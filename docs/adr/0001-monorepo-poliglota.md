# ADR 0001: Estructura de monorepo poliglota

- Estado: aceptado
- Fecha: 2026-09-30
- Revisado: 2026-10-01, para alinearlo con el codigo del primer corte

## Contexto

Git Rudder son tres productos (Desktop, VS Code y Web) construidos con cinco piezas de codigo: la
API en Rust, el motor Git del cliente en Rust, el host de escritorio con Tauri, la extension
de VS Code con su sidecar y la web en React. Comparten bastante mas que Gastonomo, que solo
compartia el contrato HTTP:

- Crates enteros. El motor (`crates/*`) lo compilan dos binarios distintos: el host de Tauri
  lo lleva dentro y `git-rudder-vscode-engine` lo expone por stdio.
- Paquetes de TypeScript que usan las tres apps: tokens y primitivas (`@git-rudder/ui`), textos
  (`@git-rudder/i18n`), logica de interfaz (`@git-rudder/domain`) y la vista de repositorio
  (`@git-rudder/repo-view`), que usan escritorio y VS Code.
- Tipos generados que cruzan de un lenguaje a otro: ts-rs escribe en `packages/bridge` los
  tipos del protocolo del motor, y openapi-typescript lee `apps/api/openapi.json` para
  generar `@git-rudder/api-client`.

Un cambio normal cruza lenguajes. Un metodo nuevo del motor toca `git-rudder-core-application`,
`git-rudder-git`, `git-rudder-protocol`, `git-rudder-engine`, los tipos generados de `@git-rudder/bridge` y un
hook de `@git-rudder/repo-view`. En repositorios separados eso son cinco PRs que hay que publicar
en orden, con versiones intermedias que no compilan juntas.

Hay dos gestores de dependencias (cargo y pnpm) y ninguno sabe nada del otro. Las opciones
eran varios repositorios, un monorepo con orquestador (Turborepo, Nx, Bazel, moon) o un
monorepo sin orquestador donde cada ecosistema conserva su toolchain, que es el modelo de
Gastonomo.

## Decision

Un solo repositorio git con esta forma:

```
Cargo.toml            workspace unico de Rust (ver abajo)
pnpm-workspace.yaml   apps/web, apps/desktop, apps/vscode, packages/*, tools/*
justfile              punto de entrada unico
apps/api              la API: cinco crates, uno por capa
apps/desktop          frontend de Vite y src-tauri (crate git-rudder-desktop)
apps/vscode           extension host y webviews
apps/vscode-engine    sidecar JSON-RPC (crate git-rudder-vscode-engine)
apps/web              React Router
crates/               el motor cliente, compartido por los dos hosts
packages/             paquetes TS compartidos, eslint-config y tsconfig
tools/checks          scripts de Node sin dependencias que valen para los dos lenguajes
tools/rust-complexity cognitiva y ciclomatica de cada funcion de Rust (ADR 0005)
tools/docker          imagen de Linux con los requisitos de Tauri
tools/scripts         utilidades de shell para el justfile, los hooks y la CI
docs/                 arquitectura, contrato, modelo de datos y estas decisiones
```

Cada ecosistema conserva su toolchain y no hay orquestador que lo envuelva todo. El pegamento
es el `justfile`, los hooks de git y la CI.

### Un solo workspace de Cargo, no uno por app

Gastonomo tiene un workspace de Cargo dentro de `apps/api` porque solo tenia una pieza en
Rust. Git Rudder tiene tres binarios (`git-rudder-api`, `git-rudder-desktop`, `git-rudder-vscode-engine`) y
crates compartidos entre dos de ellos. Por eso hay un unico `Cargo.toml` en la raiz:

- `resolver = "3"` y `edition = "2024"` para todos los miembros, con Rust 1.98.1 fijado en
  `rust-toolchain.toml` (el motivo de fijarlo esta en el
  [ADR 0005](0005-enforcement-automatizado.md)).
- Miembros: `crates/*`, `apps/api/crates/*`, `apps/vscode-engine` y `apps/desktop/src-tauri`.
- Un solo `Cargo.lock` y un solo `target/`. La version de tokio o de serde no puede divergir
  entre el motor de escritorio y el de VS Code, y los dos tests de arquitectura
  ([ADR 0002](0002-arquitectura-hexagonal.md)) leen un unico lockfile.
- Las dependencias y los lints se declaran una vez, en `[workspace.dependencies]` y
  `[workspace.lints]`, y cada crate los hereda.

**El crate de Tauri es miembro, pero no esta en `default-members`.** Tauri enlaza con
webkit2gtk en Linux, y esa libreria no esta en todas las maquinas ni en los runners de la CI
que no son de escritorio. Con esa lista, `cargo test` y `cargo clippy` a secas compilan todo
menos `git-rudder-desktop` y funcionan en cualquier maquina. `cargo build -p git-rudder-api` no compila
ni una linea de Tauri.

La consecuencia es una regla: **ninguna receta usa `--workspace`**, porque esa bandera pisa
`default-members`. El crate de escritorio se compila con `-p git-rudder-desktop` en sus recetas
(`just desktop-rust-check`, `just desktop-build`) y en su job de la CI, que instala antes las
librerias de GTK.

Aunque no se compile por defecto, `git-rudder-desktop` aparece en `Cargo.lock` con todas sus
dependencias. Eso es lo que permite que el test de arquitectura del cliente compruebe sus
dependencias directas sin compilar Tauri.

### pnpm 11.18.0 por corepack y versiones exactas

