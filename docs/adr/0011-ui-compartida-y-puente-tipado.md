# ADR 0011: Interfaz compartida por tokens semanticos y puente tipado con transportes

- Estado: aceptado
- Fecha: 2026-09-30
- Revisado: 2026-10-01, para alinearlo con el codigo del primer corte

## Contexto

La vista de repositorio (grafo, fila WIP, detalle de commit, diff, resolver de conflictos, editor
de rebase) se pinta en dos hosts muy distintos:

- **Git Rudder Desktop**, en el webview de Tauri: IBM Plex, colores oklch del diseno, tema claro y
  oscuro, filas del grafo de 40 px, elementos de menu de 32 px, iconos de Lucide.
- **Git Rudder for VS Code**, en webviews: sigue el tema del editor (incluidos los de alto contraste),
  usa la fuente del sistema, filas de 28 px, elementos de menu de 24 px, iconos Codicons, y tiene
  que leer las variables `--vscode-*`, no colores propios.

Y la web usa las mismas primitivas (botones, menus, modales, tablas) con su propia escala de
texto.

Hay tres restricciones que condicionan la decision:

- **CSP estricta en los tres sitios.** El webview de escritorio no puede hablar con la red y los
  webviews de VS Code llevan `connect-src 'none'`. Cualquier biblioteca que inyecte etiquetas
  `<style>` en tiempo de ejecucion choca con un `style-src` sin `'unsafe-inline'`.
- **El webview no ve tokens ni envia rutas.** Todo lo que sale de la interfaz pasa por el motor
  ([ADR 0010](0010-un-motor-para-escritorio-y-vscode.md)), y el motor trata al webview como un
  cliente que puede estar comprometido: pinta contenido que no es de fiar (mensajes de commit,
  nombres de rama, ficheros).
- **El contrato entre la interfaz y el motor lo define Rust.** Si los tipos de TypeScript se
  escriben a mano, se desincronizan.

El toolkit oficial de VS Code para webviews (`@vscode/webview-ui-toolkit`) esta archivado desde
enero de 2025 y npm lo marca como obsoleto.

## Decision

### Un paquete de interfaz tematizado solo con tokens semanticos

`@git-rudder/ui` contiene tokens, temas, densidades, primitivas y el proveedor de iconos. Los
componentes solo leen tokens semanticos, nunca un color literal:

| Grupo                        | Tokens                                                                                                                                                              |
| ---------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Superficies                  | `--r-bg`, `--r-panel`, `--r-surface`, `--r-surface-hover`, `--r-border`                                                                                             |
| Texto                        | `--r-text`, `--r-muted`, `--r-faint`                                                                                                                                |
| Acento, seleccion y estados  | `--r-accent`, `--r-accent-ink`, `--r-accent-soft`, `--r-selection`, `--r-selection-ink`, `--r-ok`, `--r-warn`, `--r-danger`, `--r-info`, `--r-merged` y sus `-soft` |
| Diff y estado de fichero     | `--r-add-bg`, `--r-del-bg`, `--r-status-m`, `--r-status-a`, `--r-status-d`, `--r-status-r`, `--r-status-c`                                                          |
| Carriles del grafo           | `--r-lane-0` a `--r-lane-5`, `--r-on-lane`                                                                                                                          |
| Interaccion y alto contraste | `--r-focus`, `--r-backdrop`, `--r-knob`, `--r-hatch`, `--r-on-emphasis`, `--r-hc-border`, `--r-hc-active-border`                                                    |
| Marcas                       | `--r-platform-github`, `-gitlab`, `-bitbucket`, `-gitea`, `-azure`, `--r-provider-google`                                                                           |
| Tipografia, forma y densidad | `--r-font-ui`, `--r-font-code`, `--r-font-size-base`, radios, sombra, `--r-row-h`, `--r-menu-item-h`, `--r-sidebar-item-h`, `--r-toolbar-h`                         |

Dos ficheros de tema dan valor a esos tokens:

- `themes/git-rudder.css`: los valores oklch del diseno para `[data-theme="dark"]` y
  `[data-theme="light"]`. Lo usan escritorio y web; la letra de 14 px y el claro de la web van en
  el mismo fichero bajo `[data-host="web"]`. Los tokens que no cambian con el tema (carriles,
  marcas, `-soft`, alto contraste, densidades) viven en `themes/base.css`.
