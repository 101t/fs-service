//! Turns uploaded documents into fax-ready TIFF (and anything into PDF for previews).
//!
//! Pipelines, all run without a shell, under a timeout and a concurrency limit:
//! * office / text / html  → LibreOffice → PDF → Ghostscript (`tiffg4`) → TIFF
//! * PDF                   → Ghostscript → TIFF
//! * images (jpg, png, …)  → ImageMagick → TIFF
//!
//! Outputs are cached next to the source (`<id>.fine.fax.tiff`, `<id>.pdf`) and written
//! via a temporary name + rename, so a half-written file is never served or faxed.

use std::{
    ffi::OsString,
    path::{Path, PathBuf},
    process::Stdio,
    time::Duration,
};

use serde::Deserialize;
use tokio::{process::Command, sync::Semaphore, time::timeout};
use utoipa::ToSchema;

pub const OFFICE: &[&str] =
    &["doc", "docx", "odt", "rtf", "txt", "html", "htm", "xls", "xlsx", "ods", "csv", "ppt", "pptx", "odp"];
pub const IMAGE: &[&str] = &["jpg", "jpeg", "png", "gif", "bmp", "tif", "tiff"];
pub const AUDIO: &[&str] = &["wav", "mp3"];

/// Fax resolution. `fine` = 204×196 dpi (sharper, larger), `normal` = 204×98 dpi.
#[derive(Debug, Clone, Copy, Default, Deserialize, ToSchema, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum FaxQuality {
    Normal,
    #[default]
    Fine,
}