- `packageManager: "pnpm@11.18.0"` en el `package.json` raiz y la misma version exacta en
  `pnpm/action-setup` de la CI. Las recetas y los hooks lo invocan por corepack. El pnpm
  global de una maquina puede ser otro: en la maquina de desarrollo hay un pnpm 9 instalado
  con npm que no cambia de version segun `packageManager` e ignora claves de pnpm 11 del
  workspace (`allowBuilds`), asi que su instalacion no seria equivalente. `just doctor` (lo
  corre `just setup`) falla si el pnpm que ve no es el 11.18.
- Existe pnpm 12.8.1, pero la 12 es una version mayor con cambios incompatibles publicada
  hace un mes. Se queda la 11.18.0, la misma que Gastonomo, para que las dos bases de codigo
  del mismo equipo se comporten igual. Subir a la 12 es un PR propio, con su prueba.
- Toda dependencia de npm lleva version exacta, sin `^` ni `~`. Los paquetes del workspace se
  declaran con `workspace:*`.
- Los paquetes internos se consumen como codigo fuente: su `exports` apunta a ficheros `.ts`,
  los transpila Vite o Vitest y los comprueba `tsc -b` con project references. No hay un paso
  de build de paquetes que haya que ordenar.

### just como punto de entrada unico

Un `justfile` en la raiz con recetas finas que delegan en cargo y pnpm, sin reimplementar
nada. Se agrupan por pieza (`api-*`, `engine-*`, `protocol-*`, `packages-*`, `web-*`,
`desktop-*`, `vscode-*`) y hay cuatro transversales:

| Receta                  | Para que                                                                                              |
| ----------------------- | ----------------------------------------------------------------------------------------------------- |
| `just setup`            | Instala dependencias y los hooks de lefthook                                                          |
| `just check`            | El enforcement de cada dia: formato, lint, guardas, tests unitarios, typecheck y cobertura            |
| `just ci`               | Lo mismo que la CI: `check` mas `cargo deny`, tipos generados al dia, e2e, builds y la pila compuesta |
| `just up` / `just down` | La pila de Docker de desarrollo ([ADR 0007](0007-docker-desarrollo-y-produccion.md))                  |

Los hooks los gestiona lefthook y los mensajes de commit los valida commitlint con
Conventional Commits. La CI llama a las mismas recetas: si una receta y la CI no hacen lo
mismo, manda la CI y se corrige la receta.

La CI tiene un unico workflow de entrada, `.github/workflows/ci.yml`, que decide que jobs
corren segun las rutas cambiadas (`dorny/paths-filter`) y deja un unico check requerido,
`all-green`.

### Sin orquestador, con una condicion para revisarlo

Ni Turborepo, ni Nx, ni Bazel, ni moon. Cargo ya ordena y cachea los builds de Rust, y pnpm
ya sabe filtrar lo afectado por un cambio (`pnpm --filter "...[origin/main]"`).

Este ADR se reabre si se cumple una de estas dos condiciones: el camino critico de la CI pasa
de unos 15 minutos, o algun paquete TS empieza a necesitar un build propio. Las dos son las
situaciones en las que una cache de tareas ahorra algo de verdad.

## Consecuencias

A favor:

- Un metodo nuevo del motor, con su tipo generado y la pantalla que lo usa, se ve en un solo
  commit, y la CI comprueba en ese mismo commit que los tipos generados estan al dia.
- `cargo test` funciona en una maquina sin webkit2gtk.
- Cada desarrollador sigue usando los comandos de su ecosistema: `cargo test -p git-rudder-git`
  funciona sin saber que existe un justfile.
- Dos lockfiles (`Cargo.lock` y `pnpm-lock.yaml`), no tres como en Gastonomo, y un flujo de
  Dependabot por ecosistema.
- Las reglas de calidad comunes viven en un sitio (`tools/checks`) y se aplican a los dos
  lenguajes.

En contra:

- Nada impide teclear `--workspace` a mano, salvo que las recetas no lo usan. Quien lo haga en
  Linux sin las librerias de GTK vera un error de enlace de webkit2gtk y tendra que saber por
  que.
- Un workspace unico recompila mas cuando cambia una dependencia comun: subir tokio recompila
  la API y los dos motores. Es el precio de que no puedan divergir.
- La CI tiene que filtrar por rutas para no compilar Tauri cuando solo cambia el CSS de la
  web, y el job de escritorio instala paquetes del sistema antes de compilar.
- No hay cache remota de tareas. Si la CI se vuelve lenta, este ADR se reabre con el numero en
  la mano.
- Quien clona se baja todo, aunque solo vaya a tocar la web.

## Alternativas descartadas

**Varios repositorios.** El motor lo comparten dos productos y el protocolo lo comparten Rust
y TypeScript. Cada cambio del contrato serian varios PRs publicados en orden, con una ventana
en la que las piezas publicadas no casan.

**Un workspace de Cargo por app, como Gastonomo.** Los crates del motor serian dependencias
por ruta de dos workspaces con dos lockfiles, que pueden resolver versiones distintas de la
misma dependencia. Los tests de arquitectura tendrian que leer varios lockfiles y el motor de
escritorio y el de VS Code podrian compilar con un tokio distinto sin que nadie lo notara.

**Turborepo.** Su valor es el grafo de tareas y la cache de builds de paquetes TS, y aqui los
paquetes se consumen como fuente: no hay build que cachear. Su soporte de Cargo es
experimental.

**Nx.** Pesado, con una configuracion por proyecto que choca con el modelo de recetas finas, y
su soporte de Rust son plugins de la comunidad.

**Bazel o moon.** Un toolchain mas que aprender y mantener para cinco piezas que ya tienen
build incremental propio.

**pnpm 12 desde el principio.** Mezclaria en el primer scaffold una migracion de version mayor
con todo lo demas. Va en su propio PR cuando se quiera dar el paso, idealmente a la vez en
Git Rudder y en Gastonomo.
