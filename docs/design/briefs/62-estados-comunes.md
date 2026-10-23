# Brief #62: estados comunes (C-01 a C-10, B-61)

Issue: [eabl0306/git-rudder#62](https://github.com/eabl0306/git-rudder/issues/62). Fuente: `docs/design/gaps.md`,
seccion 1.1 y fila B-61.

## Contexto

- Productos: los tres. Se disena una vez y se reutiliza en Git Rudder Desktop (`Git Rudder.dc.html`), Git Rudder for
  VS Code (`Git Rudder VS Code.dc.html`) y Git Rudder Web (`Git Rudder Web.dc.html`). En codigo vivira en `@git-rudder/ui`.
- El prototipo no tiene ni un estado de carga, de error ni sin conexion. El unico estado asincrono es
  "Generando..." de la IA. El toast `err` existe y nunca se emite.
- Componentes del prototipo que hay que reutilizar tal cual: el **toast** (abajo a la derecha, punto de
  color, `toastIn .2s`), el **modal generico** (460 o 580 px, radio 16, campos info, text, textarea,
  select, segmented, checkbox y action, boton principal acento o peligro), la **tarjeta** de ajustes
  (radio 14) con sus **filas de ajustes**, el **menu contextual**, la **caja de error** del formulario de
  autenticacion web (`au.error`), la caja punteada de **"Sin cuentas conectadas"** y la linea **"Sin
  cambios"** del staging. En VS Code: la **notificacion** con "Origen: Git Rudder", el **quick input** (600 px
  arriba), las cabeceras de seccion de 22 px y la **barra de estado**.
- Las pantallas propias de cada producto (primer arranque D-05, panel web vacio W-19, errores de git
  D-18...) son de otros briefs: aqui se fija el patron que esos briefs usan.

## Pantallas y estados

Densidades (tokens de `@git-rudder/ui`, ya definidos): escritorio y web `comfortable` (filas 40 px, menu 32,
barra lateral 30, barra de herramientas 62); VS Code `compact` (filas 28 px, menu 24, barra lateral 22,
barra de herramientas 34). Cada estado se dibuja en claro y oscuro y, donde aplica, en las dos
densidades.

### 1. C-01 Esqueletos de carga

- **Proposito.** Que un dato que tarda no parezca roto ni vacio.
- **Muestra.** Bloques grises con la forma exacta del contenido final, misma altura de fila: tarjeta de
  repo (rejilla `34px | 1.5fr | 1.2fr | 1fr | 224px`), fila de la tabla de PRs del workspace, fila de
  miembro, tarjeta de cuenta de plataforma, fila de llave de IA, fila de sesion, panel de detalle de
  commit (sha, titulo de dos lineas, autor, lista de archivos) y filas del grafo (40 px o 28 px). En la
  web, mosaicos del resumen y cada seccion del panel por separado.
- **Detalles.** Bloque `--surface2` sobre `--surface` (en VS Code, `--vscode-editorWidget-background`
  sobre el fondo del editor), radio 5 a 7. Brillo que recorre el bloque en 1,2 s; con
  `prefers-reduced-motion`, estatico. Aparece a los 150 ms (si el dato llega antes, no parpadea). Entre
  3 y 6 filas de muestra. Lector de pantalla: la region con `aria-busy` y la etiqueta "Cargando...".
- **Variantes asociadas.** Fila "Cargando mas..." al final de una lista paginada. Boton ocupado: el
  mismo boton con un indicador giratorio de 14 px a la izquierda y el texto en gerundio, deshabilitado.
  En VS Code las vistas nativas usan la barra de progreso nativa de la vista; los webviews (grafo,
  merge, ajustes) usan una barra indeterminada de 2 px arriba (`--vscode-progressBar-background`) y
  esqueletos de 28 px en el grafo.
- **Si tarda mucho.** A los 10 s, bajo el esqueleto: texto de espera y "Reintentar".
- **Estados.** Cargando, cargando mas, boton ocupado, tarda mas de lo normal.

