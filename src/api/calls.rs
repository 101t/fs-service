//! Fax and voice broadcast. Each request becomes a batch of calls, dialed at a steady rate,
//! whose outcome is tracked through CDRs.

use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use utoipa::ToSchema;

use super::{ApiError, ErrorBody, Shared, require, valid_name, valid_number};
use crate::{
    cdr::BATCH_VAR,
    convert::{AUDIO, FaxQuality, extension},
    dialer::DialJob,
};

#[derive(Deserialize, ToSchema)]
#[schema(example = json!({"gateway": "trunk1", "file": "0193a5c2-7e1f-7b3a-9c41-5f2d8e6a1b20.wav", "numbers": ["905551112233", "905554445566"]}))]
pub struct VoiceRequest {
    /// Gateway to dial out through.
    pub gateway: String,
    /// Uploaded `.wav` or `.mp3` (mp3 needs mod_shout).
    pub file: String,
    pub numbers: Vec<String>,
    /// Caller ID number presented to the callee.
    pub caller_id: Option<String>,
}

#[derive(Deserialize, ToSchema)]
#[schema(example = json!({"gateway": "trunk1", "file": "0193a5c2-7e1f-7b3a-9c41-5f2d8e6a1b20.pdf", "numbers": ["902129998877"], "quality": "fine"}))]
pub struct FaxRequest {
    pub gateway: String,
    /// Any uploaded document or image; it is converted to fax TIFF first.
    pub file: String,
    pub numbers: Vec<String>,
    /// Caller ID number, also sent as the fax station ID.
    pub caller_id: Option<String>,
    #[serde(default)]
    pub quality: FaxQuality,
}

#[derive(Serialize, ToSchema)]
pub struct QueuedCall {
    /// FreeSWITCH channel UUID of this call.
    pub uuid: String,
    pub destination: String,
}

#[derive(Serialize, ToSchema)]
pub struct BatchAccepted {
    /// Poll `GET /api/v1/batches/{batch_id}` for delivery results.
    pub batch_id: String,
    pub calls: Vec<QueuedCall>,
}

/// Play an audio file to a list of numbers.
#[utoipa::path(post, path = "/api/v1/calls/voice", tag = "calls", request_body = VoiceRequest,
    responses((status = ACCEPTED, body = BatchAccepted), (status = BAD_REQUEST, body = ErrorBody), (status = NOT_FOUND, body = ErrorBody)))]
pub async fn voice(
    State(s): State<Shared>,
    Json(r): Json<VoiceRequest>,
) -> Result<(StatusCode, Json<BatchAccepted>), ApiError> {
    let path = source(&s, &r.file)?;
    require(AUDIO.contains(&extension(&path).as_str()), "voice calls need a .wav or .mp3 file")?;
    let app = format!("&playback({})", path.display());
    queue(&s, "voice", &r.gateway, &r.file, &r.numbers, r.caller_id.as_deref(), "", &app).await
}

/// Fax a document to a list of numbers (T.38 when the far end supports it, G.711 otherwise).
#[utoipa::path(post, path = "/api/v1/calls/fax", tag = "calls", request_body = FaxRequest,
    responses((status = ACCEPTED, body = BatchAccepted), (status = BAD_REQUEST, body = ErrorBody),
              (status = NOT_FOUND, body = ErrorBody), (status = UNPROCESSABLE_ENTITY, body = ErrorBody)))]
pub async fn fax(
    State(s): State<Shared>,
    Json(r): Json<FaxRequest>,
) -> Result<(StatusCode, Json<BatchAccepted>), ApiError> {
    validate_targets(&s, &r.gateway, &r.numbers, r.caller_id.as_deref())?;
    let src = source(&s, &r.file)?;
    let tiff = s.converter.to_fax_tiff(&src, r.quality).await?;
    let ident = r.caller_id.as_deref().map(|c| format!(",fax_ident={c}")).unwrap_or_default();
    let vars = format!(
        ",absolute_codec_string='PCMU,PCMA',fax_enable_t38=true,fax_enable_t38_request=false,fax_use_ecm=false,fax_verbose=true{ident}"
    );
    let app = format!("&txfax({})", tiff.display());
    queue(&s, "fax", &r.gateway, &r.file, &r.numbers, r.caller_id.as_deref(), &vars, &app).await
}

fn source(s: &Shared, file: &str) -> Result<std::path::PathBuf, ApiError> {
    require(valid_name(file, 128), "invalid file name")?;
    let path = s.cfg.outbox_dir.join(file);
    if !path.is_file() {
        return Err(ApiError::NotFound(format!("file `{file}` not found, upload it first")));
    }
    Ok(path)
}

