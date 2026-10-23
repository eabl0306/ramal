# Convenciones del repositorio

Este fichero es para quien llegue a trabajar aqui, persona o agente. Es preciso a
proposito. El README es el resumen; esto es el detalle. El porque de cada cosa esta en
[docs/architecture.md](docs/architecture.md) y en los [ADR](docs/adr/).

## Que es este repositorio

Monorepo poliglota de Git Rudder, un cliente de Git: tres productos (escritorio, VS Code y web), una
API y un solo motor de Git en Rust que comparten escritorio y VS Code.

| Ruta                                                    | Que es                                      | Toolchain                                                | Comando de verificacion                          |
| ------------------------------------------------------- | ------------------------------------------- | -------------------------------------------------------- | ------------------------------------------------ |
| `apps/api`                                              | API HTTP (5 crates `git-rudder-api-*`)      | Cargo, Rust 1.99.1, ed. 2024, Salvo, SQLx                | `just api-check` y `just api-test-db`            |
| `apps/web`                                              | Landing prerenderizada y SPA                | pnpm 11.18, Node 24, React Router 8, Vite                | `just web-check`                                 |
| `apps/desktop`                                          | Git Rudder Desktop (`@git-rudder/desktop`)  | Tauri 2 (`src-tauri`, crate `git-rudder-desktop`) y Vite | `just desktop-check` y `just desktop-rust-check` |
| `apps/vscode`                                           | Extension de VS Code (paquete `git-rudder`) | Node 24, esbuild y Vite                                  | `just vscode-check`                              |
| `apps/vscode-engine`                                    | Sidecar del motor por stdio                 | Cargo                                                    | `just engine-check`                              |
| `crates/*`                                              | Motor de Git (5 crates)                     | Cargo                                                    | `just engine-check` y `just protocol-check`      |
| `packages/{domain,i18n,ui,repo-view,bridge,api-client}` | TypeScript compartido                       | pnpm workspace                                           | `just packages-check`                            |
| `packages/eslint-config`, `packages/tsconfig`           | Configuracion compartida                    | pnpm workspace                                           | `just root-lint` y `just tools-test`             |
| `tools/checks`                                          | Reglas comunes a los dos lenguajes          | Node puro, sin dependencias                              | `just guard` y `just tools-test`                 |
| `tools/rust-complexity`                                 | Cognitiva y ciclomatica de Rust             | Cargo (syn, sin compilar el codigo)                      | `just rust-complexity`                           |

Usa **pnpm por corepack**, nunca npm ni yarn ni un pnpm global de otra version. La version
esta fijada en `packageManager` (11.18.0). Rust lo fija `rust-toolchain.toml` (1.99.1).

## Como se escribe el nombre

El producto es **Git Rudder** ([ADR 0027](docs/adr/0027-renombrado-a-git-rudder.md)). En
identificadores hay una forma por convencion de cada sitio, y no se mezclan:

| Donde                                                             | Forma                                              |
| ----------------------------------------------------------------- | -------------------------------------------------- |
| Texto visible, titulos, categoria de comandos                     | `Git Rudder`                                       |
| Crates, paquetes, repositorio, extension, nombres de fichero      | `git-rudder`                                       |
| Scope de npm                                                      | `@git-rudder/`                                     |
| Modulos de Rust, upstreams de nginx, usuario y base de PostgreSQL | `git_rudder`                                       |
| Comandos, vistas y ajustes de VS Code; claves de almacenamiento   | `gitRudder`                                        |
| Variables de entorno y constantes                                 | `GIT_RUDDER`                                       |
| Id de la extension, y de Tauri y Flatpak                          | `git-rudder.git-rudder`, `com.gitrudder.GitRudder` |

No llevan el nombre y no hay que tocarlos: el prefijo `--r-` de los tokens de CSS (la `r` vale
igual para Rudder) y el prefijo `repo-` de los `repoId`.

## Arquitectura: la regla que no se negocia

Arquitectura hexagonal en todas las piezas. El nucleo (dominio mas casos de uso) no conoce
framework, ni HTTP, ni SQL, ni git, ni React, ni Tauri, ni VS Code. Lo minimo que hay que saber
de cada pieza, y que lo hace cumplir:

