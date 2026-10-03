# ADR 0009: Motor Git hibrido, primero con la CLI de git y sin libgit2

- Estado: aceptado
- Fecha: 2026-09-30
- Revisado: 2026-10-01, para alinearlo con el codigo del primer corte
- Ratificado: 2026-10-01, el MinGit de respaldo en Windows, con sus reglas de empaquetado y de
  actualizacion (ver [MinGit en el instalador de Windows](#mingit-en-el-instalador-de-windows))

## Contexto

Git Rudder es un cliente Git al estilo de GitKraken. El diseno pide el grafo de todas las ramas,
estado y staging por fichero, commit con amend y firma (SSH o GPG, activada por defecto en los
ajustes), fetch y pull con sus modos, push con upstream y `--force-with-lease`, ramas, tags,
stash, merge en sus tres estrategias, resolucion de conflictos, rebase interactivo, cherry-pick,
revert, reset en sus tres modos, clonado con submodulos, blame e historial de fichero, deshacer
y rehacer, y **varias cuentas por plataforma**, donde cada repositorio usa una de ellas para
fetch, push y pull requests.

Lo que hacen los productos comparables deja la tendencia clara:

| Producto                                               | Motor                                                                                                                                                                                                                                                        |
| ------------------------------------------------------ | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| GitKraken Desktop                                      | Paso de NodeGit y libgit2 al ejecutable de git para commit, fetch, push, pull, rebase, cherry-pick, stash, firma SSH y Git Credential Manager. Motivos publicados: libgit2 va muy por detras de git, rinde mal con LFS y soporta mal la configuracion de SSH |
| GitHub Desktop                                         | Git empaquetado (dugite); todas las operaciones lanzan git                                                                                                                                                                                                   |
| Tower                                                  | Empaqueta el binario oficial de git, con opcion de elegir otro                                                                                                                                                                                               |
| GitButler (Tauri y Rust, el mas parecido a Git Rudder) | Paso de `git2` a `gix` para lecturas y lanza git para fetch y push, para heredar la configuracion de SSH, el agente y los helpers de credenciales                                                                                                            |

Las opciones eran `git2` (libgit2), `gix` (gitoxide) solo, la CLI de git sola, o un hibrido. La
decision de partida del usuario: **un hibrido que empieza con la CLI para todo**, `gix` mas tarde
y solo en lecturas calientes si un benchmark lo justifica, nada de libgit2, y git 2.34 como
minimo con deteccion de capacidades. Este ADR la desarrolla.

## Decision

### Todo por la CLI de git, detras de dos puertos

El motor habla con git a traves de dos puertos de `git-rudder-core-application`:

- **`GitReader`**: log y ventana del grafo, refs, estado, detalle de commit, diff, blame, etapas
  de un conflicto.
- **`GitWriter`**: todo lo que modifica el repositorio o habla con un remoto.

`git-rudder-git` implementa los dos lanzando el ejecutable de git. Escrituras, red, hooks, firma, LFS y
credenciales van siempre por la CLI y nunca dejaran de hacerlo: son justo las partes donde la
semantica exacta de git importa y donde las bibliotecas van por detras.

### Reglas del adaptador

Las que evitan los errores clasicos de envolver una CLI:

- Siempre formatos de maquina: `-z`, `--porcelain=v2` y `--format` explicito, con separadores
  que git no deja meter en el dato. En el log del grafo, NUL entre commits y un campo por linea:
  el nombre y el correo viven en una linea de la cabecera del commit y `%s` une el asunto en una
  sola. El separador `%x1f` se abandono porque git si deja meter un 0x1f en un nombre o un correo,
  y los campos se desplazaban. En `for-each-ref`, NUL entre campos y salto de linea entre refs.
  Siempre `--no-pager`, `-c color.ui=false` y `-c core.quotepath=off`.
- Entorno fijo en cada llamada: `LC_ALL=C` (los mensajes de git no cambian con el idioma del
  usuario) y `GIT_TERMINAL_PROMPT=0` (git nunca se queda esperando una contrasena por una
  terminal que no existe). Ademas se quitan las variables que redirigen git a otro repositorio
  (`GIT_DIR`, `GIT_WORK_TREE`, `GIT_INDEX_FILE`, `GIT_CONFIG_PARAMETERS`...), que el motor
  heredaria si arranca desde un hook.
- Las lecturas anaden `GIT_OPTIONAL_LOCKS=0`, para no tomar `index.lock` mientras el usuario
  trabaja en una terminal. En el primer corte solo hay lecturas y va en todas las llamadas; las
  escrituras no lo llevaran.
- Las refs se leen con `git for-each-ref`, nunca con la decoracion `%D` del log: un nombre de ref
  puede contener comas y `%D` las usa como separador.
- Rebase interactivo sin editor: el motor escribe el fichero de tareas desde el plan del dominio
  y lanza git con `GIT_SEQUENCE_EDITOR` apuntando a un helper que lo coloca; `reword` usa
  `GIT_EDITOR` igual. Los estados detenidos (`rebase-merge`, `MERGE_HEAD`, `CHERRY_PICK_HEAD`)
  los detecta el lector.
- Operaciones largas: el progreso se lee de stderr (`--progress`) y sale como notificacion
  `$/progress`. Cancelar mata el grupo de procesos entero (un Job Object en Windows), no solo el
  proceso de git, porque git lanza hijos (ssh, helpers, hooks).
- Deshacer y rehacer se apoyan en un diario de operaciones: las refs y `HEAD` antes de cada
  escritura, con el reflog como respaldo.
- Hooks: se ejecutan como los ejecuta git, y su salida se ensena al usuario cuando fallan. Por
  eso la extension de VS Code no se activa en workspaces no confiables
  ([ADR 0010](0010-un-motor-para-escritorio-y-vscode.md)): abrir una carpeta ejecutaria sus hooks.
- **Ningun valor llega a la linea de comandos como opcion.** Refs, revisiones, rutas y URL pueden
  venir de otros (el nombre de una rama remota, la URL de un workspace compartido), y un valor que
  empieza por `-` se convertiria en una opcion de git. Por eso va `--end-of-options` antes de refs,
  revisiones y URL (git lo admite desde 2.24) y `--` antes de las rutas; los nombres de rama se
  validan con `git check-ref-format --branch` y se rechaza cualquiera que empiece por `-`.
- **Clonar y hacer fetch solo con protocolos conocidos**: cada llamada lleva
  `-c protocol.allow=never -c protocol.https.allow=always -c protocol.ssh.allow=always`. `ext::` y
  `fd::` ejecutan comandos y nunca se permiten; `file` solo para una carpeta que el usuario eligio
  en el dialogo nativo.

### Confianza en un repositorio

Abrir una carpeta ya ejecuta codigo si su `.git/config` lo pide: `git status` lanza
`core.fsmonitor` y los filtros de limpieza, y `git diff` y `git log -p` lanzan `diff.external` y
los `textconv`. Una carpeta extraida de un zip trae su propio `.git/config` y sus hooks. VS Code
tiene Workspace Trust; escritorio necesita su propio equivalente:

- Antes del primer comando de git en un repositorio que Git Rudder no clono, el motor lee
  `git config --local --list --show-origin` (con `--local` no sigue `include`, asi que no ejecuta
  nada) y mira `.git/hooks`.
- Si aparece cualquier clave que pueda lanzar un comando (`core.fsmonitor`, `core.hooksPath`,
  `core.sshCommand`, `core.pager`, `core.editor`, `sequence.editor`, `diff.external`,
  `diff.*.textconv`, `diff.*.command`, `filter.*`, `merge.*.driver`, `credential.helper`,
  `gpg*.program`, `uploadpack.*`, `include*`) o un hook que no sea `.sample`, la interfaz pregunta
  "Confiar en este repositorio" y ensena lo que encontro.
- Sin confianza, el repositorio se abre solo para leer, y las lecturas anaden
  `-c core.fsmonitor=false --no-ext-diff --no-textconv`. Las escrituras, que pueden lanzar hooks,
  piden confiar antes.
- La confianza se guarda por repositorio en el dispositivo y no se sincroniza. En VS Code, los
  repositorios dentro de carpetas de confianza ya lo son; los que se abren fuera de ellas pasan por
  la misma comprobacion.

### Version minima 2.34 y deteccion de capacidades

- Git 2.34 es la primera con firma de commits por SSH, y es la que trae Ubuntu 22.04, la base
  minima de Linux de Git Rudder ([ADR 0016](0016-actualizaciones-y-descargas.md)).
- El motor parsea `git --version` y calcula `GitCapabilities`. `engine/info` no falla con un git
  viejo: devuelve `supported: false` para que la interfaz explique que hacer. El minimo se aplica
  en `OpenRepository` (`repo/open`), que rechaza cualquier version inferior con el error
  `git_version_unsupported` antes de tocar el repositorio, y no en el adaptador.
- Lo que llego despues se detecta, no se supone: `merge-tree --write-tree` y
  `rebase --update-refs` necesitan 2.38. Si faltan, el motor usa el camino alternativo o
  desactiva la funcion con un motivo visible.
- La CI corre la suite del motor contra git 2.34 y contra la ultima version
  ([ADR 0006](0006-estrategia-de-testing.md)).

### Que ejecutable de git se usa

En este orden:

1. El ajuste explicito del usuario, si existe. En VS Code es la ruta que expone la API
   `vscode.git` (el ajuste `git.path` del editor), que el extension host pasa al sidecar al
   arrancarlo: el usuario ya le dijo a su editor que git usar.
2. El git del `PATH`, si cumple la version minima.
3. Solo en Git Rudder Desktop para Windows, un MinGit empaquetado en el instalador, con la version
   fijada y actualizada con cada publicacion (reglas abajo).

En macOS se usa el git del sistema, y si no existe se propone `xcode-select --install`. En Linux,
el del gestor de paquetes: los paquetes `.deb` y `.rpm` lo declaran como dependencia y la
AppImage lo pide si falta. La extension de VS Code nunca empaqueta git: VS Code ya lo necesita
para su propia integracion.

**Por que MinGit solo como respaldo en Windows y no siempre.** Un git del sistema ya trae la
configuracion del usuario, su Git Credential Manager y sus hooks funcionando. Forzar el
empaquetado siempre, como hace GitHub Desktop, garantiza la version a cambio de desentenderse de
esa configuracion. Git Rudder prefiere respetar lo que el usuario tiene y solo trae git cuando no hay
ninguno que valga, que es el caso normal en un Windows recien instalado.

### MinGit en el instalador de Windows

Ratificado el 2026-10-01, con estas reglas:

- **Va en los dos instaladores de Windows y en las dos arquitecturas**: NSIS y MSI, x64 y arm64
  (Git for Windows publica MinGit para arm64). Es el MinGit normal, no la variante busybox.
- **Version y sha256 fijados en el repositorio.** El empaquetado comprueba el sha256 antes de
  meterlo en el instalador, y un job de la CI avisa cuando sale una version nueva.
- **Ante una correccion de seguridad de Git for Windows, Git Rudder publica en pocos dias.** Es el precio
  de empaquetar git.
- **Pesa unos 30 MB mas en cada descarga y en cada actualizacion completa** (el zip de MinGit 2.56
  ronda los 37 MiB en x64 y 36 en arm64). El actualizador de Tauri baja el instalador NSIS entero,
  asi que tambien quien tiene git instalado recibe MinGit en cada actualizacion. El tamano real de
  cada instalador sale del pipeline.
- **Obligaciones de la GPL** ([ADR 0021](0021-licencia-fsl.md)): git es GPLv2, asi que el
  instalador lleva su licencia y la oferta de su codigo fuente.
- **El MSI no se actualiza solo** ([ADR 0016](0016-actualizaciones-y-descargas.md)), asi que su
  MinGit envejece hasta que IT actualiza Git Rudder. La tarjeta y la documentacion del MSI lo dicen.
- Hay que comprobar si MinGit trae Git Credential Manager: los repositorios que no estan ligados a
  una cuenta de Git Rudder dependen de el para HTTPS. Si no lo trae, el respaldo funciona para lo que
  usa las cuentas de Git Rudder y la pantalla de git ausente explica como instalar Git for Windows
  completo.
- La pantalla de git ausente o antiguo (gaps D-02) solo sale en Windows si MinGit falta o esta roto.

### Credenciales por cuenta y por repositorio

Es la parte dificil: con dos cuentas de GitHub, el push de un repositorio tiene que usar una y el
de otro la otra. Un helper de credenciales global no sabe hacerlo.

**HTTPS.** Solo en los repositorios ligados a una cuenta de Git Rudder, cada operacion de red se lanza
con `-c credential.helper=` (el valor vacio vacia la lista de helpers configurados) seguido de
`-c credential.helper=<helper de Git Rudder>`. El helper habla el protocolo de credenciales de git:

- Recibe protocolo, host y ruta y pide el token al motor que lanzo la operacion, por un canal
  local (socket de dominio Unix en un directorio privado, tuberia con nombre en Windows)
  autenticado con un token aleatorio por proceso que viaja en una variable de entorno. El motor
  solo responde para las operaciones que el mismo lanzo, solo con el token de la cuenta ligada a
  ese repositorio, y **solo si el protocolo es `https` y el host coincide exactamente con el de la
  cuenta** despues de normalizarlo con IDNA. La plataforma que diga el repositorio no cuenta: un
  host que se parece a `github.com` no recibe el token de GitHub
  ([ADR 0018](0018-cuentas-de-plataformas-git.md)).
- Cuando git le dice `erase` (la credencial fallo), el motor marca la cuenta como caducada y la
  interfaz propone actualizar el token.
- `GIT_ASKPASS` apunta al mismo helper para las preguntas interactivas que no pasan por el
  protocolo de credenciales.

El ejecutable del helper es el propio sidecar en modo helper en VS Code, y un binario pequeno que
se empaqueta junto a la app en escritorio (Tauri lo lleva como binario externo): el binario de la
app de escritorio en Windows es de subsistema grafico y no tiene un stdout donde hablar el
protocolo. La logica vive en `git-rudder-git` (planificado, con el primer `GitWriter` de red).

**SSH.** Por cuenta, con `GIT_SSH` apuntando al mismo helper en modo ssh, que recibe los
argumentos de git como lista y lanza `ssh -i <llave> -o IdentitiesOnly=yes` sin pasar por un
shell. No se usa `core.sshCommand`: git lo ejecuta con un shell, y una ruta de llave con espacios o
comillas se convertiria en un comando. Si el repositorio no tiene llave ligada, se respeta el
`~/.ssh/config` del usuario. Las llaves SSH viven en el equipo y nunca se sincronizan.

**Repositorios no ligados a ninguna cuenta.** Se ejecutan con la configuracion del usuario
intacta: Git Credential Manager, `osxkeychain`, el agente de SSH. Git Rudder no pisa lo que no es
suyo.

**Nunca** se escribe un token en `.git/config`, en la URL de un remoto ni en un argumento de
linea de comandos, donde lo veria cualquier `ps`.

### gix, mas tarde y solo en lecturas

`gix` entra si un benchmark sobre un repositorio grande de prueba demuestra que una lectura
concreta del CLI no alcanza (los candidatos son la ventana del grafo y el estado). Entra metodo a
metodo, como otra implementacion de `GitReader`, con la del CLI como respaldo, y detras de una
opcion que se puede apagar. `GitWriter` no se toca nunca.

Motivos para no empezar con el: gix publica versiones con cambios incompatibles cada mes (de la
0.84 a la 0.88 en cinco meses), su push no esta listo para produccion, su merge "necesita
reescribirse para demostrar que es correcto" segun su propio estado de crates, y el rebase, el
stash y los submodulos estan a medias. Y sobre todo: hasta que no haya un numero, no se sabe que
lectura necesita ser mas rapida.

### Nada de libgit2

`git2` y `libgit2-sys` se vetan en todo el workspace con `cargo deny` (`deny.toml`), para que no
entren por una dependencia transitiva.

## Consecuencias

A favor:

- Semantica exacta de git en todo: hooks, firma, LFS, `.gitattributes`, `~/.ssh/config`, Git
  Credential Manager y cada funcion nueva de git, gratis.
- Varias cuentas por plataforma funcionan de verdad, repositorio a repositorio, sin tocar la
  configuracion global del usuario.
- Ningun codigo C que compilar en tres sistemas (OpenSSL, libssh2): Git Rudder no enlaza con nada de
  git.
- Cambiar una lectura a `gix` es un cambio en `git-rudder-git` y en el `EngineBuilder`, reversible con
  una opcion.

En contra:

- Lanzar un proceso por llamada cuesta, sobre todo en Windows. Si el perfilado lo pide, las
  lecturas repetidas pasan a un proceso de larga vida (`git cat-file --batch`) antes que a `gix`.
- Parsear la salida de git es una fuente de errores. La mitigan los formatos de maquina y los
  repositorios de prueba dorados con snapshots.
- Depende de un git instalado de una version conocida. Un usuario con git 2.30 ve un error claro
  y tiene que actualizar; en Windows con MinGit no le pasa.
- MinGit en el instalador de Windows suma unos 30 MB a cada descarga y a cada actualizacion
  completa, obliga a vigilar sus avisos de seguridad, a publicar Git Rudder en pocos dias cuando Git for
  Windows corrige uno y a cumplir la GPL. En el MSI, que no se actualiza solo, envejece.
- El helper de credenciales es una pieza mas con su canal local. Cualquier proceso del mismo
  usuario que lea el entorno de un git en marcha podria ver el token del canal mientras dura la
  operacion; el motor solo responde dentro de esa operacion, y el mismo usuario ya puede leer su
  llavero en muchos sistemas.

## Alternativas descartadas

**`git2` (libgit2).** Va por detras de git (secuenciador, sparse checkout, clonado parcial), no
tiene LFS, su SSH ignora de formas sutiles `~/.ssh/config` y los agentes, no ejecuta hooks en
muchas operaciones y trae dependencias de C en tres sistemas. GitKraken lo abandono por eso.

**`gix` solo.** Push no listo, merge por reescribir, rebase y submodulos a medias, y versiones
incompatibles cada mes.

**`gix` para lecturas desde el primer dia.** Dos caminos de codigo antes de saber si hace falta,
con el riesgo de que gix y git no coincidan en el estado del indice o de la configuracion.

**Empaquetar git siempre, en todos los sistemas.** Garantiza la version, pero ignora la
instalacion y la configuracion del usuario y multiplica lo que hay que actualizar por seguridad.

**No empaquetar git en Windows** y guiar a instalar Git for Windows (enlace y
`winget install Git.Git`), como en macOS y Linux. Cero deuda de seguridad y de GPL, pero friccion
en el primer arranque de quien no tiene git, que en Windows es lo normal.

**Descargar MinGit bajo demanda** si no hay git, verificado con un sha256 del manifiesto. Instalador
mas pequeno y MinGit al dia sin publicar Git Rudder, a cambio de necesitar red en el primer arranque y
de mas codigo y mas superficie de cadena de suministro.

**Un unico helper de credenciales global.** No puede elegir la cuenta segun el repositorio, que
es lo que pide el diseno.
