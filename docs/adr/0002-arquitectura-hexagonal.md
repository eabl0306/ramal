# ADR 0002: Arquitectura hexagonal en todas las piezas

- Estado: aceptado
- Fecha: 2026-09-30
- Revisado: 2026-10-01, para alinearlo con el codigo del primer corte

## Contexto

Git Rudder va a crecer en cinco sitios a la vez: la API, el motor Git, el host de escritorio, la
extension de VS Code y la web, mas los paquetes TypeScript que comparten. La forma habitual de
que un proyecto asi se vuelva imposible de cambiar es que la logica acabe mezclada con el
framework de turno: una regla de negocio dentro de un handler de Salvo, un `git rebase`
lanzado desde un comando de Tauri, una validacion dentro de un componente de React. Cuando eso
pasa, probar obliga a levantar el framework entero y cambiar de framework obliga a reescribir
el negocio.

Git Rudder tiene ademas un motivo que Gastonomo no tenia. El mismo motor sirve a dos productos:
escritorio lo lleva en proceso y VS Code lo lanza como sidecar
([ADR 0010](0010-un-motor-para-escritorio-y-vscode.md)). Si una sola regla de Git viviera en
un comando de Tauri o en el extension host, habria que escribirla dos veces, y el protocolo
dejaria de ser el mismo en los dos productos. Tener una unica puerta de entrada al motor no es
una preferencia de estilo: es lo que hace que un producto no se desvie del otro.

## Decision

Arquitectura hexagonal (puertos y adaptadores) en todas las piezas, con el mismo vocabulario
que Gastonomo:

- **Nucleo:** dominio mas casos de uso. No conoce HTTP, ni SQL, ni React, ni Tauri, ni la API
  de VS Code, ni como se lanza un proceso.
- **Puertos:** traits de Rust o interfaces de TypeScript que declara la capa de aplicacion
  para hablar con el exterior.
- **Adaptadores driving:** quien invoca los casos de uso (handler de Salvo, `dispatch` del
  motor, comando de Tauri, bucle JSON-RPC, componente de React, comando de VS Code).
- **Adaptadores driven:** quien implementa los puertos (git CLI, SQLx, llavero, HTTP a las
  plataformas, fakes).
- **Composition root:** el unico sitio que construye adaptadores y cablea el grafo. Uno por
  proceso.

### Tres familias de hexagonos

| Pieza                                        | Nucleo                                                                                                                                                                                                                                                                                                                                                            | Driving                                                                                                           | Driven                                                                                                                                                        | Composition root                                                                |
| -------------------------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ----------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------- |
| API (`apps/api`)                             | `git-rudder-api-domain` y `git-rudder-api-application`                                                                                                                                                                                                                                                                                                            | `git-rudder-api-http` (router, catcher de problem+json y, mas adelante, idempotencia), trabajos periodicos (plan) | `git-rudder-api-infrastructure` (SQLx; luego cifrado de sobre, correo, OAuth)                                                                                 | `git-rudder-api`, en `src/bootstrap`                                            |
| Motor cliente (`crates/*`)                   | `git-rudder-core-domain` y `git-rudder-core-application`                                                                                                                                                                                                                                                                                                          | `git-rudder-engine` (`dispatch`)                                                                                  | `git-rudder-git`; mas adelante `git-rudder-keychain`, `git-rudder-vault`, `git-rudder-platforms`, `git-rudder-cloud-client` y el adaptador de `git-rudder-ai` | Dos niveles: `EngineBuilder` en `git-rudder-engine` y el bootstrap de cada host |
| Host de escritorio (`src-tauri`)             | ninguno propio                                                                                                                                                                                                                                                                                                                                                    | los comandos `rpc` y `repo_pick_and_open`                                                                         | el selector de carpetas sobre el dialogo nativo; mas adelante llavero y `EventSink`                                                                           | `src-tauri/src/bootstrap.rs`                                                    |
| Sidecar (`apps/vscode-engine`)               | ninguno propio                                                                                                                                                                                                                                                                                                                                                    | el bucle JSON-RPC                                                                                                 | peticiones inversas al extension host (secretos)                                                                                                              | `src/bootstrap.rs`                                                              |
| Extension host (`apps/vscode/src/extension`) | `application` (orquesta el sidecar, los webviews y el idioma)                                                                                                                                                                                                                                                                                                     | comandos y router de mensajes de webviews                                                                         | `infrastructure` (API de VS Code, proceso del sidecar)                                                                                                        | `app/composition-root.ts`                                                       |
| Web (`apps/web/src`)                         | `domain` y `application`                                                                                                                                                                                                                                                                                                                                          | `ui` y los modulos de ruta de `app`                                                                               | `infrastructure` (sobre `@git-rudder/api-client`)                                                                                                             | `app/composition-root.ts`                                                       |
| Frontend de escritorio y webviews            | sin dominio propio: los casos de uso viven en el motor y la logica de interfaz en `@git-rudder/domain`. En escritorio, `application` con puertos del host (`RepositoryPicker`, `AppearanceStore`) y el modelo de pestanas, para que la interfaz no importe infraestructura; en VS Code, `src/shared` con los tipos de los mensajes entre webview y extension host | `ui` y `app`                                                                                                      | `infrastructure` (transportes del puente y adaptadores de esos puertos)                                                                                       | `app/composition-root.ts`                                                       |

