//! Upload documents/audio to the outbox; download originals, conversions and received faxes.

use axum::{
    Json,
    body::Body,
    extract::{Multipart, Path, Query, State},
    http::{StatusCode, header},
    response::Response,
};
use serde::{Deserialize, Serialize};
use tokio::io::AsyncWriteExt;
use utoipa::{IntoParams, ToSchema};

use super::{ApiError, ErrorBody, Shared, require, valid_name};
use crate::convert::{AUDIO, FaxQuality, extension, is_document};

#[derive(Serialize, ToSchema)]
pub struct Uploaded {
    /// Name to use in call requests and downloads.
    #[schema(example = "0193a5c2-7e1f-7b3a-9c41-5f2d8e6a1b20.pdf")]
    pub name: String,
    pub size: u64,
    /// `document` (faxable) or `audio` (playable).
    pub kind: &'static str,
}

#[allow(dead_code)] // documents the multipart body for OpenAPI
#[derive(ToSchema)]
struct UploadForm {
    /// pdf, doc(x), odt, rtf, txt, html, xls(x), ods, csv, ppt(x), odp, jpg, png, gif, bmp, tif(f), wav, mp3
    #[schema(format = Binary, content_media_type = "application/octet-stream")]
    file: String,
}

/// Upload a document (to fax) or an audio file (to play).
#[utoipa::path(post, path = "/api/v1/files", tag = "files",
    request_body(content = UploadForm, content_type = "multipart/form-data"),
    responses((status = CREATED, body = Uploaded), (status = BAD_REQUEST, body = ErrorBody), (status = PAYLOAD_TOO_LARGE)))]
pub async fn upload(State(s): State<Shared>, mut form: Multipart) -> Result<(StatusCode, Json<Uploaded>), ApiError> {
    let bad = |e: axum::extract::multipart::MultipartError| ApiError::BadRequest(e.body_text());
    while let Some(mut field) = form.next_field().await.map_err(bad)? {
        if field.name() != Some("file") {
            continue;
        }
        let ext = extension(std::path::Path::new(field.file_name().unwrap_or_default()));
        let kind = if is_document(&ext) {
            "document"
        } else if AUDIO.contains(&ext.as_str()) {
            "audio"
        } else {
            return Err(ApiError::BadRequest(format!("file type `.{ext}` is not accepted")));
        };
        // The client's file name is never used on disk: only its (allow-listed) extension.
        let name = format!("{}.{ext}", uuid::Uuid::now_v7());
        let path = s.cfg.outbox_dir.join(&name);
        let tmp = s.cfg.outbox_dir.join(format!(".upload-{name}"));
        let mut out = tokio::fs::File::create(&tmp).await?;
        let mut size = 0u64;
        let written: Result<(), ApiError> = async {
            while let Some(chunk) = field.chunk().await.map_err(bad)? {
                size += chunk.len() as u64;
                out.write_all(&chunk).await?;
            }
            out.flush().await?;
            Ok(())
        }
        .await;
        if let Err(e) = written {
            let _ = tokio::fs::remove_file(&tmp).await;
            return Err(e);
        }
        if size == 0 {
            let _ = tokio::fs::remove_file(&tmp).await;
            return Err(ApiError::BadRequest("file is empty".into()));
        }
        tokio::fs::rename(&tmp, &path).await?;
        tracing::info!(file = name, size, "file uploaded");
        return Ok((StatusCode::CREATED, Json(Uploaded { name, size, kind })));
    }
    Err(ApiError::BadRequest("multipart field `file` is required".into()))
}

#[derive(Deserialize, ToSchema, Default, Clone, Copy, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Folder {
    /// Uploaded files and their conversions.
    #[default]
    Outbox,
    /// Faxes received by FreeSWITCH.
    Inbox,
}

#[derive(Deserialize, ToSchema, Default, Clone, Copy, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Format {
    /// The file as stored.
    #[default]
    Original,
    /// PDF rendition (great for previewing a received fax).
    Pdf,
    /// The exact fax TIFF that would be transmitted.
    Tiff,
}