fn validate_targets(s: &Shared, gateway: &str, numbers: &[String], caller_id: Option<&str>) -> Result<(), ApiError> {
    require(valid_name(gateway, 64), "invalid gateway name")?;
    require(!numbers.is_empty(), "numbers must not be empty")?;
    require(numbers.len() <= s.cfg.max_numbers, format!("at most {} numbers per request", s.cfg.max_numbers))?;
    if let Some(bad) = numbers.iter().find(|n| !valid_number(n)) {
        return Err(ApiError::BadRequest(format!("`{bad}` is not a dialable number (digits, +, *, # only)")));
    }
    require(caller_id.is_none_or(valid_number), "caller_id must be a dialable number")
}

#[allow(clippy::too_many_arguments)]
async fn queue(
    s: &Shared,
    kind: &str,
    gateway: &str,
    file: &str,
    numbers: &[String],
    caller_id: Option<&str>,
    extra_vars: &str,
    app: &str,
) -> Result<(StatusCode, Json<BatchAccepted>), ApiError> {
    validate_targets(s, gateway, numbers, caller_id)?;
    let batch_id = uuid::Uuid::now_v7().to_string();
    let now = chrono::Utc::now().timestamp();
    let cid = caller_id.map(|c| format!(",origination_caller_id_number={c}")).unwrap_or_default();

    let mut calls = Vec::with_capacity(numbers.len());
    let mut jobs = Vec::with_capacity(numbers.len());
    let mut tx = s.db.begin().await?;
    for number in numbers {
        let uuid = uuid::Uuid::now_v7().to_string();
        sqlx::query("INSERT INTO call_job (uuid, batch_id, kind, gateway, destination, file, created_at) VALUES ($1, $2, $3, $4, $5, $6, $7)")
            .bind(&uuid)
            .bind(&batch_id)
            .bind(kind)
            .bind(gateway)
            .bind(number)
            .bind(file)
            .bind(now)
            .execute(&mut *tx)
            .await?;
        // Every value here is validated or generated, so nothing can escape the {...} block.
        let originate = format!(
            "{{origination_uuid={uuid},{BATCH_VAR}={batch_id},ignore_early_media=true{cid}{extra_vars}}}sofia/gateway/{gateway}/{number} {app}"
        );
        jobs.push(DialJob { uuid: uuid.clone(), originate });
        calls.push(QueuedCall { uuid, destination: number.clone() });
    }
    tx.commit().await?;
    for job in jobs {
        s.dialer.send(job).await.map_err(|_| ApiError::Internal("dialer stopped".into()))?;
    }
    tracing::info!(batch_id, kind, gateway, calls = calls.len(), "batch queued");
    Ok((StatusCode::ACCEPTED, Json(BatchAccepted { batch_id, calls })))
}

// ---------------------------------------------------------------- reports

#[derive(Serialize, ToSchema, Clone, Copy, PartialEq, Debug)]
#[serde(rename_all = "snake_case")]
pub enum CallState {
    /// Waiting for its dial slot.
    Queued,
    /// Sent to FreeSWITCH; no CDR yet.
    InProgress,
    /// Fax confirmed by the far end, or voice call answered.
    Delivered,
    Failed,
}

#[derive(FromRow)]
struct JobRow {
    uuid: String,
    kind: String,
    destination: String,
    sent_at: Option<i64>,
    error: Option<String>,
    hangup_cause: Option<String>,
    billsec: Option<i64>,
    fax_success: Option<bool>,
    fax_result_text: Option<String>,
    fax_pages: Option<i64>,
    end_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Serialize, ToSchema)]
pub struct CallStatus {
    pub uuid: String,
    pub destination: String,
    pub state: CallState,
    /// Why the call failed, when known.
    pub reason: Option<String>,
    pub hangup_cause: Option<String>,
    pub billsec: Option<i64>,
    pub fax_pages: Option<i64>,
    pub ended_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Serialize, ToSchema)]
pub struct BatchReport {
    pub batch_id: String,
    /// `voice` or `fax`.
    pub kind: String,
    pub total: usize,
    pub queued: usize,
    pub in_progress: usize,
    pub delivered: usize,
    pub failed: usize,
    pub calls: Vec<CallStatus>,
}

impl JobRow {
    /// The CDR is the ground truth; without one, an originate error or the dial progress decides.
    fn state(&self) -> CallState {
        let Some(billsec) = self.billsec else {
            return match (&self.error, self.sent_at) {
                (Some(_), _) => CallState::Failed,
                (None, Some(_)) => CallState::InProgress,
                (None, None) => CallState::Queued,
            };
        };
        let ok = if self.kind == "fax" { self.fax_success == Some(true) } else { billsec > 0 };
        if ok { CallState::Delivered } else { CallState::Failed }
    }
}

