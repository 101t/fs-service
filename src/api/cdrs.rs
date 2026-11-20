//! Query the CDR log, ingest CDRs from mod_json_cdr, and list received faxes.

use axum::{
    Json,
    body::Bytes,
    extract::{Query, State},
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{QueryBuilder, Sqlite};
use utoipa::{IntoParams, ToSchema};

use super::{ApiError, ErrorBody, Shared, require};
use crate::cdr::{self, CallRecord};

#[derive(Deserialize, IntoParams, Default)]
pub struct CdrQuery {
    /// Calls that started at or after this time (RFC 3339).
    pub from: Option<DateTime<Utc>>,
    /// Calls that started before this time (RFC 3339).
    pub to: Option<DateTime<Utc>>,
    /// Matches caller or destination number.
    pub number: Option<String>,
    /// `inbound` or `outbound`.
    pub direction: Option<String>,
    pub batch_id: Option<String>,
    /// Only calls that carried a fax.
    #[serde(default)]
    pub fax_only: bool,
    /// Page size, 1-1000 (default 100).
    pub limit: Option<i64>,
    #[serde(default)]
    pub offset: i64,
}

#[derive(Serialize, ToSchema)]
pub struct CdrPage {
    pub items: Vec<CallRecord>,
    pub limit: i64,
    pub offset: i64,
}

async fn search(s: &Shared, q: &CdrQuery, destinations: &[String]) -> Result<CdrPage, ApiError> {
    let limit = q.limit.unwrap_or(100);
    require((1..=1000).contains(&limit) && q.offset >= 0, "limit must be 1-1000 and offset >= 0")?;
    let mut sql: QueryBuilder<Sqlite> = QueryBuilder::new("SELECT * FROM cdr WHERE 1 = 1");
    if let Some(from) = q.from {
        sql.push(" AND start_at >= ").push_bind(from.timestamp());
    }
    if let Some(to) = q.to {
        sql.push(" AND start_at < ").push_bind(to.timestamp());
    }
    if let Some(n) = &q.number {
        sql.push(" AND (caller_id_number = ")
            .push_bind(n.clone())
            .push(" OR destination_number = ")
            .push_bind(n.clone())
            .push(")");
    }
    if let Some(d) = &q.direction {
        sql.push(" AND direction = ").push_bind(d.clone());
    }
    if let Some(b) = &q.batch_id {
        sql.push(" AND batch_id = ").push_bind(b.clone());
    }
    if q.fax_only {
        sql.push(" AND fax_success IS NOT NULL");
    }
    if !destinations.is_empty() {
        sql.push(" AND destination_number IN (");
        let mut list = sql.separated(", ");
        for d in destinations {
            list.push_bind(d.clone());
        }
        sql.push(")");
    }
    sql.push(" ORDER BY start_at DESC LIMIT ").push_bind(limit).push(" OFFSET ").push_bind(q.offset);
    let items = sql.build_query_as::<CallRecord>().fetch_all(&s.db).await?;
    Ok(CdrPage { items, limit, offset: q.offset })
}

/// Search call detail records, newest first.
#[utoipa::path(get, path = "/api/v1/cdrs", tag = "cdrs", params(CdrQuery),
    responses((status = OK, body = CdrPage), (status = BAD_REQUEST, body = ErrorBody)))]
pub async fn list(State(s): State<Shared>, Query(q): Query<CdrQuery>) -> Result<Json<CdrPage>, ApiError> {
    Ok(Json(search(&s, &q, &[]).await?))
}

#[derive(Serialize, ToSchema)]
pub struct Ingested {
    /// False when this call UUID was already recorded (e.g. via the event socket).
    pub stored: bool,
}

/// Receive a CDR from FreeSWITCH mod_json_cdr (durable path: FreeSWITCH retries and spools on failure).
#[utoipa::path(post, path = "/api/v1/cdrs/ingest", tag = "cdrs",
    request_body(content = Object, description = "mod_json_cdr document (`encode` = `false`)"),
    responses((status = OK, body = Ingested), (status = BAD_REQUEST, body = ErrorBody)))]
pub async fn ingest(State(s): State<Shared>, body: Bytes) -> Result<Json<Ingested>, ApiError> {
    // Raw bytes: mod_json_cdr does not always send `Content-Type: application/json`.
    let doc: serde_json::Value =
        serde_json::from_slice(&body).map_err(|e| ApiError::BadRequest(format!("invalid JSON: {e}")))?;
    let record = cdr::from_json_cdr(&doc).ok_or_else(|| ApiError::BadRequest("CDR has no variables.uuid".into()))?;
    Ok(Json(Ingested { stored: cdr::insert(&s.db, &record).await? }))
}

