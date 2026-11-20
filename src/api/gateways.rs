//! SIP gateways: one `<name>.xml` file per gateway in the Sofia profile's include directory.

use std::{collections::BTreeMap, path::PathBuf};

use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use quick_xml::{XmlVersion, escape::escape, events::Event, reader::Reader};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use super::{ApiError, ErrorBody, Shared, require, valid_name};
use crate::esl::EslError;

const REDACTED: &str = "********";

/// Gateway settings. Anything not covered by the named fields goes in `params`.
#[derive(Deserialize, ToSchema)]
#[schema(example = json!({"username": "905551112233", "password": "s3cret", "realm": "sip.provider.example", "register": true, "params": {"from-domain": "sip.provider.example", "expire-seconds": "600"}}))]
pub struct GatewayInput {
    pub username: String,
    pub password: String,
    /// Registrar / SIP domain.
    pub realm: String,
    /// Outbound proxy, if different from `realm`.
    pub proxy: Option<String>,
    /// Register with the provider (needed for inbound calls on most trunks).
    #[serde(default)]
    pub register: bool,
    /// Extra Sofia gateway params, e.g. `from-domain`, `caller-id-in-from`, `register-transport`.
    #[serde(default)]
    pub params: BTreeMap<String, String>,
}

#[derive(Serialize, ToSchema)]
pub struct Gateway {
    pub name: String,
    /// Configured params; `password` is always redacted.
    pub params: BTreeMap<String, String>,
    /// Live registration state from Sofia (`REGED`, `NOREG`, `FAIL_WAIT`, ...); null when unknown.
    pub state: Option<String>,
    /// Live status from Sofia, e.g. `UP (ping)`.
    pub status: Option<String>,
}

fn file(s: &Shared, name: &str) -> Result<PathBuf, ApiError> {
    require(valid_name(name, 64), "gateway name: 1-64 of A-Z a-z 0-9 . _ - (not starting with . or -)")?;
    Ok(s.cfg.gateway_dir.join(format!("{name}.xml")))
}

/// List managed gateways.
#[utoipa::path(get, path = "/api/v1/gateways", tag = "gateways", responses((status = OK, body = Vec<String>, description = "Gateway names")))]
pub async fn list(State(s): State<Shared>) -> Result<Json<Vec<String>>, ApiError> {
    let mut names = Vec::new();
    let mut dir = tokio::fs::read_dir(&s.cfg.gateway_dir).await?;
    while let Some(entry) = dir.next_entry().await? {
        let path = entry.path();
        if path.extension().is_some_and(|e| e == "xml") {
            names.extend(path.file_stem().and_then(|n| n.to_str()).map(str::to_owned));
        }
    }
    names.sort();
    Ok(Json(names))
}

/// Show a gateway's configuration and live registration state.
#[utoipa::path(get, path = "/api/v1/gateways/{name}", tag = "gateways",
    params(("name" = String, Path, description = "Gateway name")),
    responses((status = OK, body = Gateway), (status = NOT_FOUND, body = ErrorBody)))]
pub async fn show(State(s): State<Shared>, Path(name): Path<String>) -> Result<Json<Gateway>, ApiError> {
    let path = file(&s, &name)?;
    let xml = match tokio::fs::read_to_string(&path).await {
        Ok(x) => x,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            return Err(ApiError::NotFound(format!("gateway `{name}` not found")));
        }
        Err(e) => return Err(e.into()),
    };
    let mut params = parse(&xml).map_err(|e| ApiError::Internal(format!("{}: {e}", path.display())))?;
    if let Some(pw) = params.get_mut("password") {
        *pw = REDACTED.into();
    }
    // Live state is best effort: the config is still useful while FreeSWITCH is down.
    let live = s.esl.api(&format!("sofia status gateway {name}")).await.unwrap_or_default();
    let field = |key: &str| {
        live.lines()
            .filter_map(|l| l.split_once(char::is_whitespace))
            .find(|(k, _)| *k == key)
            .map(|(_, v)| v.trim().to_string())
    };
    Ok(Json(Gateway { name, state: field("State"), status: field("Status"), params }))
}

/// Create or replace a gateway, then make Sofia load it. Idempotent: safe to retry.
#[utoipa::path(put, path = "/api/v1/gateways/{name}", tag = "gateways",
    params(("name" = String, Path, description = "Gateway name, used in dial strings as `sofia/gateway/<name>/...`")),
    request_body = GatewayInput,
    responses(
        (status = OK, body = Gateway, description = "Saved and loaded"),
        (status = BAD_REQUEST, body = ErrorBody),
        (status = SERVICE_UNAVAILABLE, body = ErrorBody, description = "Saved, but FreeSWITCH could not be told; it loads on the next rescan")))]
pub async fn put(
    State(s): State<Shared>,
    Path(name): Path<String>,
    Json(input): Json<GatewayInput>,
) -> Result<Json<Gateway>, ApiError> {
    let path = file(&s, &name)?;
    let xml = render(&name, &input)?;
    // Write-then-rename: FreeSWITCH never reads a half-written file.
    let tmp = path.with_extension("xml.tmp");
    let saved = async { tokio::fs::write(&tmp, xml).await.and(tokio::fs::rename(&tmp, &path).await) };
    saved.await.map_err(|e| ApiError::Internal(format!("writing {}: {e}", path.display())))?;

    let profile = &s.cfg.sip_profile;
    s.esl.api("reloadxml").await?;
    ignore_missing(s.esl.api(&format!("sofia profile {profile} killgw {name}")).await)?;
    s.esl.api(&format!("sofia profile {profile} rescan")).await?;
    tracing::info!(gateway = name, "gateway saved and loaded");
    show(State(s), Path(name)).await
}