- `themes/vscode.css`: cada token apunta a una variable de VS Code
  (`--r-bg: var(--vscode-editor-background)`,
  `--r-status-m: var(--vscode-gitDecoration-modifiedResourceForeground)`, y asi con el resto).
  El alto contraste se trata aparte, con bordes de `--vscode-contrastBorder`. El tema cambia solo
  cuando cambia el de VS Code, sin codigo.

Los colores de carril son los mismos en todos los temas, como pide el diseno.

**La regla se hace cumplir**: stylelint prohibe colores hexadecimales, nombres de color y
funciones de color fuera de `packages/ui/src/themes`, y la regla `git-rudder/no-raw-colors` hace lo
mismo en TypeScript ([ADR 0005](0005-enforcement-automatizado.md)).

### Dos densidades, sin ramas por host

`[data-density="comfortable"]` (escritorio y web) y `[data-density="compact"]` (VS Code) cambian
los tokens de tamano: 40 o 28 px de fila, 32 o 24 px de menu, 30 o 22 px de barra lateral, 62 o
34 px de barra de herramientas. El SVG del grafo necesita esos numeros en JavaScript, asi que
`@git-rudder/ui` exporta `DENSITIES`, y un test parsea el CSS y falla si los dos valores difieren.

Las diferencias entre escritorio y VS Code son tokens, densidad e iconos. **No hay ramas
`if (vscode)` en los componentes.**

### Iconos por proveedor

`<Icon name="branch" />` con `IconName` como union cerrada de nombres semanticos. El host elige el
juego en su composition root: `@git-rudder/ui/icons/lucide` (escritorio y web) o
`@git-rudder/ui/icons/codicons` (VS Code). Cada juego es un `Record<IconName, ...>`, asi que un icono
nuevo sin equivalente en uno de los dos no compila, y al ser subrutas distintas Lucide nunca entra
en el bundle de VS Code ni Codicons en el de escritorio. Los glifos Unicode del prototipo no se
usan.

### Primitivas accesibles con React Aria Components

Menus contextuales con submenus que se quedan abiertos en la seleccion multiple, dialogos con foco
atrapado, selects, busqueda por teclado dentro de una lista: escribir eso bien a mano es caro. Las
primitivas de `@git-rudder/ui` se construyen sobre **React Aria Components** (`react-aria-components`,
de Adobe), que da el comportamiento y la accesibilidad sin estilos.

**Por que React Aria y no Radix.** Los dos son buenos en accesibilidad. La diferencia es la CSP:
el bloqueo de scroll de los dialogos de Radix (`react-remove-scroll`) inyecta una etiqueta
`<style>` en tiempo de ejecucion, que una CSP con `style-src` sin `'unsafe-inline'` ni nonce
bloquea. React Aria aplica casi todos sus estilos por las propiedades `style` de React, que van
por CSSOM y la CSP no bloquea. Ademas trae submenus en su `Menu`, que el diseno usa en todos los
menus contextuales.

Hay una excepcion, descubierta al implementar: `react-aria-components` 1.21 inyecta una etiqueta
`<style>` fija (`react-aria-pressable-style`, con el `touch-action` de lo pulsable). En escritorio
se permite ese texto exacto por su hash en `style-src`, nunca con `'unsafe-inline'`, y el E2E de
Playwright falla con cualquier violacion de la CSP, asi que si React Aria lo cambia se ve. Los
webviews de VS Code se topan con el mismo bloqueo y su CSP todavia no lleva el hash: alli el
navegador descarta esa regla.

Solo `@git-rudder/ui` importa `react-aria-components`; las apps y `@git-rudder/repo-view` usan las
primitivas de `@git-rudder/ui`. La version exacta se fija al montar el paquete, como el resto de
dependencias de npm.

### CSS Modules, sin CSS-in-JS en tiempo de ejecucion

Cada componente tiene su `*.module.css`, que solo usa tokens. Coste cero en ejecucion, lo entiende
Vite en los tres hosts y no choca con ninguna CSP. Las fuentes van empaquetadas
(`@fontsource/ibm-plex-sans` y `-mono`), nunca desde Google Fonts: ni la CSP ni el uso sin
conexion lo permitirian.

Listas largas (filas del grafo, lineas de diff, paneles del resolver) con `@tanstack/react-virtual`,
y el estado que viene del motor con `@tanstack/react-query`, con las mismas claves e invalidacion
en los dos hosts.

