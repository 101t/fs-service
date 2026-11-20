# Security policy

fs-service can place calls and reconfigure a telephony switch, so we treat security reports as
top priority.

## Reporting

Please **do not open a public issue**. Use GitHub's
[private vulnerability reporting](https://github.com/101t/fs-service/security/advisories/new).
Include the version, configuration (secrets removed) and steps to reproduce. You will get an
answer within a week.

## Supported versions

Only the latest 2.x release receives fixes. v1 (Django) is unsupported.

## Deployment checklist

- [ ] `FSS_API_TOKENS` generated with `openssl rand -hex 32`, one per client, rotated by listing old and new together.
- [ ] Event socket bound to `127.0.0.1` (or a private network) with a strong, non-default password.
- [ ] fs-service behind a TLS reverse proxy if reachable beyond localhost; restrict by IP where possible.
- [ ] fs-service runs as an unprivileged user (Docker image: uid 10001; systemd unit: `freeswitch`).
- [ ] mod_json_cdr `cred` uses its own token.
- [ ] Rate limits on the proxy for `/api/v1/calls/*` — every request can cost you money (toll fraud).
