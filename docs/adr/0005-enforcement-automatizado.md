# ADR 0005: El enforcement de calidad es automatico y bloqueante

- Estado: aceptado
- Fecha: 2026-09-30

## Contexto

Las reglas de estilo que solo viven en un documento se incumplen. Las que solo comprueba un
reviewer humano se incumplen mas despacio. La unica regla que se cumple siempre es la que impide
compilar, commitear o mergear. Gastonomo lo aprendio y lo dejo en su ADR 0005; Ramal parte de
ahi con dos diferencias: hay mas piezas (dos lenguajes en cinco apps y seis paquetes) y varios
implementadores trabajando en paralelo, asi que la herramienta es la unica forma de que todos
escriban igual.

Los limites son los mismos que Gastonomo, y cambiarlos es una decision de equipo:

- Fichero de codigo de hasta 300 lineas; fichero de test de hasta 400.
- Complejidad cognitiva de hasta 15; ciclomatica de hasta 10.
- Funciones de hasta 50 lineas, hasta 5 parametros y anidamiento de hasta 4 niveles.
- Nada de codigo muerto, imports sin usar ni variables sin usar.
- Nada de marcadores de trabajo pendiente sin ticket.
- Nada de tests enfocados ni de tests apagados sin motivo escrito.
- Cobertura minima del 80% en los nucleos y del 60% global.
- Los avisos son errores.

## Decision

Cada limite se traduce a la herramienta nativa de cada lenguaje. Lo que ninguna cubre, o lo que
tiene que valer igual para los dos, lo cubren scripts de Node sin dependencias en
`tools/checks`.

| Regla                        | Rust                                                                    | TypeScript                                             | CSS                                                                                                                                    |
| ---------------------------- | ----------------------------------------------------------------------- | ------------------------------------------------------ | -------------------------------------------------------------------------------------------------------------------------------------- |
| Formato                      | `cargo fmt --check`                                                     | prettier                                               | prettier                                                                                                                               |
| Complejidad ciclomatica      | no se fuerza (ver abajo)                                                | `complexity`                                           | no aplica                                                                                                                              |
| Complejidad cognitiva        | `clippy::cognitive_complexity`                                          | `sonarjs/cognitive-complexity`                         | no aplica                                                                                                                              |
| Funcion de 50 lineas         | `clippy::too_many_lines`                                                | `max-lines-per-function`                               | no aplica                                                                                                                              |
| 5 parametros                 | `clippy::too_many_arguments`                                            | `max-params`                                           | no aplica                                                                                                                              |
| Anidamiento de 4             | `clippy::excessive_nesting` con su umbral                               | `max-depth`                                            | no aplica                                                                                                                              |
| Tamano de fichero            | `tools/checks/file-size.mjs`                                            | el mismo script, mas `max-lines`                       | el mismo script                                                                                                                        |
| Codigo muerto                | `dead_code` y `unused_*` con `-D warnings`, `unused_crate_dependencies` | `no-unused-vars` y knip                                | no aplica                                                                                                                              |
| Trabajo pendiente con ticket | `tools/checks/todo-without-ticket.mjs`                                  | el mismo script, mas la regla `ramal/ticketed-todo`    | el mismo script                                                                                                                        |
| Tests enfocados o apagados   | `tools/checks/no-focused-tests.mjs` (`#[ignore]` sin motivo)            | el mismo script (`.only`, `.skip` sin motivo)          | no aplica                                                                                                                              |
| Colores escritos a mano      | no aplica                                                               | `ramal/no-raw-colors`                                  | stylelint: `declaration-strict-value`, `color-no-hex`, `color-named` y funciones de color prohibidas fuera de `packages/ui/src/themes` |
| Texto de interfaz literal    | no aplica                                                               | `ramal/no-jsx-literal-text`                            | no aplica                                                                                                                              |
| Capas hexagonales            | Tests de arquitectura sobre `Cargo.lock` y `clippy.toml` por nucleo     | `eslint-plugin-boundaries`                             | no aplica                                                                                                                              |
| Grafo de paquetes            | Tests de cruce de hexagonos                                             | `tools/checks/package-graph.mjs` y project references  | no aplica                                                                                                                              |
| Crates vetados               | `cargo deny check` (licencias, avisos de seguridad, bans)               | no aplica                                              | no aplica                                                                                                                              |
| Tipos generados al dia       | `just protocol-check` y el test de `openapi.json`                       | `git diff --exit-code` sobre las carpetas `generated/` | no aplica                                                                                                                              |
| Cobertura                    | `cargo-llvm-cov` con umbral                                             | cobertura v8 de Vitest con umbrales por glob           | no aplica                                                                                                                              |

### Reglas propias de Rust que conviene saber

Ademas de los umbrales, el `[workspace.lints]` de la raiz fija:

- `unsafe_code = "forbid"` en todo el workspace.
- `unwrap_used`, `expect_used`, `panic`, `todo`, `unimplemented`, `dbg_macro` e
  `indexing_slicing` denegados: un fallo se modela como error, no como panico.
- `print_stdout` y `print_stderr` denegados. En el sidecar de VS Code stdout es solo protocolo
  ([ADR 0010](0010-un-motor-para-escritorio-y-vscode.md)), y un `println!` olvidado rompe la
  conexion con el extension host. Todo el log va por `tracing`.
- `clippy::all` denegado y `pedantic` y `nursery` como avisos, que con `-D warnings` bloquean
  igual. Solo se permiten tres: `module_name_repetitions`, `multiple_crate_versions` y
  `doc_markdown` (la documentacion va en castellano y cita nombres de producto).
- `missing_docs` como aviso: todo elemento publico lleva `///` en castellano.
- `excessive_nesting` es un lint de la familia restriction: no entra por `all` ni por
  `pedantic` y hay que pedirlo. Sin esa linea, el umbral de `clippy.toml` no hace nada.