### Superficies nativas de VS Code, no clones

La paleta de comandos, los dialogos (`showInputBox`, `showQuickPick`, avisos modales), las
notificaciones, la barra de estado, las vistas de arbol (ramas, remotos, PRs, stashes, tags), el
diff (`vscode.diff`) y los ajustes escalares (`contributes.configuration`) son de VS Code y se
implementan con su API desde el extension host. En React solo va lo que VS Code no sabe dibujar:
el grafo, la caja de commit, el resolver de merge como editor personalizado, el formulario de PR y
las listas de los ajustes.

`@git-rudder/repo-view` pide esas capacidades al host a traves de un puerto, `RepoViewHost` (confirmar,
pedir un texto, elegir de una lista, avisar, abrir un fichero o un diff). En escritorio lo
implementan modales y toasts de `@git-rudder/ui`; en VS Code, mensajes al extension host que llama a la
API nativa. Como una funcion no cruza `postMessage`, una peticion de texto no lleva una funcion de
validacion sino un valor declarativo (`"branch-name"`, `"tag-name"`, `"non-empty"`), y cada host
aplica el validador de `@git-rudder/domain` que corresponde.

### El puente tipado

`@git-rudder/bridge` no tiene dependencias en tiempo de ejecucion y contiene:

- `Transport`, el puerto: mandar un `RpcRequest`, recibir la respuesta, suscribirse a
  notificaciones.
- `createBridgeClient(transport)`: `client.call("graph/window", params, { signal, timeoutMs })`
  totalmente tipado por el mapa de metodos generado. 30 segundos por defecto; las operaciones
  largas informan por `$/progress` y no usan timeout. Cancelar con `signal` rechaza con
  `signal.reason`, como `fetch`, y manda `$/cancel`; todavia no hay un tipo `cancelled` y el motor
  del primer corte ignora `$/cancel`.
- `BridgeError { kind, message, data }`, donde `kind` es el `ErrorKind` cerrado del motor mas tres
  que anaden el puente y el extension host: `transport`, `timeout` y `engine_unavailable`.

Los transportes concretos viven en cada app, no en el paquete:

- `TauriTransport` en `apps/desktop`: `invoke("rpc", { request })` y un `Channel` para las
  notificaciones. `invoke` solo rechaza por fallos de transporte; un error del motor llega como
  respuesta con `error`.
- `VsCodeTransport` en `apps/vscode/src/webview`: `acquireVsCodeApi().postMessage` y el evento
  `message`.

El webview nunca envia rutas del sistema de ficheros: en VS Code, el extension host abre el
repositorio y pasa al webview solo su `repoId`; en escritorio, el comando de Tauri
`repo_pick_and_open` ensena el dialogo nativo desde Rust y devuelve el `RepoHandle`. Los dos hosts
filtran lo que el webview puede pedir con una lista de metodos por superficie o por ventana,
escrita con los nombres que salen de `git-rudder-protocol` (en VS Code tipada contra la lista generada;
en escritorio, `surfaces.rs` con los marcadores de `define_methods!`). Los metodos marcados como
del host (`repo/open`) no se aceptan nunca de un webview, y el motor lo vuelve a comprobar. Ningun metodo acepta rutas absolutas, una ruta relativa con `..` o NUL, o
que salga del arbol de trabajo despues de seguir enlaces simbolicos, se rechaza, y ninguno
devuelve material secreto a un webview.

### Tipos generados desde Rust con ts-rs

- Cada DTO de `git-rudder-protocol` deriva `Serialize`, `Deserialize` y `TS`. ts-rs 12 respeta los
  atributos de serde, asi que el TypeScript coincide con el JSON real.
- `TS_RS_EXPORT_DIR` apunta a `packages/bridge/src/generated`. ts-rs no genera valores ni el mapa
  de metodos, asi que la macro `define_methods!` declara cada metodo una vez y un test escribe
  `methods.ts` (el tipo `EngineMethods`, la constante `ENGINE_METHODS` que el extension host usa
  como lista blanca y `PROTOCOL_VERSION`) e `index.ts`.
- Reglas del contrato: campos en camelCase, `deny_unknown_fields` en los parametros, nada de
  enteros de 64 bits (ts-rs los traduce a `bigint` y romperian el JSON normal), instantes como
  texto ISO 8601.
