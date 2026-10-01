# Brief #64: entrada y ciclo de vida del repositorio (Ramal Desktop)

## Contexto

Solo Ramal Desktop: primer arranque, Git ausente, iniciar sesion, app sin sesion, lista vacia, repos
que faltan en este equipo y los flujos de abrir, iniciar, clonar y publicar. Cuelga de
`Ramal.dc.html`: barra de titulo de 46 px, Inicio > Repositorios (barra lateral de 232 px, cabecera
con Abrir, Iniciar y **Clonar**, barra de filtros, tarjeta de repo de cinco columnas), vista de
workspace, vista de repo (barra de 62 px, grafo de filas de 40 px, panel de staging) y modal generico
de 460 o 580 px. Carga, error, validacion y sin conexion son C-01, C-02, C-03 y C-05 de #62.

## Pantallas y estados

Formato de los textos: `"castellano" / "english"`. Los parametros tienen nombre en ingles y son
iguales en los dos idiomas (`{branch}`, `{host}`); los contadores usan `{count}` con claves `_one` y
`_other`.

### 1. D-01 Primer arranque

- **Proposito.** Llevar a alguien de cero a su primer repositorio sin pedirle cuenta.
- **Muestra.** Ventana completa con la barra de titulo (solo logo y controles de ventana, sin
  pestanas) y una tarjeta centrada de 560 px sobre `--bg` (`--surface`, radio 16, sombra del
  handoff). Arriba, un indicador de pasos: Cuenta, Identidad, Repositorios, Plataformas, IA. Al pie
  de la tarjeta, en mono y `--faint`, el git detectado: `✓ Git 2.56.0 · /usr/bin/git` (X-13).
- **Paso 1, Cuenta.** Logo de 48 px, titulo y dos botones del mismo ancho: "Iniciar sesión"
  (primario) y "Continuar sin cuenta" (secundario). Debajo, tres lineas con lo que anade la cuenta.
  "Iniciar sesion" abre D-03 dentro de la misma tarjeta; al volver, si la cuenta ya tiene
  configuracion en la nube, una linea informa de que se aplica (gana la nube, ADR 0014) y los pasos
  siguientes llegan rellenos.
- **Paso 2, Identidad.** Campos `user.name` y `user.email` (mono en la etiqueta, como en Ajustes >
  Git). Prellenados desde `git config --global` o desde la cuenta, con una pista de origen. Si estan
  vacios, "Continuar" sigue activo pero aparece el aviso de que sin identidad no hay commits.
  Validacion en linea del correo (C-03).
- **Paso 3, Repositorios.** "Elegir carpeta…" abre el selector nativo (se pueden anadir varias,
  como chips con ×). "Buscar" recorre las carpetas: estado en curso con contador en vivo, resultado
  como lista con casilla, insignia de plataforma (o ⌂), nombre y ruta en mono, y "Seleccionar
  todos". Sin resultados: texto propio. Carpeta sin permiso de lectura: error en linea con la ruta.
- **Paso 4, Plataformas.** Cinco tiles con las insignias del handoff (GH, GL, BB, GT, AZ). Cada tile
  abre el alta de cuenta de #65 (D-12) encima de la tarjeta. Las cuentas conectadas aparecen como
  filas con ✓, `@usuario` y host.
- **Paso 5, IA.** Explicacion corta y "Añadir llave", que abre el formulario de llave de #67 (X-10).
- **Acciones comunes.** "Atrás", "Omitir" en los pasos 2 a 5 y "Continuar". El ultimo paso termina
  en "Empezar", que lleva a Inicio (D-05 si no se anadio nada).
- **Estados.** Comprobando git al abrir (si falla, D-02 sustituye todo). Buscando repos. Sin
  resultados. Error de permisos. Sesion iniciada (paso 1 con ✓ y correo). Sin red: el paso 1 sigue
  ofreciendo las dos opciones y D-03 ensena su error; nada espera a la red.