- **API.** Un crate por capa: `domain`, `application`, `infrastructure`, `http` y `api`. El
  nucleo no depende de infraestructura ni lee el reloj (la hora llega por el puerto `Clock`).
  `git-rudder-api-http` es el unico crate que conoce Salvo y arranca el servidor; `git-rudder-api` es el
  composition root (`crates/api/src/bootstrap.rs` mas `bootstrap/<contexto>.rs`) y no depende de
  Salvo. Lo vigila `apps/api/crates/api/tests/architecture.rs`, que lee `Cargo.lock` (con las
  dev-dependencies: por eso los tests de `http` no pueden usar PostgreSQL), y el `clippy.toml`
  de cada crate de nucleo, que prohibe procesos, disco, red, entorno y reloj.
- **Motor.** `git-rudder-core-domain` (solo `thiserror`, sin serde) y `git-rudder-core-application`
  (mas `async-trait`, sin tokio). `git-rudder-protocol` no depende de ningun crate de Git Rudder: el mapeo
  dominio a DTO vive en `git-rudder-engine`, que es la unica puerta de los dos hosts. Los hosts
  (`git-rudder-desktop` y `git-rudder-vscode-engine`) solo dependen del motor y del protocolo. Lo vigilan
  `crates/engine/tests/architecture.rs` y `crates/engine/tests/core_purity.rs`. La composicion
  tiene dos niveles: `EngineBuilder` construye lo comun y cada host aporta lo suyo.
- **Escritorio.** `src-tauri` es un adaptador driving: el comando `rpc` reenvia al motor y
  `repo_pick_and_open` ensena el dialogo desde Rust. **El webview no es de fiar**: `rpc` solo deja
  pasar los metodos de la lista de su ventana (`src-tauri/src/surfaces.rs`, escrita a mano) y
  reenvia como llamada de webview, asi que el motor rechaza los metodos `host_only`. El webview
  nunca manda una ruta. En `src/`, capas `application`, `infrastructure`, `ui` y `app` con
  `eslint-plugin-boundaries`: `ui` no importa infraestructura ni Tauri.
- **VS Code.** `src/extension/application` no importa `vscode` ni `node:`; se prueba con
  Vitest. Los webviews se tratan como clientes hostiles: el host valida tamano (1 MiB), forma
  (zod), metodo y superficie (`application/surfaces.ts`) y que el `repoId` sea el del panel, y
  reescribe los ids. Un ajuste de workspace nunca elige que binario del motor se ejecuta.
- **Web.** Cinco carpetas en `apps/web/src` (`domain`, `application`, `infrastructure`, `ui`,
  `app`), vigiladas por `eslint-plugin-boundaries` con denegacion por defecto. `domain` no importa
  nada de npm; `ui` recibe los puertos por `UiPortsContext` y no importa `infrastructure` ni
  `app`. La raiz de composicion es `src/app/composition-root.ts` y los modulos de ruta son finos.
- **Paquetes.** `@git-rudder/domain`, `@git-rudder/i18n` y `@git-rudder/bridge` no tienen dependencias en
  tiempo de ejecucion y compilan sin DOM ni tipos de Node (`@git-rudder/tsconfig/library.json`).
  `@git-rudder/repo-view` habla con el motor solo por `@git-rudder/bridge`. `@git-rudder/api-client` solo lo
  importa `apps/web/src/infrastructure`. Quien depende de quien lo vigila
  `tools/checks/package-graph.mjs` (matriz `MATRIX`) y las project references de TypeScript.

Para anadir algo nuevo: dominio, puerto, caso de uso con un fake, adaptador, adaptador
driving y cableado en el composition root. En ese orden. Las recetas por pieza estan en la
seccion 9 de `docs/architecture.md`.

## El protocolo del motor

JSON-RPC 2.0, el mismo en los dos productos. Cambia el transporte: `invoke("rpc")` en
escritorio; en VS Code, `postMessage` entre webview y extension host y marcos
`Content-Length: <bytes>\r\n\r\n` por stdio hasta el sidecar (32 MiB como maximo). En el
sidecar **stdout es solo protocolo**: el log va a stderr y `print_stdout` esta denegado.

