# Brief #66: vistas que prometen los menus y diff real (Git Rudder Desktop)

## Contexto

Solo Git Rudder Desktop; VS Code reutiliza como webview el grafo, el editor de merge y crear PR. Hoy
historial y blame son toasts, "Comparar con" abre un diff fijo, el diff es el mismo para todo y el
grafo carga 19 commits. Cuelga de la vista de repo de `Git Rudder.dc.html`: centro intercambiable con
"← Grafo", barra lateral (LOCAL, REMOTOS, PULL REQUESTS, STASHES, TAGS), panel derecho de staging o
detalle, diff de `48px | 18px | codigo` con `--add-bg`, `--del-bg` y `--hatch`, resolvedor, tabla de
PRs del workspace y botones de filtro de Inicio. D-34 y D-39 son P2 y no entran.

## Pantallas y estados

Formato de los textos: `"castellano" / "english"`. Los parametros tienen nombre en ingles y son
iguales en los dos idiomas (`{branch}`, `{host}`); los contadores usan `{count}` con claves `_one` y
`_other`.

### 1. D-27 Historial de un archivo

- **Proposito.** Esta en el menu de archivo y hoy es un toast.
- **Muestra.** Vista central con cabecera de 50 px ("← Grafo", ruta en mono, "Seguir renombrados"
  como interruptor) y dos columnas: lista de commits que tocan el archivo (340 px: sha, mensaje,
  avatar con iniciales, fecha relativa y +/−; una pastilla "renombrado desde {path}" donde cambia el
  nombre) y, a la derecha, el diff de ese archivo en el commit elegido, con el componente de D-36.
  El panel derecho de la vista ensena el detalle del commit elegido.
- **Acciones.** "Ver en el grafo", "Copiar SHA", "Blame en esta versión", "Abrir esta versión en el
  editor".
- **Estados.** Cargando (C-01), sin historial (archivo sin seguimiento), "Cargar más" al final de la
  lista, archivo binario (D-36), error.

| Uso      | Texto                                                                                                                                                       |
| -------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Cabecera | "Historial de {file}" / "History of {file}" · "Seguir renombrados" / "Follow renames"                                                                       |
| Pastilla | "renombrado desde {path}" / "renamed from {path}"                                                                                                           |
| Acciones | "Ver en el grafo" / "Show in graph" · "Blame en esta versión" / "Blame at this version" · "Abrir esta versión en el editor" / "Open this version in editor" |
| Vacio    | "Este archivo aún no tiene historial." / "This file has no history yet."                                                                                    |
| Mas      | "Cargar más" / "Load more"                                                                                                                                  |

- **Variantes.** Claro y oscuro.

### 2. D-28 Blame

- **Proposito.** Menu sin vista.
- **Muestra.** Vista central: cabecera con ruta, selector de revision ("En: HEAD ▾"), interruptor
  "Ignorar espacios" y enlace a D-27. Cuerpo en mono 12,5 px: canal izquierdo de 260 px con un
  bloque por grupo de lineas del mismo commit (solo en la primera linea: avatar, autor, fecha
  relativa, sha y mensaje recortado) y barra de antiguedad de 3 px (mas reciente, mas cerca de
  `--accent`; mas antigua, mas cerca de `--border`). Al pasar por un bloque, tooltip con el commit
  completo. Las lineas sin commit dicen "Sin commit todavía".
- **Acciones por bloque.** Pulsar selecciona el commit en el detalle; "Blame del commit anterior"
  da un paso atras en la historia de esas lineas.
- **Estados.** Cargando, archivo binario (sin blame), archivo enorme (virtualizado), error.