| Uso                    | Texto                                                                                                                                                                                                 |
| ---------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Titulo paso 1          | "Te damos la bienvenida a Ramal" / "Welcome to Ramal"                                                                                                                                                 |
| Subtitulo              | "Un cliente Git para tu escritorio. Todo lo local funciona sin cuenta." / "A Git client for your desktop. Everything local works without an account."                                                 |
| Lo que anade la cuenta | "Sincroniza ajustes, workspaces y cuentas entre tus equipos." / "Sync settings, workspaces and accounts across your devices."                                                                         |
|                        | "Comparte workspaces y llaves de IA con tu equipo." / "Share workspaces and AI keys with your team."                                                                                                  |
|                        | "Puedes iniciar sesión cuando quieras desde la barra de título." / "You can sign in any time from the title bar."                                                                                     |
| Botones paso 1         | "Iniciar sesión" / "Sign in" · "Continuar sin cuenta" / "Continue without an account"                                                                                                                 |
| Nube aplicada          | "Aplicamos la configuración de tu cuenta en este equipo." / "We applied your account settings on this device."                                                                                        |
| Titulo paso 2          | "¿Quién firma tus commits?" / "Who authors your commits?"                                                                                                                                             |
| Pista de origen        | "Detectado en tu configuración global de Git" / "Found in your global Git config" · "De tu cuenta de Ramal" / "From your Ramal account"                                                               |
| Nota paso 2            | "Se guarda en git config --global y aparece como autor de cada commit." / "Saved in git config --global and shown as the author of every commit."                                                     |
| Aviso sin identidad    | "Sin nombre y correo no podrás hacer commits." / "You can't commit without a name and email."                                                                                                         |
| Titulo paso 3          | "Encuentra tus repositorios" / "Find your repositories"                                                                                                                                               |
| Botones paso 3         | "Elegir carpeta…" / "Choose folder…" · "Buscar" / "Search" · "Añadir {count} repositorios" / "Add {count} repositories"                                                                               |
| Buscando               | "Buscando… {count} repositorios encontrados" / "Searching… {count} repositories found"                                                                                                                |
| Sin resultados         | "No hay repositorios en estas carpetas." / "No repositories in these folders."                                                                                                                        |
| Sin permiso            | "Ramal no tiene permiso para leer {path}." / "Ramal can't read {path}."                                                                                                                               |
| Titulo paso 4          | "Conecta tus plataformas" / "Connect your platforms"                                                                                                                                                  |
| Nota paso 4            | "Para clonar, hacer push y ver pull requests. Puedes conectar varias cuentas por plataforma." / "To clone, push and see pull requests. You can connect several accounts per platform."                |
| Titulo paso 5          | "IA, si quieres" / "AI, if you want"                                                                                                                                                                  |
| Nota paso 5            | "Genera mensajes de commit y descripciones de PR con tu propia llave. Ollama funciona en tu equipo." / "Generate commit messages and PR descriptions with your own key. Ollama runs on your machine." |
| Botones                | "Atrás" / "Back" · "Omitir" / "Skip" · "Continuar" / "Continue" · "Añadir llave" / "Add key" · "Empezar" / "Get started"                                                                              |

- **Variantes.** Claro y oscuro. Solo densidad de escritorio.

### 2. D-02 Git ausente o anterior a 2.34

- **Proposito.** Bloquear con instrucciones utiles: sin git no hay producto (decision 1, ADR 0009).
- **Muestra.** Ventana completa con la barra de titulo y una tarjeta centrada de 560 px. Icono ⚠ en
  `--warn`, titulo, texto, un bloque de comando en mono con "Copiar" (estilo del bloque de la landing
  `code --install-extension`) y enlace a la descarga oficial. Variante por sistema operativo.