impl FaxQuality {
    /// (name, vertical dpi, page height in pixels for A4 at 1728 px width)
    fn spec(self) -> (&'static str, u32, u32) {
        match self {
            Self::Fine => ("fine", 196, 2292),
            Self::Normal => ("normal", 98, 1146),
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ConvertError {
    #[error("file type `.{0}` cannot be converted")]
    Unsupported(String),
    #[error("conversion took too long")]
    Timeout,
    #[error("`{0}` is not installed or cannot run: {1}")]
    Spawn(&'static str, std::io::Error),
    #[error("`{0}` failed: {1}")]
    Failed(&'static str, String),
    #[error(transparent)]
    Io(#[from] std::io::Error),
}

pub fn extension(path: &Path) -> String {
    path.extension().and_then(|e| e.to_str()).unwrap_or_default().to_ascii_lowercase()
}

pub fn is_document(ext: &str) -> bool {
    ext == "pdf" || OFFICE.contains(&ext) || IMAGE.contains(&ext)
}

pub struct Converter {
    permits: Semaphore,
    timeout: Duration,
    magick: String,
}

impl Converter {
    pub fn new(concurrency: usize, timeout: Duration, magick: String) -> Self {
        Self { permits: Semaphore::new(concurrency), timeout, magick }
    }

    /// Fax-ready, multi-page, 1728 px wide, 1-bit TIFF.
    pub async fn to_fax_tiff(&self, src: &Path, quality: FaxQuality) -> Result<PathBuf, ConvertError> {
        let ext = extension(src);
        if !is_document(&ext) {
            return Err(ConvertError::Unsupported(ext));
        }
        let (name, dpi, height) = quality.spec();
        let out = src.with_extension(format!("{name}.fax.tiff"));
        if out.exists() {
            return Ok(out);
        }
        let _permit = self.permits.acquire().await.expect("semaphore never closes");
        let tmp = temp_path(&out);
        if IMAGE.contains(&ext.as_str()) {
            let size = format!("1728x{height}");
            let mut target = OsString::from("tiff:");
            target.push(&tmp);
            self.run(
                "imagemagick",
                &self.magick,
                [
                    src.as_os_str().into(),
                    "-background".into(),
                    "white".into(),
                    "-alpha".into(),
                    "remove".into(),
                    "-resize".into(),
                    size.clone().into(),
                    "-gravity".into(),
                    "center".into(),
                    "-extent".into(),
                    size.into(),
                    "-monochrome".into(),
                    "-units".into(),
                    "PixelsPerInch".into(),
                    "-density".into(),
                    format!("204x{dpi}").into(),
                    "-compress".into(),
                    "Group4".into(),
                    target,
                ],
            )
            .await?;
        } else {
            let pdf = self.pdf_unlocked(src).await?;
            let mut output = OsString::from("-sOutputFile=");
            output.push(&tmp);
            self.run(
                "ghostscript",
                "gs",
                [
                    "-q".into(),
                    "-dSAFER".into(),
                    "-dBATCH".into(),
                    "-dNOPAUSE".into(),
                    "-sDEVICE=tiffg4".into(),
                    format!("-r204x{dpi}").into(),
                    format!("-g1728x{height}").into(),
                    "-dFIXEDMEDIA".into(),
                    "-dPDFFitPage".into(),
                    output,
                    pdf.into_os_string(),
                ],
            )
            .await?;
        }
        finish(&tmp, &out).await
    }

    /// PDF rendition of any supported document or image.
    pub async fn to_pdf(&self, src: &Path) -> Result<PathBuf, ConvertError> {
        let ext = extension(src);
        if !is_document(&ext) {
            return Err(ConvertError::Unsupported(ext));
        }
        if ext == "pdf" || src.with_extension("pdf").exists() {
            return Ok(src.with_extension("pdf"));
        }
        let _permit = self.permits.acquire().await.expect("semaphore never closes");
        self.pdf_unlocked(src).await
    }

    /// Callers must hold a permit.
    async fn pdf_unlocked(&self, src: &Path) -> Result<PathBuf, ConvertError> {
        let out = src.with_extension("pdf");
        if out.exists() {
            return Ok(out);
        }
        if matches!(extension(src).as_str(), "tif" | "tiff") {
            let tmp = temp_path(&out);
            self.run("tiff2pdf", "tiff2pdf", ["-o".into(), tmp.clone().into_os_string(), src.as_os_str().into()])
                .await?;
            return finish(&tmp, &out).await;
        }
        // LibreOffice picks the output name itself, so convert into a private directory.
        // A private profile per run lets several conversions run in parallel.
        let work = temp_path(&out);
        tokio::fs::create_dir_all(&work).await?;
        let mut profile = OsString::from("-env:UserInstallation=file://");
        profile.push(work.join("profile"));
        let result = self
            .run(
                "libreoffice",
                "soffice",
                [
                    "--headless".into(),
                    "--norestore".into(),
                    "--nologo".into(),
                    profile,
                    "--convert-to".into(),
                    "pdf".into(),
                    "--outdir".into(),
                    work.clone().into_os_string(),
                    src.as_os_str().into(),
                ],
            )
            .await;
        let produced = work.join(src.with_extension("pdf").file_name().unwrap_or_default());
        let result = match result {
            Ok(()) if produced.exists() => finish(&produced, &out).await,
            Ok(()) => Err(ConvertError::Failed("libreoffice", "no PDF was produced".into())),
            Err(e) => Err(e),
        };
        let _ = tokio::fs::remove_dir_all(&work).await;
        result
    }

    async fn run<const N: usize>(
        &self,
        label: &'static str,
        program: &str,
        args: [OsString; N],
    ) -> Result<(), ConvertError> {
        let child = Command::new(program)
            .args(args)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::piped())
            .kill_on_drop(true)
            .output();
        let out = timeout(self.timeout, child)
            .await
            .map_err(|_| ConvertError::Timeout)?
            .map_err(|e| ConvertError::Spawn(label, e))?;
        if out.status.success() {
            return Ok(());
        }
        let stderr = String::from_utf8_lossy(&out.stderr);
        let tail: String = stderr.lines().rev().take(3).collect::<Vec<_>>().join(" | ");
        Err(ConvertError::Failed(label, tail))
    }
}

/// Hidden sibling name: the file API refuses names starting with `.`, so it is never served.
fn temp_path(out: &Path) -> PathBuf {
    let name = out.file_name().unwrap_or_default().to_string_lossy();
    out.with_file_name(format!(".tmp-{}-{name}", uuid::Uuid::now_v7()))
}

async fn finish(tmp: &Path, out: &Path) -> Result<PathBuf, ConvertError> {
    match tokio::fs::rename(tmp, out).await {
        Ok(()) => Ok(out.to_path_buf()),
        Err(e) => {
            let _ = tokio::fs::remove_file(tmp).await;
            Err(e.into())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classifies_extensions() {
        assert_eq!(extension(Path::new("/x/A.DOCX")), "docx");
        assert!(is_document("pdf") && is_document("png") && !is_document("wav") && !is_document("exe"));
    }

    #[tokio::test]
    async fn rejects_unsupported_and_reports_missing_tools() {
        let c = Converter::new(1, Duration::from_secs(5), "definitely-not-installed-magick".into());
        assert!(matches!(
            c.to_fax_tiff(Path::new("/tmp/a.exe"), FaxQuality::Fine).await,
            Err(ConvertError::Unsupported(_))
        ));
        let dir = std::env::temp_dir().join(format!("fss-test-{}", uuid::Uuid::now_v7()));
        std::fs::create_dir_all(&dir).unwrap();
        let img = dir.join("a.png");
        std::fs::write(&img, b"not really a png").unwrap();
        let err = c.to_fax_tiff(&img, FaxQuality::Normal).await.unwrap_err();
        assert!(matches!(err, ConvertError::Spawn("imagemagick", _)), "{err}");
        std::fs::remove_dir_all(dir).unwrap();
    }
}
