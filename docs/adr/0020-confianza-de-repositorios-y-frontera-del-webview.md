# ADR 0020: Confianza de repositorios en escritorio y frontera del webview

- Estado: aceptado
- Fecha: 2026-10-23

## Contexto

Hay dos fronteras distintas en el cliente de escritorio, y las dos se cruzan al abrir un
repositorio. Este ADR las deja escritas juntas porque se confunden con facilidad y porque
protegen de cosas distintas.

**La primera: el repositorio que se abre no es de fiar.** Abrir una carpeta ya ejecuta codigo si
su `.git/config` lo pide. No hace falta hacer nada raro: `git status` lanza `core.fsmonitor` y los
filtros de limpieza, y `git diff` o `git log -p` lanzan `diff.external` y los `textconv`. Una
carpeta extraida de un zip, clonada de un repositorio ajeno o bajada de un adjunto trae su propio
`.git/config` y sus hooks. Que un cliente de Git ejecute eso por el mero hecho de abrir la carpeta
es ejecucion de codigo arbitrario con un doble clic.

VS Code resolvio esto con Workspace Trust, y la extension se apoya en el
(`capabilities.untrustedWorkspaces.supported: false`). **Escritorio no tenia nada**: Tauri no trae
un equivalente, asi que hay que construirlo.

**La segunda: el webview tampoco es de fiar.** El webview pinta mensajes de commit, nombres de
rama y rutas de ficheros, todo contenido que viene del repositorio y que puede venir de
cualquiera. Si una inyeccion en ese contenido consiguiera ejecutar JavaScript en el webview,
tendria a su disposicion todo lo que el webview pueda invocar. Lo que el webview puede pedir es,
por tanto, parte de la superficie de ataque, no un detalle de diseno de la API interna.