/// Delivery report for a batch.
#[utoipa::path(get, path = "/api/v1/batches/{batch_id}", tag = "calls",
    params(("batch_id" = String, Path, description = "Returned by POST /api/v1/calls/*")),
    responses((status = OK, body = BatchReport), (status = NOT_FOUND, body = ErrorBody)))]
pub async fn batch(State(s): State<Shared>, Path(batch_id): Path<String>) -> Result<Json<BatchReport>, ApiError> {
    let rows: Vec<JobRow> = sqlx::query_as(
        "SELECT j.uuid, j.kind, j.destination, j.sent_at, j.error,
                c.hangup_cause, c.billsec, c.fax_success, c.fax_result_text, c.fax_pages, c.end_at
         FROM call_job j LEFT JOIN cdr c ON c.uuid = j.uuid
         WHERE j.batch_id = $1 ORDER BY j.uuid",
    )
    .bind(&batch_id)
    .fetch_all(&s.db)
    .await?;
    let Some(kind) = rows.first().map(|r| r.kind.clone()) else {
        return Err(ApiError::NotFound(format!("batch `{batch_id}` not found")));
    };
    let calls: Vec<CallStatus> = rows
        .into_iter()
        .map(|r| {
            let state = r.state();
            let reason = r.error.clone().or_else(|| {
                (state == CallState::Failed).then(|| r.fax_result_text.clone().or(r.hangup_cause.clone())).flatten()
            });
            CallStatus {
                state,
                reason,
                uuid: r.uuid,
                destination: r.destination,
                hangup_cause: r.hangup_cause,
                billsec: r.billsec,
                fax_pages: r.fax_pages,
                ended_at: r.end_at,
            }
        })
        .collect();
    let count = |st: CallState| calls.iter().filter(|c| c.state == st).count();
    Ok(Json(BatchReport {
        batch_id,
        kind,
        total: calls.len(),
        queued: count(CallState::Queued),
        in_progress: count(CallState::InProgress),
        delivered: count(CallState::Delivered),
        failed: count(CallState::Failed),
        calls,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::tests::app;
    use serde_json::json;

    #[tokio::test]
    async fn voice_batch_lifecycle() {
        let app = app().await;
        std::fs::write(app.dir.join("out/hello.wav"), b"RIFF").unwrap();
        let req = |numbers: serde_json::Value| json!({"gateway": "trunk1", "file": "hello.wav", "numbers": numbers});

        assert_eq!(
            app.call("POST", "/api/v1/calls/voice", true, Some(req(json!(["1 &api shutdown"])))).await.0,
            StatusCode::BAD_REQUEST
        );
        assert_eq!(
            app.call("POST", "/api/v1/calls/voice", true, Some(req(json!([])))).await.0,
            StatusCode::BAD_REQUEST
        );
        let missing = json!({"gateway": "trunk1", "file": "nope.wav", "numbers": ["1"]});
        assert_eq!(app.call("POST", "/api/v1/calls/voice", true, Some(missing)).await.0, StatusCode::NOT_FOUND);

        let (s, accepted) = app.call("POST", "/api/v1/calls/voice", true, Some(req(json!(["100", "200"])))).await;
        assert_eq!(s, StatusCode::ACCEPTED);
        let batch = accepted["batch_id"].as_str().unwrap().to_string();
        let first = accepted["calls"][0]["uuid"].as_str().unwrap().to_string();

        // The first call's CDR arrives: answered for 12 seconds.
        sqlx::query("INSERT INTO cdr (uuid, batch_id, direction, start_at, end_at, duration, billsec, hangup_cause) VALUES ($1, $2, 'outbound', 1, 20, 19, 12, 'NORMAL_CLEARING')")
            .bind(&first).bind(&batch).execute(&app.db).await.unwrap();

        // FreeSWITCH is down in tests, so the dialer marks the other call failed; give it a moment.
        tokio::time::sleep(std::time::Duration::from_millis(200)).await;
        let (s, report) = app.call("GET", &format!("/api/v1/batches/{batch}"), true, None).await;
        assert_eq!(s, StatusCode::OK);
        assert_eq!(
            (report["total"].as_u64(), report["delivered"].as_u64(), report["failed"].as_u64()),
            (Some(2), Some(1), Some(1))
        );
        assert_eq!(report["calls"][1]["reason"], "FreeSWITCH is not connected");

        assert_eq!(app.call("GET", "/api/v1/batches/unknown", true, None).await.0, StatusCode::NOT_FOUND);
    }
}
