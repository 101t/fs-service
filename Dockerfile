# syntax=docker/dockerfile:1.7
FROM rust:1-slim-trixie AS build
WORKDIR /src
COPY Cargo.toml Cargo.lock ./
COPY src src
COPY migrations migrations
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/src/target \
    cargo build --release --locked && cp target/release/fs-service /fs-service

FROM debian:trixie-slim
# Converters for fax: LibreOffice (office docs), Ghostscript (PDF -> TIFF G4),
# ImageMagick (images), libtiff tools (TIFF -> PDF previews).
# FreeSWITCH (another user, another container) writes received faxes to inbox/, hence 1777.
RUN apt-get update && apt-get install -y --no-install-recommends \
        libreoffice-writer-nogui libreoffice-calc-nogui libreoffice-impress-nogui \
        ghostscript imagemagick libtiff-tools fonts-dejavu-core ca-certificates \
    && rm -rf /var/lib/apt/lists/* \
    && useradd --system --uid 10001 --home /var/lib/fs-service fs-service \
    && mkdir -p /var/lib/fs-service/outbox /var/lib/fs-service/inbox /etc/freeswitch/sip_profiles/external \
    && chown -R fs-service /var/lib/fs-service /etc/freeswitch/sip_profiles/external \
    && chmod 1777 /var/lib/fs-service/inbox
COPY --from=build /fs-service /usr/local/bin/fs-service
USER fs-service
ENV FSS_DATABASE_URL=sqlite:///var/lib/fs-service/fs-service.db \
    FSS_LOG_JSON=true
EXPOSE 8080
HEALTHCHECK --interval=30s --timeout=3s CMD ["bash", "-c", "exec 3<>/dev/tcp/127.0.0.1/8080"]
ENTRYPOINT ["/usr/local/bin/fs-service"]
