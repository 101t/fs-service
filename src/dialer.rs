//! Paces outbound calls so a 1,000-number fax batch does not trip FreeSWITCH's
//! `sessions-per-second` limit. One global queue, one originate per tick.

use std::{sync::Arc, time::Duration};

use sqlx::SqlitePool;
use tokio::{
    sync::mpsc,
    time::{MissedTickBehavior, interval},
};

use crate::esl::Esl;

pub struct DialJob {
    pub uuid: String,
    pub originate: String,
}

pub fn spawn(esl: Arc<Esl>, db: SqlitePool, per_second: u32) -> mpsc::Sender<DialJob> {
    let (tx, mut rx) = mpsc::channel::<DialJob>(100_000);
    tokio::spawn(async move {
        let mut tick = interval(Duration::from_secs(1) / per_second);
        tick.set_missed_tick_behavior(MissedTickBehavior::Delay);
        while let Some(job) = rx.recv().await {
            tick.tick().await;
            let now = chrono::Utc::now().timestamp();
            let res = match esl.bgapi(&format!("originate {}", job.originate), &job.uuid).await {
                Ok(()) => sqlx::query("UPDATE call_job SET sent_at = $1 WHERE uuid = $2").bind(now).bind(&job.uuid),
                Err(e) => {
                    sqlx::query("UPDATE call_job SET error = $1 WHERE uuid = $2").bind(e.to_string()).bind(&job.uuid)
                }
            }
            .execute(&db)
            .await;
            if let Err(e) = res {
                tracing::error!(error = %e, uuid = job.uuid, "failed to update call job");
            }
        }
    });
    tx
}
