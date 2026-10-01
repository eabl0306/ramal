# ADR 0006: Estrategia de testing por app y umbrales de cobertura

- Estado: aceptado
- Fecha: 2026-09-30
- Revisado: 2026-10-01, para alinearlo con el codigo del primer corte

## Contexto

"Todo tan testeado como sea posible" no es una instruccion ejecutable. Hace falta decidir que se
prueba en cada nivel, con que herramienta y con que umbral, y que ese umbral rompa el build
cuando baja.

Ramal lo pone mas dificil que Gastonomo por tres motivos:

- Un cliente Git se equivoca de formas que solo aparecen con repositorios reales: nombres de
  rama con comas, merges octopus, rebases a medias, finales de linea CRLF, submodulos, punteros
  de LFS. Un fake del puerto no ve nada de eso.
- Hay dos hosts de escritorio que no son un navegador. Un webview de Tauri y un webview de VS
  Code necesitan su host para funcionar, y los E2E reales sobre ellos son lentos y fragiles.
- La misma vista (`@ramal/repo-view`) se pinta en dos temas, dos densidades y dos juegos de
  iconos.

La arquitectura hexagonal ayuda como en Gastonomo: los nucleos no tienen dependencias, asi que se
prueban sin mocks de framework, y los adaptadores se prueban contra dobles o contra la cosa
real segun lo que cueste.

## Decision

Una piramide por app. La base son tests unitarios de milisegundos; en medio, adaptadores contra
la cosa real y contratos; arriba, la mayoria de los flujos de interfaz en Playwright contra
builds de Vite con el host simulado, y una capa fina de E2E reales.

### El motor (compartido por escritorio y VS Code)

| Nivel                  | Herramienta                                                                        | Que                                                                                                                                                              |
| ---------------------- | ---------------------------------------------------------------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Dominio y casos de uso | `cargo test` con fakes del modulo `testing` y `block_on` sin runtime               | Disposicion del grafo (carriles, aristas, ventana), parseo de marcadores de conflicto, validacion del plan de rebase, cada caso de uso contra sus puertos falsos |
| Adaptador de git       | `cargo test` contra repositorios de prueba creados con el propio git en un tempdir | Cada lectura y escritura de `ramal-git` sobre repos reales, con snapshots de `insta`                                                                             |
| Contrato del motor     | `cargo test` de `dispatch` de punta a punta                                        | Metodo, parametros y resultado serializado contra repos de prueba                                                                                                |
| Framing del sidecar    | `cargo test` en `apps/vscode-engine`                                               | Marcos `Content-Length`, marcos de mas de 32 MiB, peticiones en paralelo, cierre de stdin                                                                        |
| Comandos de Tauri      | `tauri::test` (feature `test`)                                                     | Que los comandos estan registrados y serializan. Son de una linea; no hay mas que probar                                                                         |

**Repositorios de prueba dorados.** Un constructor en `crates/git/tests/support` crea cada
escenario con git real: conflicto de merge, merge octopus, submodulo, HEAD separado, rebase en
curso, ficheros renombrados, puntero de LFS, CRLF, nombres de ref con comas, repositorio vacio.
El entorno se fija para que el resultado sea identico en cualquier maquina: `HOME` apunta al
tempdir, `GIT_CONFIG_NOSYSTEM=1`, fechas de autor y de commit fijas, `LC_ALL=C`. Sin eso, la
configuracion global de quien corre el test se cuela en el resultado.

**Dos versiones de git en la CI.** La suite del motor corre contra git 2.34, la minima
([ADR 0009](0009-motor-git-hibrido.md)), y contra la ultima. Es la unica forma de que la
deteccion de capacidades se pruebe de verdad.

### Escritorio

| Nivel                           | Herramienta                                                                 | Que                                                                                                                                          |
| ------------------------------- | --------------------------------------------------------------------------- | -------------------------------------------------------------------------------------------------------------------------------------------- |
| Paquetes y componentes          | Vitest con Testing Library                                                  | `@ramal/domain`, `@ramal/repo-view`, pantallas propias                                                                                       |
| Flujos de interfaz (la mayoria) | Playwright contra el build de Vite con `mockIPC` de `@tauri-apps/api/mocks` | Grafo y seleccion, staging, reglas de la caja de commit, resolver de conflictos, rebase, menus contextuales, los dos idiomas y los dos temas |
| E2E real (pocos)                | WebdriverIO con `@wdio/tauri-service`                                       | El binario real sobre repos de prueba: abrir, preparar, hacer commit, merge con conflicto, resolver y push a un remoto desnudo local         |

