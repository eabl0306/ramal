# syntax=docker/dockerfile:1

# Imagen de PRODUCCION de la API. El contexto de build es la RAIZ del repo,
# porque el workspace de Cargo y su Cargo.lock viven alli:
#
#   docker build -f apps/api/Dockerfile -t ramal-api:local .     (just api-docker)
#
# Que entra en el contexto lo decide apps/api/Dockerfile.dockerignore (BuildKit
# lo prefiere al .dockerignore de la raiz): solo el workspace de Rust.
#
# cargo-chef compila las dependencias en una capa propia, que solo se rehace
# cuando cambia algun Cargo.toml o el Cargo.lock. Cargo necesita el manifiesto y
# el codigo de todos los miembros del workspace para resolverlo, aunque solo se
# compile ramal-api: por eso el contexto lleva tambien crates/, el sidecar de
# VS Code y el crate de escritorio, que nunca se compilan aqui.

# Va a la par que rust-toolchain.toml: se suben juntos.
ARG RUST_VERSION=1.98.1
ARG CARGO_CHEF_VERSION=0.1.78

FROM rust:${RUST_VERSION}-slim-trixie AS chef
ARG CARGO_CHEF_VERSION
# Hilos de compilacion. "default" son todos los nucleos; en una maquina
# compartida `just api-docker` pasa el CARGO_BUILD_JOBS del entorno. Un ARG llega
# como variable de entorno a cada RUN, pero solo en la etapa que lo declara.
ARG CARGO_BUILD_JOBS=default
RUN apt-get update \
 && apt-get install --no-install-recommends --yes build-essential pkg-config \
 && rm -rf /var/lib/apt/lists/*
RUN cargo install cargo-chef --locked --version ${CARGO_CHEF_VERSION}
WORKDIR /app

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
ARG CARGO_BUILD_JOBS=default
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --locked --package ramal-api --recipe-path recipe.json
COPY . .
RUN cargo build --release --locked --package ramal-api --bin ramal-api

FROM debian:trixie-slim AS runtime
# curl solo para el HEALTHCHECK. El uid fijo (10001) deja dar permisos desde
# fuera sin adivinar que usuario crea la imagen.
RUN apt-get update \
 && apt-get install --no-install-recommends --yes ca-certificates curl \
 && rm -rf /var/lib/apt/lists/* \
 && groupadd --system --gid 10001 ramal \
 && useradd --system --uid 10001 --gid ramal --no-create-home ramal

COPY --from=builder /app/target/release/ramal-api /usr/local/bin/ramal-api

USER ramal:ramal
WORKDIR /srv

# Valores por defecto de la imagen; el resto de RAMAL__* sale del entorno del
# despliegue (.env.example es la lista completa).
ENV RAMAL__SERVER__HOST=0.0.0.0 \
    RAMAL__SERVER__PORT=8080 \
    RAMAL__TELEMETRY__LOG_LEVEL=info

EXPOSE 8080

# Liveness y no readiness: si la base cae, reiniciar el contenedor no la
# arregla. La disponibilidad la mira el orquestador en /health/ready.
HEALTHCHECK --interval=10s --timeout=3s --start-period=5s --retries=3 \
  CMD curl --fail --silent --show-error "http://127.0.0.1:${RAMAL__SERVER__PORT}/health/live" || exit 1

ENTRYPOINT ["/usr/local/bin/ramal-api"]
