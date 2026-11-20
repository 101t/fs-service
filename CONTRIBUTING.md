# Contributing

Thanks for helping! Bug reports, FreeSWITCH-version compatibility notes and carrier fax tips are
as welcome as code.

## Setup

```bash
rustup toolchain install stable      # 1.85+
cargo test                           # no FreeSWITCH needed: tests use a fake event socket
```

To try against a real switch: `docker run -d --name fs -p 127.0.0.1:8021:8021 safarov/freeswitch`
(adjust its `event_socket.conf.xml` as in [docs/freeswitch.md](docs/freeswitch.md)), then
`cargo run -- --api-tokens $(openssl rand -hex 32) --esl-password <pw>` and open
http://localhost:8080/docs.

## Before you open a PR

```bash
cargo fmt
cargo clippy --all-targets -- -D warnings
cargo test
```

* New behaviour comes with a test. API changes come with updated `#[utoipa::path]` docs — the
  OpenAPI spec is generated from them.
* Anything that reaches a dial string, a file path or the event socket must go through the
  validators in `src/api/mod.rs`.
* Schema changes are a new file in `migrations/`; never edit a released migration.
* Keep dependencies few; explain any new one in the PR.
* Update [CHANGELOG.md](CHANGELOG.md).

Commits follow [Conventional Commits](https://www.conventionalcommits.org) (`feat:`, `fix:`, `docs:` …).