El E2E real usa el driver embebido (`tauri-plugin-wdio-webdriver`), que es el unico que cubre
macOS. El plugin solo se compila detras de una feature de cargo que activan los builds de E2E:
el binario que se publica no lleva un servidor de WebDriver dentro. En Linux sin pantalla corre
bajo `xvfb-run` en los PRs que tocan escritorio o el motor. Windows y macOS iran cada noche
cuando exista ese workflow; hoy no hay ninguno.

### VS Code

| Nivel                                 | Herramienta                                                                            | Que                                                                                                                                                         |
| ------------------------------------- | -------------------------------------------------------------------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Capa de aplicacion del extension host | Vitest                                                                                 | Router de mensajes, supervision del sidecar, eleccion de repositorio, idioma. `application` no importa `vscode`, asi que no hace falta arrancar VS Code     |
| Validacion de mensajes                | Vitest                                                                                 | Mensajes malformados, metodos fuera de la superficie, ids fuera de rango, tamanos excesivos                                                                 |
| Flujos de interfaz                    | Playwright contra el bundle de webviews con un `acquireVsCodeApi` falso                | Los mismos flujos que escritorio en densidad compacta, con juegos de variables `--vscode-*` capturados de un tema oscuro, uno claro y uno de alto contraste |
| Integracion                           | `@vscode/test-cli` y `@vscode/test-electron`, bajo `xvfb-run -a` en Linux sin pantalla | Activacion, comandos, arranque y reinicio del sidecar, un repositorio de prueba real                                                                        |

Los tests de integracion esperan a eventos que expone la extension para los tests, nunca a un
`sleep`: con esperas por tiempo, estos tests son los primeros en volverse inestables.

### Web y API

| Nivel                                 | Herramienta                                                                                               | Que                                                                                                                             |
| ------------------------------------- | --------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------- |
| Dominio, aplicacion y componentes web | Vitest                                                                                                    | Validacion de formularios, casos de uso con puertos escritos a mano, pantallas con adaptadores en memoria                       |
| Nucleo de la API                      | `cargo test` con fakes y `block_on`                                                                       | Reglas de sesiones, rotacion de refresh tokens, roles, cuotas de IA, idempotencia                                               |
| Adaptadores de la API                 | `cargo test` contra PostgreSQL real ([ADR 0004](0004-postgresql-con-sqlx.md)) y `salvo::test::TestClient` | Repositorios, disparadores del registro de cambios, endpoints de punta a punta, cifrado de sobre y rotacion de la llave maestra |
| Contrato                              | Test del documento OpenAPI versionado y tipos generados sin diff                                          | Que `@ramal/api-client` y los tests de contrato de `ramal-cloud-client` ven la API real                                         |
| Flujos de interfaz                    | Playwright con la red simulada (`page.route`)                                                             | Landing con deteccion de SO y descargas, pantallas de auth, panel                                                               |
| Pila compuesta                        | `just stack-smoke` y Playwright contra la pila de Docker                                                  | La web, el gateway, la API y PostgreSQL juntos, sin simular nada                                                                |

### Criptografia

La boveda ([ADR 0012](0012-boveda-de-extremo-a-extremo.md)) y el cifrado de sobre
([ADR 0013](0013-proxy-de-ia-de-organizacion.md)) llevan vectores de prueba fijos (entrada,
llave, nonce y salida esperada), para que un cambio de crate o de version no cambie el formato
sin que nadie lo vea. Y tests negativos: un blob copiado a otra fila, a otro usuario o a otra
version de llave no descifra; unos parametros de Argon2id por debajo del minimo se rechazan.

### Tests de arquitectura

Los dos tests que leen `Cargo.lock` ([ADR 0002](0002-arquitectura-hexagonal.md)) son tests de
verdad y fallan como los demas.

### Que corre y cuando

| Momento                                      | Que                                                                                                                                                                                                                                                                                                                                             |
| -------------------------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `just check` y `pre-push`                    | Unitarios de Rust y de Vitest, tests de `tools/checks`, cobertura                                                                                                                                                                                                                                                                               |
| Cada PR (CI)                                 | Todo lo anterior, tests de PostgreSQL con `RAMAL_TEST_REQUIRE_DATABASE=1`, suite del motor con git 2.34 y con la ultima, Playwright de las tres apps con el host simulado, integracion de VS Code bajo xvfb, E2E real de escritorio en Linux si cambia escritorio o el motor, tipos generados sin diff y la pila compuesta con `smoke-stack.sh` |
| Cada noche y antes de publicar (planificado) | E2E real de escritorio en Windows y macOS, y Playwright de flujos completos contra la pila de Docker (registro, login, workspaces, cuentas ligadas con un proveedor falso, invitaciones)                                                                                                                                                        |

