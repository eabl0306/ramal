# Brief #65: credenciales y operaciones de Git (Git Rudder Desktop)

## Contexto

Solo Git Rudder Desktop: alta de cuentas, preguntas de credenciales, tokens caducados, cuenta por repo,
progreso, errores, conflictos fuera del merge, rebase detenido, mensajes de squash y merge, HEAD
separado, upstream y deshacer. Cuelga de `Git Rudder.dc.html`: Ajustes > Plataformas Git (tarjeta de
cuenta con metodo, permisos y punto de estado), barra de herramientas del repo, fila WIP, resolvedor
con su panel derecho (caja `--warn`, lista de archivos, "Completar merge"), editor de rebase, modal
de merge, modal generico y toasts. Error con detalle es C-02 de #62. D-16 y D-26 son P2 y no entran.

## Pantallas y estados

Formato de los textos: `"castellano" / "english"`. Los parametros tienen nombre en ingles y son
iguales en los dos idiomas (`{branch}`, `{host}`); los contadores usan `{count}` con claves `_one` y
`_other`.

### 1. D-12 Conectar cuenta (X-07, X-08)

- **Proposito.** Sustituir el modal de usuario y token por OAuth publico primero, con el usuario y
  los permisos leidos de la plataforma, nunca tecleados.
- **Paso 1, plataforma.** Modal de 580 px con cinco tiles (insignias GH, GL, BB, GT, AZ). En GitHub
  y GitLab, debajo, un segmentado: "github.com" o "GitHub Enterprise Server" (con campo de host);
  "gitlab.com" o "GitLab propio". Gitea y Forgejo piden siempre host.
- **Paso 2, segun la plataforma** (tabla de X-07):
  - GitHub.com: "Conectar con GitHub" genera un codigo de dispositivo. Pantalla de espera con el
    codigo grande en mono y "Copiar", "Copiar y abrir github.com/login/device", cuenta atras y
    spinner. Enlace "Usar un token de grano fino".
  - GitLab.com: "Continuar en el navegador" (PKCE). Pantalla de espera con "Abrir de nuevo" y
    "Cancelar". La pagina de exito del navegador la sirve el motor (W-08, no se dibuja aqui). Enlace
    "Usar un token personal".
  - Bitbucket Cloud: "Continuar en el navegador" (OAuth por el broker, misma espera) o "Usar un API
    token". Nunca aparece "App password".
  - GitLab propio, Gitea, Forgejo y GitHub Enterprise Server: host (validado, punycode si no es
    ASCII) y token personal en campo oculto, con la lista de permisos que hacen falta y "Crear token
    en {host}", que abre la pagina de tokens de ese host.
  - Azure DevOps: campo "Organización" (`dev.azure.com/{org}`) y token de alcance de organizacion,
    con la nota de que los tokens globales dejan de funcionar el 2026-12-01. Sin OAuth.
  - SSH: en cualquier formulario con token, casilla "Usar SSH para fetch y push" y un selector con
    las llaves de `~/.ssh` (generar llave es D-16 y no entra). SSH nunca va solo: siempre con token.
- **Validando.** "Comprobando…" con spinner; los botones se desactivan.
- **Exito.** Avatar, `@usuario`, nombre, permisos como chips, caducidad si la plataforma la da, campo
  "Etiqueta", casilla "Hacer predeterminada para {platform}" (si ya hay otra) y donde se guarda:
  "Solo en este equipo" o "Se sincroniza cifrada". Si la sincronizacion de cuentas esta activa y no
  hay boveda, al guardar se encadena K-01 de #63.
