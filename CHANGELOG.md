# Changelog

All notable changes are documented here. The format follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/); versions follow [SemVer](https://semver.org).

## [2.0.0] - 2026-11-20

Complete rewrite in Rust. See [docs/migration-from-v1.md](docs/migration-from-v1.md).

### Added
- Native async FreeSWITCH event-socket client (pipelined, auto-reconnecting) replacing the SWIG C++ binding.
- Built-in CDR recorder (event socket + optional mod_json_cdr ingest) replacing cdr-pusher/PostgreSQL.
- `GET /api/v1/cdrs` call-log search; per-batch delivery reports with failure reasons.
- Rate-limited dialer (`FSS_DIAL_RATE`).
- Live gateway registration state; extra Sofia params on gateways.
- PDF/TIFF/image conversion via Ghostscript (fax-spec 1728 px G4), cached and atomic.
- OpenAPI 3.1 spec and interactive docs at `/docs`; `/health`.
- Docker image, Docker Compose stack with FreeSWITCH, hardened systemd unit, CI.

### Changed
- REST API under `/api/v1` with proper status codes and a single error shape.
- Configuration via `FSS_*` environment variables / CLI flags.

### Security
- No default secrets; tokens ≥ 32 chars compared in constant time.
- Strict validation of numbers, names and files; no shell, no path traversal, no ESL command injection.
- Gateway passwords never returned.

### Removed
- Django admin, user accounts, unused IVR builder, Python 2 runtime.