| Metodo         | Parametros                                       | Resultado                                                               | Quien lo llama          |
| -------------- | ------------------------------------------------ | ----------------------------------------------------------------------- | ----------------------- |
| `engine/info`  | `{}`                                             | `{ engineVersion, protocolVersion, git: { path, version, supported } }` | Los hosts al arrancar   |
| `repo/open`    | `{ path }`                                       | `{ repoId, root, headRef?, headSha?, isDetached, isEmpty }`             | Solo los hosts          |
| `graph/window` | `{ repoId, offset, limit (1..1000), snapshot? }` | `{ repoId, offset, rows, hasMore, laneCount, snapshot }`                | `@git-rudder/repo-view` |

Lo que no se puede romper:

- `params` siempre es un objeto, con `deny_unknown_fields`. Campos en camelCase. Ids enteros
  menores que 2^31. Nada de enteros de 64 bits (ts-rs los da como `bigint`). Instantes como texto
  ISO 8601 estricto.
- La interfaz decide por `error.data.kind`: `invalid_params`, `repo_not_open`,
  `not_a_repository`, `git_not_found`, `git_version_unsupported`, `git_failed`, `stale_window`,
  `internal`. El bridge anade `transport` y `timeout`; el extension host, `engine_unavailable`.
  `message` va en ingles y es para el log. Un metodo inexistente o un sobre roto da `internal`.
- Los metodos con rutas absolutas o secretos se marcan `host_only` en `define_methods!`
  (`crates/protocol/src/methods.rs`). Hoy solo `repo/open`. Ningun webview los puede llamar.
- `repoId` es opaco: `repo-` mas 16 cifras hexadecimales que salen de la raiz. Reabrir la misma
  carpeta da el mismo id, tambien tras reiniciar el sidecar.
- `snapshot`: la pagina siguiente del grafo se pide con la huella de la primera; si las refs
  cambiaron, el motor responde `stale_window` y la interfaz vuelve a la fila 0.
- `PROTOCOL_VERSION` (hoy 2) la compara el extension host en cada arranque del sidecar. En los
  tests no se escribe a mano: los fakes la toman de `@git-rudder/bridge`. Un test que fijaba la 1 se
  rompio al subirla a la 2.
- Los tipos de TypeScript salen de Rust con ts-rs y se commitean en
  `packages/bridge/src/generated`. Tras cambiar un DTO o un metodo: `just protocol-gen` y
  commitear lo generado en el mismo commit. `just protocol-check` (y la CI) falla si no.
- `$/cancel` existe en el cliente pero el motor todavia lo ignora, y VS Code no lo reenvia desde
  un webview. No hay tipo `cancelled`: cancelar rechaza con `signal.reason`.

## El contrato de la API

El contrato completo esta en [docs/api-surface.md](docs/api-surface.md); del primer corte solo
existe la fase F0. La referencia ejecutable es `GET /openapi.json` (OpenAPI 3.1), versionado en
`apps/api/openapi.json`.

```
GET /health/live    200 siempre, sin tocar dependencias, checks vacio
GET /health/ready   200 si status=ok, 503 si status=degraded

{"status":"ok"|"degraded",
 "checks":[{"name":"database","status":"ok"|"error","latencyMs":3}],
 "version":"0.1.0","checkedAt":"2026-09-30T10:00:00Z"}
```

- El motivo de un fallo de salud nunca viaja en la respuesta: va al log.
- Los errores son `application/problem+json` (RFC 9457), incluidos los 404 y 405 de ruta. `title`
  y `detail` en castellano y **un `code` estable en todo problema** (`route_not_found`,
  `method_not_allowed`, `validation_failed`...) y en cada `errors[]`. Los clientes traducen el
  `code`.
- Un recurso de otra cuenta responde 404, nunca 403. Todo lo de dominio ira bajo `/v1`.
- `/swagger-ui/` solo existe con `GIT_RUDDER__SERVER__SWAGGER_UI=true` (lo ponen `compose.yaml` y
  `just api-run`). Por defecto es un 404.
- Tras cambiar una ruta o un DTO: `just api-openapi` y `just api-client`, y se commitean los
  dos. Un test de `git-rudder-api-http` y `just api-client-check` fallan si no.
- Las migraciones (`apps/api/migrations/`, solo hacia delante) se aplican en segundo plano al
  arrancar y se reintentan cada 10 segundos. Mientras tanto `/health/ready` da 503.

