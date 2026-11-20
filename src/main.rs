mod api;
mod cdr;
mod config;
mod convert;
mod dialer;
mod esl;

use std::{process::ExitCode, str::FromStr, time::Duration};

use clap::Parser;
use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions, SqliteSynchronous};
use tokio::{net::TcpListener, sync::mpsc};
use tracing_subscriber::EnvFilter;

use crate::{api::AppState, config::Config, convert::Converter, esl::Esl};

#[tokio::main]
async fn main() -> ExitCode {
    let cfg = Config::parse();
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info,tower_http=info,sqlx=warn"));
    let logs = tracing_subscriber::fmt().with_env_filter(filter);
    if cfg.log_json {
        logs.json().init()
    } else {
        logs.init()
    }

    if let Err(e) = cfg.validate() {
        tracing::error!("invalid configuration: {e}");
        return ExitCode::from(2);
    }
    match run(cfg).await {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            tracing::error!("fatal: {e}");
            ExitCode::FAILURE
        }
    }
}

async fn run(cfg: Config) -> Result<(), Box<dyn std::error::Error>> {
    for dir in [&cfg.outbox_dir, &cfg.inbox_dir] {
        tokio::fs::create_dir_all(dir).await.map_err(|e| format!("{}: {e}", dir.display()))?;
    }

    let db = SqlitePoolOptions::new()
        .max_connections(8)
        .connect_with(
            SqliteConnectOptions::from_str(&cfg.database_url)?
                .create_if_missing(true)
                .journal_mode(SqliteJournalMode::Wal) // readers never block the CDR writer
                .synchronous(SqliteSynchronous::Normal)
                .busy_timeout(Duration::from_secs(5)),
        )
        .await?;
    sqlx::migrate!().run(&db).await?;
    // Calls still waiting in the in-memory dial queue were lost with the previous process.
    let lost = sqlx::query("UPDATE call_job SET error = 'service restarted before the call was dialed' WHERE sent_at IS NULL AND error IS NULL")
        .execute(&db)
        .await?
        .rows_affected();
    if lost > 0 {
        tracing::warn!(lost, "marked undialed calls from the previous run as failed");
    }

    let (events_tx, events_rx) = mpsc::channel(10_000);
    let esl = Esl::spawn(cfg.esl_addr.clone(), cfg.esl_password.clone(), cdr::SUBSCRIPTIONS, events_tx);
    tokio::spawn(cdr::record(db.clone(), events_rx));
    let dialer = dialer::spawn(esl.clone(), db.clone(), cfg.dial_rate);
    let converter =
        Converter::new(cfg.convert_concurrency, Duration::from_secs(cfg.convert_timeout_secs), cfg.magick_bin.clone());

    let listener = TcpListener::bind(cfg.listen).await?;
    tracing::info!("fs-service {} listening on http://{} (API docs at /docs)", env!("CARGO_PKG_VERSION"), cfg.listen);
    let app = api::router(AppState { cfg, esl, db, dialer, converter });
    axum::serve(listener, app).with_graceful_shutdown(shutdown()).await?;
    tracing::info!("shut down cleanly");
    Ok(())
}

async fn shutdown() {
    let ctrl_c = tokio::signal::ctrl_c();
    #[cfg(unix)]
    {
        let mut term =
            tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate()).expect("SIGTERM handler");
        tokio::select! { _ = ctrl_c => {}, _ = term.recv() => {} }
    }
    #[cfg(not(unix))]
    let _ = ctrl_c.await;
}
