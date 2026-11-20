//! REST API: routing, authentication, errors, input validation and the OpenAPI document.

mod calls;
mod cdrs;
mod files;
mod gateways;

use std::{sync::Arc, time::Duration};

use axum::{
    Json, Router,
    extract::{DefaultBodyLimit, Request, State},
    http::{HeaderValue, StatusCode, header},
    middleware::{self, Next},
    response::{IntoResponse, Response},
    routing::get,
};
use base64::{Engine, engine::general_purpose::STANDARD as BASE64};
use serde::Serialize;
use sqlx::SqlitePool;
use tokio::sync::mpsc;
use tower_http::{
    sensitive_headers::SetSensitiveRequestHeadersLayer, set_header::SetResponseHeaderLayer, timeout::TimeoutLayer,
    trace::TraceLayer,
};
use utoipa::{
    Modify, OpenApi, ToSchema,
    openapi::security::{Http, HttpAuthScheme, SecurityScheme},
};
use utoipa_axum::{router::OpenApiRouter, routes};
use utoipa_scalar::{Scalar, Servable};

use crate::{
    config::Config,
    convert::{ConvertError, Converter},
    dialer::DialJob,
    esl::{Esl, EslError},
};

pub struct AppState {
    pub cfg: Config,
    pub esl: Arc<Esl>,
    pub db: SqlitePool,
    pub dialer: mpsc::Sender<DialJob>,
    pub converter: Converter,
}

type Shared = Arc<AppState>;

#[derive(OpenApi)]
#[openapi(
    info(
        title = "fs-service",
        description = "REST gateway for FreeSWITCH: manage SIP gateways, send faxes and voice messages, \
                       and query the built-in CDR log.\n\nAuthenticate every `/api/v1` call with \
                       `Authorization: Bearer <token>`."
    ),
    modifiers(&BearerAuth),
    security(("token" = [])),
    tags(
        (name = "gateways", description = "SIP trunks registered with a Sofia profile"),
        (name = "files", description = "Upload documents and audio, download conversions and received faxes"),
        (name = "calls", description = "Fax and voice broadcast, with per-batch delivery reports"),
        (name = "cdrs", description = "Call detail records captured by the built-in recorder"),
        (name = "system", description = "Health and metadata"),
    )
)]
struct ApiDoc;

struct BearerAuth;

impl Modify for BearerAuth {
    fn modify(&self, api: &mut utoipa::openapi::OpenApi) {
        api.components
            .get_or_insert_with(Default::default)
            .add_security_scheme("token", SecurityScheme::Http(Http::new(HttpAuthScheme::Bearer)));
    }
}

pub fn router(state: AppState) -> Router {
    let upload_limit = state.cfg.max_upload_mb * 1024 * 1024;
    let state = Arc::new(state);

    let uploads = OpenApiRouter::new().routes(routes!(files::upload)).layer(DefaultBodyLimit::max(upload_limit));
    let (api, openapi) = OpenApiRouter::with_openapi(ApiDoc::openapi())
        .routes(routes!(gateways::list))
        .routes(routes!(gateways::show, gateways::put, gateways::delete))
        .merge(uploads)
        .routes(routes!(files::download))
        .routes(routes!(calls::voice))
        .routes(routes!(calls::fax))
        .routes(routes!(calls::batch))
        .routes(routes!(cdrs::list))
        .routes(routes!(cdrs::ingest))
        .routes(routes!(cdrs::fax_inbox))
        .route_layer(middleware::from_fn_with_state(state.clone(), authenticate))
        .routes(routes!(health)) // added after the auth layer, so it stays public
        .with_state(state)
        .split_for_parts();

    api.merge(Scalar::with_url("/docs", openapi.clone()))
        .route("/openapi.json", get(move || async move { Json(openapi) }))
        .layer(DefaultBodyLimit::max(1024 * 1024))
        // Conversions (LibreOffice + Ghostscript) are the slowest legitimate requests.
        .layer(TimeoutLayer::with_status_code(StatusCode::REQUEST_TIMEOUT, Duration::from_secs(300)))
        .layer(SetResponseHeaderLayer::overriding(header::X_CONTENT_TYPE_OPTIONS, HeaderValue::from_static("nosniff")))
        .layer(SetResponseHeaderLayer::if_not_present(header::CACHE_CONTROL, HeaderValue::from_static("no-store")))
        .layer(TraceLayer::new_for_http())
        .layer(SetSensitiveRequestHeadersLayer::new([header::AUTHORIZATION]))
}

