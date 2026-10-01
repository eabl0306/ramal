# Atajos del monorepo. `just` los lista con `just` a secas.
#
# Las recetas son finas: delegan en el toolchain de cada parte (cargo, pnpm,
# docker) en lugar de reimplementar nada. Si una receta y la CI no hacen lo
# mismo, manda la CI: mirala en .github/workflows.
#
# Este fichero es un contrato. Cada app y cada paquete implementa las mismas
# recetas y los mismos scripts de package.json, aunque hoy alguna receta apunte a
# algo que todavia no existe:
#
#   - Todo paquete y app de TypeScript tiene los scripts lint, typecheck,
#     test (vitest run) y test:coverage. Las apps ademas dev, build y e2e; la
#     extension de VS Code ademas test:integration y package.
#   - Nombres: @ramal/domain, @ramal/i18n, @ramal/ui, @ramal/repo-view,
#     @ramal/bridge, @ramal/api-client, @ramal/web, @ramal/desktop y "ramal"
#     (la extension). Las recetas los llaman con `pnpm --filter <nombre> <script>`.
#   - Rust: ninguna receta usa --workspace. Pisaria default-members, que deja
#     fuera el crate de escritorio porque necesita webkit2gtk.

set shell := ["bash", "-euo", "pipefail", "-c"]
set dotenv-load := true

# pnpm siempre por corepack, que respeta el `packageManager` de package.json. El
# pnpm global de una maquina puede ser otra version mayor, y pnpm 9 ignora en
# silencio claves de pnpm-workspace.yaml que pnpm 11 necesita (allowBuilds).
pnpm := env_var_or_default("PNPM", "corepack pnpm")
# --fail-if-no-match: sin el, un filtro que no casa con ningun paquete (un nombre
# mal escrito, una app renombrada) sale con 0 y la receta pasa en verde sin haber
# comprobado nada.
pnpm_filter := pnpm + " --fail-if-no-match --filter"
compose := "docker compose"
# La pila completa (api, web, gateway y mailpit) va en el perfil `full`. Sin
# perfil, compose solo conoce la base de datos, y `just db-up` funciona aunque
# las apps todavia no tengan Dockerfile.
compose_full := "docker compose --profile full"
pg_user := env_var_or_default("POSTGRES_USER", "ramal")
pg_db := env_var_or_default("POSTGRES_DB", "ramal")
database_url := "postgres://" + pg_user + ":${POSTGRES_PASSWORD:-ramal}@127.0.0.1:${POSTGRES_PORT:-5432}/" + pg_db

# Crates de cada area, para pasarlos a cargo con -p.
api_crates := "-p ramal-api-domain -p ramal-api-application -p ramal-api-infrastructure -p ramal-api-http -p ramal-api"
engine_crates := "-p ramal-core-domain -p ramal-core-application -p ramal-protocol -p ramal-git -p ramal-engine -p ramal-vscode-engine"
# Carpetas de los paquetes de TypeScript compartidos; cada una es @ramal/<carpeta>.
ts_package_dirs := "domain i18n ui repo-view bridge api-client"
protocol_out := "packages/bridge/src/generated"
# Imagen con los requisitos de Tauri en Linux (tools/docker/tauri-linux.Dockerfile).
tauri_image := "ramal-tauri-linux:local"

# Lista todas las recetas.
default:
    @just --list --unsorted

# ---------------------------------------------------------------- monorepo ---

# Instala dependencias, engancha los hooks de git y comprueba los toolchains.
[group('monorepo')]
setup:
    {{pnpm}} install --frozen-lockfile || {{pnpm}} install
    {{pnpm}} exec lefthook install
    @just doctor

# Existe porque el pnpm global de la maquina puede ser otro: con pnpm 9 el
# install parece funcionar, pero ignora claves de pnpm-workspace.yaml.
#
# El texto de `just --list` va en [doc]: just toma como descripcion la ultima
# linea del comentario, y aqui seria un fragmento sin sentido.
[group('monorepo')]
[doc('Muestra las versiones de las herramientas y falla si pnpm no es 11.18.x.')]
doctor:
    #!/usr/bin/env bash
    set -euo pipefail
    pnpm_version="$({{pnpm}} --version)"
    echo "node    $(node --version)"
    echo "pnpm    ${pnpm_version} (via: {{pnpm}})"
    echo "cargo   $(cargo --version)"
    echo "rustc   $(rustc --version)"
    echo "just    $(just --version)"
    echo "docker  $(docker --version 2>/dev/null || echo 'no instalado')"
    case "${pnpm_version}" in
      11.18.*) ;;
      *)
        echo "doctor: hace falta pnpm 11.18.x y hay ${pnpm_version}." >&2
        echo "doctor: usa corepack (corepack pnpm) o exporta PNPM con el pnpm correcto." >&2
        exit 1
        ;;
    esac