Y los paquetes compartidos tienen tambien un papel fijo: `@git-rudder/domain` e `@git-rudder/i18n` son
nucleo puro, sin dependencias; `@git-rudder/bridge` declara el puerto `Transport`;
`@git-rudder/repo-view` es un adaptador driving que comparten escritorio y VS Code; y
`@git-rudder/api-client` es una ayuda para los adaptadores driven de la web y solo lo importa su
`infrastructure`.

### Los dos nucleos de Rust no se conocen

El servidor nunca ve un commit y el cliente no gestiona organizaciones: no hay conceptos de
negocio que compartan de verdad. Ningun crate `git-rudder-api-*` alcanza un crate del cliente y
ningun crate del cliente alcanza un `git-rudder-api-*`. Lo unico que cruza son dos cosas:

- `git-rudder-ai` (planificado), una biblioteca neutral que no depende de ningun `git-rudder-*` y que
  usan `git-rudder-engine` y `git-rudder-api-infrastructure`, para que el prompt de una llave personal y
  el de una llave de organizacion salgan de las mismas plantillas
  ([ADR 0013](0013-proxy-de-ia-de-organizacion.md)).
- El contrato HTTP, que cada lado modela con sus propios tipos. `git-rudder-cloud-client` no
  importa los DTOs de la API: se protege con tests de contrato contra `apps/api/openapi.json`.

### Los casos de uso del cliente viven en Rust, no en los webviews

Los frontends de escritorio y los webviews de VS Code son casi solo adaptadores driving. Los
casos de uso de Git, cuentas, IA y sincronizacion estan en `git-rudder-core-application`, y la
disposicion del grafo se calcula en `git-rudder-core-domain`. Lo que si vive en TypeScript es la
logica pura de estado de interfaz, en `@git-rudder/domain`: el modelo interactivo del resolver de
conflictos, el editor de rebase, filtros y agrupaciones, tiempo relativo, reglas del mensaje
de commit.

Hay una duplicacion aceptada a proposito: el editor de rebase valida el plan en TypeScript
para responder al instante, y el motor lo vuelve a validar antes de ejecutarlo. La validacion
de Rust es la que manda; la de TypeScript es comodidad.

### Nada de serde en el dominio

El dominio del cliente no depende de serde: no sabe que existe JSON. La forma del JSON la
decide `git-rudder-protocol`, que a su vez no depende de ningun otro crate de Git Rudder, y el mapeo
entre dominio y DTO vive en `git-rudder-engine` como funciones libres (la regla de huerfanos de
Rust no deja implementar `From` entre dos tipos que no son del crate).

En la API, `serde` se controla con la lista blanca de dependencias directas del dominio, igual
que en Gastonomo: `Cargo.lock` lista la dependencia opcional de `chrono` con `serde` en cuanto
otro crate del workspace activa esa feature, asi que no se puede prohibir en el cierre
transitivo.

### Los hosts son finos

El comando `rpc` de Tauri y el bucle del sidecar deserializan, llaman a `Engine::handle` y
serializan. No hay un comando de Tauri por metodo: la lista de metodos vive en
`git-rudder-protocol`. La composicion del cliente tiene dos niveles: `EngineBuilder` construye lo
que es igual en los dos productos (git CLI y, mas adelante, plataformas, IA y Git Rudder API) y
cada host aporta lo que solo el sabe construir, como el `SecretStore` (llavero del sistema en
escritorio, SecretStorage a traves del extension host en VS Code) o el `EventSink` de las
notificaciones.

### La direccion de las dependencias se fuerza, no se documenta

| Regla                                 | Rust                                                                                                              | TypeScript                                                                                    |
| ------------------------------------- | ----------------------------------------------------------------------------------------------------------------- | --------------------------------------------------------------------------------------------- |
| Capas dentro de una pieza             | Un crate por capa, y el compilador impide nombrar lo que no es dependencia                                        | Una carpeta por capa y `eslint-plugin-boundaries` con `default: "disallow"`                   |
| Dependencias externas por capa        | Tests de arquitectura que leen `Cargo.lock`: lista blanca directa en los nucleos, lista negra transitiva en todos | Politicas de `boundaries` para origenes externos y del core de Node                           |
| Dependencias entre piezas             | Los mismos tests comprueban que ninguna familia alcanza a la otra salvo `git-rudder-ai`                           | `tools/checks/package-graph.mjs` sobre los `package.json`, y project references de TypeScript |
| APIs estandar prohibidas en el nucleo | `clippy.toml` propio en cada crate de nucleo con `disallowed-methods` y `disallowed-types`                        | `node:*` prohibido en `domain` y `application`                                                |
| Crates vetados                        | `cargo deny check bans`                                                                                           | no aplica                                                                                     |

