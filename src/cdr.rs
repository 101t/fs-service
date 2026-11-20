//! Built-in CDR recorder. Replaces cdr-pusher + mod_cdr_sqlite + the Postgres side database.
//!
//! Two inputs feed the same `cdr` table (deduplicated by channel UUID):
//! * live `CHANNEL_HANGUP_COMPLETE` events from the event socket (zero FreeSWITCH config), and
//! * optional `POST /api/v1/cdrs/ingest` from mod_json_cdr, which retries and spools to disk,
//!   so CDRs survive while this service is restarting.

use chrono::{DateTime, Utc};
use serde::Serialize;
use serde_json::Value;
use sqlx::{FromRow, SqlitePool};
use tokio::sync::mpsc;
use utoipa::ToSchema;

use crate::esl::Event;

/// Events the recorder needs from FreeSWITCH.
pub const SUBSCRIPTIONS: &str = "CHANNEL_HANGUP_COMPLETE BACKGROUND_JOB";
/// Channel variable stamped on every call this service originates.
pub const BATCH_VAR: &str = "fss_batch_id";

/// One finished call leg.
#[derive(Debug, Clone, Serialize, FromRow, ToSchema)]
pub struct CallRecord {
    /// FreeSWITCH channel UUID.
    pub uuid: String,
    /// Batch that started this call, when it came from `POST /api/v1/calls/*`.
    pub batch_id: Option<String>,
    /// `inbound` or `outbound`.
    pub direction: String,
    pub caller_id_name: Option<String>,
    pub caller_id_number: Option<String>,
    pub destination_number: Option<String>,
    /// Sofia gateway the call went through.
    pub gateway: Option<String>,
    pub start_at: DateTime<Utc>,
    pub answer_at: Option<DateTime<Utc>>,
    pub end_at: DateTime<Utc>,
    /// Total seconds, ringing included.
    pub duration: i64,
    /// Seconds after answer.
    pub billsec: i64,
    /// Q.850 cause, e.g. `NORMAL_CLEARING`, `USER_BUSY`.
    pub hangup_cause: String,
    /// Present only on fax calls.
    pub fax_success: Option<bool>,
    pub fax_result_text: Option<String>,
    pub fax_pages: Option<i64>,
}

/// Builds a record from FreeSWITCH variable names (`uuid`, `start_epoch`, ...).
fn build(get: impl Fn(&str) -> Option<String>) -> Option<CallRecord> {
    let get = |k: &str| get(k).filter(|v| !v.is_empty());
    let num = |k: &str| get(k).and_then(|v| v.parse::<i64>().ok());
    let ts = |k: &str| num(k).filter(|&s| s > 0).and_then(|s| DateTime::from_timestamp(s, 0));
    let end_at = ts("end_epoch").unwrap_or_else(Utc::now);
    Some(CallRecord {
        uuid: get("uuid")?,
        batch_id: get(BATCH_VAR),
        direction: get("direction").unwrap_or_else(|| "unknown".into()),
        caller_id_name: get("caller_id_name"),
        caller_id_number: get("caller_id_number"),
        destination_number: get("destination_number"),
        gateway: get("sip_gateway_name"),
        start_at: ts("start_epoch").unwrap_or(end_at),
        answer_at: ts("answer_epoch"),
        end_at,
        duration: num("duration").unwrap_or(0),
        billsec: num("billsec").unwrap_or(0),
        hangup_cause: get("hangup_cause").unwrap_or_default(),
        fax_success: get("fax_success").map(|v| v == "1"),
        fax_result_text: get("fax_result_text"),
        fax_pages: num("fax_document_transferred_pages"),
    })
}

/// From a `CHANNEL_HANGUP_COMPLETE` event.
pub fn from_event(ev: &Event) -> Option<CallRecord> {
    build(|k| {
        let header = match k {
            "uuid" => "Unique-ID",
            "direction" => "Call-Direction",
            "caller_id_name" => "Caller-Caller-ID-Name",
            "caller_id_number" => "Caller-Caller-ID-Number",
            "destination_number" => "Caller-Destination-Number",
            "hangup_cause" => "Hangup-Cause",
            var => return ev.get(&format!("variable_{var}")).map(str::to_owned),
        };
        ev.get(header).map(str::to_owned)
    })
}

/// From a mod_json_cdr document (`encode` = `false`).
pub fn from_json_cdr(doc: &Value) -> Option<CallRecord> {
    let vars = &doc["variables"];
    let flow = &doc["callflow"];
    let profile = if flow.is_array() { &flow[0]["caller_profile"] } else { &flow["caller_profile"] };
    build(|k| {
        let v = match k {
            "caller_id_name" | "caller_id_number" | "destination_number" => profile[k].as_str().or(vars[k].as_str()),
            "direction" => vars[k].as_str().or(doc["channel_data"]["direction"].as_str()),
            _ => vars[k].as_str(),
        };
        v.map(str::to_owned)
    })
}