Configuracion por entorno con el prefijo `GIT_RUDDER__` y `__` como separador. Todo tiene valor por
defecto y **una clave desconocida hace fallar el arranque**: por eso `.env.example` deja
comentadas las que aun no se leen.

```
GIT_RUDDER__SERVER__HOST  __PORT  __SWAGGER_UI     127.0.0.1, 8080, false
GIT_RUDDER__DATABASE__URL  __MAX_CONNECTIONS  __ACQUIRE_TIMEOUT_MS
GIT_RUDDER__TELEMETRY__LOG_LEVEL
```

## Limites de calidad

Se aplican a los dos lenguajes y rompen el build ([ADR 0005](docs/adr/0005-enforcement-automatizado.md)):

- Fichero de codigo: 300 lineas. De test: 400.
- Complejidad cognitiva 15 y ciclomatica 10. En TypeScript con ESLint; en Rust con
  `tools/rust-complexity`, porque clippy no mide ninguna de las dos como Sonar.
- Funcion: 50 lineas y 5 parametros. Anidamiento: 4 niveles.
- Prohibido: codigo muerto, imports y variables sin usar (`-D warnings`, knip).
- Marcadores de trabajo pendiente solo con ticket: `TODO(RAM-123)` o `TODO(#123)`.
- Tests apagados solo con motivo escrito. Nada de `.only`.
- Ningun limite se apaga en linea (`eslint-disable` o `#[allow(clippy::...)]` de un umbral):
  `tools/checks/no-limit-suppressions.mjs`. Todo `#[allow]` lleva `reason`.
- Los tests de Rust van en su fichero (`#[cfg(test)] mod tests;` mas `<modulo>/tests.rs`),
  nunca en linea: la cobertura contaria sus lineas.
- Cobertura: 80% en cada nucleo (`git-rudder-core-domain`, `git-rudder-core-application`,
  `git-rudder-api-domain`, `git-rudder-api-application`, cada uno por separado; `@git-rudder/domain`,
  `@git-rudder/i18n`, `@git-rudder/bridge`; `domain` y `application` de la web; `application` del
  escritorio y del extension host) y 60% global, tambien en el host de Tauri.
- Colores solo desde los tokens de `@git-rudder/ui` (stylelint y `git-rudder/no-raw-colors`). Ningun texto
  visible escrito en un componente (`git-rudder/no-jsx-literal-text`): sale de `@git-rudder/i18n`.

Los avisos son errores en todas partes: `-D warnings` en Rust y `--max-warnings 0` en ESLint y
stylelint.

## Comandos que se usan de verdad

```bash
just                       # lista todas las recetas
just check                 # todo el enforcement, como la CI sin lo caro
just ci                    # check + cargo deny, tipos generados, e2e, builds y stack-smoke
just guard                 # tools/checks: tamano, tests apagados, TODO, grafo, limites, versiones, enlaces
just fmt / just fmt-check  # prettier y rustfmt
just api-test-db           # tests de la API contra el Postgres de compose; no se pueden saltar
just engine-check          # motor y sidecar: clippy, tests y cobertura
just protocol-gen          # regenera packages/bridge/src/generated
just api-openapi           # reescribe apps/api/openapi.json
just api-client            # regenera @git-rudder/api-client
just i18n-gen              # regenera los parametros de @git-rudder/i18n
just desktop-e2e           # Playwright del escritorio con el IPC simulado
just desktop-docker-rust-check  # el host de Tauri en Docker, sin webkit2gtk en la maquina
just vscode-test-integration    # VS Code de verdad con el sidecar
just up / just down        # pila de Docker de desarrollo
just stack-smoke           # pila completa aparte, prueba de humo y la tumba
```

Antes de abrir un PR: `just ci`. `just check` se queda corto a proposito, para que sea rapido:
no corre los e2e, ni `cargo deny`, ni los tipos generados, ni los builds, ni la pila.

## Commits

Conventional commits, validados por commitlint en el hook `commit-msg` y en la CI de cada PR.
Cabecera de 100 caracteres como maximo. Los scopes permitidos estan en
`commitlint.config.mjs`: `api`, `engine`, `desktop`, `vscode`, `web`, `domain`, `i18n`, `ui`,
`repo-view`, `bridge`, `api-client`, `packages`, `tools`, `ci`, `docs`, `deps`, `repo`,
`release`. El cuerpo va en castellano.