| Clave        | ES                              | EN                                |
| ------------ | ------------------------------- | --------------------------------- |
| loading      | Cargando…                       | Loading…                          |
| loadingMore  | Cargando más…                   | Loading more…                     |
| slowLoad     | Está tardando más de lo normal. | This is taking longer than usual. |
| retry        | Reintentar                      | Retry                             |
| busySaving   | Guardando…                      | Saving…                           |
| busyChecking | Comprobando…                    | Checking…                         |

### 2. C-02 Toast de error y dialogo de error con detalle

- **Proposito.** Los errores de git traen salida larga (hooks, rechazos); un toast de 3 s no la puede
  ensenar ni dejar copiarla.
- **Toast.** Cuatro tipos con el mismo componente: `ok` (punto `--ok`), `info` (punto `--info`), `warn`
  (punto `--warn`) y `error` (punto `--danger`). `ok` e `info` duran 3,2 s; `warn` 6 s; `error` se
  queda hasta que se cierra con su boton x o se pulsa su accion. Un toast puede llevar una accion de
  texto a la derecha ("Ver detalle", "Reintentar"). Como mucho tres a la vez, el mas nuevo abajo.
- **Dialogo de error.** Modal ancho (580). Titulo: que fallo, en lenguaje de usuario. Debajo, una frase
  con que hacer. Bloque "Detalle": fondo `--bg`, borde, radio 9, IBM Plex Mono 12 px, interlineado 18,
  saltos de linea respetados, altura maxima 240 px con scroll. Contiene el comando de git y su salida,
  o el `code` y el `detail` de un problem+json de la API (nunca una traza). Plegado por defecto con
  "Mostrar detalle"; desplegado por defecto cuando la salida es la causa (hook, push rechazado).
  Cabecera del bloque con "Copiar detalle" (boton de 26 px). Pie: accion principal segun el caso
  ("Reintentar") y "Cerrar".
- **Banner de error en linea.** Para una seccion o panel que no carga (lo usaran W-20 y los paneles de
  Desktop): franja con borde y fondo `--danger` al 10 %, icono, texto, "Reintentar" y "Ver detalle".
- **VS Code.** Notificacion nativa de error con botones "Ver detalle" (abre el canal de salida "Git Rudder"
  con el comando y la salida) y "Reintentar". Dentro de un webview, el banner en linea con tokens de
  VS Code (`--vscode-inputValidation-errorBorder` y `errorBackground`).
- **Estados.** Toast de los cuatro tipos (con y sin accion), dialogo plegado, dialogo desplegado con
  salida larga y scroll, detalle copiado, banner en linea.
- **Ejemplo que dibujar.** Push rechazado por el hook `pre-push` (salida de lint de 12 lineas) y un error
  de la API con `code: internal`.

| Clave         | ES                                                                       | EN                                                                 |
| ------------- | ------------------------------------------------------------------------ | ------------------------------------------------------------------ |
| errFetchToast | No se pudo hacer fetch de {remote}                                       | Couldn't fetch {remote}                                            |
| viewDetail    | Ver detalle                                                              | View details                                                       |
| showDetail    | Mostrar detalle                                                          | Show details                                                       |
| detail        | Detalle                                                                  | Details                                                            |
| copyDetail    | Copiar detalle                                                           | Copy details                                                       |
| detailCopied  | Detalle copiado                                                          | Details copied                                                     |
| close         | Cerrar                                                                   | Close                                                              |
| hookTitle     | El hook {hook} rechazó el push                                           | The {hook} hook rejected the push                                  |
| hookHelp      | Revisa la salida del hook, corrige lo que indique y vuelve a intentarlo. | Check the hook output, fix what it reports and try again.          |
| saveFailTitle | No se pudo guardar                                                       | Couldn't save                                                      |
| apiDownHelp   | Git Rudder no respondió. Inténtalo de nuevo en unos minutos.             | Git Rudder didn't respond. Try again in a few minutes.             |
| sectionFail   | No se pudo cargar esta sección.                                          | This section couldn't load.                                        |
| errTimeout    | La operación tardó demasiado y se canceló.                               | The operation took too long and was canceled.                      |
| errTransport  | Se perdió la comunicación con el motor de Git Rudder.                    | Lost contact with the Git Rudder engine.                           |
| errInternal   | Algo falló en Git Rudder. Copia el detalle si quieres avisarnos.         | Something went wrong in Git Rudder. Copy the details to report it. |