#[derive(Deserialize, IntoParams)]
pub struct InboxQuery {
    pub from: Option<DateTime<Utc>>,
    pub to: Option<DateTime<Utc>>,
    /// Comma-separated receiving numbers (your DIDs); empty means all.
    pub receivers: Option<String>,
    pub limit: Option<i64>,
    #[serde(default)]
    pub offset: i64,
}

#[derive(Serialize, ToSchema)]
pub struct InboxFax {
    #[serde(flatten)]
    pub cdr: CallRecord,
    /// Download with `GET /api/v1/files/{file}?folder=inbox`; null when no image was saved.
    pub file: Option<String>,
}

/// Received faxes, newest first.
#[utoipa::path(get, path = "/api/v1/fax/inbox", tag = "cdrs", params(InboxQuery),
    responses((status = OK, body = Vec<InboxFax>), (status = BAD_REQUEST, body = ErrorBody)))]
pub async fn fax_inbox(State(s): State<Shared>, Query(q): Query<InboxQuery>) -> Result<Json<Vec<InboxFax>>, ApiError> {
    let receivers: Vec<String> = q
        .receivers
        .iter()
        .flat_map(|r| r.split(','))
        .map(str::trim)
        .filter(|r| !r.is_empty())
        .map(str::to_owned)
        .collect();
    let query = CdrQuery {
        from: q.from,
        to: q.to,
        direction: Some("inbound".into()),
        fax_only: true,
        limit: q.limit,
        offset: q.offset,
        ..Default::default()
    };
    let mut out = Vec::new();
    for cdr in search(&s, &query, &receivers).await?.items {
        let name = format!("rxfax-{}.tiff", cdr.uuid);
        let file = tokio::fs::try_exists(s.cfg.inbox_dir.join(&name)).await.unwrap_or(false).then_some(name);
        out.push(InboxFax { cdr, file });
    }
    Ok(Json(out))
}

#[cfg(test)]
mod tests {
    use crate::api::tests::app;
    use axum::http::StatusCode;
    use serde_json::json;

    #[tokio::test]
    async fn ingest_then_query() {
        let app = app().await;
        let doc = json!({
            "variables": {"uuid": "rx-1", "direction": "inbound", "start_epoch": "1795000000", "end_epoch": "1795000060",
                          "billsec": "55", "hangup_cause": "NORMAL_CLEARING", "fax_success": "1", "fax_document_transferred_pages": "2"},
            "callflow": [{"caller_profile": {"caller_id_number": "555", "destination_number": "9001"}}]
        });
        let (s, r) = app.call("POST", "/api/v1/cdrs/ingest", true, Some(doc.clone())).await;
        assert_eq!((s, r["stored"].as_bool()), (StatusCode::OK, Some(true)));
        let (_, r) = app.call("POST", "/api/v1/cdrs/ingest", true, Some(doc)).await;
        assert_eq!(r["stored"].as_bool(), Some(false), "duplicates are ignored");
        assert_eq!(
            app.call("POST", "/api/v1/cdrs/ingest", true, Some(json!({"x": 1}))).await.0,
            StatusCode::BAD_REQUEST
        );

        std::fs::write(app.dir.join("in/rxfax-rx-1.tiff"), b"II*").unwrap();
        let (s, inbox) =
            app.call("GET", "/api/v1/fax/inbox?receivers=9001,9002&from=2026-01-01T00:00:00Z", true, None).await;
        assert_eq!(s, StatusCode::OK);
        assert_eq!((inbox[0]["uuid"].as_str(), inbox[0]["file"].as_str()), (Some("rx-1"), Some("rxfax-rx-1.tiff")));
        assert_eq!(inbox[0]["start_at"], "2026-11-18T11:06:40Z");

        let (_, page) = app.call("GET", "/api/v1/cdrs?number=555&direction=inbound", true, None).await;
        assert_eq!(page["items"].as_array().map(Vec::len), Some(1));
        let (_, page) = app.call("GET", "/api/v1/cdrs?number=999", true, None).await;
        assert_eq!(page["items"].as_array().map(Vec::len), Some(0));
        assert_eq!(app.call("GET", "/api/v1/cdrs?limit=0", true, None).await.0, StatusCode::BAD_REQUEST);
    }
}