| Uso        | Texto                                                                                                                  |
| ---------- | ---------------------------------------------------------------------------------------------------------------------- |
| Cabecera   | "Blame de {file}" / "Blame of {file}" · "En: {revision}" / "At: {revision}" · "Ignorar espacios" / "Ignore whitespace" |
| Sin commit | "Sin commit todavía" / "Not committed yet"                                                                             |
| Paso atras | "Blame del commit anterior" / "Blame previous commit"                                                                  |
| Binario    | "Los archivos binarios no tienen blame." / "Binary files have no blame."                                               |

- **Variantes.** Claro y oscuro.

### 3. D-29 Comparar ramas

- **Proposito.** "Comparar con" abre hoy un diff fijo de un solo archivo (B-26).
- **Muestra.** Vista central con cabecera: dos selectores de ref en mono ("{refA} ▾" y "{refB} ▾") con un
  boton ⇄ para intercambiar, y una linea que dice que se compara desde la base comun. Pestanas
  "Commits" (dos secciones: los que estan en A y no en B, y al reves, con contador) y "Archivos"
  (lista con estado y +/−; pulsar abre el diff de D-36 a la derecha).
- **Acciones.** "Crear pull request" (abre D-31 con esas ramas), "Merge de {refB} en {refA}…",
  "Rebase de {refA} sobre {refB}…".
- **Estados.** Cargando, ramas iguales, una contiene a la otra, error.

| Uso       | Texto                                                                                                                                                                       |
| --------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Cabecera  | "Comparar" / "Compare" · "Desde la base común {sha}" / "Since common base {sha}"                                                                                            |
| Pestanas  | "Commits" · "Archivos ({count})" / "Files ({count})"                                                                                                                        |
| Secciones | "En {refA}, no en {refB} ({count})" / "In {refA}, not in {refB} ({count})"                                                                                                  |
| Iguales   | "{refA} y {refB} apuntan al mismo commit." / "{refA} and {refB} point to the same commit."                                                                                  |
| Acciones  | "Crear pull request" / "Create pull request" · "Merge de {refB} en {refA}…" / "Merge {refB} into {refA}…" · "Rebase de {refA} sobre {refB}…" / "Rebase {refA} onto {refB}…" |

- **Variantes.** Claro y oscuro.

### 4. D-30 Detalle de pull request

- **Proposito.** Pulsar un PR hoy solo abre el repo.
- **Muestra.** Vista central. Cabecera: numero (`#` o `!` en GitLab) y titulo, insignia de estado con
  los colores del handoff, `{source} → {target}` en mono, autor y fecha. Pestanas:
  "Conversación" (descripcion en Markdown saneado, comentarios y revisiones en linea de tiempo),
  "Commits", "Archivos" (diff de D-36) y "Checks" (estado, nombre, duracion y "Ver en {platform}").
- **Acciones.** "Revisar" (desplegable: Aprobar, Pedir cambios, Comentar, con area de texto),
  "Fusionar" (metodo segun lo que permita la plataforma: merge, squash o rebase; confirmacion),
  "Checkout de la rama", "Abrir en {platform}", "Copiar enlace". En un borrador, "Marcar como
  listo".
- **Estados.** Cargando; sin cuenta para el host (llamada a conectar una); error de la plataforma
  (401, limite de peticiones, SSO); fusionar desactivado con motivo (checks fallidos, faltan
  revisiones, conflictos); conflictos con "Resolver en local"; PR propio (no se puede aprobar);
  fusionado o cerrado (sin acciones, con quien y cuando).
- **Entradas.** Fila de la tabla de PRs del workspace y elemento de PULL REQUESTS en la barra lateral.