La pila compuesta va en cada PR, no a mano como acabo en Gastonomo: su ADR 0006 la dejo fuera de
la CI por lo que tardaba, y fue el unico test que habria cazado un prefijo del gateway olvidado
([ADR 0007](0007-docker-desarrollo-y-produccion.md)).

### Umbrales

| Ambito                                                                                                                                                         | Minimo        |
| -------------------------------------------------------------------------------------------------------------------------------------------------------------- | ------------- |
| Nucleos de Rust (`ramal-core-domain`, `ramal-core-application`, `ramal-api-domain`, `ramal-api-application`)                                                   | 80% de lineas |
| Nucleos de TypeScript (`@ramal/domain`, `@ramal/i18n`, `@ramal/bridge`, `domain` y `application` de la web, `application` del escritorio y del extension host) | 80% de lineas |
| Global de cada pieza                                                                                                                                           | 60% de lineas |

Herramientas: `cargo-llvm-cov` y la cobertura v8 de Vitest con umbrales por glob. Bajar del
umbral rompe la CI.

**Los composition root tambien se cubren.** Gastonomo no podia medir su crate binario porque
`cargo-llvm-cov` no genera datos para un binario que ningun test enlaza. En Ramal el bootstrap de
`ramal-api`, `ramal-vscode-engine` y `ramal-desktop` es una funcion de biblioteca y `main.rs` es
una llamada; un test de arranque en `tests/` la ejecuta con configuracion de prueba.

### Reglas

- Se prohibe apagar un test sin dejar escrito por que: `.only`, `.skip` sin motivo y
  `#[ignore]` sin `= "..."` rompen el build ([ADR 0005](0005-enforcement-automatizado.md)).
- Nombres: en Rust, frases en ingles en snake_case
  (`a_ref_name_with_commas_is_read_whole`); en TypeScript, `describe` e `it` en castellano.
- Un componente no se prueba con un fake que devuelve siempre el mismo objeto: en Gastonomo eso
  escondio un bucle de renderizado infinito que solo cazo el E2E.

## Consecuencias

A favor:

- Un fallo de una regla de Git o de negocio falla en un test de milisegundos, no en un E2E de
  minutos.
- Los fallos que solo dan los repositorios reales se cazan en el adaptador, con escenarios que
  se pueden reproducir en cualquier maquina.
- La mayoria de la interfaz se prueba en Playwright contra Vite, que es rapido y determinista, y
  la misma suite cubre temas, densidades e idiomas.
- La cobertura no es decorativa: bajar del umbral rompe la CI.

En contra:

- El E2E real de macOS depende de un plugin joven. Si falla, macOS queda cubierto solo por la
  capa de Playwright con `mockIPC`, y por eso esa capa es la gruesa.
- Solo el VSIX de `linux-x64` se prueba en la CI con VS Code real. Los otros siete destinos
  ([ADR 0010](0010-un-motor-para-escritorio-y-vscode.md)) se compilan pero no se ejecutan en su
  plataforma hasta la prueba manual de publicacion.
- Los tests de integracion de VS Code y los E2E de escritorio son lentos. Se mantienen pocos a
  proposito; cada flujo nuevo va primero a Playwright.
- Probar contra dos versiones de git duplica el tiempo de la suite del motor en la CI.
- Los flujos completos contra la pila de Docker no corren en cada PR: la pila compuesta si, pero
  solo con las sondas de `smoke-stack.sh`.

## Alternativas descartadas

**Mockear git en los tests del adaptador.** Probaria que el adaptador llama a git como se espera,
no que entiende lo que git devuelve, que es donde estan los fallos.

**E2E reales como capa principal de interfaz.** Lentos, dependientes del sistema y dificiles de
depurar. Con `mockIPC` y el `acquireVsCodeApi` falso, el 90% de los flujos se prueban en segundos.

**`wdio-vscode-service` o `vscode-extension-tester` para los webviews de VS Code.** Posibles para
dos o tres flujos que hagan clic dentro de un webview real; no entran en el primer corte porque
Playwright sobre el bundle cubre lo mismo mas rapido.

**tauri-driver externo.** Solo cubre Linux y Windows. El driver embebido de `@wdio/tauri-service`
cubre tambien macOS.

**msw 3 para simular la red de la web.** Salio el 28 de septiembre con cambios incompatibles;
`page.route` de Playwright basta para los flujos de interfaz. Si hiciera falta simular la red
en Vitest, se usaria la 2.x.