Hay dos tests de arquitectura, uno por hexagono, en el crate que hace de composition root de
cada uno: `apps/api/crates/api/tests/architecture.rs` y `crates/engine/tests/architecture.rs`.
Leen el `Cargo.lock` de la raiz linea a linea, sin crate de TOML, como en Gastonomo, y fallan
con un mensaje que dice que crate alcanza que. El del cliente comprueba ademas que los dos
hosts (`git-rudder-desktop` y `git-rudder-vscode-engine`) solo dependen del motor y del protocolo.

Prohibir `tokio` no basta para que el nucleo no haga IO, porque `std::process::Command` y
`std::fs` estan en la biblioteca estandar. Por eso cada crate de nucleo tiene su `clippy.toml`
con `disallowed-types` y `disallowed-methods` que cubren cada puerta de entrada, no solo la mas
comun: lanzar procesos y terminarlos (`Command`, `process::exit`), todo `std::fs` y los metodos
de `Path` que tocan el disco, sockets TCP, UDP y Unix, resolver nombres, el entorno
(`std::env::var`, `args`, `current_dir`...), `stdin` y `stdout`, la hora de pared
(`SystemTime`) y los hilos (`thread::spawn`, `thread::sleep`). La hora llega siempre por el
puerto `Clock`. Clippy usa el `clippy.toml` mas cercano sin fusionarlo con el de la raiz, asi
que esos ficheros repiten los umbrales comunes. `crates/engine/tests/core_purity.rs` comprueba
que los cuatro nucleos prohiben lo mismo, que repiten los umbrales de la raiz y que su codigo no
nombra los modulos de IO; el test de arquitectura de la API repite la comprobacion de sus dos
nucleos.

En TypeScript no hay compilador que ponga la barrera, asi que la pone el linter, y las project
references son una segunda red mas tosca: importar codigo de un paquete no referenciado hace
fallar `tsc -b` aunque alguien haya configurado mal ESLint.

## Consecuencias

A favor:

- Los nucleos se prueban sin levantar nada: los casos de uso de Rust corren con fakes y un
  ejecutor sin runtime (`block_on` con `Waker::noop`), sin tokio ni red.
- El protocolo es identico en escritorio y en VS Code por construccion, porque los dos hosts
  entran por la misma funcion.
- Cambiar un adaptador es cambiar el composition root. Pasar una lectura concreta del git CLI a
  `gix` ([ADR 0009](0009-motor-git-hibrido.md)) toca `git-rudder-git` y `EngineBuilder`, nada mas.
- La conversacion de code review deja de ser "esto deberia ir en otro sitio", porque la
  herramienta lo dice antes.

En contra:

- Mas ficheros y mas indireccion. `GraphWindow` existe tres veces: en el dominio, como DTO de
  `git-rudder-protocol` y como tipo generado de TypeScript, con un mapeador entre los dos primeros.
  A simple vista parece duplicado y no lo es: es lo que permite cambiar un tipo interno sin
  cambiar el contrato.
- El mapeo con funciones libres es mas verboso que un `impl From`.
- La validacion del plan de rebase existe dos veces, en TypeScript y en Rust, y hay que
  mantenerlas de acuerdo.
- Los `clippy.toml` de los nucleos repiten los umbrales de la raiz. El test de arquitectura lo
  vigila, pero son cuatro ficheros mas que tocar si un umbral cambia.
- En TypeScript la barrera depende de que la configuracion de ESLint este bien. Se mitiga con
  las project references y comprobando en caliente que la regla salta.

## Alternativas descartadas

**Logica en los hosts.** Escribir las operaciones de Git en comandos de Tauri y, para VS Code,
en el extension host. Duplica todo y garantiza que los dos productos acaben comportandose
distinto.

**Un dominio en TypeScript para todo, incluido Git.** Obliga a mandar el historial crudo por
IPC antes del primer pintado del grafo y a escribir en TypeScript un adaptador de git para VS
Code. Es la opcion que el [ADR 0010](0010-un-motor-para-escritorio-y-vscode.md) descarta.

**Un crate por pieza con modulos por capa.** Los modulos de Rust no ponen barrera: cualquier
modulo puede importar a cualquier otro del mismo crate. La separacion quedaria en convencion.

**dependency-cruiser en TypeScript.** Cubre lo mismo que `eslint-plugin-boundaries`, que ya se
usa en Gastonomo y corre dentro del lint. Seria otra herramienta mas para la misma regla.