| Uso         | Texto                                                                                                                                                                              |
| ----------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Pestanas    | "Conversación" / "Conversation" · "Commits" · "Archivos" / "Files" · "Checks"                                                                                                      |
| Revisar     | "Revisar" / "Review" · "Aprobar" / "Approve" · "Pedir cambios" / "Request changes" · "Comentar" / "Comment"                                                                        |
| Fusionar    | "Fusionar" / "Merge" · "Crear commit de merge" / "Create a merge commit" · "Squash y fusionar" / "Squash and merge" · "Rebase y fusionar" / "Rebase and merge"                     |
| Desactivado | "Hay checks fallidos." / "Some checks are failing." · "Faltan {count} revisiones." / "{count} reviews missing." · "Tiene conflictos con {target}." / "It conflicts with {target}." |
| Conflictos  | "Resolver en local" / "Resolve locally"                                                                                                                                            |
| Propio      | "No puedes aprobar tu propio pull request." / "You can't approve your own pull request."                                                                                           |
| Sin cuenta  | "Conecta una cuenta de {host} para ver sus pull requests." / "Connect a {host} account to see its pull requests." · "Conectar cuenta…" / "Connect account…"                        |
| Fusionado   | "Fusionado por @{user} {ago}" / "Merged by @{user} {ago}"                                                                                                                          |
| Borrador    | "Marcar como listo" / "Mark as ready"                                                                                                                                              |
| Limite      | "{platform} limita las peticiones. Vuelve a intentarlo a las {time}." / "{platform} is rate limiting. Try again at {time}."                                                        |

- **Variantes.** Claro y oscuro; abierto, borrador, fusionado y con conflictos.

### 5. D-31 Crear pull request completo

- **Proposito.** El modal actual cubre el caso minimo y falla con ramas sin publicar.
- **Muestra.** Vista central "Nuevo pull request" en dos columnas. Izquierda, formulario: "Cuenta",
  "Desde" (rama), "Hacia" (repositorio destino, que ofrece el upstream si es un fork, y rama, por
  defecto la rama por defecto de la plataforma, B-11), "Título", "Descripción" con pestanas
  "Escribir" y "Vista previa" y selector de plantilla si el repo tiene varias, "✦ Generar título y
  descripción con IA", "Revisores" y "Asignados" (chips con autocompletado de la plataforma y
  "Asignarme"), "Etiquetas" (chips), casilla "Crear como borrador". Derecha: commits que entran y
  numero de archivos, con enlace a D-29.
