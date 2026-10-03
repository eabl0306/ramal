# Brief #67: ventana, ajustes que faltan y equipo (Git Rudder Desktop)

## Contexto

Solo Git Rudder Desktop: barra de titulo sin marco, ajustes sin pantalla, cuenta y equipo por rol,
workspaces compartidos, enlaces profundos, que se sincroniza e IA. Equipos, facturacion y llaves de
IA de organizacion entran en la primera version (decision #49). Cuelga de `Git Rudder.dc.html`: barra de
46 px, pestana de Ajustes con navegacion por grupos, fila de ajustes generica, tarjetas de plan, de
sesion y de miembros, lista de llaves de IA, vista de workspace y modal generico. C-01 a C-10 (#62) y
K-01 a K-09 (#63) se enlazan, no se redibujan. D-50 y D-52 son P2 y no entran.

## Pantallas y estados

Formato de los textos: `"castellano" / "english"`. Los parametros tienen nombre en ingles y son
iguales en los dos idiomas (`{branch}`, `{host}`); los contadores usan `{count}` con claves `_one` y
`_other`.

### 0. Navegacion de Ajustes

Para que quepa lo que falta, la navegacion de Ajustes pasa a:

- Usuario: Perfil · Cuenta · Equipo (Cuenta y Equipo sustituyen a "Cuenta y equipo").
- Conexiones: Plataformas Git · IA.
- Aplicacion: Git · Herramientas externas · Apariencia · Atajos de teclado · Actualizaciones.

### 1. D-42 Barra de titulo de Tauri

- **Proposito.** Sin marco del sistema no se puede mover, minimizar ni cerrar la ventana.
- **Muestra.** La misma barra de 46 px con tres variantes:
  - macOS: hueco de 78 px a la izquierda para los semaforos nativos (centrados en vertical), y
    despues logo y pestanas.
  - Windows: controles propios a la derecha, tras el avatar: Minimizar, Maximizar o Restaurar y
    Cerrar, de 46 x 46 px, glifos de 10 px; hover en `--surface2`; Cerrar con hover en `--danger` y
    glifo blanco.
  - Linux: los mismos controles que Windows.
  - Zona de arrastre: el hueco entre la ultima pestana (o el boton +) y los controles de la derecha,
    mas el bloque del logo. Doble clic maximiza (zoom en macOS). Las pestanas y los botones no
    arrastran.
  - Pastilla "Actualización lista" (`--accent-soft`) junto a ⚙︎ cuando D-45 tiene una version
    descargada.
  - Sin sesion, el avatar es el boton "Iniciar sesión" de D-04 (#64).
- **Tamano minimo.** 1024 x 640. Dibuja la vista de repo a 1024 px: el grafo esconde las columnas
  Autor y Fecha antes de encoger el panel derecho por debajo de 280 px.
- **Estados.** Ventana activa e inactiva (texto de pestanas en `--faint`), maximizada, con muchas
  pestanas (el desbordamiento es D-52, P2: basta con que no se rompa).

| Uso      | Texto                                                                                              |
| -------- | -------------------------------------------------------------------------------------------------- |
| Tooltips | "Minimizar" / "Minimize" · "Maximizar" / "Maximize" · "Restaurar" / "Restore" · "Cerrar" / "Close" |
| Pastilla | "Actualización lista" / "Update ready"                                                             |

- **Variantes.** macOS, Windows y Linux; claro y oscuro; activa e inactiva.

### 2. D-43 Ajustes de Git: ejecutable

- **Proposito.** Con git del sistema (decision 1), el usuario tiene que ver cual se usa y cambiarlo.
- **Muestra.** Tarjeta nueva al principio de Ajustes > Git, "Ejecutable de Git": ruta en mono,
  version y una pastilla de origen ("Del PATH", "Elegido por ti" o "Incluido con Git Rudder"). Botones
  "Cambiar ruta…" (selector nativo) y "Usar el del sistema" (solo si hay ruta propia).
- **Estados.** Valido; elegido no es git; version anterior a 2.34 (error, no se guarda); version
  entre 2.34 y 2.38 (nota `--faint` de funciones que van por un camino mas lento); MinGit en Windows
  con la nota de como se actualiza; comprobando.
- **Ajustes > Git, de paso:** "Firmar commits" apagado por defecto y con la descripcion de que usa la
  llave de firma de la configuracion de Git; la fila "Generar llave SSH" se quita hasta D-16 (P2).

| Uso             | Texto                                                                                                                                                                                                               |
| --------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Titulo          | "Ejecutable de Git" / "Git executable" · "Git {version}" / "Git {version}"                                                                                                                                          |
| Origen          | "Del PATH" / "From PATH" · "Elegido por ti" / "Chosen by you" · "Incluido con Git Rudder" / "Bundled with Git Rudder"                                                                                               |
| Botones         | "Cambiar ruta…" / "Change path…" · "Usar el del sistema" / "Use the system one"                                                                                                                                     |
| No es git       | "Ese archivo no es un ejecutable de Git." / "That file isn't a Git executable."                                                                                                                                     |
| Antiguo         | "Git Rudder necesita Git 2.34 o posterior. Ese es {version}." / "Git Rudder needs Git 2.34 or later. That one is {version}."                                                                                        |
| Entre versiones | "Con Git 2.38 o posterior, el merge y el rebase de ramas apiladas van más rápido." / "With Git 2.38 or later, merge and stacked-branch rebase are faster."                                                          |
| MinGit          | "El Git incluido se actualiza cuando actualizas Git Rudder. En instalaciones MSI, cuando lo haga tu equipo de IT." / "The bundled Git updates when you update Git Rudder. On MSI installs, when your IT team does." |
| Firma           | "Usa la llave de firma de tu configuración de Git (user.signingkey)." / "Uses the signing key from your Git config (user.signingkey)."                                                                              |

- **Variantes.** Claro y oscuro; Windows con MinGit; macOS y Linux con git del sistema.

### 3. D-44 Herramientas externas

- **Proposito.** "Abrir en el editor" y "Abrir terminal aquí" existen en los menus sin ajuste detras.
- **Muestra.** Pagina nueva con cuatro filas de selector: "Editor", "Terminal", "Herramienta de diff" y
  "Herramienta de merge". Cada selector lista lo detectado en el equipo (lo no instalado aparece
  desactivado con "no encontrada"), la opcion de la configuracion de Git (`diff.tool`, `merge.tool`)
  donde aplica, "Ninguna" en diff y merge, y "Personalizada…", que despliega "Comando" y "Argumentos"
  en mono con los marcadores `{path}` y `{line}`. Boton "Probar" en cada fila.
- **Efecto visible.** Con diff configurada, el diff de #66 ensena "Abrir en herramienta de diff"; con
  merge configurada, el resolvedor ensena "Abrir en herramienta de merge".
- **Estados.** Detectando, probado con exito (toast), fallo al abrir (error en linea).

| Uso           | Texto                                                                                                                                                                              |
| ------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Titulo        | "Herramientas externas" / "External tools" · "Las apps que Git Rudder abre por ti." / "The apps Git Rudder opens for you."                                                         |
| Filas         | "Editor" · "Terminal" · "Herramienta de diff" / "Diff tool" · "Herramienta de merge" / "Merge tool"                                                                                |
| Opciones      | "La del sistema" / "System default" · "La de tu configuración de Git" / "From your Git config" · "Ninguna" / "None" · "Personalizada…" / "Custom…" · "no encontrada" / "not found" |
| Personalizada | "Comando" / "Command" · "Argumentos" / "Arguments" · "Usa {path} para la ruta y {line} para la línea." / "Use {path} for the path and {line} for the line."                        |
| Probar        | "Probar" / "Test" · "Se abrió {tool}" / "{tool} opened" · "No se pudo abrir {tool}: {error}" / "Couldn't open {tool}: {error}"                                                     |

- **Variantes.** Claro y oscuro; con personalizada desplegada.

### 4. D-45 Actualizaciones

- **Proposito.** Sin esto la app firmada nunca avisa de nada (X-09, ADR 0016).
- **Muestra.** Pagina "Actualizaciones": tarjeta de version ("Git Rudder Desktop {version}", sistema,
  arquitectura y formato instalado; pastilla "beta" en arm64 de Windows y Linux), "Canal"
  (segmentado Estable o Beta), interruptor "Buscar actualizaciones automáticamente" con lo que se
  envia, y "Buscar ahora".
- **Estados.**
  1. Comprobando.
  2. Al dia, con la hora de la ultima comprobacion.
  3. Hay version nueva: version, fecha, notas (Markdown saneado, con scroll) y "Descargar e instalar".
  4. Descargando: barra, porcentaje y tamano.
  5. Lista: "Reiniciar ahora" y "Más tarde" (deja la pastilla de D-42).
  6. Error de red con "Reintentar"; firma que no verifica (no se instala).
  7. `.deb` y `.rpm`: "Hay una versión nueva" con "Descargar", que abre la seccion de descargas de
     `git-rudder.com`.
  8. MSI: sin controles de instalar; texto de que la gestiona IT y, si hay version mas reciente, cual.
  9. Busqueda apagada: solo "Buscar ahora".
- **Version minima.** Banner `--warn` en toda la app cuando la version queda por debajo del minimo
  soportado: la nube deja de sincronizar y lo local sigue.

| Uso            | Texto                                                                                                                                                                                                                                                                                                                 |
| -------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Version        | "Git Rudder Desktop {version}" / "Git Rudder Desktop {version}" · "{os} {arch} · instalado con {format}" / "{os} {arch} · installed with {format}"                                                                                                                                                                    |
| Canal          | "Canal" / "Channel" · "Estable" / "Stable" · "Beta" · "Recibes las versiones antes. Pueden tener fallos." / "You get versions earlier. They may have bugs."                                                                                                                                                           |
| Automatico     | "Buscar actualizaciones automáticamente" / "Check for updates automatically" · "Envía tu versión y un número del 0 al 99 para el despliegue gradual. Nada que te identifique, y sin cuenta." / "Sends your version and a number from 0 to 99 for staged rollout. Nothing that identifies you, and no account needed." |
| Boton          | "Buscar ahora" / "Check now"                                                                                                                                                                                                                                                                                          |
| Al dia         | "Tienes la última versión · comprobado {ago}" / "You're up to date · checked {ago}"                                                                                                                                                                                                                                   |
| Nueva          | "Git Rudder {version} está disponible" / "Git Rudder {version} is available" · "Descargar e instalar" / "Download and install"                                                                                                                                                                                        |
| Descargando    | "Descargando {pct} % · {size}" / "Downloading {pct}% · {size}"                                                                                                                                                                                                                                                        |
| Lista          | "Reinicia Git Rudder para terminar de actualizar." / "Restart Git Rudder to finish updating." · "Reiniciar ahora" / "Restart now" · "Más tarde" / "Later"                                                                                                                                                             |
| Error          | "No pudimos buscar actualizaciones." / "We couldn't check for updates." · "La actualización no superó la verificación de firma y no se instaló." / "The update failed signature verification and wasn't installed."                                                                                                   |
| deb y rpm      | "Los paquetes .deb y .rpm no se actualizan solos." / ".deb and .rpm packages don't update themselves." · "Descargar" / "Download"                                                                                                                                                                                     |
| MSI            | "Esta instalación la gestiona tu equipo de IT y no se actualiza sola." / "Your IT team manages this install and it doesn't update itself." · "La versión más reciente es {version}." / "The latest version is {version}."                                                                                             |
| Version minima | "Actualiza Git Rudder para seguir sincronizando. Todo lo local sigue funcionando." / "Update Git Rudder to keep syncing. Everything local keeps working." · "Actualizar" / "Update"                                                                                                                                   |

- **Variantes.** Claro y oscuro; un estado por formato (DMG, NSIS, AppImage, MSI, deb, rpm).

### 5. D-46 Atajos de teclado y paleta de comandos

- **Proposito.** Los menus muestran atajos que no hacen nada.
- **Pagina "Atajos de teclado".** Buscador, grupos ("General", "Pestañas", "Repositorio", "Grafo",
  "Staging y commit") y filas con la accion y el atajo en chips. "Cambiar" pone la fila en modo grabacion; un
  atajo repetido avisa en linea con "Reemplazar". "Restablecer" por fila y "Restablecer todos".
  Los glifos dependen del sistema (⌘ y ⌥ en macOS, Ctrl y Alt en Windows y Linux; B-43).
- **Paleta.** Capa de 600 px arriba al centro (radio 12, sombra del handoff): campo de busqueda,
  grupos "Recientes", "Comandos", "Repositorios" y "Ramas", filas con el atajo a la derecha, vacio,
  y pie con la ayuda de teclado.
- **Atajos por defecto (propuesta a validar al implementar):**

| Accion                         | Atajo (Mod = ⌘ o Ctrl)           |
| ------------------------------ | -------------------------------- |
| Paleta de comandos             | Mod+Mayus+P                      |
| Ajustes                        | Mod+,                            |
| Abrir repositorio              | Mod+O                            |
| Cerrar pestana                 | Mod+W                            |
| Pestana siguiente y anterior   | Ctrl+Tab · Ctrl+Mayus+Tab        |
| Buscar commits                 | Mod+F                            |
| Fetch                          | Mod+Mayus+R                      |
| Pull · Push                    | Mod+Mayus+↓ · Mod+Mayus+↑        |
| Nueva rama                     | Mod+B                            |
| Preparar o quitar seleccionado | Mod+S                            |
| Preparar todo                  | Mod+Mayus+S                      |
| Commit                         | Mod+Intro (en la caja de commit) |
| Deshacer · Rehacer             | Mod+Z · Mod+Mayus+Z              |

| Uso         | Texto                                                                                                                                                                  |
| ----------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Pagina      | "Atajos de teclado" / "Keyboard shortcuts" · "Buscar atajos" / "Search shortcuts" · "Cambiar" / "Change" · "Restablecer" / "Reset" · "Restablecer todos" / "Reset all" |
| Grabando    | "Pulsa la combinación. Esc para cancelar." / "Press the keys. Esc to cancel."                                                                                          |
| Repetido    | "{shortcut} ya lo usa «{action}»." / "{shortcut} is already used by “{action}”." · "Reemplazar" / "Replace"                                                            |
| Paleta      | "Escribe un comando o busca un repositorio, rama o commit" / "Type a command or search for a repository, branch or commit"                                             |
| Grupos      | "Recientes" / "Recent" · "Comandos" / "Commands" · "Repositorios" / "Repositories" · "Ramas" / "Branches"                                                              |
| Vacio y pie | "Sin resultados" / "No results" · "↑↓ para moverte · ↵ para ejecutar · Esc para cerrar" / "↑↓ to move · ↵ to run · Esc to close"                                       |

- **Variantes.** Claro y oscuro; macOS y Windows para los glifos.

### 6. D-47 Cuenta y equipo

- **Proposito.** Hoy todo asume titular y no hay selector de organizacion.
- **Pagina Cuenta.**
  - Tarjeta de sesion: "Sesión iniciada como {email}", "Cambiar correo o contraseña" (abre la web) y
    "Cerrar sesión…", que abre el dialogo K-09 de #63 (texto y casilla de la decision #57, abajo).
  - "Qué se sincroniza" (D-51).
  - "Sesiones activas": dispositivo, sistema, app o navegador y ultima actividad, con la pastilla
    "Este dispositivo"; **sin ubicacion** (decision #53). "Cerrar" en cada otra sesion y "Cerrar
    todas las demás", las dos con confirmacion que no promete un borrado remoto (X-03).
- **Pagina Equipo.**
  - Selector de organizacion arriba ("Personal", cada organizacion con su rol, "Crear
    organización…").
  - Invitaciones para mi: tarjeta por invitacion con organizacion, rol, quien invita y caducidad, y
    "Aceptar" y "Rechazar". Con el correo sin verificar, la lista sale vacia y una linea lo explica.
  - Contexto personal: tarjeta que explica Git Rudder Pro (prueba de 14 dias sin tarjeta, precio por
    asiento) con "Crear organización". Los importes llegan de la API, no se escriben en la app.
  - Tarjeta de plan, por estado de la suscripcion: prueba (dias que quedan), activa (renovacion,
    asientos, mensual o anual), pago pendiente (`--warn`), cancelada (fecha de fin), inactiva (la
    organizacion queda en solo lectura; es la propuesta de ADR 0026, pendiente de la issue de
    reglas de facturacion, asi que dibujala como estado aparte facil de cambiar). Los botones solo
    los ve la titular y abren el navegador (checkout o portal de Paddle); un miembro ve el estado
    sin botones.
  - Organizacion: nombre, contador separado de miembros, invitaciones y asientos (B-52), "Invitar
    miembro" y un `⋯` con "Renombrar…" (admin), "Salir de la organización…", "Transferir
    titularidad…" y "Eliminar organización…" (titular; abren la web porque piden reautenticacion).
  - Miembros por rol: titular, admin, miembro. Etiqueta "Titular" (B-50).
- **Vistas por rol** (C-08 de #62 para el estilo de solo lectura):
  - Miembro: sin "Invitar", sin `⋯` en las filas de otros y una linea que dice por que.
  - Admin: invita, cambia roles entre admin y miembro, reenvia, revoca y quita; la fila de la titular
    solo tiene "Copiar correo".
  - Titular: todo, mas plan y transferir o eliminar.
- **Invitar sin asientos libres** (W-15): el modal ensena el error en linea; la titular ve "Añadir
  asientos" (abre la web con precio y confirmacion), un admin ve que se lo pida a la titular.
- **Crear organizacion.** Modal con "Nombre" y la linea de la prueba. Errores: correo sin verificar,
  limite de 5 organizaciones.
- **Salir.** Confirmacion de peligro; la titular no puede salir sin transferir antes.

| Uso                                       | Texto                                                                                                                                                                                                                                                                                                                                                                                                                                                          |
| ----------------------------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Sesion                                    | "Sesión iniciada como {email}" / "Signed in as {email}" · "Cambiar correo o contraseña" / "Change email or password" · "Cerrar sesión…" / "Sign out…"                                                                                                                                                                                                                                                                                                          |
| K-09 (claves `k9Body` y `k9Check` de #63) | "Al cerrar sesión se borran de este equipo la llave de tu bóveda y las cuentas y llaves que llegaron de otros equipos. Las que añadiste aquí se quedan para que Git siga funcionando." / "Signing out removes your vault key and the accounts and keys that came from other devices. The ones you added here stay so Git keeps working." · casilla apagada: "Borrar también las añadidas en este equipo" / "Also delete the ones added on this device"         |
| Sesiones                                  | "Sesiones activas" / "Active sessions" · "{os} · {app} · {ago}" / "{os} · {app} · {ago}" · "Este dispositivo" / "This device" · "Cerrar" / "Sign out" · "Cerrar todas las demás" / "Sign out all others"                                                                                                                                                                                                                                                       |
| Cerrar otra                               | "¿Cerrar la sesión de {device}?" / "Sign out {device}?" · "Perderá el acceso a tu cuenta la próxima vez que se conecte. Lo que ya tenía descifrado puede seguir en ese equipo: te recomendaremos rotar la llave de la bóveda." / "It loses access to your account the next time it connects. What it already decrypted may stay on that device: we'll recommend rotating your vault key."                                                                      |
| Selector                                  | "Organización" / "Organization" · "Personal" · "Crear organización…" / "Create organization…"                                                                                                                                                                                                                                                                                                                                                                  |
| Invitacion para mi                        | "Te han invitado a {org} como {role}" / "You're invited to {org} as {role}" · "De {person} · caduca el {date}" / "From {person} · expires on {date}" · "Aceptar" / "Accept" · "Rechazar" / "Decline"                                                                                                                                                                                                                                                           |
| Sin verificar                             | "Verifica tu correo para ver tus invitaciones." / "Verify your email to see your invitations." · "Verificar en la web" / "Verify on the web"                                                                                                                                                                                                                                                                                                                   |
| Personal                                  | "Trabaja en equipo con Git Rudder Pro" / "Work as a team with Git Rudder Pro" · "Invita a tu equipo, comparte workspaces y usa llaves de IA de la organización. Prueba 14 días sin tarjeta." / "Invite your team, share workspaces and use organization AI keys. Try it for 14 days, no card needed." · "Después, {yearly} por asiento al mes en pago anual o {monthly} en mensual." / "Then {yearly} per seat per month billed yearly, or {monthly} monthly." |
| Plan                                      | "Prueba de Git Rudder Pro · quedan {days} días" / "Git Rudder Pro trial · {days} days left" · "Git Rudder Pro · se renueva el {date} · {seats} asientos" / "Git Rudder Pro · renews on {date} · {seats} seats" · "Anual" / "Yearly" · "Mensual" / "Monthly"                                                                                                                                                                                                    |
| Pago pendiente                            | "No pudimos cobrar la suscripción. Actualiza el método de pago antes del {date}." / "We couldn't charge the subscription. Update the payment method before {date}." · "Actualizar método de pago" / "Update payment method"                                                                                                                                                                                                                                    |
| Cancelada                                 | "Se cancela el {date}. Después, {org} queda en solo lectura." / "Ends on {date}. After that, {org} becomes read-only." · "Reactivar" / "Reactivate"                                                                                                                                                                                                                                                                                                            |
| Inactiva                                  | "La suscripción no está activa: {org} está en solo lectura." / "The subscription isn't active: {org} is read-only." · "Elegir plan" / "Choose plan"                                                                                                                                                                                                                                                                                                            |
| Botones de plan                           | "Gestionar suscripción" / "Manage subscription" · "Se abre en el navegador." / "Opens in your browser." · "Solo la titular gestiona la suscripción." / "Only the owner manages the subscription."                                                                                                                                                                                                                                                              |
| Contador                                  | "{members} miembros y {invitations} invitaciones pendientes · {used} de {seats} asientos" / "{members} members and {invitations} pending invitations · {used} of {seats} seats"                                                                                                                                                                                                                                                                                |
| Roles                                     | "Titular" / "Owner" · "Admin" · "Miembro" / "Member"                                                                                                                                                                                                                                                                                                                                                                                                           |
| Miembro                                   | "Solo los admins pueden invitar y cambiar roles." / "Only admins can invite and change roles."                                                                                                                                                                                                                                                                                                                                                                 |
| Sin asientos                              | "No quedan asientos libres ({used} de {seats}). Las invitaciones pendientes ocupan asiento." / "No free seats ({used} of {seats}). Pending invitations take a seat." · "Añadir asientos" / "Add seats" · "Pide a la titular que añada asientos." / "Ask the owner to add seats."                                                                                                                                                                               |
| Crear                                     | "Crear organización" / "Create organization" · "Crear y empezar la prueba" / "Create and start trial" · "Verifica tu correo antes de crear una organización." / "Verify your email before creating an organization." · "Ya creaste 5 organizaciones." / "You've already created 5 organizations."                                                                                                                                                              |
| Salir                                     | "¿Salir de {org}?" / "Leave {org}?" · "Dejas de ver sus workspaces compartidos y de usar sus llaves de IA en todos tus equipos." / "You'll stop seeing its shared workspaces and using its AI keys on all your devices." · "Transfiere la titularidad antes de salir." / "Transfer ownership before leaving."                                                                                                                                                  |
| En la web                                 | "Transferir titularidad…" / "Transfer ownership…" · "Eliminar organización…" / "Delete organization…" · "Se abre en la web porque pide confirmar tu identidad." / "Opens on the web because it asks you to confirm your identity."                                                                                                                                                                                                                             |

- **Variantes.** Claro y oscuro; contexto personal, miembro, admin y titular; cada estado del plan;
  sin sesion (tarjeta de invitacion de D-04).

### 7. D-48 Workspace: color, orden, compartido y fetch de todos

- **Proposito.** La web permite compartir y el escritorio no lo muestra.
- **Muestra.**
  - "Editar workspace" sustituye a "Renombrar": "Nombre", "Color" (seis muestras de la paleta de
    carriles) e interruptor "Compartir con {org}" con su descripcion. Sin organizacion, la linea
    "Crea una organización para compartir workspaces."; sin sesion, la de D-04.
  - Barra lateral: WORKSPACES se parte en "Personales" y "{org}" si hay organizacion. Al pasar, un
    asa para arrastrar y reordenar con linea de destino en `--accent`.
  - Cabecera del workspace compartido: pastilla "Compartido con {org}" y quien lo creo. Un miembro
    no edita ni borra los compartidos de otros: opciones desactivadas con motivo.
  - "Fetch de todos": cada tarjeta ensena su estado (spinner, "Al día", "↓{count} nuevos", error con
    motivo, "No clonado" para los de D-06 de #64) y al final un toast con resumen y "Ver errores".
  - Borrar un compartido: confirmacion que dice que desaparece para todo el equipo.

| Uso           | Texto                                                                                                                                                                                                                                                                         |
| ------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Modal         | "Editar workspace" / "Edit workspace" · "Color" · "Compartir con {org}" / "Share with {org}"                                                                                                                                                                                  |
| Compartir     | "Tu equipo verá este workspace y sus repositorios. Ni las cuentas ni las rutas se comparten." / "Your team will see this workspace and its repositories. Accounts and paths aren't shared."                                                                                   |
| Sin org       | "Crea una organización para compartir workspaces." / "Create an organization to share workspaces."                                                                                                                                                                            |
| Barra lateral | "Personales" / "Personal"                                                                                                                                                                                                                                                     |
| Cabecera      | "Compartido con {org} · creado por {person}" / "Shared with {org} · created by {person}"                                                                                                                                                                                      |
| Solo lectura  | "Solo quien lo creó o un admin puede editarlo." / "Only its creator or an admin can edit it."                                                                                                                                                                                 |
| Fetch         | "Al día" / "Up to date" · "↓{count} nuevos" / "↓{count} new" · "No clonado" / "Not cloned" · "Fetch de {count} repositorios: {okCount} correctos, {errorCount} con error" / "Fetched {count} repositories: {okCount} OK, {errorCount} failed" · "Ver errores" / "View errors" |
| Borrar        | "¿Eliminar {workspace}? Desaparece para todo el equipo. Los repositorios no se tocan." / "Delete {workspace}? It disappears for the whole team. Repositories aren't touched."                                                                                                 |

- **Variantes.** Claro y oscuro; personal y de organizacion; vista de miembro.

### 8. D-49 Enlaces profundos

- **Proposito.** "Abrir en Git Rudder Desktop" (web W-23 y VS Code V-18) y los enlaces de clonar no llevan
  a ninguna parte.
- **Formatos (propuesta para ingenieria):** `git-rudder://open?repo={remote}&ref={ref}`,
  `git-rudder://clone?url={url}` y `git-rudder://settings/accounts` (la llamada de la web para anadir cuentas,
  X-02). Nunca una ruta local.
- **Lo que hace Desktop:**
  1. Repo clonado aqui: abre su pestana y selecciona la ref; si la ref no esta, toast con "Fetch".
  2. Varias copias: modal "¿Qué copia abro?" con las rutas.
  3. No clonado aqui: modal con la identidad remota y "Clonar" (D-08 relleno).
  4. Clonar: siempre confirma, con la URL completa (punycode si el host no es ASCII), la cuenta que
     encaja por host exacto o el aviso de que no hay ninguna.
  5. Ajustes: navega a Plataformas Git.
  6. Enlace no valido: modal de error con el enlace recortado.
  7. Si llega durante D-01 o D-02 (#64), espera a que terminen.
- **Copiar enlace.** El menu de la tarjeta de repo gana "Copiar enlace de Git Rudder" (solo con remoto).

| Uso         | Texto                                                                                                                                                   |
| ----------- | ------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Boton comun | "Abrir en Git Rudder Desktop" / "Open in Git Rudder Desktop"                                                                                            |
| Varias      | "¿Qué copia abro?" / "Which copy should I open?"                                                                                                        |
| No clonado  | "{repo} no está en este equipo." / "{repo} isn't on this device." · "Clonar" / "Clone"                                                                  |
| Clonar      | "¿Clonar desde este enlace?" / "Clone from this link?" · "Comprueba el servidor antes de seguir: {host}" / "Check the server before continuing: {host}" |
| Ref         | "No encontramos {ref} en tu copia." / "We can't find {ref} in your copy." · "Fetch"                                                                     |
| No valido   | "Este enlace de Git Rudder no es válido." / "This Git Rudder link isn't valid."                                                                         |
| Menu        | "Copiar enlace de Git Rudder" / "Copy Git Rudder link"                                                                                                  |

- **Variantes.** Claro y oscuro.

### 9. D-51 "Que se sincroniza" en Ajustes > Cuenta

- **Proposito.** Los interruptores solo estan en la web, que no puede crear la boveda.
- **Muestra.** Tarjeta con los seis interruptores de la web y sus mismos textos: "Workspaces y
  repositorios", "Cuentas vinculadas", "Preferencias de IA", "Llaves de IA", "Preferencias de Git" y
  "Apariencia". Bajo cada fila, el estado de sincronizacion de C-06 (#62). Bajo las dos de secretos,
  el estado de la boveda. Cada interruptor guarda al cambiar (B-57).
- **Comportamiento.**
  - Encender un interruptor de secretos sin boveda abre K-01 (#63); con la boveda bloqueada en este
    equipo, ofrece desbloquear (K-04).
  - Apagar uno de secretos: confirmacion de peligro (borra de la nube metadatos y copias cifradas;
    cada equipo conserva las suyas).
  - Apagar uno de los otros cuatro: toast `info` de pausa.
  - Sin sesion: tarjeta de invitacion de D-04.

| Uso             | Texto                                                                                                                                                                                                                                                                                                                                                                                                                                                                          |
| --------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| Titulo          | "Qué se sincroniza" / "What syncs" · "Se guarda en tu cuenta y se aplica en cada equipo con sesión iniciada." / "Stored in your account and applied on every signed-in device."                                                                                                                                                                                                                                                                                                |
| Filas           | "Workspaces y repositorios" / "Workspaces and repositories" · "Cuentas vinculadas" / "Linked accounts" · "Preferencias de IA" / "AI preferences" · "Llaves de IA" / "AI keys" · "Preferencias de Git" / "Git preferences" · "Apariencia" / "Appearance"                                                                                                                                                                                                                        |
| Descripciones   | "Tokens cifrados de extremo a extremo con tu frase de sincronización" / "Tokens end-to-end encrypted with your sync passphrase" · llaves apagado: "Desactivado: cada equipo guarda sus propias llaves" / "Off: each device keeps its own keys" · llaves encendido: "Cifradas de extremo a extremo con tu frase de sincronización" / "End-to-end encrypted with your sync passphrase" · "En VS Code se usa siempre el tema del editor" / "VS Code always uses the editor theme" |
| Boveda          | "Bóveda configurada" / "Vault set up" · "Sin bóveda: te pediremos una frase al activarlo" / "No vault: we'll ask for a passphrase when you turn it on" · "Bóveda bloqueada en este equipo" / "Vault locked on this device" · "Desbloquear" / "Unlock"                                                                                                                                                                                                                          |
| Apagar secretos | "¿Dejar de sincronizar {category}?" / "Stop syncing {category}?" · "Borramos de la nube sus datos y sus copias cifradas. Cada equipo conserva las suyas." / "We delete its data and encrypted copies from the cloud. Each device keeps its own." · "Dejar de sincronizar" / "Stop syncing"                                                                                                                                                                                     |
| Pausa           | "En pausa: los datos se quedan en la nube y este equipo deja de aplicarlos." / "Paused: the data stays in the cloud and this device stops applying it."                                                                                                                                                                                                                                                                                                                        |

- **Variantes.** Claro y oscuro; con y sin boveda; boveda bloqueada.

### 10. X-05 y X-10 Ajustes > IA: llaves de la organizacion y modelo de la lista

- **Proposito.** Las llaves de organizacion viven en el servidor (decision 3) y los modelos cambian
  cada pocos meses (decision #54).
- **Lista de llaves en dos secciones.**
  - "Tus llaves": como hoy, con la pastilla "Solo en este equipo" o "Sincronizada" en vez de
    "Pertenece a".
  - "De la organización · {org}": de solo lectura, con la pastilla "De la organización", mascara
    calculada en el servidor y la nota de X-03. Los admins ven "Probar conexión" y "Gestionar en la
    web" (W-16); los miembros no tienen menu. Linea con la cuota del miembro. Sin organizacion, una
    linea que invita a crearla; sin sesion, la tarjeta de D-04.
- **Formulario de llave personal.** Sin el selector de alcance. Campos: "Proveedor", "Etiqueta",
  "API key" (oculto; al editar ensena la mascara como placeholder, nunca el valor), "URL base"
  (OpenRouter, Ollama y compatibles con OpenAI; se guarda de verdad) y "Modelo predeterminado",
  que es un selector que se llena tras "Validar".
- **Selector de modelo** (en el formulario y en cada funcion): opciones con el id en mono y el nombre
  del proveedor; la recomendada para esa funcion lleva la insignia "Recomendado · Rápido" (mensajes
  de commit) o "Recomendado · Equilibrado" (PRs y resumenes). La recomendacion solo preselecciona al
  crear y nunca cambia lo ya elegido. Estados: antes de validar (desactivado con motivo), cargando,
  error con "Reintentar", endpoint compatible con OpenAI sin lista (texto libre), modelo guardado que
  ya no esta, Ollama sin modelos, Ollama inalcanzable, sin recomendaciones (sin red ni copia en
  cache: la lista del proveedor sin insignias; en OpenRouter, ordenada por precio). Ollama y los
  endpoints personalizados nunca llevan insignia. Sin coste (ADR 0025).
- **"Dónde usar la IA".** Cada funcion: interruptor, "Llave" (grupos "Tus llaves" y "De la
  organización") y "Modelo" ("El de la llave ({model})" por defecto). Con una llave de la
  organizacion, el modelo es texto fijo: lo fija el admin por funcion.
- **Primera generacion con llave de la organizacion:** dialogo que avisa de que el diff pasa por los
  servidores de Git Rudder (X-03), con "Entendido, generar".
- **Excluir archivos sensibles:** el interruptor se sustituye por una linea fija (la lista sensible
  se aplica siempre) y un campo "Patrones adicionales".
- **Errores de generacion con llave de la organizacion:** cuota agotada, llave que no funciona
  (avisa a un admin) y proveedor que no responde.

| Uso                | Texto                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                   |
| ------------------ | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Secciones          | "Tus llaves" / "Your keys" · "De la organización · {org}" / "From the organization · {org}"                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                             |
| Pastillas          | "Solo en este equipo" / "Only on this device" · "Sincronizada" / "Synced" · "De la organización" / "From the organization"                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                              |
| Nota X-03          | "Cifradas en los servidores de Git Rudder; Git Rudder las usa para generar en nombre de tu organización." / "Encrypted on Git Rudder's servers; Git Rudder uses them to generate on behalf of your organization."                                                                                                                                                                                                                                                                                                                                                                                                                       |
| Cuota              | "Te quedan {remaining} de {limit} generaciones hoy." / "You have {remaining} of {limit} generations left today."                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                        |
| Admin              | "Probar conexión" / "Test connection" · "Gestionar en la web" / "Manage on the web"                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                     |
| Sin organizacion   | "Con Git Rudder Pro, tu organización puede compartir llaves de IA." / "With Git Rudder Pro, your organization can share AI keys."                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                       |
| Formulario         | "Validar" / "Validate" · "Modelo predeterminado" / "Default model" · "URL base" / "Base URL"                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                            |
| Guia               | "Para los mensajes de commit te recomendamos un modelo rápido y barato. Para PRs y resúmenes, uno equilibrado. Cada generación la paga tu llave." / "For commit messages we recommend a fast, cheap model. For PRs and summaries, a balanced one. Your key pays for every generation."                                                                                                                                                                                                                                                                                                                                                  |
| Insignias          | "Recomendado · Rápido" / "Recommended · Fast" · "Recomendado · Equilibrado" / "Recommended · Balanced"                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                  |
| Estados del modelo | "Valida la llave para ver sus modelos." / "Validate the key to see its models." · "Cargando modelos de {provider}…" / "Loading {provider} models…" · "No pudimos cargar los modelos de {provider}." / "We couldn't load {provider} models." · "Escribe el id del modelo" / "Type the model id" · "{model} ya no está en la lista de {provider}. Elige otro." / "{model} is no longer in {provider}'s list. Pick another." · "No hay modelos instalados en Ollama. Instala uno con ollama pull." / "No models installed in Ollama. Install one with ollama pull." · "No encontramos Ollama en {url}." / "We can't find Ollama at {url}." |
| Funciones          | "Llave" / "Key" · "Modelo" / "Model" · "El de la llave ({model})" / "The key's model ({model})" · "Fijado por tu organización: {model}" / "Set by your organization: {model}"                                                                                                                                                                                                                                                                                                                                                                                                                                                           |
| Primera vez        | "Esta generación pasa por Git Rudder" / "This generation goes through Git Rudder" · "Con una llave de la organización, los cambios preparados viajan a los servidores de Git Rudder para generar el texto con {provider}. Git Rudder no guarda ni registra su contenido." / "With an organization key, your staged changes travel to Git Rudder's servers to generate the text with {provider}. Git Rudder doesn't store or log their content." · "Entendido, generar" / "Got it, generate"                                                                                                                                             |
| Exclusion          | "Git Rudder nunca envía .env, *.pem, *.key ni otros archivos sensibles." / "Git Rudder never sends .env, *.pem, *.key or other sensitive files." · "Patrones adicionales" / "Additional patterns"                                                                                                                                                                                                                                                                                                                                                                                                                                       |
| Errores            | "Has llegado al límite diario de tu organización." / "You've reached your organization's daily limit." · "La llave de la organización no funciona. Avisa a un admin." / "The organization key isn't working. Tell an admin." · "{provider} no responde." / "{provider} isn't responding."                                                                                                                                                                                                                                                                                                                                               |

- **Variantes.** Claro y oscuro; miembro y admin; cada estado del selector.

## Reglas

- **Llaves de organizacion de solo lectura y sin selector de alcance** en Desktop (X-05): lo que se
  anade aqui es siempre personal. El alta, la rotacion y la baja estan en la web (W-16, #70).
- **El modelo es un selector que se llena con la lista del proveedor** (X-10, decision #54), con el
  nivel recomendado marcado, sin coste, y sin ningun modelo fijo en la app: quita los modelos por
  defecto de `git-rudder-data.js`. Con llave de la organizacion, el admin fija el modelo por funcion.
- **Sin ubicacion en las sesiones** (decision #53) y **sin prometer borrado remoto** (X-03).
- **La facturacion ocurre en el navegador**: Desktop nunca ensena campos de pago; los botones abren
  el checkout o el portal de Paddle en la web. Los precios llegan de la API. Transferir y eliminar la
  organizacion, cambiar correo y contrasena tambien abren la web, porque piden reautenticacion.
- **Sin cuentas de plataforma compartidas con el equipo** (X-01): compartir un workspace comparte
  metadatos, nunca cuentas ni rutas.
- **Los interruptores de secretos dicen el estado de la boveda** y apagarlos borra de la nube con
  confirmacion de peligro (X-04).
- **Los enlaces `git-rudder://` son entrada no fiable**: nunca llevan rutas, nunca ejecutan nada que
  escriba o clone sin confirmar, y el host se ensena en punycode si no es ASCII.
- **Los comandos de herramientas externas se lanzan sin shell**, con la lista de argumentos.
- **Actualizaciones** (ADR 0016, decision #55): la busqueda se puede apagar y no pide cuenta; el MSI
  x64 dice "sin actualizaciones automáticas"; `.deb` y `.rpm` avisan con enlace; arm64 de Windows y
  Linux lleva la pastilla beta.
- Bugs que no se copian: B-29 (destructivas con el modal de peligro de #71, foco inicial en "Cancelar"), B-34 (mascara del motor o
  la API), B-37, B-41 (claves propias como `copyName` o `copyEmail`), B-43 (atajos por sistema), B-46,
  B-50 ("Titular"), B-51 (nada de comprar asientos sin precio ni confirmacion), B-52 (contadores
  separados), B-57 (guardado inmediato por control), B-61 (toast de informacion en `--info`).
- Textos al diccionario `D`, colores de los tokens. Dominio `git-rudder.com`; la API y el
  manifiesto del actualizador en `api.git-rudder.com`. Los correos de ejemplo, con `example.com`.

## Entregable

- Zip `git-rudder-handoff-67-ajustes.zip` con la estructura del original: `README.md` (pantallas nuevas y
  la navegacion nueva de Ajustes), `Git Rudder Ajustes.dc.html`, `git-rudder-data.js` (original mas claves y
  datos: organizaciones con los tres roles, invitaciones para mi, estados de suscripcion, llaves de
  organizacion, lista de modelos con recomendaciones) y `support.js`.
- La barra de titulo de D-42 es la que usaran los archivos de #64, #65 y #66: dejala como bloque
  facil de copiar.
- Props: `theme`, `lang`, `os` (`macos`, `windows`, `linux`), `session` (`signedIn`, `signedOut`),
  `role` (`personal`, `member`, `admin`, `owner`), `install` (`dmg`, `nsis`, `msi`, `appimage`,
  `deb`, `rpm`) y `screen`: `d42-barra`, `d42-minimo`, `d43-git`, `d44-herramientas`,
  `d45-al-dia`, `d45-nueva`, `d45-lista`, `d45-paquete`, `d45-msi`, `d45-minima`, `d46-atajos`,
  `d46-paleta`, `d47-cuenta`, `d47-equipo`, `d47-sin-asientos`, `d47-crear-org`, `d48-editar`,
  `d48-fetch`, `d49-clonar`, `d49-varias`, `d51-sync`, `x05-ia`, `x10-formulario`, `x10-selector`,
  `x05-primera-vez`.
- Juntos en el lienzo: ventana (D-42); Ajustes de aplicacion (D-43 a D-46); cuenta, equipo y
  sincronizacion (D-47 y D-51); workspaces y enlaces (D-48 y D-49); IA (X-05 y X-10).

## Hecho cuando

- [ ] D-42 a D-49 y D-51 diseñados
- [ ] Las llaves de organización aparecen de solo lectura y sin selector de alcance
- [ ] El campo modelo es un selector que se rellena con la lista del proveedor