```
feat(engine): leer el estado del arbol de trabajo
fix(vscode): el test del reintento usa el protocolo actual
docs: alinear la arquitectura con el codigo del primer corte
```

Los hooks se saltan con `LEFTHOOK=0` y solo en emergencias. La CI no se puede saltar.

## Lo que NO hay que hacer

- No metas logica de negocio en un handler, un comando de Tauri o un componente. Va en un caso
  de uso.
- No importes infraestructura desde el nucleo. No compila, o el linter o un test lo rechaza.
- No uses `npm install`, `yarn` ni un pnpm global: rompen el lockfile o ignoran `allowBuilds`.
- No uses `--workspace` con cargo: pisa `default-members` y obliga a compilar Tauri, que necesita
  webkit2gtk. El crate de escritorio va con `-p git-rudder-desktop`.
- No bajes un umbral ni apagues un limite en linea para que pase la CI. Escribe el test o parte
  la funcion.
- No escribas `unwrap()`, `expect()`, `panic!` ni indexes con `[]` en Rust de produccion: clippy
  los rechaza.
- No edites lo generado (`packages/bridge/src/generated`, `packages/api-client/src/generated`,
  `packages/i18n/src/generated`, `apps/api/openapi.json`): se regenera con su receta.
- No anadas un metodo del motor a la lista de un webview si lleva rutas absolutas o secretos:
  se marca `host_only`.
- No relajes la CSP con `'unsafe-inline'`. Un `<style>` inyectado se permite por su hash.
- No edites una migracion que ya esta en `main`: SQLx guarda su suma, las migraciones fallarian
  en cada reintento y `/health/ready` se quedaria en 503.
- No toques `pnpm-workspace.yaml` a mano salvo que sepas lo que haces: cada `allowBuilds` es una
  decision de seguridad.
- No esperes a un diseno que no esta: la fase de diseno se cerro el 2026-10-02. Si una pantalla o
  un estado no sale en los prototipos de `docs/design/handoff/`, disenalo en codigo con los tokens,
  los componentes y la densidad del prototipo mas parecido. Antes, mira el issue #523 y el
  comentario de diseno pendiente del issue de implementacion.

## Detalles del entorno que sorprenden

- **pnpm tiene que ser el 11 de corepack.** En la maquina original habia un pnpm 9 global que
  ignora en silencio claves de pnpm 11 (`allowBuilds`). Las recetas y los hooks llaman a
  `corepack pnpm` (o a `$PNPM`). Si hace falta: `npm rm -g pnpm && corepack enable pnpm`.
  `just doctor` falla si no es 11.18.
- **webkit2gtk puede faltar.** Sin el no compila `git-rudder-desktop`, y por eso no esta en
  `default-members`: ni todas las maquinas ni los runners que no son de escritorio lo tienen. Con
  los paquetes del README instalados, las recetas `desktop-*` funcionan en nativo, el E2E nativo
  incluido. Sin ellos, las recetas `just desktop-docker-*` corren la receta de escritorio
  dentro de `tools/docker/tauri-linux.Dockerfile` (Ubuntu 22.04, la Linux soportada mas antigua).
  Montan el repo en la misma ruta y con tu uid, asi que antes hace falta `just setup` en la
  maquina. Cargo usa volumenes propios (`git-rudder-desktop-cargo`, `git-rudder-desktop-target`). `tauri dev`
  no tiene receta en Docker y no se ha probado.
- **`.claude/worktrees/` esta en `.gitignore`.** Son copias completas del repo que crean los
  agentes; sin eso git, ESLint y prettier las verian como codigo propio.
- **Puertos.** Gateway 8090, API 8080, web 5173, PostgreSQL 5432, Mailpit 8025 y 1025. Vite de
  escritorio 1420 (lo espera `tauri dev`) y de los webviews de VS Code 5175. Playwright: web 4173,
  escritorio 4174, VS Code 4175. Docker publica todo solo en `127.0.0.1` (`GIT_RUDDER_BIND_IP`).