/// Unregister and remove a gateway.
#[utoipa::path(delete, path = "/api/v1/gateways/{name}", tag = "gateways",
    params(("name" = String, Path, description = "Gateway name")),
    responses((status = NO_CONTENT), (status = NOT_FOUND, body = ErrorBody)))]
pub async fn delete(State(s): State<Shared>, Path(name): Path<String>) -> Result<StatusCode, ApiError> {
    let path = file(&s, &name)?;
    match tokio::fs::remove_file(&path).await {
        Ok(()) => {}
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            return Err(ApiError::NotFound(format!("gateway `{name}` not found")));
        }
        Err(e) => return Err(e.into()),
    }
    s.esl.api("reloadxml").await?;
    ignore_missing(s.esl.api(&format!("sofia profile {} killgw {name}", s.cfg.sip_profile)).await)?;
    tracing::info!(gateway = name, "gateway deleted");
    Ok(StatusCode::NO_CONTENT)
}

/// `killgw` on a gateway Sofia never loaded is fine; losing FreeSWITCH is not.
fn ignore_missing(res: Result<String, EslError>) -> Result<(), ApiError> {
    match res {
        Ok(_) | Err(EslError::Command(_)) => Ok(()),
        Err(e) => Err(e.into()),
    }
}

fn render(name: &str, g: &GatewayInput) -> Result<String, ApiError> {
    let mut params: BTreeMap<&str, &str> = BTreeMap::new();
    for (k, v) in &g.params {
        require(
            (1..=64).contains(&k.len()) && k.bytes().all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-'),
            format!("param name `{k}`: use lowercase letters, digits and -"),
        )?;
        params.insert(k, v);
    }
    params.insert("username", &g.username);
    params.insert("password", &g.password);
    params.insert("realm", &g.realm);
    params.insert("register", if g.register { "true" } else { "false" });
    if let Some(proxy) = &g.proxy {
        params.insert("proxy", proxy);
    }
    for (k, v) in &params {
        require(
            v.len() <= 512 && !v.chars().any(char::is_control),
            format!("param `{k}`: at most 512 characters, no control characters"),
        )?;
    }
    require(!g.username.is_empty() && !g.realm.is_empty(), "username and realm are required")?;

    let mut xml = format!("<include>\n  <gateway name=\"{}\">\n", escape(name));
    for (k, v) in params {
        xml += &format!("    <param name=\"{}\" value=\"{}\"/>\n", escape(k), escape(v));
    }
    xml += "  </gateway>\n</include>\n";
    Ok(xml)
}

fn parse(xml: &str) -> Result<BTreeMap<String, String>, quick_xml::Error> {
    let mut reader = Reader::from_str(xml);
    let mut out = BTreeMap::new();
    loop {
        match reader.read_event()? {
            Event::Empty(e) | Event::Start(e) if e.name().as_ref() == "param" => {
                let (mut name, mut value) = (None, None);
                for attr in e.attributes().flatten() {
                    let v = attr.normalized_value(XmlVersion::Implicit1_0)?.into_owned();
                    match attr.key.as_ref() {
                        "name" => name = Some(v),
                        "value" => value = Some(v),
                        _ => {}
                    }
                }
                if let (Some(n), Some(v)) = (name, value) {
                    out.insert(n, v);
                }
            }
            Event::Eof => return Ok(out),
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::tests::app;
    use serde_json::json;

    fn input(password: &str) -> GatewayInput {
        GatewayInput {
            username: "1001".into(),
            password: password.into(),
            realm: "sip.example.com".into(),
            proxy: None,
            register: true,
            params: BTreeMap::from([("from-domain".into(), "example.com".into())]),
        }
    }

    #[test]
    fn xml_round_trips_and_escapes() {
        let xml = render("trunk1", &input(r#"p"<&>'w"#)).unwrap();
        assert!(xml.contains("&quot;&lt;&amp;&gt;"), "{xml}");
        let params = parse(&xml).unwrap();
        assert_eq!(params["password"], r#"p"<&>'w"#);
        assert_eq!(params["register"], "true");
        assert_eq!(params["from-domain"], "example.com");
        assert!(render("t", &input("a\nb")).is_err());
    }

    #[tokio::test]
    async fn put_writes_file_and_reports_freeswitch_down() {
        let app = app().await;
        let body = json!({"username": "1001", "password": "pw", "realm": "sip.example.com"});
        let (s, _) = app.call("PUT", "/api/v1/gateways/trunk1", true, Some(body.clone())).await;
        assert_eq!(s, StatusCode::SERVICE_UNAVAILABLE, "saved, but FreeSWITCH is down in tests");
        assert!(app.dir.join("gw/trunk1.xml").exists());

        let (s, gw) = app.call("GET", "/api/v1/gateways/trunk1", true, None).await;
        assert_eq!((s, gw["params"]["password"].as_str()), (StatusCode::OK, Some(REDACTED)));
        let (s, list) = app.call("GET", "/api/v1/gateways", true, None).await;
        assert_eq!((s, list), (StatusCode::OK, json!(["trunk1"])));

        assert_eq!(app.call("PUT", "/api/v1/gateways/..%2Fevil", true, Some(body)).await.0, StatusCode::BAD_REQUEST);
        assert_eq!(app.call("GET", "/api/v1/gateways/missing", true, None).await.0, StatusCode::NOT_FOUND);
    }
}