/// Accepts `Bearer <token>`, or HTTP Basic with the token as password (for mod_json_cdr).
async fn authenticate(State(s): State<Shared>, req: Request, next: Next) -> Result<Response, ApiError> {
    let header = req.headers().get(header::AUTHORIZATION).and_then(|v| v.to_str().ok()).unwrap_or_default();
    let presented = if let Some(token) = header.strip_prefix("Bearer ") {
        Some(token.trim().to_string())
    } else if let Some(basic) = header.strip_prefix("Basic ") {
        BASE64
            .decode(basic.trim())
            .ok()
            .and_then(|raw| String::from_utf8(raw).ok())
            .and_then(|pair| pair.split_once(':').map(|(_, pw)| pw.to_string()))
    } else {
        None
    };
    match presented {
        Some(p) if s.cfg.api_tokens.iter().any(|t| constant_time_eq(t.trim().as_bytes(), p.as_bytes())) => {
            Ok(next.run(req).await)
        }
        _ => Err(ApiError::Unauthorized),
    }
}

/// Compares without an early exit, so response timing does not reveal how much of a token matched.
fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
    a.len() == b.len() && a.iter().zip(b).fold(0u8, |acc, (x, y)| acc | (x ^ y)) == 0
}

#[derive(Serialize, ToSchema)]
struct Health {
    /// Always `ok` while the process serves requests.
    status: &'static str,
    /// Whether the event socket session to FreeSWITCH is up.
    freeswitch_connected: bool,
    version: &'static str,
}

/// Liveness and FreeSWITCH connectivity. No authentication.
#[utoipa::path(get, path = "/health", tag = "system", security(()), responses((status = OK, body = Health)))]
async fn health(State(s): State<Shared>) -> Json<Health> {
    Json(Health { status: "ok", freeswitch_connected: s.esl.is_connected(), version: env!("CARGO_PKG_VERSION") })
}

// ---------------------------------------------------------------- validation

/// File and gateway names: letters, digits, `.`, `_`, `-`; never a path, never hidden, never an option.
pub fn valid_name(s: &str, max: usize) -> bool {
    (1..=max).contains(&s.len())
        && !s.starts_with(['.', '-'])
        && s.bytes().all(|b| b.is_ascii_alphanumeric() || b"._-".contains(&b))
}

/// Dialable numbers: digits plus `+`, `*`, `#`. Nothing that could break out of a dial string.
pub fn valid_number(s: &str) -> bool {
    (1..=32).contains(&s.len()) && s.bytes().all(|b| b.is_ascii_digit() || b"+*#".contains(&b))
}

pub fn require(ok: bool, msg: impl Into<String>) -> Result<(), ApiError> {
    if ok { Ok(()) } else { Err(ApiError::BadRequest(msg.into())) }
}

// ---------------------------------------------------------------- errors