- **Lo generado se commitea**, para que el TypeScript compile sin cargo. `just protocol-gen`
  regenera y `just protocol-check` regenera y falla si `git diff` no esta limpio; la CI lo corre en
  cada PR. `@git-rudder/api-client` sigue el mismo patron con `apps/api/openapi.json` como fuente.

### CSP

| Host                | Politica                                                                                                                                                                                                                                                                                                                        |
| ------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Tauri               | `default-src 'none'`, `script-src 'self'`, `style-src 'self'` mas el hash del `<style>` de React Aria, `font-src 'self'`, `img-src 'self' data:`, `connect-src ipc: http://ipc.localhost`, `frame-src 'none'`, `object-src 'none'`, `base-uri 'none'`, `form-action 'none'`; `freezePrototype: true` y `withGlobalTauri: false` |
| Webviews de VS Code | `default-src 'none'`, `script-src 'nonce-...'` con un nonce nuevo en cada carga, `style-src`, `font-src` e `img-src` limitados a `webview.cspSource` (mas `data:` en imagenes), `connect-src 'none'`, `frame-src 'none'`, `base-uri 'none'`, `form-action 'none'`; `localResourceRoots` limitado a `dist/webview`               |
| Web                 | La pone el gateway como cabecera. `script-src` con los hashes de los scripts de hidratacion que genera el build, nunca `'unsafe-inline'`                                                                                                                                                                                        |

`connect-src` sin `https:` en escritorio es lo que garantiza que el webview no puede hablar con la
red aunque alguien lo intente: toda salida pasa por el comando `rpc`. En desarrollo hay una CSP
aparte que permite el servidor de Vite y su websocket de HMR, y solo se usa en modo desarrollo.

## Consecuencias

A favor:

- Un solo componente sirve a los dos hosts, y la diferencia entre ellos es un fichero de tema, un
  atributo de densidad y un juego de iconos.
- Si un DTO cambia en Rust y nadie regenera, la CI falla con el diff a la vista.
- Ningun webview puede hablar con la red ni ver un token.
- VS Code se ve como VS Code, con los temas del usuario, en vez de como un clon del escritorio.

En contra:

- Con un tema de VS Code del usuario, la version de VS Code no se parecera pixel a pixel al
  prototipo. Es lo esperado: quien usa VS Code quiere su tema.
- Hay que comprobar `oklch()` y `color-mix(in oklch, ...)`, que el diseno usa mucho, en el
  WebKitGTK mas antiguo que se soporta en Linux, y tener un respaldo si falla.
- React Aria trae textos de accesibilidad propios para muchos idiomas. Solo hacen falta castellano
  e ingles, y hay que recortar el resto del bundle.
- Los numeros de densidad existen dos veces (CSS y `DENSITIES`), vigilados por un test.
- Lo generado vive en el repositorio y ensucia algunos diffs. Esta fuera de ESLint, knip y el
  control de tamano.

## Alternativas descartadas

**Una interfaz por host, o el paquete comun con envoltorios por host.** Duplica pantallas o llena
los componentes de condiciones; los tokens resuelven lo mismo sin ninguna de las dos cosas.

**Radix UI.** Ver arriba: la inyeccion de `<style>` en el bloqueo de scroll choca con la CSP.

**Primitivas escritas a mano.** Menus con submenus, foco atrapado y busqueda por teclado son justo
lo que mas cuesta hacer accesible, y no es donde Git Rudder aporta nada.

**Tailwind v4.** Tambien es de coste cero en ejecucion y podria mapear los tokens. CSS Modules
encaja mejor con tokens que ya son variables CSS y con las reglas de stylelint sobre CSS normal, y
es una dependencia menos.

**CSS-in-JS en tiempo de ejecucion.** Inyecta `<style>` y necesita nonces en las tres CSP.

**`@vscode/webview-ui-toolkit`.** Archivado y obsoleto.

**`@vscode-elements/elements`.** Componentes web con aspecto nativo de VS Code. Solo tendria
sentido para una pantalla tipo ajustes que necesitara controles identicos a los del editor; los
componentes propios con `--vscode-*` bastan.

**tauri-specta o specta para los tipos.** Siguen en release candidate (rc.25), y solo cubririan
Tauri: el sidecar necesita los mismos tipos. ts-rs 12 es estable y no depende del host.

**vscode-messenger para los mensajes de los webviews.** Es solo de VS Code, asi que el cliente
compartido seguiria necesitando otro transporte para Tauri.
