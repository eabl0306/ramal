# syntax=docker/dockerfile:1
#
# Imagen para compilar y probar el lado nativo de Git Rudder Desktop (crate
# git-rudder-desktop) en maquinas sin webkit2gtk. No es una imagen de produccion: es
# el entorno de Linux que Tauri 2 necesita, con las mismas versiones que fija el
# repo, para correr dentro las recetas desktop-rust-check, desktop-build y
# desktop-e2e-native (ver las recetas desktop-docker-* del justfile).
#
#   just desktop-docker-image
#
# La base es Ubuntu 22.04 a proposito: es la Linux soportada mas antigua (ADR
# 0016), la que fija el minimo de glibc de los builds de Linux, y trae git 2.34,
# la version minima que admite el motor (ADR 0009).
#
# El contexto de build es esta carpeta y no la raiz: la imagen no copia el repo,
# lo monta la receta al ejecutar.
FROM ubuntu:22.04

ARG DEBIAN_FRONTEND=noninteractive
# Las tres versiones van a la par que el repo: rust-toolchain.toml, .nvmrc y el
# `packageManager` de package.json. Se suben juntas.
ARG RUST_VERSION=1.98.1
ARG RUSTUP_VERSION=1.29.1
ARG NODE_VERSION=24.21.0
ARG PNPM_VERSION=11.18.0
ARG JUST_VERSION=1.58.0
ARG CARGO_LLVM_COV_VERSION=0.9.0
ARG TARGETARCH