Las dos cosas estaban descritas a trozos: la confianza en
[ADR 0009](0009-motor-git-hibrido.md#confianza-en-un-repositorio) y en
[el motor cliente](../architecture.md#42-el-motor-cliente-crates) de architecture.md, y la
frontera del webview en
[el host de escritorio](../architecture.md#43-el-host-de-escritorio-appsdesktop) y
[el protocolo del motor](../architecture.md#65-el-protocolo-del-motor). Faltaba el documento que
dice por que la lista es una lista y no un `allow` generico.

## Decision

### Antes del primer comando, el motor mira la configuracion sin ejecutarla

En un repositorio que Git Rudder no clono, antes del primer comando de git, el motor lee
`git config --local --list --show-origin` y mira `.git/hooks`.

El `--local` no es un detalle: con el, git **no sigue las directivas `include` ni
`includeIf`**, asi que leer la configuracion no puede ejecutar nada ni cargar un fichero de otro
sitio. Es la unica forma de inspeccionar la configuracion sin pagar por adelantado lo que se
quiere evitar.

### Las claves que obligan a preguntar

Si aparece cualquiera de estas claves, o un hook en `.git/hooks` que no termine en `.sample`, el
caso de uso devuelve `untrusted_repository` y la interfaz pregunta **"Confiar en este
repositorio"**, ensenando lo que encontro:

| Clave                               | Que ejecuta                                                           |
| ----------------------------------- | --------------------------------------------------------------------- |
| `core.fsmonitor`                    | Un programa en cada `git status`                                      |
| `core.hooksPath`                    | Mueve los hooks a otra carpeta, fuera de `.git/hooks`                 |
| `core.sshCommand`                   | El comando de SSH, con shell de por medio                             |
| `core.pager`                        | El paginador de cualquier salida larga                                |
| `core.editor`, `sequence.editor`    | El editor de mensajes y de la lista de un rebase interactivo          |
| `diff.external`                     | Un programa en vez del diff de git                                    |
| `diff.*.textconv`, `diff.*.command` | Un programa por fichero al calcular un diff                           |
| `filter.*`                          | Los filtros de limpieza y de borron, en cada checkout y cada commit   |
| `merge.*.driver`                    | Un programa al fusionar un fichero                                    |
| `credential.helper`                 | Un programa al que git le pasa las credenciales                       |
| `gpg*.program`                      | El programa de firma                                                  |
| `uploadpack.*`                      | Lo que se ejecuta al servir objetos                                   |
| `include*`                          | Carga otra configuracion, que puede traer cualquiera de las de arriba |

La lista se mantiene a mano y crece: cada version de git puede traer una clave nueva que lanza
un comando. Es una **lista de lo que obliga a preguntar**, no una lista de lo que esta prohibido;
la diferencia importa, porque una clave que no este en la lista no se bloquea, solo no dispara la
pregunta.

### Sin confianza se lee, no se escribe

Un repositorio sin confianza **se abre**, y eso es deliberado: negarse a abrirlo empujaria a la
gente a confiar sin mirar, que es el fallo que Workspace Trust tiene cuando molesta demasiado.

- **Las lecturas pasan**, con `-c core.fsmonitor=false --no-ext-diff --no-textconv` anadido: las
  tres cosas que una lectura puede lanzar, apagadas en la linea de comandos, que gana a lo que diga
  el fichero.
- **Las escrituras piden confiar antes.** Una escritura puede lanzar hooks (`pre-commit`,
  `pre-push`, `post-checkout`), y no hay un `--no-hooks` general en el que confiar para todas.
- La confianza se guarda **por repositorio y en el dispositivo** (puerto `TrustStore`), y **no se
  sincroniza**. Confiar es una decision sobre una carpeta de un equipo concreto; que viajara a los
  demas dispositivos convertiria un si en un portatil en un si en todos.
- En VS Code, los repositorios dentro de carpetas de confianza del workspace ya lo son, y los que
  se abren fuera pasan por esta misma comprobacion. Un ajuste del workspace nunca decide que
  binario del motor se ejecuta.

### El webview solo llega a los metodos de su lista

El comando `rpc` de Tauri no reenvia al motor todo lo que le llegue. Hay tres barreras, y cada una
para una cosa distinta:

1. **La lista por ventana** (`apps/desktop/src-tauri/src/surfaces.rs`). Cada ventana tiene los
   metodos que puede pedir; en el primer corte la principal solo puede pedir `graph/window`. Un
   metodo que no este en la lista **se responde como si no existiera**, no con un "prohibido": al
   webview no se le cuenta que metodos hay detras. La lista se escribe a mano, pero con los
   nombres de los marcadores de `define_methods!` (`git-rudder-protocol`), nunca con texto literal,
   asi que un metodo que desaparece del motor no compila aqui.
2. **`host_only` en el protocolo.** `define_methods!` marca los metodos que solo puede llamar un
   host, hoy `repo/open` y los que se anadan con rutas absolutas o secretos. El motor los rechaza
   cuando la llamada llega como de webview (`CallContext::webview()`), y `rpc` reenvia siempre como
   llamada de webview. Esta barrera es la que sigue en pie si alguien anade `repo/open` a una lista
   por error.
3. **Los permisos de Tauri.** `build.rs` declara los comandos con
   `tauri_build::AppManifest::commands(...)`, asi que cada comando necesita un permiso explicito en
   `capabilities/default.json` y una ventana no puede invocar lo que no se le concedio.

El extension host de VS Code hace lo mismo por superficie en
`apps/vscode/src/extension/application/surfaces.ts`, tipado contra la lista generada. `engine/info`
no es `host_only` y aun asi no esta en ninguna lista: lo llama solo el host, al arrancar.

### El webview nunca manda una ruta

Abrir un repositorio no es un metodo del motor que el webview pueda pedir: es otro comando de
Tauri, `repo_pick_and_open`, que ensena el dialogo nativo **desde Rust** y devuelve el
`RepoHandle`. El webview no elige la carpeta, no la ve antes de abrirla y no la manda.

Para lo demas, la regla sobre los parametros:

- **Ningun parametro que venga de un webview lleva una ruta absoluta.**
- Las rutas relativas al repositorio (preparar, descartar, resolver con contenido) se rechazan si
  contienen `..` o NUL, o si **despues de seguir los enlaces simbolicos** salen del arbol de
  trabajo. Lo ultimo es lo que hace falta de verdad: un `..` se ve a simple vista, y un enlace
  simbolico dentro del repositorio apuntando a `/etc` no.
- Las acciones sobre ficheros que llegaran ("Mostrar en carpeta", "Copiar ruta") van por `repoId`
  mas ruta relativa, y el motor las valida.
- **Nada secreto hacia un webview.** Ningun metodo devuelve tokens, llaves, la clave de la boveda
  ni la frase. Los resultados pueden traer rutas para ensenarlas (`root` de `RepoHandle`), nunca al
  reves.

## Consecuencias

- El motor necesita un puerto `TrustStore` y un caso de uso que decida, antes de cualquier otro
  comando, si el repositorio hace falta confiarlo. El tipo de error `untrusted_repository` entra en
  el enum cerrado de `ErrorKind` y la interfaz lo traduce con `@git-rudder/i18n`.
- `GitReader` y `GitWriter` dejan de ser simetricos en esto: el lector sabe anadir los tres flags
  de lectura sin confianza y el escritor sabe que no puede correr sin confianza. Es una razon mas
  para que sean dos puertos.
- La interfaz tiene que **ensenar lo que encontro** al preguntar. Un "¿confias?" sin decir que
  claves o que hooks aparecieron entrena a la gente a decir si.
- La lista de claves es mantenimiento recurrente: cada version de git que suba el minimo pide
  revisarla.
- La lista por ventana crece con cada pantalla, y cada metodo nuevo obliga a decidir en que
  ventanas entra. Los tests comprueban que ninguna lista contiene un metodo del host, asi que la
  decision no se puede olvidar en silencio.
- Un metodo del motor que necesite una ruta absoluta o un secreto **no se anade a ninguna lista de
  webview**: se marca `host_only` y lo llama el host.

## Alternativas descartadas

- **No tener confianza en escritorio.** Es lo que habia, y deja la ejecucion de codigo a un doble
  clic sobre una carpeta descargada. VS Code lo arreglo hace anos con Workspace Trust.
- **Negarse a abrir un repositorio sin confianza.** Mas seguro sobre el papel. En la practica
  convierte la pregunta en un peaje que se cruza sin leer, y hace imposible lo que mas se quiere
  hacer con un repositorio ajeno: mirarlo.
- **Preguntar siempre, confie o no.** Sin memoria, el aviso pierde todo su significado al tercer
  dia.
- **Sincronizar la confianza entre dispositivos.** Comodo, y convierte un si dado en un equipo en
  un si en todos. Una decision sobre una carpeta local se queda en la maquina que la tiene.
- **Una lista de claves prohibidas en vez de una lista de lo que obliga a preguntar.** Prohibir
  claves romperia repositorios legitimos que usan `filter.*` para LFS o `credential.helper` del
  sistema. La pregunta deja decidir a quien sabe si ese repositorio es suyo.
- **Un `allow` generico para el webview** (que el webview pida cualquier metodo del motor y el
  motor decida). Es una barrera menos y, sobre todo, mueve la decision a un sitio donde no se ve:
  la lista por ventana se lee de un tiro y dice exactamente que puede pedir cada pantalla.
- **Un comando de Tauri por metodo del motor.** Daria permisos de Tauri por metodo, a cambio de
  duplicar en `commands.rs` la lista de metodos que ya vive en `git-rudder-protocol` y de que las
  dos se separen con el tiempo.
- **Dejar que el webview mande la ruta a abrir**, con validacion en el host. El dialogo nativo
  desde Rust cuesta lo mismo y quita el problema entero: no hay ruta que validar si el webview
  nunca la escribe.
