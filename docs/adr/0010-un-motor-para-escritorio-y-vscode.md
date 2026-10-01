# ADR 0010: Un solo motor en Rust para escritorio y VS Code, como sidecar JSON-RPC en VS Code

- Estado: aceptado
- Fecha: 2026-09-30
- Revisado: 2026-10-01, para alinearlo con el codigo del primer corte
- Ratificado: 2026-10-01, los ocho destinos de VSIX; y se decide quien sincroniza cada ajuste de
  la extension (ver [Ajustes y sincronizacion](#ajustes-y-sincronizacion))

## Contexto

Ramal for VS Code ensena la misma vista de repositorio que Ramal Desktop, sin lista de
repositorios ni workspaces: el grafo, los cambios con staging, ramas, remotos, PRs, stashes y
tags, el resolver de conflictos, el diff, los ajustes, commit y commit con push, mensaje de commit
con IA, la barra de estado y "Abrir en Ramal Desktop". Por debajo necesita lo mismo que
escritorio: git con varias cuentas por repositorio, las APIs de las plataformas, los proveedores
de IA, la sesion de Ramal y la boveda de secretos.

La extension corre en el extension host de VS Code, que es Node. Las opciones para darle un motor
eran:

- La API `vscode.git` que trae VS Code.
- Lanzar el git CLI desde TypeScript, que es el modelo de GitLens.
- El mismo motor de Rust que usa escritorio, como binario sidecar.
- El mismo motor como addon nativo de Node (napi-rs).
- Git en WebAssembly (isomorphic-git).

La decision de partida del usuario es la tercera: **un solo motor en Rust**, que escritorio usa
en proceso desde el host de Tauri y VS Code lleva como sidecar `ramal-vscode-engine` por stdio
con JSON-RPC, en paquetes VSIX por plataforma (el modelo de rust-analyzer), con el extension host
fino. Este ADR la desarrolla.

## Decision

### `ramal-engine` es la unica puerta

Los dos hosts llaman a la misma funcion, `Engine::handle(RpcRequest) -> Option<RpcResponse>` (o
`handle_with`, que ademas dice si la peticion viene del host o de un webview), que deserializa los
parametros, llama al caso de uso y serializa el resultado. Lo que viene de un webview no puede
llamar a los metodos marcados como del host. Por eso el protocolo es
identico en los dos productos: los metodos, los DTOs y los errores los define `ramal-protocol` una
vez ([ADR 0011](0011-ui-compartida-y-puente-tipado.md)), y ningun host tiene una lista propia de
metodos.

- **Escritorio:** el motor vive en el proceso de Tauri. Un unico comando de negocio, `rpc`, recibe
  un `RpcRequest` y devuelve la respuesta; las notificaciones (progreso, cambios del repositorio,
  tokens de IA) salen por un `Channel` de Tauri.
- **VS Code:** el motor vive en un proceso hijo, `ramal-vscode-engine`, que habla JSON-RPC 2.0 por
  stdin y stdout.

### El sidecar: JSON-RPC 2.0 con marcos de estilo LSP

- Marcos `Content-Length: <bytes>\r\n\r\n` seguidos de exactamente esos bytes de JSON en UTF-8,
  como LSP. Otras cabeceras se ignoran. Un marco de mas de 32 MiB se rechaza y se cierra la
  conexion.
- Una tarea por peticion y una unica tarea que escribe en stdout, para que dos respuestas nunca se
  mezclen.
- **stdout es solo protocolo.** Todo el log va a stderr con `tracing-subscriber`, y el extension
  host lo vuelca en el canal de salida "Ramal Engine". Un `println!` romperia el protocolo, y por
  eso `print_stdout` esta denegado en clippy en todo el workspace
  ([ADR 0005](0005-enforcement-automatizado.md)).
- El sidecar termina cuando se cierra su stdin: si el extension host muere, el motor no se queda
  huerfano.
- Notificaciones de control con prefijo `$/`: `$/cancel` para cancelar una peticion y
  `$/progress` para las operaciones largas, con los codigos de LSP donde existen (-32800 para una
  peticion cancelada). En el primer corte el motor todavia ignora `$/cancel` y no emite
  `$/progress`.
- Un JSON roto da -32700 y un sobre invalido (id de texto o fuera de rango, sin metodo, un array)
  da -32600, con id `null` si no se pudo leer. Un metodo que no existe da -32601. Los tres llevan
  `kind: internal`: la interfaz solo llama a los metodos del mapa generado, asi que son fallos de
  empaquetado, no algo que el usuario pueda arreglar.

### El extension host se queda fino

Hace lo que solo el puede hacer y nada mas:

- Activacion, comandos, paneles y vistas de VS Code, barra de estado, `AuthenticationProvider` y
  `UriHandler` para el inicio de sesion ([ADR 0008](0008-autenticacion-con-tokens.md)).
- Arrancar, vigilar y reiniciar el sidecar.
- Validar y enrutar los mensajes de los webviews. Un webview pinta contenido no confiable
  (mensajes de commit, nombres de rama, contenido de ficheros) y se trata como un cliente hostil:
  el host comprueba origen, tamano (1 MiB), forma (zod), que el metodo este en la lista generada
  de metodos y en la de su superficie, y reescribe los ids para que un webview no pueda chocar con
  otro ni cancelar peticiones ajenas. El motor vuelve a validar despues.
- Resolver las peticiones nativas (dialogos, QuickPick, abrir un fichero, copiar) con la API de VS
  Code, sin pasar por el sidecar. En el primer corte solo existe copiar (`clipboard/writeText`).
- Guardar secretos en `SecretStorage`.
- Leer de la API `vscode.git` solo tres cosas: la ruta de git configurada (que pasa al sidecar),
  el descubrimiento de repositorios en workspaces con varias carpetas y los avisos de "el
  repositorio cambio" como pista para refrescar.

No lanza git, no parsea salida de git, no llama a ninguna API de plataforma ni de IA y no calcula
el grafo.

### Secretos por peticion inversa

El motor no guarda secretos en disco. Cuando necesita el refresh token de Ramal, el token de una
plataforma o la llave de la boveda, manda al extension host una peticion JSON-RPC en sentido
inverso (del servidor al cliente, como hace LSP), y el extension host responde con
`context.secrets`. En escritorio el mismo puerto `SecretStore` lo implementa el llavero del
sistema. El caso de uso no sabe cual de los dos tiene delante.

### Ciclo de vida

- El extension host arranca el sidecar la primera vez que hace falta, no al activarse la
  extension.
- Cada vez que arranca un proceso, llama a `engine/info` y compara `PROTOCOL_VERSION` con la que
  trae generada. Como los dos viajan en el mismo VSIX, una diferencia es un fallo de empaquetado,
  se ensena como tal y no se reintenta.
- Si `engine/info` contesta con un error (no hay git, o el git no arranca), el proceso esta sano y
  reiniciarlo no arregla nada: no cuenta como caida. El host lo para, la peticion recibe el error
  del motor con su tipo (`git_not_found`, por ejemplo) y la siguiente prueba con un proceso
  nuevo, que ya ve el arreglo. Cuando contarlo como caida metia el motor en el bucle de
  reinicios, el usuario veia "motor no disponible" y nunca "no se encuentra Git". Un git
  anterior a 2.34 tampoco es un error aqui: `engine/info` devuelve `supported: false`.
- Si el sidecar muere, lo reinicia tras 1, 2, 4, 8 y 16 segundos. Con cinco caidas en tres
  minutos deja de intentarlo y muestra una notificacion con "Reintentar". Las peticiones
  pendientes fallan con el tipo `engine_unavailable`, que genera el propio host, en vez de
  quedarse colgadas.
- Tras un reinicio el registro de repositorios del motor esta vacio. Antes de dejar pasar ninguna
  peticion, el host vuelve a abrir con `repo/open` la raiz de cada panel. El `repoId` sale de la
  raiz (`repo-` y 16 cifras hexadecimales de su hash), asi que es el mismo que tenia el panel. Si
  la carpeta ya no es un repositorio, cierra sus paneles y avisa.
- Los webviews reciben los tipos de error del motor tal cual, mas `engine_unavailable` cuando el
  motor no esta.
- Cada ventana de VS Code tiene su extension host y por tanto su sidecar. Todos comparten
  `SecretStorage`, y de ahi sale la ventana de 30 segundos del refresco de sesion
  ([ADR 0008](0008-autenticacion-con-tokens.md)).

### VSIX por plataforma

- Un VSIX por destino, con `vsce package --target` y `ovsx publish --target`: `win32-x64`,
  `win32-arm64`, `darwin-x64`, `darwin-arm64`, `linux-x64`, `linux-arm64`, `alpine-x64` y
  `alpine-arm64`. Cada uno lleva solo su binario en `dist/bin/ramal-vscode-engine` (con `.exe` en
  Windows). Los de Alpine se compilan para musl, que es lo que usan muchos contenedores de
  desarrollo. Ratificado el 2026-10-01: los ocho salen en el lanzamiento, porque `linux-arm64` y
  `alpine-*` son justo los contenedores de desarrollo (un Mac con Apple Silicon, Graviton), y
  compilarlos cuesta poco. Los cuatro destinos de Linux y Alpine salen de un solo job con
  compilacion cruzada, y los binarios de macOS y Windows se firman con las mismas identidades que
  escritorio ([ADR 0016](0016-actualizaciones-y-descargas.md)).
- Se publica en el Marketplace de Visual Studio y en Open VSX, que es de donde instalan VSCodium,
  Cursor y otros derivados.
- `extensionKind: ["workspace"]`: en Remote SSH, WSL y contenedores, la extension y el motor
  corren junto al repositorio, y VS Code instala el VSIX de la plataforma remota.
- `capabilities.virtualWorkspaces: false` (no hay git local en vscode.dev ni en github.dev) y
  `capabilities.untrustedWorkspaces.supported: false`, porque ejecutar git en una carpeta no
  confiable puede ejecutar sus hooks y su configuracion.
- **La ruta del binario no la elige un ajuste en produccion.** Siempre es la del VSIX. Solo en modo
  desarrollo (`ExtensionMode.Development`) se acepta el ajuste `ramal.engine.path` para apuntar a
  `target/debug`. Un ajuste de workspace que eligiera el binario seria ejecucion de codigo con solo
  abrir una carpeta.

### Ajustes y sincronizacion

Decidido el 2026-10-01 (gaps V-22). VS Code sincroniza sus ajustes entre equipos con Settings Sync,
y Ramal sincroniza los suyos con su nube. Si los dos sincronizaran la misma clave, se pisarian. La
regla es que **cada ajuste tiene un solo dueno**:

| Ajuste                                                                                    | Donde vive                              | Quien lo sincroniza                                                 |
| ----------------------------------------------------------------------------------------- | --------------------------------------- | ------------------------------------------------------------------- |
| Preferencias de Git (fetch automatico, modo de pull, firmar, podar)                       | Motor, documento `git`, como en Desktop | La nube de Ramal                                                    |
| Configuracion de IA (funciones, llave y modelo por funcion, idioma, Conventional Commits) | Motor, documento `ai`, como en Desktop  | La nube de Ramal                                                    |
| Cuentas de plataforma y llaves de IA                                                      | `SecretStorage` y la boveda             | La nube de Ramal (boveda). VS Code nunca sincroniza `SecretStorage` |
| `ramal.engine.path`, `ramal.engine.logLevel` y "usar la nube" en esta instalacion         | `settings.json`, con `scope: "machine"` | Nadie: Settings Sync no sincroniza los ajustes `machine`            |
| Tema e idioma                                                                             | VS Code                                 | VS Code                                                             |
| Confianza de repositorios                                                                 | Motor, por equipo                       | Nadie                                                               |

- Lo compartido con Desktop (Git e IA) se edita en el webview de ajustes de Ramal, el mismo que ya
  tiene las cuentas y las llaves. No sale en el buscador de ajustes de VS Code, a cambio de que no
  haya dos sincronizaciones sobre la misma clave ni que la extension escriba en el `settings.json`
  del usuario.
- En `settings.json` solo quedan los ajustes de esta maquina, todos con `scope: "machine"`, que
  tampoco se pueden fijar por workspace, asi que la regla del binario se mantiene.
- Sin cuenta, o con la nube apagada en esta instalacion, Git e IA son locales a ese VS Code.
- La extension no registra estado propio con `globalState.setKeysForSync`.
- **Hoy `package.json` declara `ramal.engine.path` y `ramal.engine.logLevel` con
  `scope: "application"`**, y Settings Sync si sincroniza ese ambito: un `trace` puesto en un
  equipo viajaria a todos. Pasarlos a `machine` es una issue de seguimiento (#301).

## Consecuencias

A favor:

- Una sola implementacion de git, credenciales por cuenta, plataformas, IA, sesion y boveda para
  los dos productos. Un fallo se arregla una vez.
- El mismo protocolo en los dos hosts, asi que `@ramal/repo-view` no sabe en cual esta.
- VS Code recibe el grafo ya calculado en Rust y por ventanas, sin mandar el historial entero a un
  webview.
- Funciona en Remote SSH, WSL y contenedores, porque el motor corre donde esta el repositorio.
- Si el motor falla, cae un proceso hijo, no el extension host ni el editor.

En contra:

- Ocho VSIX que compilar, publicar en dos tiendas y, en macOS y Windows, firmar. La CI de
  publicacion es mas larga y los destinos de Alpine necesitan compilacion cruzada para musl.
- Cada VSIX pesa varios MB mas que una extension en TypeScript puro.
- No hay version para vscode.dev ni github.dev. Con git local es imposible de todos modos.
- Un proceso mas por ventana de VS Code, con su memoria.
- Hay que gestionar el ciclo de vida del proceso: arranque, reinicio, cancelacion y versiones.
- Si una politica corporativa bloquea binarios nativos en extensiones, Ramal for VS Code no
  funciona ahi. No hay plan B en el primer corte; el precedente de rust-analyzer, Ruff o Biome dice
  que es un caso raro.

## Alternativas descartadas

**La API `vscode.git`.** No da el grafo con padres, ni rebase interactivo, ni cherry-pick, ni un
modelo de conflictos, ni credenciales por cuenta, y depende de que la extension de git de VS Code
este activada. Se usa solo como fuente de pistas.

**Git CLI desde TypeScript, el modelo de GitLens.** Obligaria a escribir por segunda vez, en otro
lenguaje, todo el adaptador de git (parseo de log, estado, diff y conflictos, el driver de rebase,
las credenciales por cuenta) y ademas los clientes de plataformas e IA, cada uno con sus propios
fallos.

**Addon nativo de Node con napi-rs.** En proceso y sin JSON-RPC, a cambio de problemas de ABI con
el Electron de VS Code y de que un fallo del motor tumbe el extension host. El coste de empaquetar
por plataforma es el mismo que el del sidecar.

**isomorphic-git en WebAssembly.** Lento, sin hooks, sin firma, sin LFS y sin SSH. No sirve para un
cliente Git.

**Un VSIX universal con los ocho binarios.** Una sola publicacion, a cambio de que cada usuario se
descargue ocho veces el motor.

**Una biblioteca de LSP (tower-lsp) para el sidecar.** El framing es el mismo, pero la semantica de
LSP (documentos, capacidades del lenguaje) no encaja, y el marco propio son unas decenas de lineas
con sus tests.

**Un comando de Tauri por metodo.** El protocolo de escritorio divergiria del del sidecar, que es
justo lo que un solo `rpc` impide.