# Todo el enforcement del monorepo, igual que la CI pero en local.
[group('monorepo')]
check: guard root-lint lint-css fmt-check tools-test knip rust-lint rust-test api-check engine-check packages-check web-check desktop-check vscode-check

# Lo mismo que check mas lo que la CI corre aparte: auditoria, protocolo, e2e, builds y la pila.
[group('monorepo')]
ci: check rust-deny protocol-check api-client-check e2e build stack-smoke

# Formatea todo: prettier para JS, TS, CSS, JSON, YAML y Markdown; rustfmt para Rust.
[group('monorepo')]
fmt:
    {{pnpm}} run format
    cargo fmt --all

# Falla si algo no esta formateado.
[group('monorepo')]
fmt-check:
    {{pnpm}} run format:check
    cargo fmt --all --check

# Lint de todo: raiz, CSS, Rust, paquetes y apps.
[group('monorepo')]
lint: root-lint lint-css rust-lint packages-lint web-lint desktop-lint vscode-lint

# Tests unitarios de todo.
[group('monorepo')]
test: tools-test rust-test packages-test web-test desktop-test vscode-test

# Cobertura de todo, con sus umbrales.
[group('monorepo')]
cov: api-cov engine-cov packages-cov web-cov desktop-cov vscode-cov

# E2E: Playwright en web, escritorio y el webview de VS Code, integracion de VS Code y la API contra Postgres.
[group('monorepo')]
e2e: web-e2e desktop-e2e vscode-e2e vscode-test-integration api-test-db

# Build de produccion de lo que se despliega o se distribuye.
[group('monorepo')]
build: api-build web-build desktop-web-build vscode-build

# Reglas comunes a todos los lenguajes: tamano, tests apagados, marcadores sin ticket y grafo de paquetes.
[group('monorepo')]
guard:
    {{pnpm}} run check:quality

# ESLint de la raiz: la configuracion compartida y los scripts de tools/.
[group('monorepo')]
root-lint:
    {{pnpm}} run lint

# Stylelint sobre el CSS de apps y paquetes (colores solo desde los tokens).
[group('monorepo')]
lint-css:
    {{pnpm}} run lint:css

# Tests de los scripts de tools/checks y de las reglas propias de ESLint.
[group('monorepo')]
tools-test:
    {{pnpm}} run check:tools-test

# Codigo muerto: exports, ficheros y dependencias que nadie usa.
[group('monorepo')]
knip:
    {{pnpm}} run knip