Los umbrales viven en `clippy.toml` en la raiz. Cada crate de nucleo tiene ademas el suyo, con
las APIs de la biblioteca estandar prohibidas ([ADR 0002](0002-arquitectura-hexagonal.md)), y
como clippy no fusiona ficheros, repite los umbrales; el test de arquitectura comprueba que
coinciden con los de la raiz.

### El formato del ticket

Un marcador de trabajo pendiente solo vale con ticket: una tarjeta del tablero (`RAM-123`) o una
issue de GitHub (`#123`), entre parentesis justo detras de la palabra. El script distingue
mayusculas porque "todo" es una palabra normal en castellano y no puede saltar en un comentario
que la use.

### La ciclomatica no se fuerza en Rust

Clippy no tiene lint de complejidad ciclomatica: el unico que existe es
`clippy::cognitive_complexity`, que mide la cognitiva. Gastonomo comprobo que una funcion con
ciclomatica 32 (un `match` plano de 31 ramas) pasa `cargo clippy -- -D warnings` limpia.

Lo que la acota en la practica es la combinacion de `too_many_lines` (50), `too_many_arguments`
(5), `cognitive_complexity` (15) y `excessive_nesting` (4): un `match` gigante se cuela, pero
casi nada mas. En Ramal hay un sitio donde eso va a tentar, la tabla metodo a handler de
`ramal-engine`, y por eso `dispatch` se parte por area (`dispatch/graph.rs`, `dispatch/repo.rs`)
desde el primer dia. Forzarla de verdad pediria rust-code-analysis o lizard, otro toolchain en
el repositorio, asi que se asume el hueco y se documenta en vez de afirmar lo contrario.

### El toolchain de Rust esta fijado

Gastonomo usa `stable` en `rust-toolchain.toml` y acepta que un clippy nuevo rompa la CI sin
que nadie toque el codigo. Ramal fija la version exacta (1.98.1) por dos motivos: con varios
implementadores en paralelo, un lint nuevo que aparece a mitad de semana rompe el trabajo de
todos a la vez; y la maquina de desarrollo, la CI y las imagenes de Docker tienen que compilar
con el mismo rustc. Se sube a mano o con Dependabot, en un PR propio que arregla lo que traigan
los lints nuevos y mueve a la vez el `ARG RUST_VERSION` de los Dockerfile.

### Tres momentos

1. **En local**, `just check` corre el mismo enforcement que la CI salvo lo caro. `cargo deny`,
   la comprobacion de tipos generados, los e2e, los builds y la pila compuesta van en `just ci`.
   Antes de abrir un PR se corre `just ci`.
2. **En cada commit**, lefthook bloquea. `pre-commit` mira solo lo que esta en el index: arregla
   (ESLint, stylelint, rustfmt y prettier con autofix) y comprueba (tamano de fichero, tests
   enfocados, marcadores sin ticket, grafo de paquetes y un escaner de secretos y ficheros
   enormes). `commit-msg` valida el mensaje con commitlint. `pre-push` corre clippy con `-D
warnings` y los tests de Rust, el typecheck y los tests de TypeScript, y los tests de
   `tools/checks`.
3. **En cada push y cada PR**, GitHub Actions repite todo y deja un unico check requerido,
   `all-green`.

Los avisos son errores en todas partes: `-D warnings` en Rust, `--max-warnings 0` en ESLint y
en stylelint.

### Lo que queda fuera del control de tamano y del lint

El codigo generado (`packages/bridge/src/generated`, `packages/api-client/src/generated`) esta
fuera de ESLint, de knip y del control de tamano: nadie lo escribe a mano y su frescura la
comprueba la CI regenerandolo. Las migraciones SQL tambien quedan fuera del control de tamano.

## Consecuencias

A favor:

- El codigo que llega a `main` cumple, sin depender de la memoria de nadie ni de que todos los
  implementadores hayan leido lo mismo.
- El reviewer habla de diseno, no de estilo.
- Las reglas estan escritas una vez y se ejecutan en tres sitios.
- Los temas y el idioma no se pueden romper por descuido: un color escrito a mano o un texto
  literal en un componente no pasan el lint.

En contra:

- Los hooks alargan el commit. Por eso `pre-commit` solo mira lo que esta en el index y lo caro
  vive en `pre-push`, que se puede saltar con `LEFTHOOK=0` cuando de verdad hace falta.
- El toolchain fijado no se actualiza solo: si nadie lo sube, se queda atras. Dependabot abre
  el PR; alguien tiene que arreglar los lints nuevos.
- `pedantic`, `nursery` e `indexing_slicing` hacen el Rust mas verboso (`.get()` en vez de
  `[]`, conversiones explicitas). Se acepta: es verbosidad que evita panicos.
- Las reglas de tamano se miden en lineas fisicas en el script comun y saltando blancos y
  comentarios en ESLint. El script es el techo comun.
- La ciclomatica en Rust es un hueco documentado, no cerrado.

## Alternativas descartadas

**Reglas solo en el documento y en la revision.** Es lo que el contexto explica que no
funciona.

**rust-code-analysis o lizard para la ciclomatica de Rust.** Cerrarian el hueco a cambio de otro
toolchain que instalar en cada maquina y en la CI, para un caso que las otras cuatro reglas ya
acotan casi del todo.

**`stable` en `rust-toolchain.toml`, como Gastonomo.** Ver arriba: con varios implementadores
en paralelo, que la CI se rompa sola cuesta mas que subir la version a mano.

**Un linter por app sin scripts comunes.** Cada ecosistema cuenta las lineas a su manera y cada
uno tiene su sintaxis de tests enfocados; la regla dejaria de ser la misma en los dos lenguajes.