#[derive(Debug, thiserror::Error)]
pub enum ApiError {
    #[error("{0}")]
    BadRequest(String),
    #[error("missing or invalid API token")]
    Unauthorized,
    #[error("{0}")]
    NotFound(String),
    #[error(transparent)]
    FreeSwitch(#[from] EslError),
    #[error(transparent)]
    Convert(#[from] ConvertError),
    #[error("internal error")]
    Internal(String),
}

impl From<sqlx::Error> for ApiError {
    fn from(e: sqlx::Error) -> Self {
        Self::Internal(format!("database: {e}"))
    }
}

impl From<std::io::Error> for ApiError {
    fn from(e: std::io::Error) -> Self {
        Self::Internal(format!("io: {e}"))
    }
}

/// Every error response has this shape.
#[derive(Serialize, ToSchema)]
pub struct ErrorBody {
    /// Stable machine-readable code, e.g. `bad_request`, `freeswitch_unavailable`.
    pub error: &'static str,
    /// Human-readable explanation.
    pub message: String,
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, code) = match &self {
            Self::BadRequest(_) | Self::FreeSwitch(EslError::InvalidCommand) => {
                (StatusCode::BAD_REQUEST, "bad_request")
            }
            Self::Unauthorized => (StatusCode::UNAUTHORIZED, "unauthorized"),
            Self::NotFound(_) => (StatusCode::NOT_FOUND, "not_found"),
            Self::FreeSwitch(EslError::Command(_)) => (StatusCode::BAD_GATEWAY, "freeswitch_error"),
            Self::FreeSwitch(_) => (StatusCode::SERVICE_UNAVAILABLE, "freeswitch_unavailable"),
            Self::Convert(ConvertError::Unsupported(_) | ConvertError::Failed(..)) => {
                (StatusCode::UNPROCESSABLE_ENTITY, "conversion_failed")
            }
            Self::Convert(ConvertError::Timeout) => (StatusCode::UNPROCESSABLE_ENTITY, "conversion_timeout"),
            Self::Convert(_) | Self::Internal(_) => (StatusCode::INTERNAL_SERVER_ERROR, "internal"),
        };
        if status.is_server_error() {
            tracing::error!(error = ?self, "request failed");
        }
        let message = match &self {
            Self::Internal(_) | Self::Convert(ConvertError::Io(_)) => "internal error, see server logs".to_string(),
            e => e.to_string(),
        };
        let mut res = (status, Json(ErrorBody { error: code, message })).into_response();
        if status == StatusCode::UNAUTHORIZED {
            res.headers_mut().insert(header::WWW_AUTHENTICATE, HeaderValue::from_static("Bearer"));
        }
        res
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use axum::body::Body;
    use http_body_util::BodyExt;
    use tower::ServiceExt;

    pub const TOKEN: &str = "test-token-0123456789abcdef0123456789";

    pub struct TestApp {
        pub router: Router,
        pub db: SqlitePool,
        pub dir: std::path::PathBuf,
    }

    pub async fn app() -> TestApp {
        let dir = std::env::temp_dir().join(format!("fss-api-{}", uuid::Uuid::now_v7()));
        for sub in ["out", "in", "gw"] {
            std::fs::create_dir_all(dir.join(sub)).unwrap();
        }
        let path = |sub: &str| dir.join(sub).to_string_lossy().into_owned();
        let cfg = <Config as clap::Parser>::parse_from([
            "fs-service".to_string(),
            "--api-tokens".into(),
            TOKEN.into(),
            "--esl-addr".into(),
            "127.0.0.1:1".into(), // nothing listens: FreeSWITCH is "down"
            "--esl-password".into(),
            "x".into(),
            "--outbox-dir".into(),
            path("out"),
            "--inbox-dir".into(),
            path("in"),
            "--gateway-dir".into(),
            path("gw"),
        ]);
        let db = sqlx::sqlite::SqlitePoolOptions::new().max_connections(1).connect("sqlite::memory:").await.unwrap();
        sqlx::migrate!().run(&db).await.unwrap();
        let (etx, _erx) = mpsc::channel(1);
        let esl = Esl::spawn(cfg.esl_addr.clone(), "x".into(), "ALL", etx);
        let dialer = crate::dialer::spawn(esl.clone(), db.clone(), 100);
        let converter = Converter::new(1, Duration::from_secs(5), "convert".into());
        TestApp { router: router(AppState { cfg, esl, db: db.clone(), dialer, converter }), db, dir }
    }

    impl TestApp {
        pub async fn call(
            &self,
            method: &str,
            uri: &str,
            auth: bool,
            body: Option<serde_json::Value>,
        ) -> (StatusCode, serde_json::Value) {
            let mut req = Request::builder().method(method).uri(uri);
            if auth {
                req = req.header(header::AUTHORIZATION, format!("Bearer {TOKEN}"));
            }
            let body = match body {
                Some(b) => {
                    req = req.header(header::CONTENT_TYPE, "application/json");
                    Body::from(b.to_string())
                }
                None => Body::empty(),
            };
            self.send(req.body(body).unwrap()).await
        }

        pub async fn send(&self, req: Request) -> (StatusCode, serde_json::Value) {
            let res = self.router.clone().oneshot(req).await.unwrap();
            let status = res.status();
            let bytes = res.into_body().collect().await.unwrap().to_bytes();
            (status, serde_json::from_slice(&bytes).unwrap_or(serde_json::Value::Null))
        }
    }

    #[tokio::test]
    async fn auth_is_enforced() {
        let app = app().await;
        let (s, body) = app.call("GET", "/health", false, None).await;
        assert_eq!((s, body["freeswitch_connected"].as_bool()), (StatusCode::OK, Some(false)));
        assert_eq!(app.call("GET", "/api/v1/gateways", false, None).await.0, StatusCode::UNAUTHORIZED);
        assert_eq!(app.call("GET", "/api/v1/gateways", true, None).await.0, StatusCode::OK);

        let with = |auth: String| {
            Request::get("/api/v1/gateways").header(header::AUTHORIZATION, auth).body(Body::empty()).unwrap()
        };
        assert_eq!(app.send(with("Bearer wrong-token".into())).await.0, StatusCode::UNAUTHORIZED);
        let basic = format!("Basic {}", BASE64.encode(format!("freeswitch:{TOKEN}")));
        assert_eq!(app.send(with(basic)).await.0, StatusCode::OK);

        let (s, spec) = app.call("GET", "/openapi.json", false, None).await;
        assert_eq!(s, StatusCode::OK);
        assert!(spec["paths"]["/api/v1/calls/fax"].is_object());
    }

    #[test]
    fn validators_block_injection() {
        assert!(valid_number("+905551234567") && valid_number("*99#"));
        assert!(!valid_number("123 &shutdown") && !valid_number("1}") && !valid_number(""));
        assert!(valid_name("0190f-ab.pdf", 128) && valid_name("my_trunk", 64));
        assert!(!valid_name("../etc/passwd", 128) && !valid_name(".hidden", 128) && !valid_name("-rf", 128));
        assert!(!valid_name("a/b", 128) && !valid_name("a b", 128));
    }
}
