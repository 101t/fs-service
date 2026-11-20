use std::{net::SocketAddr, path::PathBuf};

use clap::Parser;

/// Every option can be given as a CLI flag or an environment variable (`FSS_*`).
#[derive(Parser, Clone)]
#[command(version, about, long_about = None)]
pub struct Config {
    /// Address the HTTP API listens on.
    #[arg(long, env = "FSS_LISTEN", default_value = "0.0.0.0:8080")]
    pub listen: SocketAddr,

    /// Comma-separated API bearer tokens (each at least 32 characters).
    #[arg(long, env = "FSS_API_TOKENS", value_delimiter = ',', required = true, hide_env_values = true)]
    pub api_tokens: Vec<String>,

    /// FreeSWITCH event socket address (mod_event_socket).
    #[arg(long, env = "FSS_ESL_ADDR", default_value = "127.0.0.1:8021")]
    pub esl_addr: String,

    /// FreeSWITCH event socket password. No default on purpose: never ship `ClueCon`.
    #[arg(long, env = "FSS_ESL_PASSWORD", hide_env_values = true)]
    pub esl_password: String,

    /// SQLite database for CDRs and call jobs.
    #[arg(long, env = "FSS_DATABASE_URL", default_value = "sqlite://fs-service.db")]
    pub database_url: String,

    /// Sofia SIP profile that owns the managed gateways.
    #[arg(long, env = "FSS_SIP_PROFILE", default_value = "external")]
    pub sip_profile: String,

    /// Directory included by that SIP profile, where one `<name>.xml` is written per gateway.
    #[arg(long, env = "FSS_GATEWAY_DIR", default_value = "/etc/freeswitch/sip_profiles/external")]
    pub gateway_dir: PathBuf,

    /// Uploaded files and fax conversions. FreeSWITCH must see the same path.
    #[arg(long, env = "FSS_OUTBOX_DIR", default_value = "/var/lib/fs-service/outbox")]
    pub outbox_dir: PathBuf,

    /// Received faxes, written by `rxfax` as `rxfax-<uuid>.tiff`.
    #[arg(long, env = "FSS_INBOX_DIR", default_value = "/var/lib/fs-service/inbox")]
    pub inbox_dir: PathBuf,

    /// Maximum upload size in MiB.
    #[arg(long, env = "FSS_MAX_UPLOAD_MB", default_value_t = 25)]
    pub max_upload_mb: usize,

    /// Maximum destination numbers in one call request.
    #[arg(long, env = "FSS_MAX_NUMBERS", default_value_t = 1000)]
    pub max_numbers: usize,

    /// Calls started per second across all batches (keep below FreeSWITCH `sessions-per-second`).
    #[arg(long, env = "FSS_DIAL_RATE", default_value_t = 10)]
    pub dial_rate: u32,

    /// Document conversions allowed to run at once.
    #[arg(long, env = "FSS_CONVERT_CONCURRENCY", default_value_t = 2)]
    pub convert_concurrency: usize,

    /// Kill a conversion that takes longer than this many seconds.
    #[arg(long, env = "FSS_CONVERT_TIMEOUT_SECS", default_value_t = 120)]
    pub convert_timeout_secs: u64,

    /// ImageMagick binary (`convert` for IM6, `magick` for IM7).
    #[arg(long, env = "FSS_MAGICK_BIN", default_value = "convert")]
    pub magick_bin: String,

    /// Log as JSON lines instead of human-readable text.
    #[arg(long, env = "FSS_LOG_JSON", default_value_t = false)]
    pub log_json: bool,
}

impl Config {
    pub fn validate(&self) -> Result<(), String> {
        if self.api_tokens.iter().any(|t| t.trim().len() < 32) {
            return Err("every FSS_API_TOKENS entry must be at least 32 characters (try `openssl rand -hex 32`)".into());
        }
        if self.esl_password.is_empty() {
            return Err("FSS_ESL_PASSWORD must not be empty".into());
        }
        if self.dial_rate == 0 || self.convert_concurrency == 0 {
            return Err("FSS_DIAL_RATE and FSS_CONVERT_CONCURRENCY must be at least 1".into());
        }
        Ok(())
    }
}