### 3. C-03 Errores de validacion en linea en los modales

- **Proposito.** Hoy un modal invalido no hace nada y no dice por que (`onSubmit` devuelve `false`).
- **Muestra.** Campo con error: borde `--danger`, mensaje de 12 px en `--danger` debajo del campo con
  icono de aviso. Texto de ayuda en `--faint` cuando no hay error. Error del servidor: caja de error
  arriba del cuerpo del modal, con el mismo estilo que la caja de error de la autenticacion web.
- **Comportamiento.** Se valida al salir del campo y al enviar. Al enviar con errores, el foco va al
  primer campo invalido y el boton nunca falla en silencio. Mientras corre una comprobacion asincrona
  (nombre de rama ya existe), el boton muestra "Comprobando..." deshabilitado. Los textos informativos
  respetan los saltos de linea (B-60).
- **Campos.** Text, textarea, select, segmented y checkbox (terminos). Dibujar: crear rama con nombre
  invalido (B-31), clonar con URL no reconocida, invitar con email invalido, y un error del servidor.
- **VS Code.** El quick input nativo muestra el mensaje en su linea de validacion (severidad Error); el
  diseno solo fija el texto. En webviews, el mismo patron con tokens de VS Code.
- **Estados.** Vacio, valido con ayuda, error de campo, varios errores, comprobando, error del servidor.

