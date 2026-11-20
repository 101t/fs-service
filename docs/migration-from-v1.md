# Migrating from v1 (Django)

v2 is a rewrite. The HTTP API is now REST-shaped and versioned under `/api/v1`, so clients need
updating; FreeSWITCH-side files are compatible.

## What went away

| v1 | v2 |
|---|---|
| Python 2 / Django / DRF | One Rust binary |
| SWIG C++ `ESL` module (`_ESL.so`) | Native async event-socket client |
| cdr-pusher + mod_cdr_sqlite + PostgreSQL | Built-in CDR recorder → SQLite |
| Django users + DRF tokens + admin | Static API tokens (`FSS_API_TOKENS`) |
| Hard-coded paths in `vars.py` | `FSS_*` environment variables |
| `{"status": "OK", "code": 200, "body": …}` envelopes | Plain JSON + real HTTP status codes |
| Unused IVR builder | Removed |

## Endpoint mapping

| v1 | v2 |
|---|---|
| `POST /api/gateway/operation/` `{"service":"new"\|"edit", …}` | `PUT /api/v1/gateways/{username}` |
| `POST /api/gateway/operation/` `{"service":"delete", …}` | `DELETE /api/v1/gateways/{username}` |
| `POST /api/gateway/check/` | `GET /api/v1/gateways/{name}` (password no longer returned) |
| `POST /api/service/file/upload/` field `filename` | `POST /api/v1/files` field **`file`** |
| `POST /api/service/file/download/` `{"filename","format","directory"}` | `GET /api/v1/files/{name}?format=pdf\|tiff&folder=outbox\|inbox` |
| `POST /api/service/voice/originate/` `{"username","filename","numbers":"a,b"}` | `POST /api/v1/calls/voice` `{"gateway","file","numbers":["a","b"]}` |
| `POST /api/service/fax/originate/` `{…, "is_enhanced": true}` | `POST /api/v1/calls/fax` `{…, "quality": "fine"}` |
| `POST /api/service/fax/report/` `{"uuid"}` | `GET /api/v1/batches/{batch_id}` |
| `POST /api/service/fax/inbox/` `{"start_date","end_date","receivers"}` | `GET /api/v1/fax/inbox?from=…&to=…&receivers=a,b` |

Notes:

* `username` (gateway name) is now `gateway`; numbers are a JSON array, not a comma string.
* Dates are RFC 3339 (`2026-11-01T00:00:00Z`) instead of `"%Y-%m-%d %H:%M"`.
* Fax reports now count a fax delivered only when spandsp confirms it (`fax_success`),
  not when `duration > 0`, and include per-call reasons.
* `Authorization: Token …` becomes `Authorization: Bearer …`.

## Steps

1. Generate tokens (`openssl rand -hex 32`) and a new ESL password (`FSS_ESL_PASSWORD`); install
   the [event socket config](../contrib/freeswitch/autoload_configs/event_socket.conf.xml) that
   reads it (see [freeswitch.md](freeswitch.md#1-event-socket-required)).
2. Point `FSS_GATEWAY_DIR` at the old gateway directory: existing gateway XML files keep working.
3. Point `FSS_INBOX_DIR` at the old `RXFAX_DIR`: old `rxfax-<uuid>.tiff` files stay downloadable
   (they appear in the inbox list only for calls recorded by v2).
4. Stop cdr-pusher. Old CDRs stay in its PostgreSQL database; v2 starts a fresh log.
5. Update API clients using the table above. `/docs` has the full schema.
