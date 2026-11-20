# FreeSWITCH setup

fs-service works with **FreeSWITCH 1.10.x and 1.11.x** (tested live against 1.11.3). It needs
three things from the switch; ready-made files are in [`contrib/freeswitch/`](../contrib/freeswitch).

## 1. Event socket (required)

[`event_socket.conf.xml`](../contrib/freeswitch/autoload_configs/event_socket.conf.xml) → `autoload_configs/`

```xml
<X-PRE-PROCESS cmd="env-set" data="fss_esl_password=$FSS_ESL_PASSWORD"/>
<param name="listen-ip" value="127.0.0.1"/>
<param name="password" value="$${fss_esl_password}"/>
<param name="apply-inbound-acl" value="loopback.auto"/>
```

The stock config listens on all interfaces with password `ClueCon`. Anyone who reaches that port
controls the switch. This one binds to loopback and reads the password from the
`FSS_ESL_PASSWORD` environment variable, the same one fs-service uses, so the password lives only
in `.env`:

* **Docker Compose:** nothing to do; compose passes the variable and refuses to start without it.
* **systemd:** `systemctl edit freeswitch` → `[Service]` `EnvironmentFile=/etc/fs-service.env`,
  or replace `$${fss_esl_password}` with the password itself.

> [!WARNING]
> If the variable is missing, the password becomes empty and mod_event_socket silently accepts
> `ClueCon`. fs-service then fails to log in (`auth rejected` in its log, `freeswitch_connected:
> false`), which is your signal. Check with `fs_cli -x status`: it must **fail** without `-p`.

Apply with `fs_cli -p "$FSS_ESL_PASSWORD" -x 'reload mod_event_socket'`, or restart FreeSWITCH.

Check: `curl localhost:8080/health` → `"freeswitch_connected": true`.

Using the safarov/freeswitch image yourself? Its built-in healthcheck runs `fs_cli` with the
default password and reports `unhealthy` once you change it; `docker-compose.yml` overrides it with
`fs_cli -p "$FSS_ESL_PASSWORD" -x status`.

## 2. Gateways directory (required for `/api/v1/gateways`)

fs-service writes one `<name>.xml` per gateway into `FSS_GATEWAY_DIR` and runs
`reloadxml` + `sofia profile <profile> rescan`. The stock `sip_profiles/external.xml` already
includes that directory:

```xml
<gateways>
  <X-PRE-PROCESS cmd="include" data="external/*.xml"/>
</gateways>
```

The fs-service user needs write access to that directory; FreeSWITCH needs read access.
Gateways you defined by hand elsewhere keep working and can still be dialed through the API —
they just are not listed or editable.

## 3. Shared files (required for fax and voice)

FreeSWITCH opens the files itself (`txfax`, `playback`, `rxfax`), so **`FSS_OUTBOX_DIR` and
`FSS_INBOX_DIR` must be the same paths on both sides**: same host, or the same mount point in
both containers (see [`docker-compose.yml`](../docker-compose.yml)).

* outbox: written by fs-service, read by FreeSWITCH.
* inbox: written by FreeSWITCH, read by fs-service.

Fax needs `mod_spandsp` (loaded by default). MP3 playback needs `mod_shout`.

## Receiving faxes

Install [`50_fax_inbound.xml`](../contrib/freeswitch/dialplan/public/50_fax_inbound.xml) in
`dialplan/public/`, put your DIDs in the regex, `reloadxml`. Received faxes then show up at
`GET /api/v1/fax/inbox`. The file must be named `rxfax-${uuid}.tiff` inside `FSS_INBOX_DIR`.

## Zero-loss CDRs (optional, recommended for billing)

The event-socket recorder misses calls that end while fs-service is down. To close that gap,
enable mod_json_cdr with [`json_cdr.conf.xml`](../contrib/freeswitch/autoload_configs/json_cdr.conf.xml):

1. Set `cred` to `freeswitch:<one of FSS_API_TOKENS>` and keep `encode` = `false`.
2. Add `<load module="mod_json_cdr"/>` to `modules.conf.xml`, then `load mod_json_cdr`.

FreeSWITCH retries failed posts and spools them to `err-log-dir`. Both sources write the same
table; duplicates are ignored.

## Fax tuning

Outbound faxes use `fax_enable_t38=true`, `fax_enable_t38_request=false`, `fax_use_ecm=false` and
`absolute_codec_string='PCMU,PCMA'` — the most compatible combination across carriers. If your
carrier handles T.38 re-INVITEs well, enabling `fax_enable_t38_request` improves success rates on
lossy links; it is set in `src/api/calls.rs`.

## Troubleshooting

| Symptom | Check |
|---|---|
| `freeswitch_connected: false` | `fs_cli -x 'event_socket status'`, password, ACL, `FSS_ESL_ADDR`. fs-service logs the exact reason. |
| Gateway `PUT` → 503 | Saved, but FreeSWITCH unreachable; retry the `PUT`. |
| Gateway `state` stays `FAIL_WAIT` | Wrong credentials/realm. `fs_cli -x 'sofia status gateway <name>'`. |
| Batch call failed with `INVALID_GATEWAY` | The gateway name does not exist in Sofia. |
| Fax fails with a `fax_result_text` | spandsp's verdict, e.g. `Far end cannot receive at the resolution of the image` → try `"quality": "normal"`. |
| `conversion_failed` | Converter missing or document broken; the message names the tool. |