# Borra artefactos de build, cobertura e informes.
[group('monorepo')]
clean:
    cargo clean
    rm -rf apps/*/dist apps/*/build apps/*/coverage apps/*/playwright-report apps/*/test-results
    rm -rf packages/*/dist packages/*/coverage
    rm -rf apps/vscode/out apps/vscode/.vscode-test apps/vscode/*.vsix

# -------------------------------------------------------------------- rust ---

# Formatea todo el Rust del workspace.
[group('rust')]
rust-fmt:
    cargo fmt --all

# Clippy en modo estricto sobre default-members: cualquier aviso es un error.
[group('rust')]
rust-lint:
    cargo clippy --all-targets --all-features -- -D warnings

# Tests de Rust de default-members (todo menos el escritorio).
[group('rust')]
rust-test:
    cargo test

# Auditoria de dependencias: avisos de seguridad, licencias y fuentes.
[group('rust')]
rust-deny:
    cargo deny check

# Informe de cobertura de default-members. Los umbrales viven en api-cov y engine-cov.
[group('rust')]
rust-cov:
    cargo llvm-cov --summary-only

# --------------------------------------------------------------------- api ---

# Todo el enforcement de la API: clippy, tests y cobertura de sus crates.
[group('api')]
api-check:
    cargo clippy {{api_crates}} --all-targets --all-features -- -D warnings
    cargo test {{api_crates}}
    @just api-cov

# Tests de la API que no necesitan base de datos.
[group('api')]
api-test:
    cargo test {{api_crates}}

# RAMAL_TEST_REQUIRE_DATABASE convierte el salto en fallo: si la base no esta,
# esto se pone en rojo en vez de pasar sin ejecutar nada. `--tests` y no una
# lista de suites: en Gastonomo la lista se quedo vieja al partir un fichero en
# dos y una bateria entera dejo de correr sin ponerse en rojo.
#
# Si RAMAL_TEST_DATABASE_URL ya viene puesta (la CI, con su servicio de
# postgres), se usa esa y no se levanta compose: dos Postgres en el 5432 chocan.
[group('api')]
[doc('Tests de la API contra el Postgres de docker compose, sin permitir que se salten.')]
api-test-db:
    #!/usr/bin/env bash
    set -euo pipefail
    if [ -z "${RAMAL_TEST_DATABASE_URL:-}" ]; then
      just db-up
      export RAMAL_TEST_DATABASE_URL="{{database_url}}"
    fi
    RAMAL_TEST_REQUIRE_DATABASE=1 cargo test {{api_crates}} --tests -- --nocapture

# Cobertura de la API: 80% en el nucleo (dominio y aplicacion) y 60% global.
[group('api')]
api-cov:
    cargo llvm-cov -p ramal-api-domain -p ramal-api-application --summary-only --fail-under-lines 80
    cargo llvm-cov {{api_crates}} --summary-only --fail-under-lines 60

# Arranca la API en local contra el Postgres de docker compose.
[group('api')]
api-run: db-up
    RAMAL__DATABASE__URL="{{database_url}}" cargo run --bin ramal-api

# Compila el binario de la API en modo release.
[group('api')]
api-build:
    cargo build --release -p ramal-api

# El contexto es la raiz porque el workspace de Cargo y el Cargo.lock viven aqui.
# CARGO_BUILD_JOBS del entorno limita los hilos de cargo dentro de la imagen.
[group('api')]
[doc('Construye la imagen de produccion de la API.')]
api-docker:
    docker build -f apps/api/Dockerfile --build-arg CARGO_BUILD_JOBS="${CARGO_BUILD_JOBS:-default}" -t ramal-api:local .

# Contrato del binario: con `--openapi <ruta>` escribe el documento y sale sin
# tocar la base de datos. Se versiona y @ramal/api-client genera su cliente de el.
[group('api')]
[doc('Escribe el documento OpenAPI de la API en apps/api/openapi.json.')]
api-openapi:
    cargo run --quiet --bin ramal-api -- --openapi apps/api/openapi.json

# El cliente de la web se genera desde el documento versionado, nunca desde la
# API en marcha: asi la CI puede comprobar que no hay diff sin levantar nada.
[group('api')]
[doc('Regenera los tipos de @ramal/api-client desde apps/api/openapi.json.')]
api-client:
    {{pnpm_filter}} @ramal/api-client generate

# Falla si el cliente generado no coincide con el que esta en git.
[group('api')]
api-client-check: api-client
    #!/usr/bin/env bash
    set -euo pipefail
    out=packages/api-client/src/generated
    git diff --exit-code -- "${out}"
    # git diff no ve los ficheros nuevos: un esquema nuevo sin commitear tiene que fallar igual.
    if [ -n "$(git status --porcelain -- "${out}")" ]; then
      git status --short -- "${out}"
      echo "api-client-check: el cliente no esta al dia. Ejecuta just api-client y commitea." >&2
      exit 1
    fi

# ------------------------------------------------------------------ engine ---

# Todo el enforcement del motor de Git: clippy, tests y cobertura de crates/* y del sidecar.
[group('engine')]
engine-check:
    cargo clippy {{engine_crates}} --all-targets --all-features -- -D warnings
    cargo test {{engine_crates}}
    @just engine-cov

# Tests del motor de Git y del sidecar de VS Code.
[group('engine')]
engine-test:
    cargo test {{engine_crates}}

# Cobertura del motor: 80% en el nucleo (dominio y aplicacion) y 60% global.
[group('engine')]
engine-cov:
    cargo llvm-cov -p ramal-core-domain -p ramal-core-application --summary-only --fail-under-lines 80
    cargo llvm-cov {{engine_crates}} --summary-only --fail-under-lines 60

# ts-rs escribe los tipos cuando corren sus tests `export_bindings_*`. La carpeta
# se borra antes para que un tipo eliminado en Rust desaparezca tambien en TS.
[group('engine')]
[doc('Regenera los tipos de TypeScript del protocolo en packages/bridge.')]
protocol-gen:
    rm -rf {{protocol_out}}
    TS_RS_EXPORT_DIR="{{justfile_directory()}}/{{protocol_out}}" cargo test -p ramal-protocol export_bindings

# Falla si los tipos generados no coinciden con los que estan en git.
[group('engine')]
protocol-check: protocol-gen
    #!/usr/bin/env bash
    set -euo pipefail
    git diff --exit-code -- {{protocol_out}}
    # git diff no ve los ficheros nuevos: un tipo anadido en Rust y sin
    # commitear en TS tiene que fallar igual.
    if [ -n "$(git status --porcelain -- {{protocol_out}})" ]; then
      git status --short -- {{protocol_out}}
      echo "protocol-check: los tipos generados no estan al dia. Ejecuta just protocol-gen y commitea." >&2
      exit 1
    fi

# Compila el sidecar de la extension en modo release.
[group('engine')]
vscode-engine-build:
    cargo build --release -p ramal-vscode-engine

# ---------------------------------------------------------------- packages ---

# Todo el enforcement de los paquetes compartidos: lint, tipos y cobertura.
[group('packages')]
packages-check: packages-lint packages-typecheck packages-cov

# ESLint de los paquetes compartidos.
[group('packages')]
packages-lint: (_packages-run "lint")

# TypeScript en modo estricto en los paquetes compartidos.
[group('packages')]
packages-typecheck: (_packages-run "typecheck")

# Tests unitarios de los paquetes compartidos.
[group('packages')]
packages-test: (_packages-run "test")

# Cobertura de los paquetes compartidos, con sus umbrales.
[group('packages')]
packages-cov: (_packages-run "test:coverage")

# Regenera los tipos de los parametros de @ramal/i18n desde el diccionario castellano.
[group('packages')]
i18n-gen:
    {{pnpm_filter}} @ramal/i18n generate

# Los paquetes compartidos llegan de varias ramas (bridge, repo-view y
# api-client los montan otros), asi que una carpeta que todavia no existe se
# salta con un aviso en vez de fallar. Lo que no se esconde: un paquete que
# existe tiene que llamarse @ramal/<carpeta> y declarar el script del contrato
# (si no, pnpm lo saltaria en silencio), y cualquier fallo de su script pone la
# receta en rojo. Sin ningun paquete, falla.
[private]
_packages-run script:
    #!/usr/bin/env bash
    set -euo pipefail
    filters=()
    for dir in {{ts_package_dirs}}; do
      manifest="packages/${dir}/package.json"
      if [ ! -f "${manifest}" ]; then
        echo "packages: packages/${dir} todavia no existe, se salta" >&2
        continue
      fi
      info="$(node -p "const p = require('./${manifest}'); [p.name, Boolean(p.scripts && p.scripts['{{script}}'])].join(' ')")"
      if [ "${info}" != "@ramal/${dir} true" ]; then
        echo "packages: ${manifest} incumple el contrato (nombre @ramal/${dir} y script {{script}}): ${info}" >&2
        exit 1
      fi
      filters+=(--filter "@ramal/${dir}")
    done
    if [ "${#filters[@]}" -eq 0 ]; then
      echo "packages: no hay ningun paquete compartido que comprobar" >&2
      exit 1
    fi
    {{pnpm}} --fail-if-no-match "${filters[@]}" run {{script}}

# --------------------------------------------------------------------- web ---

# Todo el enforcement de la web (lint, tipos, cobertura y build).
[group('web')]
web-check: web-lint web-typecheck web-cov web-build

# Servidor de desarrollo con HMR.
[group('web')]
web-dev:
    {{pnpm_filter}} @ramal/web dev

# ESLint de la web (capas, complejidad, hooks, accesibilidad, textos visibles).
[group('web')]
web-lint:
    {{pnpm_filter}} @ramal/web lint

# TypeScript en modo estricto.
[group('web')]
web-typecheck:
    {{pnpm_filter}} @ramal/web typecheck

# Tests unitarios (vitest).
[group('web')]
web-test:
    {{pnpm_filter}} @ramal/web test

# Cobertura de la web con sus umbrales.
[group('web')]
web-cov:
    {{pnpm_filter}} @ramal/web test:coverage

# Build de produccion.
[group('web')]
web-build:
    {{pnpm_filter}} @ramal/web build

# E2E de Playwright (levanta el servidor el solo).
[group('web')]
web-e2e:
    {{pnpm_filter}} @ramal/web e2e

# Descarga el navegador que usa Playwright.
[group('web')]
web-e2e-install:
    {{pnpm_filter}} @ramal/web exec playwright install --with-deps chromium

# Sin tocar la red real: la pila se levanta y se deja arriba para seguir
# probando a mano; `just down` la tumba. Las rutas que Playwright pide son las
# del gateway, asi que una ruta de la API que acabe en la SPA se ve aqui.
[group('web')]
[doc('E2E de Playwright contra la pila Docker, entrando por el gateway.')]
e2e-stack:
    {{compose_full}} up -d --build --wait
    E2E_USE_REAL_API=1 E2E_BASE_URL="http://127.0.0.1:${GATEWAY_PORT:-8090}" \
      {{pnpm_filter}} @ramal/web e2e

# Construye la imagen de produccion de la web (contexto en la raiz).
[group('web')]
web-docker:
    docker build -f apps/web/Dockerfile -t ramal-web:local .

# Construir no prueba nada: arranca la imagen y comprueba lo que sirve.
[group('web')]
web-docker-smoke: web-docker
    bash apps/web/scripts/smoke-image.sh ramal-web:local

# ----------------------------------------------------------------- desktop ---

# Todo el enforcement del frontend de escritorio: lint, tipos, cobertura y build de Vite.
[group('desktop')]
desktop-check: desktop-lint desktop-typecheck desktop-cov desktop-web-build

# App de escritorio en desarrollo (tauri dev). Necesita webkit2gtk.
[group('desktop')]
desktop-dev:
    {{pnpm_filter}} @ramal/desktop exec tauri dev

# ESLint del frontend de escritorio.
[group('desktop')]
desktop-lint:
    {{pnpm_filter}} @ramal/desktop lint

# TypeScript en modo estricto.
[group('desktop')]
desktop-typecheck:
    {{pnpm_filter}} @ramal/desktop typecheck

# Tests unitarios del frontend (vitest).
[group('desktop')]
desktop-test:
    {{pnpm_filter}} @ramal/desktop test

# Cobertura del frontend con sus umbrales.
[group('desktop')]
desktop-cov:
    {{pnpm_filter}} @ramal/desktop test:coverage

# Build de Vite del frontend, sin empaquetar la app nativa.
[group('desktop')]
desktop-web-build:
    {{pnpm_filter}} @ramal/desktop build

# E2E de Playwright contra el build de Vite, con el IPC de Tauri simulado.
[group('desktop')]
desktop-e2e:
    {{pnpm_filter}} @ramal/desktop e2e

# Descarga el navegador que usa Playwright.
[group('desktop')]
desktop-e2e-install:
    {{pnpm_filter}} @ramal/desktop exec playwright install --with-deps chromium

# Clippy y tests del lado nativo. Necesita webkit2gtk (libwebkit2gtk-4.1-dev).
[group('desktop')]
desktop-rust-check:
    cargo clippy -p ramal-desktop --all-targets --all-features -- -D warnings
    cargo test -p ramal-desktop

# El empaquetado (.deb, .AppImage, .dmg, instaladores) llega con el workflow de
# release; hasta entonces el build compila el binario con el frontend embebido y
# se para ahi. Sin --no-bundle, la AppImage descargaria herramientas en cada build.
[group('desktop')]
[doc('Compila la app de escritorio (tauri build sin empaquetar). Necesita webkit2gtk.')]
desktop-build:
    {{pnpm_filter}} @ramal/desktop exec tauri build --no-bundle

# El binario del E2E se compila con la feature `e2e`, que mete el servidor
# WebDriver embebido y deja elegir la carpeta por variable de entorno (el dialogo
# del sistema no se puede manejar con WebDriver). En Linux sin DISPLAY (CI, SSH,
# Docker) se arranca un servidor X virtual con xvfb-run. En macOS y Windows no
# hay X ni xvfb-run: la app abre su ventana sin mas.
[group('desktop')]
[doc('E2E nativos con WebdriverIO y la app real. En Linux necesita webkit2gtk.')]
desktop-e2e-native:
    #!/usr/bin/env bash
    set -euo pipefail
    {{pnpm_filter}} @ramal/desktop exec tauri build --debug --no-bundle --features e2e
    if [ "$(uname -s)" != Linux ] || [ -n "${DISPLAY:-}" ]; then
      {{pnpm_filter}} @ramal/desktop e2e:native
    else
      xvfb-run -a {{pnpm_filter}} @ramal/desktop e2e:native
    fi

# La imagen trae webkit2gtk y el resto de requisitos de Tauri en Ubuntu 22.04,
# la Linux soportada mas antigua. Sirve en maquinas sin esas librerias.
[group('desktop')]
[doc('Construye la imagen de Linux con los requisitos de Tauri (tools/docker).')]
desktop-docker-image:
    docker build -f tools/docker/tauri-linux.Dockerfile -t {{tauri_image}} tools/docker

# Clippy y tests de ramal-desktop dentro de la imagen de Tauri.
[group('desktop')]
desktop-docker-rust-check: desktop-docker-image
    @just _desktop-docker desktop-rust-check

# tauri build sin empaquetar dentro de la imagen de Tauri.
[group('desktop')]
desktop-docker-build: desktop-docker-image
    @just _desktop-docker desktop-build

# E2E nativo bajo xvfb dentro de la imagen de Tauri.
[group('desktop')]
desktop-docker-e2e-native: desktop-docker-image
    @just _desktop-docker desktop-e2e-native

# Corre una receta de escritorio en la imagen de Tauri. El repo se monta en la
# misma ruta que en la maquina (asi valen los enlaces de node_modules, que hay que
# tener instalados con `just setup`) y con el uid del usuario, para que dist/ y
# los esquemas de Tauri no queden de root. Cargo usa volumenes propios: su target
# no se mezcla con el de la maquina, que se compila con otra glibc.
[private]
_desktop-docker recipe:
    docker run --rm --init \
      --user "$(id -u):$(id -g)" \
      -v "{{justfile_directory()}}:{{justfile_directory()}}" \
      -w "{{justfile_directory()}}" \
      -v ramal-desktop-cargo:/cargo-home \
      -v ramal-desktop-target:/cargo-target \
      -e CARGO_BUILD_JOBS \
      -e CI \
      {{tauri_image}} just {{recipe}}

# ------------------------------------------------------------------ vscode ---

# Todo el enforcement de la extension: lint, tipos, cobertura y build.
[group('vscode')]
vscode-check: vscode-lint vscode-typecheck vscode-cov vscode-build

# ESLint del host de la extension y de los webviews.
[group('vscode')]
vscode-lint:
    {{pnpm_filter}} ramal lint

# TypeScript en modo estricto.
[group('vscode')]
vscode-typecheck:
    {{pnpm_filter}} ramal typecheck

# Tests unitarios (vitest).
[group('vscode')]
vscode-test:
    {{pnpm_filter}} ramal test

# Cobertura de la extension con sus umbrales.
[group('vscode')]
vscode-cov:
    {{pnpm_filter}} ramal test:coverage

# Build del host (esbuild) y de los webviews.
[group('vscode')]
vscode-build:
    {{pnpm_filter}} ramal build

# E2E de Playwright contra el bundle del webview, con acquireVsCodeApi falso (puerto 4175).
[group('vscode')]
vscode-e2e:
    {{pnpm_filter}} ramal e2e

# En Linux sin DISPLAY (CI, SSH) se arranca un servidor X virtual con xvfb-run;
# en macOS y Windows VS Code abre su ventana sin mas. Necesita el sidecar
# compilado (lo ejecuta de verdad) y el build de la extension.
[group('vscode')]
[doc('Tests de integracion dentro de un VS Code de verdad (@vscode/test-cli).')]
vscode-test-integration: vscode-engine-build vscode-build
    #!/usr/bin/env bash
    set -euo pipefail
    if [ "$(uname -s)" != Linux ] || [ -n "${DISPLAY:-}" ]; then
      {{pnpm_filter}} ramal test:integration
    else
      xvfb-run -a {{pnpm_filter}} ramal test:integration
    fi

# Contrato: el sidecar se copia a apps/vscode/dist/bin/ (ramal-vscode-engine, con
# .exe en Windows), la ruta fija que ejecuta la extension instalada
# (architecture.md 4.4), y el script `package` de la extension recibe --target con
# la plataforma de vsce. Un .vsix por plataforma, porque lleva un binario nativo.
[group('vscode')]
[doc('Empaqueta la extension (.vsix) para esta plataforma, con el binario del motor dentro.')]
vscode-package: vscode-engine-build vscode-build
    #!/usr/bin/env bash
    set -euo pipefail
    case "$(uname -s)-$(uname -m)" in
      Linux-x86_64) target=linux-x64 ;;
      Linux-aarch64 | Linux-arm64) target=linux-arm64 ;;
      Darwin-x86_64) target=darwin-x64 ;;
      Darwin-arm64) target=darwin-arm64 ;;
      MINGW* | MSYS* | CYGWIN*) target=win32-x64 ;;
      *) echo "vscode-package: plataforma no soportada: $(uname -s)-$(uname -m)" >&2; exit 1 ;;
    esac
    binary=ramal-vscode-engine
    case "${target}" in win32-*) binary="${binary}.exe" ;; esac
    mkdir -p apps/vscode/dist/bin
    cp "target/release/${binary}" "apps/vscode/dist/bin/${binary}"
    {{pnpm_filter}} ramal run package --target "${target}"

# ------------------------------------------------------------------ docker ---

# Levanta la pila de desarrollo completa (db, api, web, gateway y mailpit).
[group('docker')]
up:
    {{compose_full}} up -d --build

# Para la pila y borra los volumenes.
[group('docker')]
down:
    {{compose_full}} down -v

# Logs de la pila. just logs api
[group('docker')]
logs service="":
    {{compose_full}} logs -f --tail 100 {{service}}

# Estado de los servicios.
[group('docker')]
ps:
    {{compose_full}} ps

# Existe por un fallo real en Gastonomo: el gateway enrutaba solo /health y la
# web, asi que las rutas de /v1 acababan en el servidor de la web. Los tests de
# cada imagen por separado pasaban igual, porque ninguno miraba la composicion.
[group('docker')]
[doc('Comprueba que por el gateway se llega a la API y a la web.')]
smoke url="":
    bash tools/scripts/smoke-stack.sh {{ if url == "" { "http://127.0.0.1:${GATEWAY_PORT:-8090}" } else { url } }}

# Levanta la pila completa, le pasa la prueba de humo y la tumba.
[group('docker')]
stack-smoke:
    #!/usr/bin/env bash
    set -euo pipefail
    trap '{{compose_full}} down -v >/dev/null 2>&1 || true' EXIT
    {{compose_full}} up -d --build --wait
    bash tools/scripts/smoke-stack.sh "http://127.0.0.1:${GATEWAY_PORT:-8090}"

# Levanta solo Postgres y espera a que responda.
[group('docker')]
db-up:
    {{compose}} up -d --wait db

# Para Postgres sin borrar sus datos.
[group('docker')]
db-down:
    {{compose}} stop db

# Abre una psql contra la base de datos de desarrollo.
[group('docker')]
psql: db-up
    {{compose}} exec db psql -U "{{pg_user}}" -d "{{pg_db}}"

# Aplica las migraciones a mano. La API tambien las aplica sola al arrancar.
[group('docker')]
migrate: db-up
    #!/usr/bin/env bash
    set -euo pipefail
    shopt -s nullglob
    for file in apps/api/migrations/*.sql; do
      echo "aplicando ${file}"
      {{compose}} exec -T db psql -v ON_ERROR_STOP=1 -U "{{pg_user}}" -d "{{pg_db}}" < "${file}"
    done

# Se borra la BASE DE DATOS, no el volumen: el volumen es del proyecto de
# compose y su nombre cambia con COMPOSE_PROJECT_NAME, asi que borrarlo a mano
# es fragil. DROP DATABASE ... WITH (FORCE) echa a quien este conectado, la API
# incluida, y vuelve a crearla vacia sin tumbar el contenedor.
[group('docker')]
[doc('Borra la base de datos y la vuelve a crear vacia.')]
db-reset: db-up
    {{compose}} exec -T db psql -v ON_ERROR_STOP=1 -U "{{pg_user}}" -d postgres \
      -c 'DROP DATABASE IF EXISTS "{{pg_db}}" WITH (FORCE)' \
      -c 'CREATE DATABASE "{{pg_db}}" OWNER "{{pg_user}}"'
    @just migrate