- **Acciones.** "Volver a comprobar" (primario), "Elegir ejecutable de Git…" (selector nativo; mismo
  validador que D-43 de #67), enlace "Por qué Ramal usa el Git de tu sistema".
- **Estados.**
  - Ausente en macOS: comando `xcode-select --install`.
  - Ausente en Linux: `sudo apt install git` (Debian y Ubuntu) y `sudo dnf install git` (Fedora), en
    dos lineas.
  - Ausente en Windows: solo aparece si falta o esta roto el MinGit que trae el instalador (decision
    #59). Comando `winget install Git.Git` y "Reinstalar Ramal".
  - Version antigua: ensena version y ruta detectadas y el minimo.
  - Comprobando (spinner en el boton). Sigue sin encontrarse tras comprobar (error en linea).
  - Encontrado: la tarjeta pasa a ✓ un momento y continua (a D-01 o a Inicio).

| Uso                | Texto                                                                                                                                                                 |
| ------------------ | --------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Titulo ausente     | "Ramal necesita Git" / "Ramal needs Git"                                                                                                                              |
| Texto ausente      | "Ramal usa el Git de tu sistema para todo. Instálalo y vuelve a comprobar." / "Ramal uses your system's Git for everything. Install it and check again."              |
| Texto Windows      | "Falta el Git que viene con Ramal. Reinstala Ramal o instala Git for Windows." / "The Git bundled with Ramal is missing. Reinstall Ramal or install Git for Windows." |
| Titulo antiguo     | "Tu Git es demasiado antiguo" / "Your Git is too old"                                                                                                                 |
| Texto antiguo      | "Ramal necesita Git 2.34 o posterior. Tienes {version} en {path}." / "Ramal needs Git 2.34 or later. You have {version} at {path}."                                   |
| Botones            | "Volver a comprobar" / "Check again" · "Elegir ejecutable de Git…" / "Choose Git executable…" · "Copiar" / "Copy"                                                     |
| Sigue sin aparecer | "Seguimos sin encontrar Git." / "Still can't find Git."                                                                                                               |
| Enlace             | "Por qué Ramal usa el Git de tu sistema" / "Why Ramal uses your system's Git"                                                                                         |

- **Variantes.** macOS, Windows y Linux; claro y oscuro.

### 3. D-03 Iniciar sesion desde el escritorio

- **Proposito.** La unica forma de entrar desde Desktop: navegador con PKCE y redireccion local, y
  codigo de dispositivo como alternativa (ADR 0008). Desktop nunca muestra un campo de contrasena.
- **Muestra.** Modal de 460 px (o el cuerpo de la tarjeta de D-01). Se abre desde el boton de la
  barra de titulo sin sesion (D-04), desde las tarjetas de invitacion y desde D-01.
- **Estados.**
  1. Esperando al navegador: spinner, texto, "Abrir de nuevo", "Copiar enlace" y "Cancelar". Enlace
     inferior "Usar un código".
  2. Codigo de dispositivo: codigo de 8 caracteres en dos grupos de 4 (mono, 28 px,
     `letter-spacing:.08em`) con "Copiar", la direccion `ramal.bahoque.com/activate`, cuenta atras de
     caducidad y spinner de espera. Si el receptor local no se pudo abrir, se llega aqui solo, con una
     linea que lo explica.
  3. Exito: ✓, correo de la cuenta y, la primera vez, "Aplicando la configuración de tu cuenta…".
     Se cierra solo y lanza un toast `ok`.
  4. Errores: cancelado en el navegador, codigo caducado, sin conexion o API caida (texto que deja
     claro que Git sigue funcionando), y error generico con "Copiar detalle" (C-02).

| Uso          | Texto                                                                                                                                                                                                                                                  |
| ------------ | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| Titulo       | "Iniciar sesión en Ramal" / "Sign in to Ramal"                                                                                                                                                                                                         |
| Esperando    | "Continúa en tu navegador. Hemos abierto ramal.bahoque.com para que inicies sesión y autorices Ramal Desktop en este equipo." / "Continue in your browser. We opened ramal.bahoque.com so you can sign in and authorize Ramal Desktop on this device." |
| Botones      | "Abrir de nuevo" / "Open again" · "Copiar enlace" / "Copy link" · "Cancelar" / "Cancel" · "Usar un código" / "Use a code"                                                                                                                              |
| Codigo       | "Abre ramal.bahoque.com/activate en cualquier dispositivo y escribe este código." / "Open ramal.bahoque.com/activate on any device and enter this code."                                                                                               |
| Caducidad    | "Caduca en {remaining}" / "Expires in {remaining}"                                                                                                                                                                                                     |
| Espera       | "Esperando la aprobación…" / "Waiting for approval…"                                                                                                                                                                                                   |
| Sin receptor | "Este equipo no permite la redirección local. Usa el código." / "This device blocks the local redirect. Use the code instead."                                                                                                                         |
| Exito        | "Sesión iniciada como {email}" / "Signed in as {email}"                                                                                                                                                                                                |
| Cancelado    | "Cancelaste la autorización en el navegador." / "You cancelled the authorization in the browser." · "Intentar de nuevo" / "Try again"                                                                                                                  |
| Caducado     | "El código ha caducado." / "The code has expired." · "Pedir otro código" / "Get a new code"                                                                                                                                                            |
| Sin conexion | "No podemos conectar con Ramal ahora. Git sigue funcionando sin cuenta." / "We can't reach Ramal right now. Git keeps working without an account." · "Reintentar" / "Retry"                                                                            |

- **Variantes.** Claro y oscuro. Modal y dentro de la tarjeta de D-01.

### 4. D-04 Estado sin sesion en toda la app (X-06)

- **Proposito.** Que las zonas de nube inviten a entrar en vez de desaparecer o romperse.
- **Muestra.**
  - Barra de titulo: el avatar se sustituye por un boton "Iniciar sesión" (alto 30, borde
    `--border`, fondo `--bg`), que abre D-03.
  - Un componente nuevo, la **tarjeta de invitacion**: tarjeta de ajustes (radio 14) con titulo,
    dos o tres lineas de valor, "Iniciar sesión" primario y la nota "No hace falta para usar Git.".
    Se usa en Ajustes > Perfil, Cuenta, Equipo y en la seccion de llaves de organizacion de IA.
  - Plataformas Git: cada cuenta lleva la pastilla "Solo en este equipo" (K-05 de #63).
  - Workspaces: funcionan como locales. En el modal de editar workspace, el interruptor de compartir
    se sustituye por la linea de invitacion.
- **Estados.** Sin sesion (todas las zonas anteriores). La sesion caducada es C-07 de #62.

| Uso                   | Texto                                                                                                                                                   |
| --------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Boton barra de titulo | "Iniciar sesión" / "Sign in"                                                                                                                            |
| Nota comun            | "No hace falta para usar Git." / "Not needed to use Git."                                                                                               |
| Perfil                | "Tu perfil de Ramal es lo que ve tu equipo. Inicia sesión para tenerlo." / "Your Ramal profile is what your team sees. Sign in to get one."             |
| Cuenta                | "Inicia sesión para sincronizar ajustes, workspaces y cuentas entre tus equipos." / "Sign in to sync settings, workspaces and accounts across devices." |
| Equipo                | "Inicia sesión para crear una organización o aceptar una invitación." / "Sign in to create an organization or accept an invitation."                    |
| IA de organizacion    | "Inicia sesión para usar las llaves de IA de tu organización." / "Sign in to use your organization's AI keys."                                          |
| Workspace compartido  | "Inicia sesión para compartir este workspace con tu equipo." / "Sign in to share this workspace with your team."                                        |

- **Variantes.** Claro y oscuro. Barra de titulo con y sin sesion.

### 5. D-05 Sin repositorios y sin repositorio abierto

- **Proposito.** La primera pantalla de cualquier usuario nuevo.
- **Muestra.** Inicio > Repositorios con la cabecera normal (Abrir, Iniciar, Clonar). Sin repos, la
  barra de filtros y el contador se esconden y en su lugar va un estado vacio: titulo, texto y tres
  tiles de un tercio cada uno (glifo, titulo, una linea), mas el enlace "Buscar repositorios en una
  carpeta" que abre el paso 3 de D-01 como modal. La seccion WORKSPACES vacia ensena una linea y
  "+ Nuevo workspace". Con sesion y repos en la nube sin clonar, en vez del vacio va la lista de D-06
  con un aviso arriba.
- **Estados.** Vacio total. Vacio con filtros (se mantiene "Ningún repositorio coincide con los
  filtros."). Cargando la lista (C-01). Con repos solo en la nube.

| Uso                 | Texto                                                                                                                                                                                                             |
| ------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Titulo              | "Aún no hay repositorios" / "No repositories yet"                                                                                                                                                                 |
| Texto               | "Abre una carpeta con un repositorio, clona uno o empieza uno nuevo." / "Open a folder with a repository, clone one or start a new one."                                                                          |
| Tiles               | "Abrir carpeta" / "Open folder" · "Clonar" / "Clone" · "Iniciar" / "Initialize"                                                                                                                                   |
| Lineas de los tiles | "Un repositorio que ya tienes en disco" / "A repository already on disk" · "Desde una URL o tus cuentas" / "From a URL or your accounts" · "Un repositorio nuevo en una carpeta" / "A new repository in a folder" |
| Enlace              | "Buscar repositorios en una carpeta" / "Find repositories in a folder"                                                                                                                                            |
| Workspaces vacio    | "Agrupa repositorios en workspaces." / "Group repositories into workspaces."                                                                                                                                      |
| Aviso de la nube    | "Tienes {count} repositorios en tu cuenta que no están en este equipo." / "You have {count} repositories in your account that aren't on this device."                                                             |

- **Variantes.** Claro y oscuro; ventana de 1280 y de 1024 px de ancho.

### 6. D-06 Repositorios de la nube que no estan en este equipo

- **Proposito.** Los workspaces y la lista se sincronizan, los clones no: el segundo equipo los ve
  sin carpeta.
- **Muestra.** Variante de la tarjeta de repo: columna 2 con alias y la identidad remota en mono
  (`github.com/acme/web-app`), nunca una ruta de otro equipo (B-06); columna 3 con la cuenta de este
  equipo cuyo host coincide, o "Sin cuenta para {host}"; columna 4 con la pastilla "No clonado en este
  equipo" (`--muted` sobre `--bg`); columna 5 con un boton pequeno "Clonar" y `⋯`. Pulsar la tarjeta
  abre D-08 relleno, no una pestana. El filtro Tipo gana la opcion "No clonados aquí".
- **Menu** `⋯`: "Clonar…", "Copiar URL", "Workspaces ›" (como hoy) y "Quitar de mi cuenta…" (peligro,
  con confirmacion: lo quita en todos los equipos).
- **"Abrir todos" de un workspace con repos que faltan.** Modal: cuantos faltan, lista con casillas
  (marcadas), carpeta de destino comun con "Elegir…", y dos salidas.
- **Estados.** Normal, clonando (la tarjeta ensena la barra de progreso de D-08 y "Cancelar"), error.

| Uso                  | Texto                                                                                                                                                                                                                                      |
| -------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| Pastilla             | "No clonado en este equipo" / "Not cloned on this device"                                                                                                                                                                                  |
| Sin cuenta           | "Sin cuenta para {host}" / "No account for {host}"                                                                                                                                                                                         |
| Filtro               | "No clonados aquí" / "Not cloned here"                                                                                                                                                                                                     |
| Quitar               | "¿Quitar {repo} de tu cuenta?" / "Remove {repo} from your account?" · "Desaparece de la lista en todos tus equipos. Las copias en disco no se tocan." / "It disappears from the list on all your devices. Copies on disk are not touched." |
| Abrir todos, titulo  | "Faltan {count} de {total} repositorios" / "{count} of {total} repositories are missing"                                                                                                                                                   |
| Abrir todos, botones | "Clonar y abrir todos" / "Clone and open all" · "Abrir solo los que hay" / "Open only those here"                                                                                                                                          |

- **Variantes.** Claro y oscuro; tarjeta de lista y tarjeta de la rejilla de workspace.

### 7. D-07 Ruta desaparecida

- **Proposito.** El repo se movio o se borro en disco; no puede quedarse roto sin explicacion.
- **Muestra.** Tarjeta de repo con la ruta en mono tachada y en `--faint`, la pastilla "No se
  encuentra la carpeta" en `--warn` y, en la columna 5, "Localizar…" y `⋯` ("Localizar…", "Copiar
  ruta anterior", "Quitar de la lista…"). Si la pestana estaba abierta, el centro de la vista de repo
  ensena el mismo mensaje como estado vacio, con la barra de herramientas desactivada.
- **Localizar.** Selector nativo. Si es el mismo repositorio (mismo remoto o mismo primer commit),
  se actualiza la ruta con un toast. Si es otro repositorio o no es un repositorio, modal con el
  motivo y "Elegir otra carpeta".

| Uso                | Texto                                                                                                                                                 |
| ------------------ | ----------------------------------------------------------------------------------------------------------------------------------------------------- |
| Pastilla           | "No se encuentra la carpeta" / "Folder not found"                                                                                                     |
| Estado en la vista | "No encontramos {repo} en {path}. Puede que lo hayas movido o borrado." / "We can't find {repo} at {path}. It may have been moved or deleted."        |
| Botones            | "Localizar…" / "Locate…" · "Quitar de la lista…" / "Remove from list…" · "Copiar ruta anterior" / "Copy previous path"                                |
| Actualizada        | "Ruta actualizada" / "Path updated"                                                                                                                   |
| Otro repo          | "Esa carpeta es otro repositorio ({remote})." / "That folder is a different repository ({remote})." · "Elegir otra carpeta" / "Choose another folder" |

- **Variantes.** Claro y oscuro; tarjeta y vista de repo.

### 8. D-08 Clonar con progreso

- **Proposito.** Clonar tarda minutos en un repo grande; hoy es un toast inmediato.
- **Formulario** (modal de 580 px):
  1. "Cuenta": cuentas conectadas mas "Sin cuenta (credenciales del sistema)". Va primero porque
     `owner/repo` se resuelve contra el host de la cuenta.
  2. "URL o owner/repo" en mono, con autocompletado de los repos de la cuenta elegida (avatar del
     propietario, `owner/nombre`, candado si es privado, descripcion en una linea; estado "Buscando en
     {platform}…"). Debajo, la URL ya analizada: `github.com · acme/web-app`, o el error en linea.
     Si se pega una URL de otro host, la cuenta cambia sola a una de ese host exacto o a "Sin
     cuenta", con la linea informativa y "Conectar cuenta…".
  3. "Protocolo": segmentado HTTPS o SSH. SSH desactivado, con motivo, si la cuenta no tiene llave
     SSH. Debajo, la URL final en mono `--faint`.
  4. "Carpeta de destino": ruta de solo lectura en mono con "Elegir…" (selector nativo) y la ruta
     final: "Se clonará en {path}".
  5. "Workspace": como hoy.
  6. Casilla "Incluir submódulos" con `--recurse-submodules` en mono como descripcion.
- **Progreso** (el mismo modal; los campos se pliegan a una linea de resumen): fases en lista con la
  actual resaltada (conectando, contando, recibiendo, resolviendo, extrayendo, submodulos), barra en
  `--accent`, porcentaje, tamano y velocidad. Botones "Seguir en segundo plano" (cierra el modal y la
  tarjeta del repo aparece arriba de la lista con la barra y "Cancelar") y "Cancelar".
- **Exito.** Se abre la pestana del repo y un toast `ok`.
- **Errores** (caja de error en el modal con "Copiar detalle"): autenticacion (con "Actualizar
  credenciales" y "Usar otra cuenta"), no existe o sin acceso, destino ocupado (y variante "ya lo
  tienes ahi" con "Abrirlo"), red con "Reintentar", host SSH desconocido (abre D-13 de #65) y aviso
  final de Git LFS ausente.

| Uso                 | Texto                                                                                                                                                                                                                                                                                 |
| ------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Titulo              | "Clonar repositorio" / "Clone repository"                                                                                                                                                                                                                                             |
| URL invalida        | "No reconocemos esta URL. Prueba con https://, git@ o owner/repo." / "We don't recognize this URL. Try https://, git@ or owner/repo."                                                                                                                                                 |
| Sin cuenta del host | "No tienes una cuenta de {host} en Ramal. Se usarán las credenciales del sistema." / "You don't have a {host} account in Ramal. System credentials will be used." · "Conectar cuenta…" / "Connect account…"                                                                           |
| SSH desactivado     | "Esta cuenta no tiene llave SSH." / "This account has no SSH key."                                                                                                                                                                                                                    |
| Destino             | "Se clonará en {path}" / "Will be cloned into {path}" · "Elegir…" / "Choose…"                                                                                                                                                                                                         |
| Submodulos          | "Incluir submódulos" / "Include submodules"                                                                                                                                                                                                                                           |
| Progreso, titulo    | "Clonando {repo}" / "Cloning {repo}"                                                                                                                                                                                                                                                  |
| Fases               | "Conectando" / "Connecting" · "Contando objetos" / "Counting objects" · "Recibiendo objetos" / "Receiving objects" · "Resolviendo deltas" / "Resolving deltas" · "Extrayendo archivos" / "Checking out files" · "Submódulos {current} de {total}" / "Submodules {current} of {total}" |
| Detalle             | "{pct} % · {size} · {speed}/s" / "{pct}% · {size} · {speed}/s"                                                                                                                                                                                                                        |
| Botones             | "Clonar" / "Clone" · "Seguir en segundo plano" / "Continue in background" · "Cancelar" / "Cancel"                                                                                                                                                                                     |
| Cancelado           | "Clonado cancelado. Borramos la carpeta a medias." / "Clone cancelled. We removed the partial folder."                                                                                                                                                                                |
| Exito               | "{repo} clonado" / "{repo} cloned"                                                                                                                                                                                                                                                    |
| Auth                | "{platform} rechazó las credenciales de @{user}." / "{platform} rejected @{user}'s credentials." · "Actualizar credenciales" / "Update credentials" · "Usar otra cuenta" / "Use another account"                                                                                      |
| No existe           | "{repo} no existe o tu cuenta no tiene acceso." / "{repo} doesn't exist or your account has no access."                                                                                                                                                                               |
| Ocupado             | "{path} ya existe y no está vacía." / "{path} already exists and isn't empty." · "Ya tienes este repositorio en {path}." / "You already have this repository at {path}." · "Abrirlo" / "Open it"                                                                                      |
| Red                 | "No podemos conectar con {host}." / "We can't reach {host}." · "Reintentar" / "Retry"                                                                                                                                                                                                 |
| LFS                 | "Este repositorio usa Git LFS y no está instalado: verás punteros en vez de archivos." / "This repository uses Git LFS and it isn't installed: you'll see pointers instead of files."                                                                                                 |

- **Variantes.** Claro y oscuro; formulario, progreso, error y tarjeta clonando en la lista.

### 9. D-09 Selector de carpeta nativo, "No es un repositorio" y confianza

- **Proposito.** Abrir, Iniciar, Localizar y el destino de Clonar dependen del selector nativo. El
  webview nunca escribe ni manda una ruta.
- **Abrir.** El boton "Abrir" lanza el dialogo del sistema directamente; desaparece el modal de ruta
  en texto del prototipo. Resultados:
  - Subcarpeta de un repo: se abre la raiz y un toast `info` lo dice.
  - Ya esta en la lista: se enfoca su pestana.
  - No es un repositorio: modal con la ruta en mono, "Iniciar aquí", "Elegir otra carpeta" y
    "Cancelar".
  - Git antiguo: D-02.
  - **Confianza (ADR 0009).** Si el repo no lo clono Ramal y su `.git/config` o sus hooks pueden
    ejecutar programas, modal "¿Confías en este repositorio?" con la lista de lo encontrado en mono
    (`core.fsmonitor = …`, `.git/hooks/pre-commit`) y dos salidas. Sin confianza, la vista de repo
    abre con un banner `--warn` de solo lectura y todas las acciones que escriben desactivadas con
    su motivo en el tooltip.
- **Iniciar.** Selector nativo (permite crear carpeta) y despues modal "Iniciar repositorio": ruta en
  solo lectura con "Cambiar…", "Rama inicial" (rellena con la rama por defecto de Ajustes > Git y
  validada con las reglas de refs de git, B-31), nota si la carpeta ya tiene archivos, casilla
  "Publicar en una plataforma al terminar" (abre D-11). Si ya es un repo: "Ya es un repositorio" con
  "Abrirlo".

| Uso                | Texto                                                                                                                                                                                                                                     |
| ------------------ | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Raiz abierta       | "Abrimos la raíz del repositorio: {path}" / "Opened the repository root: {path}"                                                                                                                                                          |
| No es repo         | "{path} no es un repositorio de Git" / "{path} isn't a Git repository" · "Iniciar aquí" / "Initialize here" · "Elegir otra carpeta" / "Choose another folder"                                                                             |
| Confianza, titulo  | "¿Confías en este repositorio?" / "Do you trust this repository?"                                                                                                                                                                         |
| Confianza, texto   | "Su configuración puede ejecutar programas cuando Ramal usa Git. Si no sabes de dónde viene, ábrelo solo para leer." / "Its configuration can run programs when Ramal uses Git. If you don't know where it came from, open it read-only." |
| Confianza, botones | "Abrir solo para leer" / "Open read-only" · "Confiar y abrir" / "Trust and open"                                                                                                                                                          |
| Banner             | "Solo lectura: no confías en este repositorio. Las acciones que escriben están desactivadas." / "Read-only: you don't trust this repository. Actions that write are disabled." · "Confiar…" / "Trust…"                                    |
| Iniciar            | "Iniciar repositorio" / "Initialize repository" · "Rama inicial" / "Initial branch" · "Publicar en una plataforma al terminar" / "Publish to a platform when done"                                                                        |
| Con archivos       | "La carpeta ya tiene archivos: quedarán sin seguimiento hasta que los prepares." / "The folder already has files: they stay untracked until you stage them."                                                                              |
| Ya es repo         | "Esta carpeta ya es un repositorio." / "This folder is already a repository." · "Abrirlo" / "Open it"                                                                                                                                     |

- **Variantes.** Claro y oscuro. El dialogo nativo no se dibuja: basta una nota en el lienzo.

### 10. D-10 Repositorio vacio

- **Proposito.** Todo repo recien iniciado tiene la rama sin nacer y el grafo no sabe dibujarlo.
- **Muestra.** Centro del grafo con estado vacio (titulo y una linea). Si hay archivos, la fila WIP
  sigue arriba con su nodo discontinuo. Selector de rama: `main` y la nota "sin commits". LOCAL en la
  barra lateral: `main` con "sin nacer" en `--faint`. Panel de staging normal; el boton de commit dice
  "Primer commit en {branch}"; amend desactivado.
- **Desactivados con motivo:** Deshacer, Rama, Stash y Pop; Fetch y Pull si no hay remoto. Push pasa
  a llamarse "Publicar" y abre D-11 si no hay remoto.

| Uso      | Texto                                                                                                                                                                                                                    |
| -------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| Vacio    | "Este repositorio aún no tiene commits" / "This repository has no commits yet" · "La rama {branch} nacerá con tu primer commit." / "The {branch} branch is born with your first commit."                                 |
| Rama     | "sin commits" / "no commits" · "sin nacer" / "unborn"                                                                                                                                                                    |
| Boton    | "Primer commit en {branch}" / "First commit on {branch}"                                                                                                                                                                 |
| Motivos  | "Haz el primer commit para crear ramas." / "Make the first commit to create branches." · "No hay commits que deshacer." / "No commits to undo." · "Este repositorio no tiene remoto." / "This repository has no remote." |
| Publicar | "Publicar" / "Publish"                                                                                                                                                                                                   |

- **Variantes.** Claro y oscuro; sin archivos y con archivos sin seguimiento.

### 11. D-11 Publicar al iniciar

- **Proposito.** El flujo Iniciar termina hoy sin remoto.
- **Muestra.** Modal de 580 px "Publicar repositorio": "Cuenta" (si no hay ninguna, linea con
  "Conectar cuenta…"), "Propietario" (`@usuario` y organizaciones, grupos, workspaces o proyectos
  segun la plataforma, cargados de la plataforma), "Nombre" (relleno con la carpeta, con
  comprobacion en vivo de que esta libre), "Visibilidad" ("Privado" por defecto, "Público", e "Interno"
  solo donde la plataforma lo tiene), "Descripción", ".gitignore" (plantillas), "Licencia", casilla
  "README", "Nombre del remoto" (`origin`) y "Protocolo" (HTTPS o SSH).
- **Progreso.** Tres pasos con estado: crear el repo en la plataforma, anadir el remoto y push (con
  la barra de D-17 de #65).
- **Exito.** Toast y, en el modal, "Abrir en {platform}" y "Copiar enlace".
- **Errores.** Nombre ocupado, permisos insuficientes del token, la organizacion no deja crear
  repos, SSO de GitHub (enlaza al estado de D-12 de #65), red, y fallo parcial (creado pero sin push)
  con "Reintentar push".
- **Entradas.** Casilla de D-09 Iniciar, boton "Publicar" de D-10 y "Publicar…" en REMOTOS vacio.

| Uso            | Texto                                                                                                                                                                                         |
| -------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Titulo         | "Publicar repositorio" / "Publish repository"                                                                                                                                                 |
| Campos         | "Propietario" / "Owner" · "Visibilidad" / "Visibility" · "Privado" / "Private" · "Público" / "Public" · "Interno" / "Internal" · "Licencia" / "License" · "Nombre del remoto" / "Remote name" |
| Ya existe      | "Ya existe" / "Already exists" (junto a .gitignore, licencia o README si el archivo ya esta)                                                                                                  |
| Commit inicial | "Los archivos elegidos se añaden en un commit nuevo." / "The chosen files are added in a new commit."                                                                                         |
| Nombre ocupado | "{owner}/{name} ya existe." / "{owner}/{name} already exists."                                                                                                                                |
| Pasos          | "Creando {repo} en {platform}" / "Creating {repo} on {platform}" · "Añadiendo el remoto {remote}" / "Adding remote {remote}" · "Haciendo push de {branch}" / "Pushing {branch}"               |
| Exito          | "Publicado en {url}" / "Published to {url}" · "Abrir en {platform}" / "Open on {platform}"                                                                                                    |
| Sin permiso    | "Tu token no puede crear repositorios. Le falta el permiso {scope}." / "Your token can't create repositories. It lacks the {scope} scope."                                                    |
| Organizacion   | "{org} no deja crear repositorios a sus miembros." / "{org} doesn't let members create repositories."                                                                                         |
| Parcial        | "El repositorio se creó, pero el push falló." / "The repository was created, but the push failed." · "Reintentar push" / "Retry push"                                                         |

- **Variantes.** Claro y oscuro; GitHub, GitLab (con Interno) y Azure DevOps (proyecto en vez de
  propietario).

## Reglas

- **Sin muro de cuenta** (ADR 0014): ninguna pantalla de entrada exige iniciar sesion, y las zonas de
  nube invitan con la tarjeta de invitacion en vez de desaparecer.
- **Desktop no tiene campo de contrasena** (ADR 0008): se entra por el navegador o con el codigo.
- **El webview nunca manda una ruta**: Abrir, Iniciar, Localizar y el destino de Clonar usan el
  selector nativo. Los campos de ruta son de solo lectura, con boton "Elegir…".
- **Las credenciales se eligen por host exacto** (ADR 0018): clonar solo ofrece cuentas cuyo host es
  identico al de la URL; un host que no es ASCII se ensena en punycode.
- **Git es un requisito visible** (X-13): version y ruta al pie de D-01, D-02 bloqueante, minimo
  2.34. En Windows, D-02 solo aparece si falta el MinGit incluido.
- Bugs del prototipo que no se copian: B-04 (cada opcion de un formulario tiene efecto, incluida la
  casilla de mover a la papelera al quitar un repo), B-05 (Local o Remoto segun el remoto de git, no
  segun la cuenta), B-06 (ninguna ruta de otro equipo), B-30 (analizar URL https, ssh, estilo scp,
  `owner/repo` y rutas de Azure), B-31 (nombre de rama validado), B-42 (plurales), B-44 (parametros
  con nombre), B-46 (ningun texto dice "simulado"), B-61 (el toast de informacion usa `--info`).
- Toda accion destructiva confirma con el modal de peligro de #71 (pregunta, consecuencia en una
  caja, boton `--danger`, foco inicial en "Cancelar"): "Quitar de mi cuenta", "Quitar de la lista".
- Ningun texto vive en el marcado: todo va al diccionario `D` de `ramal-data.js`, en castellano e
  ingles. Colores solo de los tokens del handoff.
- Dominio real: `ramal.bahoque.com`. Nada de `ramal.dev` ni `ramal.app`, tampoco en los datos de
  ejemplo (usa `example.com`).

## Entregable

- Un zip de handoff nuevo, `ramal-handoff-64-entrada.zip`, con la misma estructura que el original:
  `README.md` (solo la seccion de pantallas nuevas, con los ids de este brief), `Ramal Entrada.dc.html`,
  `ramal-data.js` (el original mas las claves nuevas en `D`) y `support.js` sin cambios.
- `Ramal Entrada.dc.html` copia la cabecera de estilos y la barra de titulo de `Ramal.dc.html`. Si
  D-42 (#67) ya existe en el proyecto, usa esa barra de titulo.
- Props: `theme` (`dark`, `light`), `lang` (`es`, `en`), `os` (`macos`, `windows`, `linux`, para D-02
  y la barra de titulo), `session` (`signedIn`, `signedOut`) y `screen`, un enum con un valor por
  estado: `d01-cuenta`, `d01-identidad`, `d01-repos`, `d01-plataformas`, `d01-ia`, `d02-ausente`,
  `d02-antiguo`, `d03-navegador`, `d03-codigo`, `d03-error`, `d04-sin-sesion`, `d05-vacio`,
  `d05-solo-nube`, `d06-lista`, `d06-abrir-todos`, `d07-tarjeta`, `d07-vista`, `d08-formulario`,
  `d08-progreso`, `d08-error`, `d09-no-repo`, `d09-confianza`, `d09-solo-lectura`, `d09-iniciar`,
  `d10-vacio`, `d11-formulario`, `d11-progreso`, `d11-error`.
- Las transiciones principales funcionan con clics (formulario a progreso, progreso a exito, pasos
  de D-01), igual que el prototipo original.

## Hecho cuando

- [ ] D-01 a D-11 diseñados
- [ ] Las zonas de nube invitan a entrar en vez de desaparecer
- [ ] Git aparece como requisito visible