- **Dos proyectos de compose.** El de desarrollo se llama `git-rudder` (contenedores `git-rudder-db-1`...).
  `just stack-smoke` usa `git-rudder-smoke` con puertos que elige Docker, asi que no toca la base local.
  `just down` borra los volumenes de `git-rudder`, la base incluida; `just db-down` la para sin borrar.
  `just e2e-stack` deja la pila arriba.
- **`just api-test-db`** levanta `db` con compose y exporta `GIT_RUDDER_TEST_REQUIRE_DATABASE=1`, que
  convierte el salto en fallo. Si ya hay `GIT_RUDDER_TEST_DATABASE_URL` (la CI), usa esa: dos Postgres
  en el 5432 chocan.
- **La primera `just up` tarda.** La imagen de desarrollo de la API compila el workspace con
  `cargo watch` (hasta cinco minutos de `start_period`).
- **React Aria inyecta un `<style>`.** `react-aria-components` 1.21 mete
  `react-aria-pressable-style`. La CSP de escritorio lo permite por su hash en
  `apps/desktop/src-tauri/tauri.conf.json`; si subes React Aria y cambia, `just desktop-e2e`
  falla con la violacion y hay que poner el hash nuevo. Los webviews de VS Code todavia no llevan
  el hash.
- **E2E nativo de escritorio.** El binario se compila con la feature `e2e`, que mete el servidor
  WebDriver y deja que `GIT_RUDDER_E2E_FOLDER` sustituya el dialogo. `@wdio/tauri-service` quiere
  `withGlobalTauri` para enfocar la ventana; como esta apagado, `wdio.conf.ts` llama a
  `switchToWindow`.
- **xvfb solo en Linux sin pantalla.** `just vscode-test-integration` y `just desktop-e2e-native`
  usan la pantalla si hay `DISPLAY`, y `xvfb-run` si no (la CI). El VS Code de los tests se
  descarga en `apps/vscode/.vscode-test/`, ignorado.
- **La extension se llama `git-rudder`, no `@git-rudder/vscode`.** Las recetas la filtran con
  `pnpm --filter git-rudder`, siempre con `--fail-if-no-match` para que un nombre mal escrito no pase
  en verde. En desarrollo usa el motor de `gitRudder.engine.path`, `GIT_RUDDER_ENGINE_PATH` o `target/`;
  instalada, siempre `dist/bin/`.
- **`just` carga `.env`** (`dotenv-load`), asi que las recetas ven los mismos valores que
  compose. Una variable `GIT_RUDDER__` que la API no conozca hace fallar su arranque.
- **knip no entiende WebdriverIO:** `@wdio/local-runner` y `@wdio/tauri-service` estan en su
  `ignoreDependencies`.
- **La licencia es `FSL-1.1-ALv2`** ([ADR 0021](docs/adr/0021-licencia-fsl.md)). `LICENSE.md` es el texto
  canonico y prettier no lo toca. La VSIX la lleva porque el script `package` de la extension la
  copia de la raiz justo antes de llamar a vsce; esa copia no se versiona.
- **La landing promete lo que dice el manifiesto.** Un test de la extension falla si
  `engines.vscode` y la tarjeta de VS Code de la web no dicen la misma version.

## CI

`.github/workflows/ci.yml` es el unico punto de entrada. Mira que ha cambiado con
`dorny/paths-filter` y llama como workflows reutilizables a `quality.yml`, `rust.yml`,
`api.yml`, `engine.yml`, `packages.yml`, `web.yml`, `desktop.yml`, `vscode.yml` y `stack.yml`.
El job `all-green` es el unico check que hay que marcar como required: un job saltado por los
filtros cuenta como verde. Cambiar el `justfile` o `ci.yml` lo ejecuta todo.

- Los runners van fijados a `ubuntu-24.04`, salvo el host de Tauri, en `ubuntu-22.04` a
  proposito (el minimo de glibc de los builds de Linux).
- El motor se prueba con el git del runner y con git 2.34, el minimo.
- La CI de escritorio corre el host de Tauri, `tauri build --no-bundle` y el E2E nativo bajo
  xvfb. La de VS Code, la integracion bajo xvfb y la VSIX de `linux-x64`.
- `stack.yml` levanta la pila con credenciales efimeras, corre Playwright por el gateway y
  `just smoke`.
- No hay workflow de release todavia: ni instaladores de escritorio, ni VSIX de otras
  plataformas, ni imagen publicada.