#[derive(Deserialize, IntoParams)]
pub struct DownloadQuery {
    #[serde(default)]
    pub folder: Folder,
    #[serde(default)]
    pub format: Format,
    /// Only for `format=tiff`.
    #[serde(default)]
    pub quality: FaxQuality,
}

/// Download a file, optionally converted to PDF or fax TIFF.
#[utoipa::path(get, path = "/api/v1/files/{name}", tag = "files",
    params(("name" = String, Path, description = "File name returned by upload, or a received fax name"), DownloadQuery),
    responses(
        (status = OK, content_type = "application/octet-stream", body = Vec<u8>),
        (status = NOT_FOUND, body = ErrorBody),
        (status = UNPROCESSABLE_ENTITY, body = ErrorBody, description = "Conversion failed")))]
pub async fn download(
    State(s): State<Shared>,
    Path(name): Path<String>,
    Query(q): Query<DownloadQuery>,
) -> Result<Response, ApiError> {
    require(valid_name(&name, 128), "invalid file name")?;
    let dir = if q.folder == Folder::Inbox { &s.cfg.inbox_dir } else { &s.cfg.outbox_dir };
    let src = dir.join(&name);
    if !tokio::fs::try_exists(&src).await? {
        return Err(ApiError::NotFound(format!("file `{name}` not found")));
    }
    let path = match q.format {
        Format::Original => src,
        Format::Pdf => s.converter.to_pdf(&src).await?,
        Format::Tiff => s.converter.to_fax_tiff(&src, q.quality).await?,
    };
    let mime = match extension(&path).as_str() {
        "pdf" => "application/pdf",
        "tif" | "tiff" => "image/tiff",
        "wav" => "audio/wav",
        "mp3" => "audio/mpeg",
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        _ => "application/octet-stream",
    };
    let file_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("download");
    // shortcut: buffers the whole file; fine under the upload cap, stream it if files grow past ~100 MB.
    let bytes = tokio::fs::read(&path).await?;
    Ok(Response::builder()
        .header(header::CONTENT_TYPE, mime)
        .header(header::CONTENT_DISPOSITION, format!("attachment; filename=\"{file_name}\""))
        .body(Body::from(bytes))
        .expect("static headers are valid"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::tests::{TOKEN, app};
    use axum::extract::Request;

    fn multipart(filename: &str, content: &[u8]) -> Request {
        let mut body = format!(
            "--XX\r\nContent-Disposition: form-data; name=\"file\"; filename=\"{filename}\"\r\nContent-Type: application/octet-stream\r\n\r\n"
        )
        .into_bytes();
        body.extend_from_slice(content);
        body.extend_from_slice(b"\r\n--XX--\r\n");
        Request::post("/api/v1/files")
            .header(header::AUTHORIZATION, format!("Bearer {TOKEN}"))
            .header(header::CONTENT_TYPE, "multipart/form-data; boundary=XX")
            .body(Body::from(body))
            .unwrap()
    }

    #[tokio::test]
    async fn upload_then_download() {
        let app = app().await;
        let (s, up) = app.send(multipart("../../etc/Report.PDF", b"%PDF-1.4 fake")).await;
        assert_eq!(s, StatusCode::CREATED);
        let name = up["name"].as_str().unwrap();
        assert!(name.ends_with(".pdf") && !name.contains('/'), "{name}");
        assert_eq!(up["kind"], "document");

        let res = app.router.clone();
        let req = Request::get(format!("/api/v1/files/{name}"))
            .header(header::AUTHORIZATION, format!("Bearer {TOKEN}"))
            .body(Body::empty())
            .unwrap();
        let resp = tower::ServiceExt::oneshot(res, req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        assert_eq!(resp.headers()[header::CONTENT_TYPE], "application/pdf");

        assert_eq!(app.send(multipart("evil.exe", b"MZ")).await.0, StatusCode::BAD_REQUEST);
        assert_eq!(app.send(multipart("empty.wav", b"")).await.0, StatusCode::BAD_REQUEST);
        assert_eq!(app.call("GET", "/api/v1/files/.upload-x.pdf", true, None).await.0, StatusCode::BAD_REQUEST);
        assert_eq!(app.call("GET", "/api/v1/files/nope.pdf?folder=inbox", true, None).await.0, StatusCode::NOT_FOUND);
    }
}