| Clave        | ES                                                                                               | EN                                                                                     |
| ------------ | ------------------------------------------------------------------------------------------------ | -------------------------------------------------------------------------------------- |
| reqName      | Escribe un nombre.                                                                               | Enter a name.                                                                          |
| errEmail     | Escribe un email válido.                                                                         | Enter a valid email.                                                                   |
| badBranch    | Nombre no válido: sin espacios, sin «..», sin ~ ^ : ? * [ \ y sin terminar en «/» ni en «.lock». | Invalid name: no spaces, no "..", no ~ ^ : ? * [ \ and it can't end in "/" or ".lock". |
| branchExists | Ya existe una rama con ese nombre.                                                               | A branch with that name already exists.                                                |
| tagExists    | Ya existe un tag con ese nombre.                                                                 | A tag with that name already exists.                                                   |
| badUrl       | No reconocemos esta dirección. Usa https://…, git@servidor:ruta o propietario/repo.              | We don't recognize this address. Use https://…, git@host:path or owner/repo.           |
| serverErr    | No se pudo completar: {reason}                                                                   | Couldn't complete it: {reason}                                                         |

### 4. C-04 Estados vacios

- **Proposito.** Es lo primero que ve un usuario nuevo; hoy solo algunos estan dibujados.
- **Tres tamanos del mismo patron.**
  - Seccion de barra lateral: una linea de 12 px en `--faint`, como "Sin cambios".
  - Panel o grupo: caja punteada como "Sin cuentas conectadas", con titulo, una linea de explicacion y,
    si hay, un boton secundario.
  - Area grande: bloque centrado con icono de 24 px en `--faint`, titulo de 15 px seminegrita, texto
    `--muted` y hasta dos botones (principal y secundario). Sin ilustraciones.
- **Busqueda sin resultados.** Repite el termino y ofrece "Limpiar busqueda" o "Limpiar filtros".
- **VS Code.** Los arboles vacios usan `viewsWelcome` nativo (texto y botones); el diseno fija el texto y
  las etiquetas. Los webviews usan el patron en densidad compacta.
- **Dibujar.** Stashes, tags, remotos, submodulos, PRs (y PRs sin cuenta para el remoto), cuentas
  (grupo vacio y pagina entera vacia), llaves de IA, workspace sin repos, arbol de trabajo limpio,
  busqueda de repos y de commits sin resultados. El contenido de primer arranque (D-05, V-02, W-19) es
  de los briefs #64, #68 y #70.

| Clave         | ES                                                             | EN                                                 |
| ------------- | -------------------------------------------------------------- | -------------------------------------------------- |
| noStashes     | No hay stashes                                                 | No stashes                                         |
| noStashesD    | Guarda cambios sin hacer commit con Stash.                     | Save changes without committing using Stash.       |
| noTags        | Sin tags                                                       | No tags                                            |
| noRemotes     | Este repositorio no tiene remotos                              | This repository has no remotes                     |
| addRemote     | Añadir remoto                                                  | Add remote                                         |
| noSubmodules  | Sin submódulos                                                 | No submodules                                      |
| noPrs         | No hay pull requests abiertos                                  | No open pull requests                              |
| prsNeedAcc    | Conecta una cuenta de {platform} para ver los pull requests.   | Connect a {platform} account to see pull requests. |
| connectAcc    | Conectar cuenta                                                | Connect account                                    |
| noAccounts    | Sin cuentas conectadas                                         | No connected accounts                              |
| noAccountsAll | Aún no has conectado ninguna plataforma                        | You haven't connected any platform yet             |
| noKeys        | Aún no hay llaves. Añade una para activar las funciones de IA. | No keys yet. Add one to enable AI features.        |
| addKey        | Añadir llave                                                   | Add key                                            |
| wsEmpty       | Este workspace está vacío                                      | This workspace is empty                            |
| addRepos      | Añadir repositorios                                            | Add repositories                                   |
| cleanTree     | Todo al día: no hay cambios en el árbol de trabajo.            | All clear: no changes in the working tree.         |
| noResultsQ    | Nada coincide con «{query}»                                    | Nothing matches "{query}"                          |
| noCommitsQ    | Ningún commit coincide con «{query}»                           | No commits match "{query}"                         |
| noResults     | Ningún repositorio coincide con los filtros.                   | No repositories match the filters.                 |
| clearSearch   | Limpiar búsqueda                                               | Clear search                                       |
| clearFilters  | Limpiar filtros                                                | Clear filters                                      |

### 5. C-05 Sin conexion o API de Git Rudder caida

- **Proposito.** Git local sigue funcionando sin red (ADR 0014); el estado tiene que decir que solo se
  degrada la nube.
- **Dos casos distintos.** Sin red en el equipo ("Sin conexion") y red bien pero la API no responde
  ("Git Rudder no responde"). Fetch, pull y push muestran sus propios errores (D-18, brief #65).
- **Desktop.** Indicador en la barra de titulo, a la izquierda del selector ES/EN: chip con icono de nube
  tachada y texto, fondo `--warn` al 14 %. El chip con texto solo sale si algo va mal; el resto del
  tiempo es el icono de C-06. Al pulsarlo, popover (radio 12, como un menu) con titulo, explicacion, "Cambios pendientes: N" y
  "Reintentar ahora". Las zonas de nube (Cuenta y equipo, llaves de organizacion, sincronizacion)
  muestran un banner de aviso y la ultima copia guardada. Lo que necesita la API en vivo (invitar, crear
  o desbloquear la boveda, generar con una llave de organizacion, probarla) queda deshabilitado con
  tooltip.
- **VS Code.** Elemento nativo en la barra de estado con icono `cloud-offline` y tooltip; banner en el
  webview de ajustes.
- **Web.** Banner arriba del panel ("Reintentando..."); en el login, la caja de error. Enlace a la pagina
  de estado `https://git-rudder.com/status`.
- **Estados.** Sin conexion, Git Rudder no responde, reconectando, conexion recuperada (toast `ok`), control
  deshabilitado con tooltip, banner de zona de nube con copia guardada, N cambios pendientes.

| Clave         | ES                                                                                                                                  | EN                                                                                                                  |
| ------------- | ----------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------- |
| offline       | Sin conexión                                                                                                                        | Offline                                                                                                             |
| offlineD      | Git local sigue funcionando. Lo que cambies en los ajustes se guarda en este equipo y se sincroniza cuando vuelva la conexión.      | Local Git keeps working. Settings you change are saved on this device and sync when you're back online.             |
| apiDown       | Git Rudder no responde                                                                                                              | Git Rudder isn't responding                                                                                         |
| apiDownD      | Tu conexión funciona, pero los servidores de Git Rudder no responden. Git local sigue funcionando y lo volveremos a intentar solos. | Your connection works, but Git Rudder's servers aren't responding. Local Git keeps working and we'll retry for you. |
| statusLink    | Ver el estado del servicio                                                                                                          | View service status                                                                                                 |
| pending_one   | {count} cambio pendiente de sincronizar                                                                                             | {count} change waiting to sync                                                                                      |
| pending_other | {count} cambios pendientes de sincronizar                                                                                           | {count} changes waiting to sync                                                                                     |
| retryNow      | Reintentar ahora                                                                                                                    | Retry now                                                                                                           |
| reconnecting  | Reconectando…                                                                                                                       | Reconnecting…                                                                                                       |
| backOnline    | Conexión recuperada                                                                                                                 | Back online                                                                                                         |
| needsNet      | Necesitas conexión para esto                                                                                                        | You need a connection for this                                                                                      |
| cachedCopy    | Sin conexión. Ves la última copia guardada ({ago}).                                                                                 | Offline. You're seeing the last saved copy ({ago}).                                                                 |
| loginNoApi    | No podemos conectar con Git Rudder. Inténtalo de nuevo en unos minutos.                                                             | We can't reach Git Rudder. Try again in a few minutes.                                                              |

### 6. C-06 Estado de sincronizacion

- **Proposito.** Los ajustes sincronizados usan concurrencia optimista (412 `version_conflict`) y una
  cola de escrituras sin conexion; el usuario tiene que ver si algo se sincronizo y cuando.
- **Desktop.** El mismo indicador de la barra de titulo de C-05, en su estado bueno: icono de nube sin
  texto; tooltip "Sincronizado hace 2 min". Estados: sincronizado, sincronizando (icono girando),
  pendiente (N), error (punto `--danger`), sin conexion (C-05) y sin sesion (no aparece: la zona de
  cuenta invita a entrar, D-04). En Ajustes, bajo el titulo de cada seccion que se sincroniza (Git, IA,
  Apariencia), una linea de 12 px en `--faint`: "Se sincroniza con tu cuenta de Git Rudder · hace 2 min", o
  "Solo en este equipo" sin sesion o con la categoria apagada.
- **Cambio en otro dispositivo.** Cuando llega un cambio de otro equipo con la pantalla abierta, la fila
  afectada se resalta 2 s con `--info` al 10 % y sale un toast `info`. Una escritura descartada porque la
  categoria se apago en otro equipo (409 `sync_category_disabled`) da un toast `warn`.
- **Guardado inmediato (B-57).** Un solo modelo en los tres productos: cada control guarda solo, sin
  boton "Guardar" global. Los campos de texto guardan al salir del campo y muestran "Guardando..." y
  "Guardado" (con marca, se desvanece a los 2 s) junto al control; si falla, "No se pudo guardar" con
  "Reintentar".
- **VS Code.** Linea bajo el titulo de "Configuracion de Git Rudder" y tooltip del elemento de barra de
  estado. **Web.** W-25 (brief #70) reutiliza este componente.
- **Estados.** Sincronizado, sincronizando, pendiente, error con detalle, actualizado desde otro
  dispositivo (fila resaltada), escritura descartada, guardando, guardado, fallo al guardar, solo en este
  equipo.

| Clave       | ES                                                                                                            | EN                                                                                      |
| ----------- | ------------------------------------------------------------------------------------------------------------- | --------------------------------------------------------------------------------------- |
| synced      | Sincronizado                                                                                                  | Synced                                                                                  |
| syncedAgo   | Sincronizado {ago}                                                                                            | Synced {ago}                                                                            |
| syncing     | Sincronizando…                                                                                                | Syncing…                                                                                |
| syncFail    | No se pudo sincronizar                                                                                        | Couldn't sync                                                                           |
| syncsWith   | Se sincroniza con tu cuenta de Git Rudder · {ago}                                                             | Syncs with your Git Rudder account · {ago}                                              |
| localOnly   | Solo en este equipo                                                                                           | This device only                                                                        |
| localOnlyD  | Inicia sesión para sincronizarlo.                                                                             | Sign in to sync it.                                                                     |
| updatedElse | Actualizado desde otro dispositivo                                                                            | Updated from another device                                                             |
| discarded   | No se guardó en la nube: la sincronización de {category} está desactivada. El cambio se queda en este equipo. | Not saved to the cloud: {category} sync is turned off. The change stays on this device. |
| saving      | Guardando…                                                                                                    | Saving…                                                                                 |
| saved       | Guardado                                                                                                      | Saved                                                                                   |
| saveFail    | No se pudo guardar                                                                                            | Couldn't save                                                                           |

### 7. C-07 Sesion caducada o revocada desde otro dispositivo

- **Proposito.** El refresh token puede caducar (90 dias sin uso en las apps, 14 en la web, 365 como
  maximo) o revocarse desde el panel; hoy no hay forma de enterarse.
- **Dos variantes, segun el `code` del 401 al refrescar (ADR 0008 y ADR 0012).**
  - `session_ended` (revocada, caducada o reutilizacion detectada): el motor ya borro la llave de la
    boveda y lo que llego de otros equipos. El dialogo lo dice.
  - Cualquier otro 401 (`unauthorized`, un proxy, un fallo del servidor): no se borra nada. El dialogo
    pide volver a iniciar sesion y dice que las cuentas de este equipo siguen ahi.
  - Un fallo de red o un 409 `refresh_superseded` no ensenan dialogo: como mucho el "Reconectando..."
    de C-05.
- **Desktop.** Modal (460) que no se cierra al pulsar fuera. Titulo, motivo, la linea de limpieza solo
  en la variante `session_ended` (regla de K-09, brief #63) y dos botones. Al cerrarlo, la app queda en
  el estado sin sesion (D-04).
- **VS Code.** Notificacion nativa de aviso con "Iniciar sesion"; el menu de cuentas de VS Code pide
  entrar.
- **Web.** Redireccion al login con un banner informativo arriba de la tarjeta y vuelta a la pagina de
  origen al entrar (W-09, brief #69).
- **Estados.** `session_ended` con limpieza, otro 401 sin limpieza, web con banner. El `code` no
  distingue revocada de caducada, asi que el motivo es una sola frase.

| Clave         | ES                                                                                                                                                                             | EN                                                                                                                                                              |
| ------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ | --------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| sessEndTitle  | Tu sesión de Git Rudder se cerró                                                                                                                                               | Your Git Rudder session has ended                                                                                                                               |
| sessEnded     | Se cerró desde otro dispositivo o desde el panel web, o caducó tras un tiempo sin usar Git Rudder en este equipo.                                                              | It was ended from another device or the web dashboard, or it expired after a while without using Git Rudder on this device.                                     |
| sessRenewFail | No pudimos renovar tu sesión. Vuelve a iniciarla: tus cuentas y llaves de este equipo siguen aquí.                                                                             | We couldn't renew your session. Sign in again: your accounts and keys on this device are still here.                                                            |
| sessCleanup   | Por seguridad, borramos de este equipo la llave de tu bóveda y las cuentas y llaves que llegaron de otros equipos. Lo que añadiste aquí sigue disponible y Git funciona igual. | For security, we removed your vault key and the accounts and keys that came from other devices. What you added here is still available and Git works as before. |
| signIn        | Iniciar sesión                                                                                                                                                                 | Sign in                                                                                                                                                         |
| noAccount     | Seguir sin cuenta                                                                                                                                                              | Continue without an account                                                                                                                                     |
| webExpired    | Tu sesión caducó. Vuelve a iniciar sesión para continuar.                                                                                                                      | Your session expired. Sign in again to continue.                                                                                                                |

### 8. C-08 Permiso denegado y vistas de solo lectura por rol

- **Proposito.** Todo el diseno asume que quien mira es titular de la organizacion. Roles: titular,
  admin y miembro (etiqueta "Titular", nunca "Propietaria": B-50).
- **Reglas de dibujo.** Una accion que el rol nunca puede hacer no aparece en los menus. Un control que
  se ve pero no se puede tocar (el selector de rol de otro miembro) va deshabilitado con tooltip.
  Arriba de una seccion de solo lectura, banner informativo (`--info` al 10 %, icono, texto).
- **403 inesperado** (el permiso cambio mientras miraba): toast `error` y la vista se recarga con el rol
  nuevo. **404** de algo de otra cuenta: mensaje neutro, sin decir si existe.
- **Dibujar.** Equipo visto por un miembro (Desktop "Cuenta y equipo" y web "Equipo"), llaves de IA de
  la organizacion vistas por un miembro, facturacion vista por un admin (solo el titular la gestiona).
- **Estados.** Solo lectura con banner, control deshabilitado con tooltip, toast de 403, vacio de 404.

| Clave      | ES                                                                                                 | EN                                                                                   |
| ---------- | -------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------ |
| roleOwner  | Titular                                                                                            | Owner                                                                                |
| roleAdmin  | Admin                                                                                              | Admin                                                                                |
| roleMember | Miembro                                                                                            | Member                                                                               |
| roTeam     | Eres miembro de {org}. Puedes ver el equipo; invitar y cambiar roles lo hacen los administradores. | You're a member of {org}. You can see the team; admins handle invitations and roles. |
| roOrgKeys  | Las llaves de la organización las gestionan los administradores.                                   | Organization keys are managed by admins.                                             |
| roBilling  | Solo el titular gestiona la suscripción.                                                           | Only the owner manages the subscription.                                             |
| onlyAdmins | Solo los administradores pueden cambiar esto                                                       | Only admins can change this                                                          |
| onlyOwner  | Solo el titular puede hacer esto                                                                   | Only the owner can do this                                                           |
| forbidden  | Ya no tienes permiso para hacer esto. Pide a un administrador de {org} que lo haga.                | You no longer have permission to do this. Ask an admin of {org} to do it.            |
| notFound   | No encontramos esto, o ya no tienes acceso.                                                        | We couldn't find this, or you no longer have access.                                 |

### 9. C-09 Tema del sistema e idioma del sistema

- **Proposito.** Hoy solo hay claro y oscuro y el idioma arranca fijo en castellano.
- **Tema.** Tres opciones: Sistema (por defecto), Claro y Oscuro. En Ajustes > Apariencia de Desktop, un
  segmented de tres. El conmutador rapido de la barra de titulo de Desktop y el de la web pasan de dos a
  tres botones de icono (medio circulo, sol y luna) con tooltip. Con "Sistema", la app cambia en vivo
  cuando cambia el sistema.
- **Idioma.** Regla de ADR 0017: `es` si el idioma del sistema empieza por `es`; si no, `en`. Desktop lo
  aplica en el primer arranque y despues manda el ajuste (ES o EN, sin opcion "Sistema"). La web toma el
  idioma de la URL en la landing y la preferencia guardada en autenticacion y panel. VS Code sigue a VS
  Code y no tiene selectores (el texto `followsVs` ya existe).
- **Estados.** Segmented en cada opcion, conmutador de tres en claro y oscuro, aviso de idioma detectado
  (lo usa el primer arranque, brief #64).

| Clave        | ES                                                        | EN                                                    |
| ------------ | --------------------------------------------------------- | ----------------------------------------------------- |
| themeSystem  | Sistema                                                   | System                                                |
| themeLight   | Claro                                                     | Light                                                 |
| themeDark    | Oscuro                                                    | Dark                                                  |
| themeSystemD | «Sistema» sigue el modo claro u oscuro de tu equipo.      | "System" follows your device's light or dark mode.    |
| langDetected | Detectado de tu sistema. Puedes cambiarlo cuando quieras. | Detected from your system. You can change it anytime. |

### 10. C-10 Foco de teclado visible

- **Proposito.** El prototipo no dibuja foco; sin el, la app no se usa con teclado.
- **Regla.** Solo con `:focus-visible` (teclado), nunca al pulsar con el raton. Anillo de 2 px en
  `--r-focus` (el acento) con 2 px de separacion y el radio del control. Campos de texto: borde
  `--r-focus`. Filas de lista, tarjetas de repo y elementos de menu: fondo `--surface2` mas anillo
  interior de 2 px. Fila del grafo con foco: anillo interior. En VS Code: contorno de 1 px en
  `--vscode-focusBorder`, interior en filas, como hace VS Code.
- **Dibujar en una hoja de estados.** Para boton principal, secundario, de peligro, de icono, toggle,
  segmented, checkbox, select, campo de texto, pestana de la barra de titulo, fila de lista, tarjeta de
  repo, elemento de menu contextual y fila del grafo: reposo, hover, foco, pulsado y deshabilitado.
- **Teclado.** En modales el foco queda atrapado; Esc cierra; Enter envia, salvo en dialogos de peligro,
  donde el foco inicial va a "Cancelar". En la web, enlace "Saltar al contenido" visible al recibir foco.
  Todo boton de solo icono lleva etiqueta accesible.

| Clave       | ES                  | EN              |
| ----------- | ------------------- | --------------- |
| skipContent | Saltar al contenido | Skip to content |
| moreActions | Más acciones        | More actions    |
| closeLabel  | Cerrar              | Close           |

## Reglas

- **B-61:** el toast `info` usa `--info`. `--accent` queda para la accion principal y la marca. Corrige
  tambien la linea del toast en `Git Rudder.dc.html` (`x.kind === 'info' ? 'var(--accent)'`).
- Solo tokens existentes (tabla del README del handoff; en codigo, los `--r-*` de `@git-rudder/ui`). Ningun
  color nuevo; los fondos suaves son el color al 10 o 14 % con `color-mix`.
- Ningun texto dice "simulado" (B-46). Plurales con claves `_one` y `_other` sobre `{count}` (B-42,
  ADR 0017). Marcadores con nombre (`{count}`, `{remote}`), nunca `{n}` o `{m}` ambiguos (B-44). Glifos
  fuera de las cadenas (B-43).
- Saltos de linea respetados en cajas informativas (B-60).
- En VS Code lo nativo se especifica, no se dibuja (X-11): notificaciones, quick input, barra de estado y
  `viewsWelcome` se describen con su texto.
- Solo un 401 con `code` `session_ended` borra secretos de este equipo (decision #57, ADR 0012). Ningun
  texto de otro 401 o de un fallo de red dice que se borro algo.
- Los mensajes de error para el usuario salen del `code`, nunca del `message` en ingles del motor o de la
  API, que es para el log.

## Entregable

- Un zip `git-rudder-handoff-62-estados.zip` con la misma estructura que el handoff original:
  - `README.md` con solo lo nuevo: componentes, medidas, tokens usados y la tabla de estados.
  - `Git Rudder Estados.dc.html`: un Design Component con props `product` (`desktop`, `vscode`, `web`),
    `theme` (`dark`, `light`), `lang` (`es`, `en`) y `state` (`C-01` a `C-10`, o `all` para ver la
    rejilla entera). Cada estado es una lamina con todas sus variantes lado a lado.
  - `git-rudder-data.js` con un objeto nuevo `STATES` de pares `[es, en]` con las claves de este brief.
  - `Git Rudder.dc.html` con la correccion de B-61 y nada mas.
  - `support.js` sin cambios.

## Hecho cuando

- [ ] Cada estado de C-01 a C-10 tiene diseño en claro y oscuro
- [ ] Hay variantes para la densidad de escritorio y la de VS Code
- [ ] El toast de información usa `--info` y no `--accent`