- **Errores.** Token invalido; permisos insuficientes (lista de los que faltan y "Crear un token
  nuevo"); la organizacion de GitHub no aprobo la OAuth App de Git Rudder (enlace de solicitud); la
  organizacion exige autorizar con SSO (la cuenta se anade igual, con aviso y "Autorizar en GitHub");
  codigo caducado; autorizacion cancelada; host inalcanzable o certificado no valido; cuenta ya
  conectada (con "Actualizar credenciales").
- **Actualizar credenciales.** El mismo formulario con el metodo conservado (B-35), el token vacio y
  oculto (B-37) y la etiqueta.

| Uso                   | Texto                                                                                                                                                                                                                                                            |
| --------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Titulo                | "Conectar una cuenta" / "Connect an account" · "Actualizar credenciales" / "Update credentials"                                                                                                                                                                  |
| Variantes de host     | "GitHub Enterprise Server" · "GitLab propio" / "Self-managed GitLab" · "Servidor" / "Host"                                                                                                                                                                       |
| GitHub                | "Conectar con GitHub" / "Connect with GitHub" · "Copiar y abrir github.com/login/device" / "Copy and open github.com/login/device"                                                                                                                               |
| Espera de codigo      | "Escribe este código en GitHub para autorizar Git Rudder." / "Enter this code on GitHub to authorize Git Rudder." · "Esperando a que autorices…" / "Waiting for you to authorize…"                                                                               |
| Navegador             | "Continuar en el navegador" / "Continue in the browser" · "Autoriza Git Rudder en {platform} y vuelve aquí." / "Authorize Git Rudder on {platform} and come back here."                                                                                          |
| Alternativas          | "Usar un token de grano fino" / "Use a fine-grained token" · "Usar un token personal" / "Use a personal access token" · "Usar un API token" / "Use an API token"                                                                                                 |
| Token                 | "Token de acceso" / "Access token" · "Permisos necesarios: {scopes}" / "Required scopes: {scopes}" · "Crear token en {host}" / "Create token on {host}"                                                                                                          |
| Azure DevOps          | "Organización" / "Organization" · "El token tiene que ser de esta organización. Los tokens globales dejan de funcionar el 1 de diciembre de 2026." / "The token must belong to this organization. Global tokens stop working on December 1, 2026."               |
| SSH                   | "Usar SSH para fetch y push" / "Use SSH for fetch and push" · "Llave SSH" / "SSH key"                                                                                                                                                                            |
| Validando             | "Comprobando con {platform}…" / "Checking with {platform}…"                                                                                                                                                                                                      |
| Exito                 | "Conectada como @{user}" / "Connected as @{user}" · "Caduca el {date}" / "Expires on {date}" · "Hacer predeterminada para {platform}" / "Make default for {platform}"                                                                                            |
| Donde se guarda       | "Solo en este equipo" / "Only on this device" · "Se sincroniza cifrada de extremo a extremo" / "Syncs end-to-end encrypted"                                                                                                                                      |
| Token invalido        | "{platform} no acepta este token." / "{platform} doesn't accept this token."                                                                                                                                                                                     |
| Permisos              | "A este token le faltan permisos: {scopes}." / "This token is missing scopes: {scopes}." · "Crear un token nuevo" / "Create a new token"                                                                                                                         |
| OAuth App no aprobada | "La organización {org} todavía no ha aprobado Git Rudder. Pide a un admin que lo apruebe." / "The {org} organization hasn't approved Git Rudder yet. Ask an admin to approve it." · "Solicitar aprobación" / "Request approval"                                  |
| SSO                   | "{org} exige autorizar Git Rudder con su inicio de sesión único. Sin eso, sus repositorios fallarán." / "{org} requires authorizing Git Rudder with its single sign-on. Without it, its repositories will fail." · "Autorizar en GitHub" / "Authorize on GitHub" |
| Caducado              | "El código ha caducado." / "The code has expired." · "Pedir otro" / "Get a new one"                                                                                                                                                                              |
| Cancelado             | "Cancelaste la autorización." / "You cancelled the authorization."                                                                                                                                                                                               |
| Host                  | "No podemos conectar con {host}." / "We can't reach {host}." · "El certificado de {host} no es válido." / "{host}'s certificate isn't valid."                                                                                                                    |
| Ya conectada          | "@{user} ya está conectada." / "@{user} is already connected."                                                                                                                                                                                                   |

- **Variantes.** Claro y oscuro; un estado por plataforma; exito, cada error.

### 2. D-13 Peticiones de credenciales durante fetch, pull, push y clone

- **Proposito.** git las pide por `GIT_ASKPASS` y `SSH_ASKPASS` cuando ninguna cuenta encaja; si
  Git Rudder no las dibuja, la operacion se cuelga.
- **Muestra.** Modal de 460 px con contexto arriba: la operacion y el remoto en mono
  (`git push origin feature/graph-ui`). Tres variantes:
  1. Usuario y token HTTPS: campos "Usuario" y "Token o contraseña" (oculto), casilla "Guardar como
     cuenta de Git Rudder para {host}" (si se marca, valida como D-12) y, en github.com o gitlab.com,
     enlace "Conectar con OAuth". "Cancelar" aborta la operacion.
  2. Frase de una llave SSH: ruta de la llave en mono, campo oculto, casilla "Recordar hasta cerrar
     Git Rudder" (apagada). Error en linea si la frase no vale.
  3. Huella de un host desconocido: host, tipo de llave y huella SHA256 en mono, y el consejo de
     comprobarla. "Confiar y continuar" la anade a `known_hosts`.
  4. Huella cambiada: variante de peligro sin boton de continuar, solo "Cerrar" y explicacion.
- **Fuera.** La frase de una llave GPG la pide `pinentry` del sistema (X-13): no se dibuja como
  dialogo propio. El boton de commit ensena "Esperando a la ventana de GPG…" mientras tanto.

| Uso              | Texto                                                                                                                                                                                                                                                                                                                                          |
| ---------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Titulo HTTPS     | "Credenciales para {host}" / "Credentials for {host}"                                                                                                                                                                                                                                                                                          |
| Campos           | "Usuario" / "Username" · "Token o contraseña" / "Token or password" · "Guardar como cuenta de Git Rudder para {host}" / "Save as a Git Rudder account for {host}" · "Conectar con OAuth" / "Connect with OAuth"                                                                                                                                |
| Titulo SSH       | "Frase de la llave SSH" / "SSH key passphrase" · "Recordar hasta cerrar Git Rudder" / "Remember until Git Rudder closes" · "Desbloquear" / "Unlock"                                                                                                                                                                                            |
| Frase incorrecta | "La frase no es correcta." / "That passphrase is wrong."                                                                                                                                                                                                                                                                                       |
| Titulo huella    | "¿Confiar en {host}?" / "Trust {host}?"                                                                                                                                                                                                                                                                                                        |
| Texto huella     | "Es la primera vez que te conectas a este servidor. Comprueba la huella con quien lo administra." / "This is your first connection to this server. Check the fingerprint with whoever runs it." · "Confiar y continuar" / "Trust and continue"                                                                                                 |
| Huella cambiada  | "La huella de {host} ha cambiado" / "{host}'s fingerprint has changed" · "Puede ser un cambio legítimo del servidor o alguien interceptando la conexión. Git Rudder no continuará. Revisa tu known_hosts." / "It may be a legitimate server change or someone intercepting the connection. Git Rudder won't continue. Check your known_hosts." |
| GPG              | "Esperando a la ventana de GPG…" / "Waiting for the GPG window…"                                                                                                                                                                                                                                                                               |

- **Variantes.** Claro y oscuro; las cuatro variantes.

### 3. D-14 Token caducado o revocado

- **Proposito.** Los tokens caducan (GitHub con refresco, Bitbucket a las 2 horas, Azure DevOps como
  mucho al ano) y hoy no hay forma de enterarse.
- **Muestra.**
  - Punto de estado real en la tarjeta de cuenta (B-39): `--ok` valida con "Comprobada {ago}",
    `--warn` "Caduca en {days} días", `--danger` "Caducada o revocada", `--faint` "Sin comprobar".
  - Aviso en la vista de repo cuando una operacion falla por autenticacion: banner bajo la barra de
    herramientas con "Actualizar credenciales" (abre D-12 en modo actualizar, o la reconexion OAuth)
    y "Usar otra cuenta" (abre D-15).
  - Aviso previo en Ajustes > Plataformas Git cuando una cuenta caduca en menos de 14 dias.
  - Cuenta sincronizada sin token en este equipo: "Actualizar credenciales" (la boveda bloqueada es
    K-04 de #63).

| Uso          | Texto                                                                                                                                                                                                          |
| ------------ | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Estados      | "Comprobada {ago}" / "Checked {ago}" · "Caduca en {days} días" / "Expires in {days} days" · "Caducada o revocada" / "Expired or revoked" · "Sin comprobar" / "Not checked"                                     |
| Banner       | "El token de @{user} en {platform} caducó o se revocó." / "@{user}'s {platform} token expired or was revoked." · "Actualizar credenciales" / "Update credentials" · "Usar otra cuenta" / "Use another account" |
| Aviso previo | "El token de @{user} caduca el {date}. Crea uno nuevo antes para no quedarte sin push." / "@{user}'s token expires on {date}. Create a new one before then to keep pushing."                                   |
| Sin token    | "Esta cuenta llegó de otro equipo sin su token." / "This account came from another device without its token."                                                                                                  |

- **Variantes.** Claro y oscuro; tarjeta en cada estado; banner.

### 4. D-15 Elegir la cuenta de cada repositorio

- **Proposito.** "Varias cuentas por plataforma" solo se elige hoy al clonar.
- **Muestra.** En el menu de la tarjeta de repo y en el desplegable del nombre del repo de la barra
  de herramientas, el submenu "Cuenta ›" con las cuentas cuyo host es identico al del remoto (✓ en la
  actual), "Sin cuenta (credenciales del sistema)" y "Conectar una cuenta de {host}…". La columna 3
  de la tarjeta refleja el cambio. Si el repo tiene remotos en otros hosts, una linea en el submenu
  dice que esos usan las credenciales del sistema.

| Uso         | Texto                                                                                                                                                               |
| ----------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Submenu     | "Cuenta" / "Account" · "Sin cuenta (credenciales del sistema)" / "No account (system credentials)" · "Conectar una cuenta de {host}…" / "Connect a {host} account…" |
| Otros hosts | "Los remotos de otros servidores usan las credenciales del sistema." / "Remotes on other servers use system credentials."                                           |
| Toast       | "{repo} usa ahora @{user}" / "{repo} now uses @{user}"                                                                                                              |

- **Variantes.** Claro y oscuro.

### 5. D-17 Progreso de operaciones largas

- **Proposito.** Fetch, pull, push, checkout y rebase duran segundos o minutos en un repo real.
- **Muestra.**
  - Barra de operacion de 36 px bajo la barra de herramientas: glifo de la operacion, etiqueta,
    fase y porcentaje ("Recibiendo objetos 62 % · 3,4 MiB"), barra fina en `--accent` y "Cancelar".
    Sin porcentaje conocido, barra indeterminada.
  - El boton de la operacion ensena un spinner en vez del glifo; los demas botones que escriben se
    desactivan con el tooltip "Espera a que termine {operation}".
  - Si la operacion corre en otra pestana, la insignia de plataforma de esa pestana lleva un anillo
    de progreso.
  - "Cancelar" esta en fetch, pull, push, clone y checkout. Un rebase no se cancela: se aborta (D-20).
- **Estados.** En curso, cancelando, cancelado (toast `info`), terminado (toast `ok` y la barra se va),
  fallo (pasa a D-18).

| Uso       | Texto                                                                                                                                                                                                                                                                  |
| --------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Etiquetas | "Fetch de {remote}" / "Fetching {remote}" · "Pull de {branch}" / "Pulling {branch}" · "Push a {remote}/{branch}" / "Pushing to {remote}/{branch}" · "Checkout de {branch}" / "Checking out {branch}" · "Rebase: {current} de {total}" / "Rebase: {current} of {total}" |
| Tooltip   | "Espera a que termine {operation}" / "Wait until {operation} finishes"                                                                                                                                                                                                 |
| Cancelado | "{operation} cancelada" / "{operation} cancelled"                                                                                                                                                                                                                      |

- **Variantes.** Claro y oscuro; determinada e indeterminada; en otra pestana.

### 6. D-18 Errores de operacion

- **Proposito.** Los errores diarios de cualquier cliente Git. Se dibujan con el dialogo de error de
  C-02 (texto, salida en mono con scroll, "Copiar detalle") y acciones propias.
- **Casos:**
  1. Push rechazado por no ser fast-forward: "Pull y reintentar" (primario), "Force push con
     lease…" (peligro, abre la confirmacion que ya existe) y "Cancelar".
  2. Autenticacion: lleva a D-14.
  3. Red: "Reintentar".
  4. `index.lock` presente: "Reintentar" y "Borrar el bloqueo y reintentar" (peligro, confirma).
  5. Hook fallido: nombre del hook, salida completa y "Reintentar"; "Commit sin hooks" como accion
     secundaria en `--warn`, con confirmacion.
  6. Arbol sucio que bloquea el checkout: lista de archivos que chocan, "Hacer stash y cambiar"
     (primario), "Descartar y cambiar…" (peligro) y "Cancelar".
  7. Firma fallida: "Abrir ajustes de Git" y "Commit sin firmar".
  8. Rechazo del servidor (rama protegida, hook remoto): la salida del remoto tal cual.

| Uso            | Texto                                                                                                                                                                                                                                                                                    |
| -------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Push rechazado | "{remote}/{branch} tiene commits que tú no tienes." / "{remote}/{branch} has commits you don't have." · "Pull y reintentar" / "Pull and retry" · "Force push con lease…" / "Force push with lease…"                                                                                      |
| Red            | "No podemos conectar con {host}." / "We can't reach {host}." · "Reintentar" / "Retry"                                                                                                                                                                                                    |
| index.lock     | "Otro proceso de Git está usando este repositorio." / "Another Git process is using this repository." · "Si no tienes otro Git abierto, puedes borrar el bloqueo." / "If no other Git is running, you can remove the lock." · "Borrar el bloqueo y reintentar" / "Remove lock and retry" |
| Hook           | "El hook {hook} rechazó la operación." / "The {hook} hook rejected the operation." · "Commit sin hooks" / "Commit without hooks"                                                                                                                                                         |
| Arbol sucio    | "Tus cambios se perderían al cambiar a {branch}." / "Your changes would be lost by switching to {branch}." · "Hacer stash y cambiar" / "Stash and switch" · "Descartar y cambiar…" / "Discard and switch…"                                                                               |
| Firma          | "No se pudo firmar el commit." / "The commit couldn't be signed." · "Abrir ajustes de Git" / "Open Git settings" · "Commit sin firmar" / "Commit unsigned"                                                                                                                               |
| Servidor       | "El servidor rechazó el push:" / "The server rejected the push:"                                                                                                                                                                                                                         |

- **Variantes.** Claro y oscuro; un dialogo por caso.

### 7. D-19 Conflictos fuera del merge

- **Proposito.** Pull, rebase, cherry-pick, revert y aplicar un stash tambien paran con conflictos.
- **Muestra.** El mismo resolvedor. Cambian la caja `--warn` del panel derecho, el texto de la fila
  WIP, las etiquetas A y B (B-23) y los botones:

| Operacion                | Caja del panel                           | A                    | B                                | Botones                                                |
| ------------------------ | ---------------------------------------- | -------------------- | -------------------------------- | ------------------------------------------------------ |
| Merge o pull con merge   | "Merge en curso"                         | Tu rama (`{branch}`) | Entrante (`{source}`)            | Completar merge · Abortar merge                        |
| Rebase o pull con rebase | "Rebase en curso · {current} de {total}" | Base (`{onto}`)      | Commit que se reaplica (`{sha}`) | Continuar rebase · Omitir este commit · Abortar rebase |
| Cherry-pick              | "Cherry-pick en curso"                   | Tu rama (`{branch}`) | Commit elegido (`{sha}`)         | Continuar · Omitir · Abortar                           |
| Revert                   | "Revert en curso"                        | Tu rama (`{branch}`) | Revert de `{sha}`                | Continuar · Omitir · Abortar                           |
| Aplicar stash            | "Stash aplicado con conflictos"          | Tu rama (`{branch}`) | `stash@{index}`                  | Terminar · Eliminar el stash (tras resolver)           |

- Al terminar un stash con conflictos, una linea recuerda que el stash no se borro.
- "Continuar" se activa cuando todos los archivos estan resueltos; "Omitir" pide confirmacion.

| Uso       | Texto                                                                                                                                                                                   |
| --------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Fila WIP  | "// Rebase en curso · {current} de {total} · resuelve los conflictos" / "// Rebase in progress · {current} of {total} · resolve conflicts" (y equivalentes por operacion)               |
| Etiquetas | "Base" / "Base" · "Commit que se reaplica" / "Commit being replayed" · "Commit elegido" / "Picked commit" · "Revert de {sha}" / "Revert of {sha}"                                       |
| Botones   | "Continuar rebase" / "Continue rebase" · "Omitir este commit" / "Skip this commit" · "Abortar rebase" / "Abort rebase" · "Terminar" / "Finish" · "Eliminar el stash" / "Drop the stash" |
| Stash     | "El stash no se eliminó porque hubo conflictos." / "The stash wasn't dropped because there were conflicts."                                                                             |
| Omitir    | "¿Omitir {sha}? Sus cambios no se aplicarán." / "Skip {sha}? Its changes won't be applied."                                                                                             |

- **Variantes.** Claro y oscuro; una por operacion. Densidad compacta de 28 px para el panel del
  resolvedor, porque VS Code reutiliza el editor de merge como webview (V-07 lo especifica #68).

### 8. D-20 Rebase detenido en `edit`

- **Proposito.** El editor de rebase ofrece `edit`, pero no existe la pantalla de rebase detenido.
- **Muestra.** Banner `--warn` bajo la barra de herramientas con el commit (sha en mono y mensaje),
  el progreso y tres botones. El panel de staging arranca con amend marcado y el mensaje del commit,
  y una linea de ayuda. El nodo del commit detenido lleva el anillo de seleccion.

| Uso     | Texto                                                                                                                                                                                |
| ------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| Banner  | "Rebase detenido para editar {sha} «{message}» · {current} de {total}" / "Rebase stopped to edit {sha} “{message}” · {current} of {total}"                                           |
| Ayuda   | "Cambia lo que quieras, prepáralo y pulsa Modificar último commit. Después, Continuar rebase." / "Change what you need, stage it and click Amend last commit. Then Continue rebase." |
| Botones | "Continuar rebase" / "Continue rebase" · "Omitir commit" / "Skip commit" · "Abortar rebase" / "Abort rebase"                                                                         |

- **Variantes.** Claro y oscuro.

### 9. D-21 Mensaje del `squash` durante el rebase

- **Proposito.** git pide el mensaje del commit combinado; sin editor, el squash no se completa bien.
- **Muestra.** Modal de 580 px: lista de los commits combinados (sha y mensaje), un area de texto con
  los mensajes concatenados (separados por linea en blanco, sin las lineas de comentario de git),
  contador de 72 en la primera linea (grafemas, B-09) y "✦ Generar con IA". No se puede cerrar sin
  elegir: "Continuar rebase" o "Abortar rebase". `fixup` no pregunta; `reword` usa el texto ya escrito
  en el editor.

| Uso     | Texto                                                                                                            |
| ------- | ---------------------------------------------------------------------------------------------------------------- |
| Titulo  | "Mensaje del commit combinado" / "Combined commit message"                                                       |
| Sub     | "Se combinan {count} commits. Edita el mensaje final." / "{count} commits are combined. Edit the final message." |
| Botones | "Continuar rebase" / "Continue rebase" · "Abortar rebase" / "Abort rebase"                                       |

- **Variantes.** Claro y oscuro; con y sin IA generando.

### 10. D-22 Mensaje de merge y de squash merge

- **Proposito.** La casilla "Editar el mensaje antes de confirmar" existe y no abre nada (B-04).
- **Muestra.** Si la casilla esta marcada, o la estrategia es Squash, al terminar el merge sin
  conflictos se abre el mismo editor de D-21 con el mensaje por defecto: "Merge branch '{source}' into
  {target}", o para squash "Squash de {source}" mas la lista de commits en el cuerpo. Botones "Crear commit" y
  "Cancelar". Cancelar un merge pide confirmar que se deshace; cancelar un squash ofrece "Dejar los
  cambios preparados" o "Deshacer el squash".
- **Fast-forward imposible.** Si la estrategia es solo fast-forward y no se puede, error en el modal
  de merge con "Crear commit de merge" como salida.

| Uso      | Texto                                                                                                                                                              |
| -------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| Titulos  | "Mensaje del merge" / "Merge message" · "Mensaje del squash" / "Squash message"                                                                                    |
| Botones  | "Crear commit" / "Create commit" · "Dejar los cambios preparados" / "Keep changes staged" · "Deshacer el squash" / "Undo the squash"                               |
| Cancelar | "¿Cancelar el merge? Se deshace y la rama vuelve a como estaba." / "Cancel the merge? It's undone and the branch goes back to how it was."                         |
| FF       | "No se puede hacer fast-forward: las ramas se han separado." / "Can't fast-forward: the branches have diverged." · "Crear commit de merge" / "Create merge commit" |

- **Variantes.** Claro y oscuro.

### 11. D-23 HEAD separado

- **Proposito.** Hoy es un toast y se pueden perder commits sin enterarse.
- **Muestra.** Banner `--info` sobre el grafo con "Crear rama aquí" (primario pequeno) y "Volver a
  {branch}". El selector de rama de la barra ensena "HEAD separado · {sha}" en `--warn`. El boton de
  commit dice "Commit en HEAD separado". El menu de commit dice "Checkout {sha} (HEAD separado)". Al
  salir con commits que no estan en ninguna rama, confirmacion con "Crear rama" y "Salir igualmente"
  (peligro).

| Uso      | Texto                                                                                                                                                                                                                      |
| -------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Banner   | "HEAD separado en {sha}. Los commits que hagas aquí no pertenecen a ninguna rama." / "Detached HEAD at {sha}. Commits made here don't belong to any branch."                                                               |
| Botones  | "Crear rama aquí" / "Create branch here" · "Volver a {branch}" / "Back to {branch}"                                                                                                                                        |
| Selector | "HEAD separado · {sha}" / "Detached HEAD · {sha}"                                                                                                                                                                          |
| Commit   | "Commit en HEAD separado" / "Commit on detached HEAD"                                                                                                                                                                      |
| Salir    | "Vas a dejar {count} commits sin rama. Si no creas una, git acabará borrándolos." / "You're leaving {count} commits without a branch. Without one, git will eventually delete them." · "Salir igualmente" / "Leave anyway" |

- **Variantes.** Claro y oscuro.

### 12. D-24 Establecer upstream y primer push

- **Proposito.** El menu lo ofrece sin dialogo y el primer push de cada rama lo necesita.
- **Muestra.** Modal de 460 px: "Remoto" (selector), "Rama remota" (mono, rellena con el mismo
  nombre, validada con las reglas de refs, B-31) y casilla "Hacer push ahora". Pulsar Push en una
  rama sin upstream abre este modal con titulo "Publicar {branch}" y boton "Publicar". Si la rama
  remota ya existe con otra historia, D-18 caso 1.

| Uso     | Texto                                                                                   |
| ------- | --------------------------------------------------------------------------------------- |
| Titulos | "Establecer upstream" / "Set upstream" · "Publicar {branch}" / "Publish {branch}"       |
| Campos  | "Remoto" / "Remote" · "Rama remota" / "Remote branch" · "Hacer push ahora" / "Push now" |
| Botones | "Guardar" / "Save" · "Publicar" / "Publish"                                             |
| Toast   | "{branch} sigue a {remote}/{branch}" / "{branch} now tracks {remote}/{branch}"          |

- **Variantes.** Claro y oscuro.

### 13. D-25 Deshacer y rehacer

- **Proposito.** Los botones existen; detras hay un diario de operaciones en el motor y la interfaz
  tiene que decir sus limites.
- **Muestra.**
  - Tooltip del boton con lo que se va a deshacer ("Deshacer: commit {sha} «{message}»"). Sin nada,
    desactivado con "Nada que deshacer".
  - El boton Deshacer gana un ▾ con el historial: las ultimas 20 operaciones (icono, descripcion,
    hora); las que no se pueden deshacer, en `--faint` con su motivo. Pulsar una deshace hasta ella,
    con confirmacion.
  - Toast `info` tras deshacer, con accion "Rehacer".
  - Deshacer algo que ya esta en el remoto: confirmacion `--warn` que avisa de reescribir historia
    publicada.
- **Se pueden deshacer:** commit, amend, checkout, crear, borrar y renombrar rama, merge, rebase,
  reset, cherry-pick, revert y stash. **No:** push, fetch, descartar cambios sin commit y borrar un
  stash.

| Uso       | Texto                                                                                                                                                                                                             |
| --------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Tooltips  | "Deshacer: {operation}" / "Undo: {operation}" · "Rehacer: {operation}" / "Redo: {operation}" · "Nada que deshacer" / "Nothing to undo"                                                                            |
| Historial | "Historial de operaciones" / "Operation history" · "Deshacer hasta aquí" / "Undo to here"                                                                                                                         |
| Motivos   | "Ya está en el remoto: no se puede deshacer desde aquí." / "It's already on the remote: can't be undone from here." · "Los cambios descartados no se pueden recuperar." / "Discarded changes can't be recovered." |
| Toast     | "Deshecho: {operation}" / "Undone: {operation}" · "Rehacer" / "Redo"                                                                                                                                              |
| Publicado | "{sha} ya está en {remote}. Deshacerlo reescribe historia que otros pueden tener." / "{sha} is already on {remote}. Undoing it rewrites history others may have."                                                 |

- **Variantes.** Claro y oscuro; historial abierto.

## Reglas

- **OAuth publico primero y usuario detectado** (X-07, ADR 0018): el usuario y los permisos se leen
  de la plataforma; nunca hay un campo "Usuario" en el alta, salvo en D-13.
- **"App password" no aparece en ningun sitio** (X-08); Azure DevOps pide organizacion y no ofrece
  OAuth.
- **Sin cuentas compartidas con el equipo** (X-01): ninguna casilla ni etiqueta de equipo en cuentas.
- **La frase de GPG no se dibuja como dialogo propio** (X-13): la pide `pinentry`.
- **Credenciales por host exacto**: D-15 y D-13 solo ofrecen cuentas cuyo host coincide exactamente
  con el del remoto; punycode para hosts que no son ASCII.
- **Campos de secreto ocultos** que nunca vuelven a ensenar el valor guardado (B-37); editar una
  cuenta conserva el metodo (B-35); quitar la predeterminada promueve otra (B-36).
- Bugs que no se copian: B-04 (la estrategia de merge y "editar mensaje" tienen efecto), B-09
  (contador por grafemas), B-12 (`-x` en linea nueva con sha completo), B-23 (etiquetas segun la
  operacion), B-25 (rebase real, validado antes), B-26 (las acciones del menu de rama actuan sobre
  esa rama), B-27, B-29 (toda accion destructiva confirma), B-31, B-42, B-61 (toast de informacion
  en `--info`).
- Acciones destructivas con el modal de peligro de #71 (boton `--danger`, foco inicial en
  "Cancelar"): force push, borrar el bloqueo, descartar y cambiar, salir de HEAD separado con commits
  sueltos, abortar.
- Textos al diccionario `D`, colores de los tokens, ningun "simulado" (B-46). Datos de ejemplo con
  correos de `example.com`.

## Entregable

- Zip `git-rudder-handoff-65-git.zip` con la estructura del original: `README.md` (seccion de pantallas
  nuevas con los ids), `Git Rudder Git.dc.html`, `git-rudder-data.js` (original mas claves nuevas) y
  `support.js`.
- `Git Rudder Git.dc.html` reutiliza la vista de repositorio y Ajustes > Plataformas Git de
  `Git Rudder.dc.html`. Si D-42 (#67) ya existe, usa su barra de titulo.
- Props: `theme`, `lang`, `density` (`desktop` de 40 px, `vscode` de 28 px, solo para el resolvedor
  de D-19) y `screen`: `d12-plataforma`, `d12-github-codigo`, `d12-navegador`, `d12-token`,
  `d12-azure`, `d12-exito`, `d12-error-permisos`, `d12-error-sso`, `d12-actualizar`, `d13-https`,
  `d13-ssh`, `d13-huella`, `d13-huella-cambiada`, `d14-tarjetas`, `d14-banner`, `d15-submenu`,
  `d17-progreso`, `d18-push-rechazado`, `d18-index-lock`, `d18-hook`, `d18-arbol-sucio`, `d18-firma`,
  `d19-pull`, `d19-rebase`, `d19-cherry-pick`, `d19-revert`, `d19-stash`, `d20-edit`, `d21-squash`,
  `d22-merge`, `d23-head-separado`, `d24-upstream`, `d25-historial`.
- Juntos en el mismo lienzo: D-12 a D-15 (cuentas) y D-17 a D-25 (operaciones), en dos secciones.

## Hecho cuando

- [ ] D-12 a D-15 y D-17 a D-25 diseñados
- [ ] El alta de cuentas ofrece OAuth primero y detecta el usuario
- [ ] La frase de GPG no se dibuja como diálogo propio