# Requisitos de Tauri 2 en Linux (guia oficial de prerrequisitos, pestana
# Debian), mas lo que piden las pruebas:
#   - webkit2gtk-driver trae WebKitWebDriver, el driver externo de WebKitGTK. En
#     Ubuntu 26.04 el paquete se llama webkitgtk-webdriver.
#   - xvfb y xauth: servidor X virtual para la app real sin pantalla (xvfb-run).
#   - git: el motor lanza el git del sistema, y los tests crean repos con el.
#   - dbus-x11 y fuentes: WebKitGTK arranca mas limpio con un bus de sesion y
#     necesita alguna fuente de sistema para el texto que no es IBM Plex.
RUN apt-get update \
  && apt-get install -y --no-install-recommends \
    build-essential \
    ca-certificates \
    curl \
    dbus-x11 \
    file \
    fonts-dejavu-core \
    git \
    libayatana-appindicator3-dev \
    librsvg2-dev \
    libssl-dev \
    libwebkit2gtk-4.1-dev \
    libxdo-dev \
    pkg-config \
    webkit2gtk-driver \
    wget \
    xauth \
    xvfb \
    xz-utils \
  && rm -rf /var/lib/apt/lists/*

# Node desde nodejs.org con su suma SHA-256: los repos de Ubuntu 22.04 traen una
# version muy anterior. Corepack viene con Node 24 y fija pnpm al del repo.
ENV COREPACK_HOME=/opt/corepack \
  COREPACK_ENABLE_DOWNLOAD_PROMPT=0
RUN set -eux; \
  case "${TARGETARCH:-amd64}" in \
    amd64) node_arch=x64; node_sha=fd8e59d5a511510f6a298afb548f18c7d2b1be404d8b4a27d94fbe49f56cb2d6 ;; \
    arm64) node_arch=arm64; node_sha=6ad1325edbdb5649c379b75a237147a666c95d4f9ae8d340fef2d1575d289ad2 ;; \
    *) echo "arquitectura no soportada: ${TARGETARCH}" >&2; exit 1 ;; \
  esac; \
  tarball="node-v${NODE_VERSION}-linux-${node_arch}.tar.xz"; \
  curl -fsSL --retry 5 --retry-all-errors -o "/tmp/${tarball}" \
    "https://nodejs.org/dist/v${NODE_VERSION}/${tarball}"; \
  echo "${node_sha}  /tmp/${tarball}" | sha256sum -c -; \
  tar -xJf "/tmp/${tarball}" -C /usr/local --strip-components=1 --no-same-owner; \
  rm "/tmp/${tarball}"; \
  corepack enable; \
  corepack install -g "pnpm@${PNPM_VERSION}"; \
  chmod -R a+rwX "${COREPACK_HOME}"; \
  node --version; pnpm --version

# Rust con rustup, en la version exacta de rust-toolchain.toml y con sus
# componentes, para que rustup no tenga que descargar nada al ejecutar.
ENV RUSTUP_HOME=/usr/local/rustup \
  PATH=/usr/local/cargo/bin:${PATH}
RUN set -eux; \
  case "${TARGETARCH:-amd64}" in \
    amd64) rust_triple=x86_64-unknown-linux-gnu; rustup_sha=dda7234360b7f578ca8b0ddcb80145646fa61a67c1720a5abc7051b35c9fcb71 ;; \
    arm64) rust_triple=aarch64-unknown-linux-gnu; rustup_sha=15f6e4ce9f583b929c996c91562bad6d4454f3281de858b02cdfdef615fac433 ;; \
    *) echo "arquitectura no soportada: ${TARGETARCH}" >&2; exit 1 ;; \
  esac; \
  curl -fsSL --retry 5 --retry-all-errors -o /tmp/rustup-init \
    "https://static.rust-lang.org/rustup/archive/${RUSTUP_VERSION}/${rust_triple}/rustup-init"; \
  echo "${rustup_sha}  /tmp/rustup-init" | sha256sum -c -; \
  chmod +x /tmp/rustup-init; \
  CARGO_HOME=/usr/local/cargo /tmp/rustup-init -y --no-modify-path --profile minimal \
    --default-toolchain "${RUST_VERSION}" -c clippy -c rustfmt -c llvm-tools-preview; \
  rm /tmp/rustup-init; \
  chmod -R a+rX "${RUSTUP_HOME}" /usr/local/cargo; \
  rustc --version; cargo --version

# just, el punto de entrada de todas las recetas, con la suma de su release.
RUN set -eux; \
  case "${TARGETARCH:-amd64}" in \
    amd64) just_triple=x86_64-unknown-linux-musl; just_sha=4a5cc2f53e6f0f8c59092a6cc38291eb729d46a7dd95d3ae582008881b84931d ;; \
    arm64) just_triple=aarch64-unknown-linux-musl; just_sha=748237128c4c40cbdabc65e841d05ceba13cc23a91eaba395495894c1d9764df ;; \
    *) echo "arquitectura no soportada: ${TARGETARCH}" >&2; exit 1 ;; \
  esac; \
  tarball="just-${JUST_VERSION}-${just_triple}.tar.gz"; \
  curl -fsSL --retry 5 --retry-all-errors -o "/tmp/${tarball}" \
    "https://github.com/casey/just/releases/download/${JUST_VERSION}/${tarball}"; \
  echo "${just_sha}  /tmp/${tarball}" | sha256sum -c -; \
  tar -xzf "/tmp/${tarball}" -C /usr/local/bin just; \
  rm "/tmp/${tarball}"; \
  just --version

# cargo-llvm-cov, con la suma de su release: desktop-rust-check mide la cobertura
# de git-rudder-desktop (60% global, ADR 0006). La misma version que instala la CI.
RUN set -eux; \
  case "${TARGETARCH:-amd64}" in \
    amd64) cov_triple=x86_64-unknown-linux-musl; cov_sha=bc2122fd7bd9b6d3eb371690ed085f82e5a8bf1851f4645ef521d12ed04b2616 ;; \
    arm64) cov_triple=aarch64-unknown-linux-musl; cov_sha=3c299780e109d59fd77044e64734421d43c0067b92937a49be302fee66d04727 ;; \
    *) echo "arquitectura no soportada: ${TARGETARCH}" >&2; exit 1 ;; \
  esac; \
  tarball="cargo-llvm-cov-${cov_triple}.tar.gz"; \
  curl -fsSL --retry 5 --retry-all-errors -o "/tmp/${tarball}" \
    "https://github.com/taiki-e/cargo-llvm-cov/releases/download/v${CARGO_LLVM_COV_VERSION}/${tarball}"; \
  echo "${cov_sha}  /tmp/${tarball}" | sha256sum -c -; \
  tar -xzf "/tmp/${tarball}" -C /usr/local/bin cargo-llvm-cov; \
  rm "/tmp/${tarball}"; \
  cargo llvm-cov --version

# La receta ejecuta el contenedor con el uid del usuario de la maquina, para que
# lo que escribe en el repo montado (dist/, los esquemas de Tauri) no quede de
# root. Por eso los volumenes de cargo (registro y target, separados del target/
# de la maquina) nacen escribibles por cualquiera: un volumen nuevo copia los
# permisos de su punto de montaje en la imagen.
ENV CARGO_HOME=/cargo-home \
  CARGO_TARGET_DIR=/cargo-target \
  HOME=/tmp/home
RUN mkdir -p /cargo-home /cargo-target /tmp/home \
  && chmod 1777 /cargo-home /cargo-target /tmp/home