/// Stores a record; returns false when that UUID was already recorded.
pub async fn insert(db: &SqlitePool, r: &CallRecord) -> sqlx::Result<bool> {
    let res = sqlx::query(
        "INSERT OR IGNORE INTO cdr (uuid, batch_id, direction, caller_id_name, caller_id_number,
            destination_number, gateway, start_at, answer_at, end_at, duration, billsec,
            hangup_cause, fax_success, fax_result_text, fax_pages)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16)",
    )
    .bind(&r.uuid)
    .bind(&r.batch_id)
    .bind(&r.direction)
    .bind(&r.caller_id_name)
    .bind(&r.caller_id_number)
    .bind(&r.destination_number)
    .bind(&r.gateway)
    .bind(r.start_at.timestamp())
    .bind(r.answer_at.map(|t| t.timestamp()))
    .bind(r.end_at.timestamp())
    .bind(r.duration)
    .bind(r.billsec)
    .bind(&r.hangup_cause)
    .bind(r.fax_success)
    .bind(&r.fax_result_text)
    .bind(r.fax_pages)
    .execute(db)
    .await?;
    Ok(res.rows_affected() == 1)
}

/// Consumes event-socket events forever.
pub async fn record(db: SqlitePool, mut events: mpsc::Receiver<Event>) {
    while let Some(ev) = events.recv().await {
        match ev.get("Event-Name") {
            Some("CHANNEL_HANGUP_COMPLETE") => {
                let Some(r) = from_event(&ev) else { continue };
                if let Err(e) = insert(&db, &r).await {
                    // Full record in the log so it can be replayed by hand.
                    tracing::error!(error = %e, cdr = ?r, "failed to store CDR");
                }
            }
            // Result of an originate we queued with bgapi; only failures matter here.
            Some("BACKGROUND_JOB") => {
                let (Some(job), Some(body)) = (ev.get("Job-UUID"), ev.body.as_deref()) else { continue };
                let Some(err) = body.strip_prefix("-ERR") else { continue };
                let res = sqlx::query("UPDATE call_job SET error = $1 WHERE uuid = $2")
                    .bind(err.trim())
                    .bind(job)
                    .execute(&db)
                    .await;
                if let Err(e) = res {
                    tracing::error!(error = %e, job, "failed to store originate failure");
                }
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::esl::parse_event;
    use serde_json::json;

    #[test]
    fn maps_hangup_event() {
        let ev = parse_event(
            "Event-Name: CHANNEL_HANGUP_COMPLETE\nUnique-ID: u1\nCall-Direction: outbound\n\
             Caller-Caller-ID-Number: 100\nCaller-Destination-Number: 200\nHangup-Cause: NORMAL_CLEARING\n\
             variable_start_epoch: 1795000000\nvariable_answer_epoch: 0\nvariable_end_epoch: 1795000042\n\
             variable_duration: 42\nvariable_billsec: 30\nvariable_fss_batch_id: b1\n\
             variable_fax_success: 1\nvariable_fax_result_text: OK\nvariable_fax_document_transferred_pages: 3\n",
        );
        let r = from_event(&ev).unwrap();
        assert_eq!((r.uuid.as_str(), r.direction.as_str(), r.batch_id.as_deref()), ("u1", "outbound", Some("b1")));
        assert_eq!((r.duration, r.billsec, r.fax_pages, r.fax_success), (42, 30, Some(3), Some(true)));
        assert_eq!(r.start_at.timestamp(), 1795000000);
        assert!(r.answer_at.is_none(), "answer_epoch 0 means never answered");
        assert!(from_event(&parse_event("Event-Name: CHANNEL_HANGUP_COMPLETE\n")).is_none());
    }

    #[test]
    fn maps_json_cdr() {
        let doc = json!({
            "channel_data": {"direction": "inbound"},
            "variables": {"uuid": "u2", "start_epoch": "1795000000", "end_epoch": "1795000010",
                          "billsec": "8", "hangup_cause": "NORMAL_CLEARING"},
            "callflow": [{"caller_profile": {"caller_id_number": "555", "destination_number": "900"}}]
        });
        let r = from_json_cdr(&doc).unwrap();
        assert_eq!((r.direction.as_str(), r.caller_id_number.as_deref(), r.billsec), ("inbound", Some("555"), 8));
        assert!(r.fax_success.is_none());
    }
}