- **Estados.**
  - Rama sin publicar: banner con "Publicar rama y continuar" (abre D-24 de #65); "Crear" desactivado.
  - Ya hay un PR abierto para esta rama: aviso con "Ver PR" (D-30).
  - Sin diferencias entre ramas: "Crear" desactivado con motivo.
  - Cargando revisores, etiquetas y plantillas; error de la plataforma; permiso insuficiente.
  - Exito: pantalla con "Abrir en {platform}", "Ver en Git Rudder" y "Copiar enlace".
- **Solo se muestra lo que la plataforma admite** (etiquetas, asignados, borrador).

| Uso          | Texto                                                                                                                                                                            |
| ------------ | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Titulo       | "Nuevo pull request" / "New pull request"                                                                                                                                        |
| Campos       | "Hacia" / "Into" · "Plantilla" / "Template" · "Escribir" / "Write" · "Vista previa" / "Preview" · "Asignados" / "Assignees" · "Asignarme" / "Assign me" · "Etiquetas" / "Labels" |
| Sin publicar | "{branch} no está publicada en {remote}." / "{branch} isn't published to {remote}." · "Publicar rama y continuar" / "Publish branch and continue"                                |
| Ya existe    | "Ya hay un pull request abierto para {branch}: #{number}." / "There's already an open pull request for {branch}: #{number}." · "Ver PR" / "View PR"                              |
| Sin cambios  | "No hay diferencias entre {source} y {target}." / "There are no differences between {source} and {target}."                                                                      |
| Derecha      | "{commits} commits · {files} archivos" / "{commits} commits · {files} files"                                                                                                     |
| Exito        | "Pull request #{number} creado" / "Pull request #{number} created" · "Ver en Git Rudder" / "View in Git Rudder"                                                                  |

- **Variantes.** Claro y oscuro. Densidad compacta de 28 px, porque VS Code reutiliza este
  formulario como webview.

### 6. D-32 Gestion de remotos

- **Proposito.** Solo existe editar la URL.
- **Muestra.** Boton "+" en la cabecera de REMOTOS y menu de cada remoto: "Fetch {remote}", "Renombrar…",
  "Editar URL…", "Copiar URL" y "Quitar…" (peligro). Modal "Añadir remoto": "Nombre", "URL" (con la URL
  analizada debajo y la cuenta que encaja por host), casilla "Hacer fetch al añadir". "Editar URL"
  permite una URL de push distinta. REMOTOS vacio: linea con "Añadir remoto" y "Publicar…" (D-11 de
  #64).
- **Estados.** Validacion en linea: nombre repetido o invalido, URL que no se reconoce.

| Uso     | Texto                                                                                                                                                                                                                            |
| ------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Menu    | "Renombrar…" / "Rename…" · "Editar URL…" / "Edit URL…" · "Copiar URL" / "Copy URL" · "Quitar…" / "Remove…"                                                                                                                       |
| Alta    | "Añadir remoto" / "Add remote" · "Hacer fetch al añadir" / "Fetch after adding" · "URL de push distinta" / "Different push URL"                                                                                                  |
| Quitar  | "¿Quitar el remoto {remote}?" / "Remove remote {remote}?" · "Se borran de tu copia las ramas remotas de {remote}. El servidor no cambia." / "Remote branches of {remote} are removed from your copy. The server doesn't change." |
| Errores | "Ya hay un remoto llamado {remote}." / "A remote named {remote} already exists."                                                                                                                                                 |
| Vacio   | "Sin remotos" / "No remotes"                                                                                                                                                                                                     |

- **Variantes.** Claro y oscuro.

### 7. D-33 Vista previa de un stash y stash de archivos sueltos

- **Proposito.** Aplicar un stash a ciegas es la forma habitual de liarla.
- **Muestra.** Pulsar un stash lo selecciona y el panel derecho ensena su detalle, con el mismo
  esquema que el detalle de commit: mensaje, fecha, commit base (sha y rama), archivos con +/− (los
  sin seguimiento marcados) y diff al pulsar. Acciones: "Aplicar", "Pop", "Crear rama desde el stash"
  y "Eliminar…" (peligro, B-29).
- **Stash de archivos sueltos.** Con varios archivos seleccionados (D-37), la accion "Stash de los
  seleccionados…" abre el modal de stash con una linea informativa del numero de archivos. Opciones
  con efecto: mensaje, "Incluir archivos sin seguimiento" y "Mantener lo preparado".

| Uso      | Texto                                                                                                                                                                               |
| -------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Detalle  | "stash@{index}" / "stash@{index}" · "Sobre {sha} en {branch}" / "On {sha} in {branch}" · "sin seguimiento" / "untracked"                                                            |
| Acciones | "Aplicar" / "Apply" · "Pop" · "Crear rama desde el stash" / "Create branch from stash" · "Eliminar…" / "Drop…"                                                                      |
| Sueltos  | "Stash de los seleccionados…" / "Stash selected…" · "Solo los {count} archivos seleccionados" / "Only the {count} selected files" · "Mantener lo preparado" / "Keep staged changes" |

- **Variantes.** Claro y oscuro.

### 8. D-35 Staging y descarte por bloque y por linea

- **Proposito.** Es la funcion por la que se elige un cliente asi; "Parcial" existe en el diccionario
  sin uso.
- **Muestra.** En el diff de un archivo sin preparar, cada cabecera de bloque lleva "Preparar bloque"
  y "Descartar bloque" (este en `--danger`, con confirmacion). Las lineas se seleccionan pulsando el
  canal de numeros (Mayus para un rango); aparece una barra flotante con "Preparar {count} líneas" y
  "Descartar {count} líneas". En un archivo preparado, "Quitar bloque" y "Quitar {count} líneas". En lado a
  lado, las lineas quitadas se eligen a la izquierda y las anadidas a la derecha. En la lista de
  staging, un archivo a medias aparece en las dos listas con la pastilla "Parcial".
- **Desactivado.** Binarios, punteros LFS y archivos en conflicto, con motivo.

| Uso       | Texto                                                                                                                                                        |
| --------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| Bloque    | "Preparar bloque" / "Stage hunk" · "Descartar bloque" / "Discard hunk" · "Quitar bloque" / "Unstage hunk"                                                    |
| Lineas    | "Preparar {count} líneas" / "Stage {count} lines" · "Descartar {count} líneas" / "Discard {count} lines" · "Quitar {count} líneas" / "Unstage {count} lines" |
| Confirmar | "¿Descartar estas líneas? No se puede deshacer." / "Discard these lines? This can't be undone."                                                              |
| Pastilla  | "Parcial" / "Partial"                                                                                                                                        |
| Motivo    | "No se puede preparar por partes un archivo binario." / "Binary files can't be partially staged."                                                            |

- **Variantes.** Claro y oscuro; lado a lado y unificado.

### 9. D-36 Diff real

- **Proposito.** El prototipo usa el mismo diff de muestra para todo.
- **Muestra y estados** (todos en lado a lado y unificado):
  - Cabeceras de bloque `@@ -12,7 +12,9 @@` con el contexto de funcion, en una franja `--panel`.
  - Regiones sin cambios plegadas: "⋯ {count} líneas sin cambios" con "Mostrar 20 más" arriba y abajo.
  - Resaltado de sintaxis sin colores nuevos, con los tokens que ya hay: palabras clave en `--info`,
    cadenas en `--ok`, numeros y constantes en `--warn`, comentarios en `--faint`, tipos en
    `--accent` y funciones en el segundo color de carril (`oklch(0.68 0.2 300)`). Comprueba el
    contraste sobre `--add-bg` y `--del-bg` en claro y oscuro.
  - Cambios por palabra: solo las palabras cambiadas llevan un fondo mas intenso, `--ok` o
    `--danger` al 28 % con `color-mix`.
  - Menu de espacios: "Mostrar todo", "Ignorar espacios al final" e "Ignorar todos los espacios".
  - Renombrado con cambios ("Renombrado de {path} · {pct} % parecido") y sin cambios.
  - Cambio de permisos (`100644 → 100755`) y solo finales de linea (CRLF y LF).
  - Binario: tamano antes y despues, sin diff.
  - Imagen: antes y despues sobre fondo de cuadros, con dimensiones y peso.
  - Archivo enorme: aviso y "Mostrar igualmente".
  - Sin seguimiento, nuevo y borrado.
  - Puntero de Git LFS: oid, tamano y si el contenido esta descargado.
  - Submodulo: `{sha} → {sha}` con "Abrir submódulo".
  - Falta salto de linea al final.
  - El unificado agrupa como git: primero las lineas quitadas y despues las anadidas (B-17).
  - "Abrir en herramienta de diff" en la cabecera si hay una configurada (D-44 de #67).

| Uso        | Texto                                                                                                                                                                         |
| ---------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Plegado    | "{count} líneas sin cambios" / "{count} unchanged lines" · "Mostrar 20 más" / "Show 20 more"                                                                                  |
| Espacios   | "Espacios" / "Whitespace" · "Mostrar todo" / "Show all" · "Ignorar espacios al final" / "Ignore trailing whitespace" · "Ignorar todos los espacios" / "Ignore all whitespace" |
| Renombrado | "Renombrado de {path} · {pct} % parecido" / "Renamed from {path} · {pct}% similar" · "Renombrado sin cambios" / "Renamed without changes"                                     |
| Permisos   | "Permisos {before} → {after}" / "Mode {before} → {after}"                                                                                                                     |
| Finales    | "Solo cambian los finales de línea ({before} → {after})." / "Only line endings change ({before} → {after})."                                                                  |
| Binario    | "Archivo binario · {before} → {after}" / "Binary file · {before} → {after}"                                                                                                   |
| Enorme     | "Este diff tiene {count} líneas y puede tardar." / "This diff has {count} lines and may be slow." · "Mostrar igualmente" / "Show anyway"                                      |
| Estados    | "Archivo nuevo sin seguimiento" / "New untracked file" · "Archivo borrado" / "Deleted file"                                                                                   |
| LFS        | "Puntero de Git LFS · {size}" / "Git LFS pointer · {size}" · "Contenido no descargado" / "Content not downloaded"                                                             |
| Submodulo  | "Submódulo {path}" / "Submodule {path}" · "Abrir submódulo" / "Open submodule"                                                                                                |
| Salto      | "Sin salto de línea al final" / "No newline at end of file"                                                                                                                   |
| Externa    | "Abrir en herramienta de diff" / "Open in diff tool"                                                                                                                          |

- **Variantes.** Claro y oscuro; cada estado.

### 10. D-37 Multiseleccion en staging y en el grafo

- **Proposito.** Esperado en cualquier lista; en el grafo habilita el squash de commits elegidos.
- **Staging.** Clic, Mayus+clic (rango) y Mod+clic (sumar). Filas elegidas en `--accent-soft`. Barra
  de acciones encima de la lista: "{count} seleccionados", "Preparar", "Descartar…", "Stash…" (D-33) e
  "Ignorar".
- **Grafo.** Mismos gestos. El panel derecho pasa a "{count} commits seleccionados" con la lista y las
  acciones: "Cherry-pick de {count} commits", "Squash en uno…" (solo commits seguidos de la rama actual;
  si ya estan en el remoto, aviso de reescribir historia), "Revertir {count} commits", "Comparar el
  primero con el último" (D-29) y "Copiar SHAs". Desactivadas con su motivo.

| Uso      | Texto                                                                                                                                                                                          |
| -------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Contador | "{count} seleccionados" / "{count} selected" · "{count} commits seleccionados" / "{count} commits selected"                                                                                    |
| Acciones | "Cherry-pick de {count} commits" / "Cherry-pick {count} commits" · "Squash en uno…" / "Squash into one…" · "Revertir {count} commits" / "Revert {count} commits" · "Copiar SHAs" / "Copy SHAs" |
| Motivo   | "Solo commits seguidos de la rama actual." / "Only consecutive commits on the current branch."                                                                                                 |

- **Variantes.** Claro y oscuro.

### 11. D-38 Conflictos binarios, borrado frente a modificado y renombrados

- **Proposito.** El resolvedor asume bloques de texto; con estos conflictos no se puede terminar.
- **Muestra.** Para esos archivos, el centro del resolvedor cambia la mitad superior por dos
  tarjetas A y B (miniatura si es imagen, tamano, ultimo commit) y la inferior por la eleccion a
  nivel de archivo. En la lista del panel derecho, un icono por tipo de conflicto. "Abrir en
  herramienta de merge" si hay una configurada (D-44 de #67).
- **Casos.** Binario ("Usar la versión de A" o "de B"); borrado frente a modificado ("Mantener
  borrado" o "Mantener la versión modificada"); renombrado en los dos lados (elegir el nombre).

| Uso        | Texto                                                                                                                                                                                                  |
| ---------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| Binario    | "Conflicto en un archivo binario" / "Conflict in a binary file" · "Usar la versión de A" / "Use version A" · "Usar la versión de B" / "Use version B"                                                  |
| Borrado    | "{side} borró {file} y {other} lo modificó." / "{side} deleted {file} and {other} modified it." · "Mantener borrado" / "Keep deleted" · "Mantener la versión modificada" / "Keep the modified version" |
| Renombrado | "Los dos lados renombraron {file}." / "Both sides renamed {file}." · "Quedarse con {name}" / "Keep {name}"                                                                                             |
| Externa    | "Abrir en herramienta de merge" / "Open in merge tool"                                                                                                                                                 |

- **Variantes.** Claro y oscuro. Densidad compacta de 28 px (editor de merge de VS Code).

### 12. D-40 Grafo paginado y virtualizado

- **Proposito.** Un repo real tiene cientos de miles de commits.
- **Muestra.** Filas virtualizadas. Al acercarse al final, se pide la siguiente pagina (el protocolo
  sirve de 1 a 1000 filas con la huella `snapshot`): filas esqueleto con nodos grises y el texto
  "Cargando más commits…". Respaldo: boton "Cargar más". Al llegar al commit raiz, una fila final con
  el total. Las aristas hacia padres que aun no estan cargados bajan hasta el borde inferior (B-16).
  Si las refs cambian entre paginas (`stale_window`), toast `info` y vuelta a la fila 0.
- **Estados.** Primera carga (esqueleto de 12 filas), cargando mas, fin, error de pagina con
  "Reintentar".

| Uso      | Texto                                                                               |
| -------- | ----------------------------------------------------------------------------------- |
| Cargando | "Cargando más commits…" / "Loading more commits…" · "Cargar más" / "Load more"      |
| Fin      | "Commit inicial · {count} commits" / "Initial commit · {count} commits"             |
| Stale    | "El historial cambió. Volvemos al principio." / "History changed. Back to the top." |

- **Variantes.** Claro y oscuro; densidad de 40 px y compacta de 28 px.

### 13. D-41 Columna dinamica, ramas ocultas, filtros, busqueda y tooltips

- **Proposito.** La columna es fija de 130 px y la busqueda solo mira las filas cargadas.
- **Columna.** Ancho = 16 + 20 por carril, hasta 10 carriles; a partir de ahi, separacion de 12 px y
  divisor arrastrable. Color = carril modulo 6 (B-14). Las aristas de merge no cruzan nodos (B-15).
- **Ocultar o aislar ramas.** En la barra lateral, al pasar por una rama: ojo para "Ocultar" y menu
  con "Mostrar solo esta rama". Sobre el grafo, chip "{count} ramas ocultas · Mostrar todas".
- **Filtros.** Fila de botones de filtro como los de Inicio sobre el grafo: "Autor" (multiple, con
  avatar) y "Fecha" (Desde, Hasta y atajos "Última semana", "Último mes"), con "Limpiar filtros".
- **Busqueda en todo el historial.** El buscador de la barra busca en el motor (mensaje, autor y
  SHA), no solo en lo cargado: contador de resultados, flechas ↑↓ que saltan y cargan la fila, e
  interruptor "Solo coincidencias". Sustituye al atenuado del prototipo.
- **Tooltips.** Nodo: sha, autor, fecha, mensaje (3 lineas) y refs. Pildora de ref: nombre, tipo,
  ↑↓ respecto a su upstream y ultimo commit. Pildora "+n": la lista completa.

| Uso      | Texto                                                                                                                                                                         |
| -------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Ramas    | "Ocultar" / "Hide" · "Mostrar solo esta rama" / "Show only this branch" · "{count} ramas ocultas" / "{count} hidden branches" · "Mostrar todas" / "Show all"                  |
| Filtros  | "Autor" / "Author" · "Fecha" / "Date" · "Desde" / "From" · "Hasta" / "To" · "Última semana" / "Last week" · "Último mes" / "Last month" · "Limpiar filtros" / "Clear filters" |
| Busqueda | "{current} de {total} coincidencias" / "{current} of {total} matches" · "Solo coincidencias" / "Matches only" · "Sin coincidencias en el historial" / "No matches in history" |
| Tooltip  | "↑{ahead} ↓{behind} respecto a {upstream}" / "↑{ahead} ↓{behind} vs {upstream}" · "Último commit {ago}" / "Last commit {ago}"                                                 |

- **Variantes.** Claro y oscuro; 3, 8 y 14 carriles; densidad de 40 px y compacta de 28 px.

## Reglas

- **Cada opcion de un formulario tiene efecto o no se muestra** (B-04). Repaso de los formularios
  que tocan estas vistas y las de #64 y #65: Merge (estrategia y editar mensaje), Rebase (autostash e
  interactivo), Cherry-pick (`-x` y sin commit), Crear tag (push del tag), Stash (sin seguimiento,
  solo seleccionados, mantener lo preparado), Eliminar rama (forzar), Quitar repo (mover a la
  papelera), Clonar (submodulos) y Crear PR (borrador, revisores, asignados, etiquetas solo si la
  plataforma los admite). Lo que no se pueda cumplir, se quita del diseno.
- Bugs que no se copian: B-03 (todo sale del motor: diff por archivo, archivos de commit reales,
  enlaces de PR reales), B-11 (destino del PR = rama por defecto de la plataforma), B-13 a B-16
  (carriles, color modulo, aristas que no cruzan nodos y que siguen hasta el borde), B-17 (unificado
  como git), B-26 (comparar actua sobre la rama del menu), B-29 (descartar, quitar remoto y eliminar stash confirman con el modal de peligro de #71),
  B-32 (contadores de la barra lateral cuentan lo filtrado), B-42 (plurales).
- **Nada se carga entero**: historial, blame, PRs y diff enormes paginan o virtualizan, con los
  esqueletos de C-01.
- Los PRs, checks y revisiones los pide el motor a la plataforma con la cuenta del repo (ADR 0018):
  sin cuenta para ese host, la vista lo dice y ofrece conectar una.
- Markdown de descripciones y comentarios: saneado, sin HTML.
- Colores solo de los tokens del handoff, tambien para la sintaxis y los cambios por palabra: ningun
  color nuevo. Textos al diccionario `D`. Toasts de informacion en `--info` (B-61).

## Entregable

- Zip `git-rudder-handoff-66-vistas.zip` con la estructura del original: `README.md` (pantallas nuevas y
  la correspondencia de sintaxis con los tokens), `Git Rudder Vistas.dc.html`,
  `git-rudder-data.js` (original mas claves y datos de ejemplo: varios diffs distintos, un historial de
  archivo, un PR con checks y revisiones) y `support.js`.
- `Git Rudder Vistas.dc.html` reutiliza la vista de repositorio de `Git Rudder.dc.html`. Si D-42 (#67) ya
  existe, usa su barra de titulo.
- Props: `theme`, `lang`, `density` (`desktop`, `vscode`; aplica a D-31, D-38, D-40 y D-41) y
  `screen`: `d27-historial`, `d28-blame`, `d29-comparar`, `d30-pr-abierto`, `d30-pr-conflictos`,
  `d30-pr-fusionado`, `d31-nuevo-pr`, `d31-sin-publicar`, `d31-exito`, `d32-remotos`, `d33-stash`,
  `d35-lineas`, `d36-bloques`, `d36-renombrado`, `d36-binario`, `d36-imagen`, `d36-enorme`, `d36-lfs`,
  `d36-submodulo`, `d37-staging`, `d37-grafo`, `d38-binario`, `d38-borrado`, `d40-paginado`,
  `d41-filtros`, `d41-busqueda`.
- Juntos en el lienzo: historial, blame y comparar (D-27 a D-29); PRs (D-30 y D-31); barra lateral
  (D-32 y D-33); diff y staging (D-35 a D-37); conflictos (D-38); grafo (D-40 y D-41).

## Hecho cuando

- [ ] D-27 a D-33 y D-35 a D-38 diseñados
- [ ] D-40 y D-41 diseñados: grafo paginado, filtros y columna dinámica
- [ ] Cada opción de un formulario tiene efecto o no se muestra
