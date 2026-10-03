# Brief #71: revisar el diseno contra los bugs del prototipo (B-01 a B-62)

Issue: [eabl0306/git-rudder#71](https://github.com/eabl0306/git-rudder/issues/71). Fuente: `docs/design/gaps.md`,
seccion 3.

## Contexto

- Productos: los tres prototipos base, `Git Rudder.dc.html` (Desktop), `Git Rudder VS Code.dc.html` y
  `Git Rudder Web.dc.html`, mas `git-rudder-data.js`. Quien implementa mira el prototipo: si un bug sigue dibujado,
  se copia.
- Este brief no anade pantallas nuevas. Corrige en sitio lo que esta mal dibujado y deja escrito, para
  cada bug, si se corrigio en el diseno, si es una regla de implementacion (no hay nada que dibujar) o si
  lo resuelve otro brief.
- Se reutiliza todo lo existente: el **modal generico** con `danger: true` (boton `--danger`, texto
  blanco) para las confirmaciones, el **menu contextual**, el **toast**, la **tarjeta de cuenta**, la
  **fila de llave**, el **resolvedor** (paneles A y B, Resultado, panel de archivos), el **editor de
  rebase** y la **tabla de repos** de la web. Estados de error, vacio y validacion: los del brief #62.
- Conviene hacerlo justo despues de #62, para que el resto de briefs parta de prototipos corregidos.

## Pantallas y estados

Leyenda de la columna "Tratamiento": **Diseno** (se redibuja en el prototipo), **Regla** (solo
implementacion: va a la tabla del README del handoff y no se dibuja), **#6x** (lo dibuja ese brief; aqui
solo se comprueba que el prototipo base no lo contradiga). Todo cambio se dibuja en claro y oscuro, en ES
y EN, y en Desktop y VS Code cuando el bug esta en los dos.

### 1. Estado y datos (B-01 a B-06)

| Id   | Donde            | Tratamiento    | Que cambia                                                                                                                                                                                                 |
| ---- | ---------------- | -------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| B-01 | Desktop          | Regla          | Una sesion por pestana: cambiar de pestana cambia grafo, staging, conflicto y stashes.                                                                                                                     |
| B-02 | Desktop, VS Code | Diseno + Regla | El prototipo arranca en Inicio (Desktop) y en un grafo sin conflicto (VS Code). El conflicto se alcanza con "Merge hotfix/login en actual" y no depende de un interruptor (`conflictUsed`). Lo decide git. |
| B-03 | Desktop, VS Code | Regla          | Diff, archivos de commit y enlaces de PR salen del motor.                                                                                                                                                  |
| B-04 | Desktop          | Regla          | Cada opcion de un formulario tiene efecto o no se muestra (merge, tag, "mover a la papelera"). Los formularios los revisa #66.                                                                             |
| B-05 | Desktop          | Diseno         | La insignia de plataforma de la tarjeta de repo sale del host del remoto, haya cuenta ligada o no. "⌂" solo sin remoto. Remoto sin cuenta: chip "Sin cuenta" con "Elegir cuenta" (D-15).                   |
| B-06 | Web              | Diseno         | La tabla de repos no ensena rutas locales: segunda linea con `propietario/repo`; repos sin remoto, "Solo local · {device}".                                                                                |

| Clave     | ES                    | EN                    |
| --------- | --------------------- | --------------------- |
| noAccChip | Sin cuenta            | No account            |
| chooseAcc | Elegir cuenta         | Choose account        |
| localOn   | Solo local · {device} | Local only · {device} |

### 2. Commit y mensajes (B-07 a B-12)

| Id   | Donde            | Tratamiento | Que cambia                                                                                                                                                                                                                                                                                  |
| ---- | ---------------- | ----------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| B-07 | VS Code          | Regla       | El commit usa el mensaje entero: resumen, linea en blanco y cuerpo.                                                                                                                                                                                                                         |
| B-08 | Desktop          | Regla       | Igual: la descripcion entra en el commit.                                                                                                                                                                                                                                                   |
| B-09 | Ambos            | Regla       | El contador de 72 cuenta grafemas.                                                                                                                                                                                                                                                          |
| B-10 | Desktop, VS Code | Diseno      | El resumen de IA del detalle de commit es bajo demanda: boton "Resumir con IA" (con el glifo de IA); estados generando (esqueleto de tres lineas), hecho (pie "Generado con {provider} · {model}"), error con "Reintentar", e IA desactivada o sin llave (texto con enlace a Ajustes > IA). |
| B-11 | Desktop, VS Code | Regla (#66) | Destino del PR = rama por defecto en la plataforma; origen elegido por el usuario.                                                                                                                                                                                                          |
| B-12 | Desktop          | Regla       | `-x` anade la linea `(cherry picked from commit <sha completo>)`.                                                                                                                                                                                                                           |

| Clave       | ES                                | EN                                  |
| ----------- | --------------------------------- | ----------------------------------- |
| summarize   | Resumir con IA                    | Summarize with AI                   |
| generatedBy | Generado con {provider} · {model} | Generated with {provider} · {model} |
| summaryFail | No se pudo generar el resumen.    | Couldn't generate the summary.      |

### 3. Grafo y diff (B-13 a B-18)

| Id   | Donde              | Tratamiento    | Que cambia                                                                                                                                                                          |
| ---- | ------------------ | -------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| B-13 | git-rudder-data.js | Regla          | Carriles calculados en el motor (`assignLanes`), no escritos en los datos.                                                                                                          |
| B-14 | Desktop, VS Code   | Diseno + Regla | Color = carril modulo 6. Los datos de muestra anaden un septimo carril, que repite el color del primero.                                                                            |
| B-15 | Desktop, VS Code   | Diseno         | Aristas de merge enrutadas por el carril donde vivia la reserva: curva, vertical, curva. Ninguna cruza un nodo.                                                                     |
| B-16 | Desktop, VS Code   | Diseno         | Una arista a un padre fuera de las filas cargadas baja hasta el borde inferior, como una rama que sigue.                                                                            |
| B-17 | Desktop            | Diseno         | Vista unificada por hunks: primero el bloque de lineas quitadas y despues el de anadidas, nunca intercaladas.                                                                       |
| B-18 | VS Code            | Diseno (#68)   | Titulos de pestana de diff como los fija #68 (V-12): "{file} (Índice ↔ Árbol de trabajo)", "{file} (HEAD ↔ Índice)" para lo preparado y "{file} ({sha})" para un archivo de commit. |

### 4. Resolvedor de conflictos (B-19 a B-23)

| Id   | Donde   | Tratamiento    | Que cambia                                                                                                                                                                                                            |
| ---- | ------- | -------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| B-19 | Ambos   | Diseno + Regla | Aviso `--warn` en la cabecera del Resultado si hay lineas repetidas. Git da los marcadores en `zdiff3`.                                                                                                               |
| B-20 | Ambos   | Diseno         | Cada bloque tiene decision explicita: sin decidir, lineas elegidas o vacio. "Quitar bloque" (que no hacia nada) pasa a "Dejar vacío"; vacio cuenta como resuelto y el Resultado ensena una linea fina "Bloque vacío". |
| B-21 | Ambos   | Diseno + Regla | Se detecta tambien el marcador base `\|\|\|\|\|\|\|`. Accion "Marcar como resuelto igualmente" en la fila del archivo y en la edicion manual, con confirmacion si quedan marcadores.                                  |
| B-22 | Desktop | Diseno         | La edicion manual se guarda como borrador por archivo. Tocar una eleccion con una edicion manual pide confirmar.                                                                                                      |
| B-23 | Ambos   | Diseno         | Las etiquetas de A y B dependen de la operacion (tabla de abajo). El subtitulo del Resultado pasa a uno generico.                                                                                                     |

| Operacion   | A (ES / EN)                                 | B (ES / EN)                                                    |
| ----------- | ------------------------------------------- | -------------------------------------------------------------- |
| Merge       | Tu rama ({branch}) / Your branch ({branch}) | Entrante ({source}) / Incoming ({source})                      |
| Pull        | Tu rama ({branch}) / Your branch ({branch}) | Remoto ({upstream}) / Remote ({upstream})                      |
| Rebase      | Base ({onto}) / Base ({onto})               | Commit que se reaplica ({sha}) / Commit being replayed ({sha}) |
| Cherry-pick | Tu rama ({branch}) / Your branch ({branch}) | Commit elegido ({sha}) / Picked commit ({sha})                 |
| Revert      | Tu rama ({branch}) / Your branch ({branch}) | Reversión de {sha} / Revert of {sha}                           |
| Stash       | Árbol de trabajo / Working tree             | Stash ({stash}) / Stash ({stash})                              |

| Clave        | ES                                                                                        | EN                                                                                      |
| ------------ | ----------------------------------------------------------------------------------------- | --------------------------------------------------------------------------------------- |
| dupWarn      | Hay líneas repetidas en el resultado. Revísalas antes de completar.                       | There are repeated lines in the result. Check them before completing.                   |
| leaveEmpty   | Dejar vacío                                                                               | Leave empty                                                                             |
| emptyBlock   | Bloque vacío                                                                              | Empty block                                                                             |
| outputSub    | así quedará {file} al continuar                                                           | how {file} will look when you continue                                                  |
| markResolved | Marcar como resuelto igualmente                                                           | Mark as resolved anyway                                                                 |
| markResTitle | ¿Marcar {file} como resuelto?                                                             | Mark {file} as resolved?                                                                |
| markResBody  | Aún hay líneas que parecen marcadores de conflicto. Si son contenido real, puedes seguir. | Some lines still look like conflict markers. If they're real content, you can continue. |
| markResBtn   | Marcar como resuelto                                                                      | Mark as resolved                                                                        |
| discManTitle | ¿Descartar tu edición manual?                                                             | Discard your manual edit?                                                               |
| discManBody  | Elegir líneas de A o B sustituye lo que escribiste a mano en {file}.                      | Choosing lines from A or B replaces what you typed by hand in {file}.                   |
| discManBtn   | Descartar edición                                                                         | Discard edit                                                                            |

### 5. Rebase interactivo (B-24 y B-25)

| Id   | Donde   | Tratamiento    | Que cambia                                                                                                                                                                                               |
| ---- | ------- | -------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| B-24 | Desktop | Regla          | La lista es `merge-base(onto, HEAD)..HEAD`, sin merges ni parches ya en upstream.                                                                                                                        |
| B-25 | Desktop | Diseno + Regla | Validacion en linea en el editor (patron C-03): el primer commit no puede ser squash ni fixup; reword necesita mensaje. "Iniciar rebase" deshabilitado con el error a la vista. Git hace el rebase real. |

| Clave       | ES                                                                                            | EN                                                                                    |
| ----------- | --------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------- |
| firstSquash | El primer commit no puede ser squash ni fixup: no tiene un commit anterior con el que unirse. | The first commit can't be squash or fixup: there's no earlier commit to combine with. |
| rewordEmpty | Escribe el mensaje nuevo.                                                                     | Enter the new message.                                                                |

### 6. Acciones de Git (B-26 a B-33)

| Id   | Donde            | Tratamiento    | Que cambia                                                                                                                       |
| ---- | ---------------- | -------------- | -------------------------------------------------------------------------------------------------------------------------------- |
| B-26 | Desktop, VS Code | Diseno + Regla | El menu de una rama nombra la rama sobre la que actua y actua sobre ella.                                                        |
| B-27 | Desktop, VS Code | Diseno         | Dos elementos: "Eliminar rama local..." y "Eliminar {remote}/{branch} del remoto...", cada uno con su confirmacion (seccion 10). |
| B-28 | VS Code          | #68            | La pildora de tag abre el menu de tag (contribucion nativa).                                                                     |
| B-29 | Desktop, VS Code | Diseno         | Confirmacion de peligro en toda accion destructiva (seccion 10).                                                                 |
| B-30 | Desktop          | Regla          | El nombre al clonar sale de analizar la URL (https, ssh, scp, `owner/repo`, Azure).                                              |
| B-31 | Desktop          | Diseno         | Error en linea en "Crear rama" (texto `badBranch` del brief #62).                                                                |
| B-32 | Desktop          | Regla          | El contador de Remotos cuenta lo filtrado.                                                                                       |
| B-33 | VS Code          | Regla (#68)    | Barra de estado con ahead, behind y cambios reales.                                                                              |

| Clave        | ES                                     | EN                                    |
| ------------ | -------------------------------------- | ------------------------------------- |
| pushOf       | Push de {branch}                       | Push {branch}                         |
| pullOf       | Pull de {branch}                       | Pull {branch}                         |
| compareOf    | Comparar {branch} con…                 | Compare {branch} with…                |
| deleteLocal  | Eliminar rama local…                   | Delete local branch…                  |
| deleteRemote | Eliminar {remote}/{branch} del remoto… | Delete {remote}/{branch} from remote… |

### 7. Cuentas y llaves (B-34 a B-40)

| Id   | Donde            | Tratamiento  | Que cambia                                                                                                                                                                                         |
| ---- | ---------------- | ------------ | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| B-34 | Desktop, VS Code | Diseno       | Mascaras de muestra iguales en los tres productos: prefijo conocido mas los 4 ultimos (`sk-ant-…7f3a`, `sk-proj-…91c0`); menos de 12 caracteres, `••••••••`.                                       |
| B-35 | Desktop          | Regla (#65)  | Editar una cuenta conserva su metodo y solo cambia el token si se escribe uno.                                                                                                                     |
| B-36 | Desktop, VS Code | Regla        | Al quitar la predeterminada se promueve la primera que quede (toast `info` `defT`, que ya existe).                                                                                                 |
| B-37 | Web, Desktop     | Diseno       | Campo de secreto oculto, con mostrar u ocultar solo mientras se escribe. Al editar, vacio con la ayuda de abajo.                                                                                   |
| B-38 | Los tres         | Diseno       | Placeholder del host segun la plataforma (tabla de abajo), de una sola tabla compartida.                                                                                                           |
| B-39 | Desktop          | Diseno       | El punto de la tarjeta de cuenta refleja el estado real (tabla de abajo). Los estados de boveda son del brief #63.                                                                                 |
| B-40 | VS Code          | Diseno (#68) | Barra de estado con la identidad de Git Rudder, no el usuario de GitHub. Textos y estados: los de #68 (`$(cloud) Git Rudder` con la identidad en el tooltip, "Git Rudder: sin sesión" sin sesion). |

| Plataforma        | Placeholder del host (ES / EN)                                  |
| ----------------- | --------------------------------------------------------------- |
| GitHub            | github.com                                                      |
| GitHub Enterprise | github.tu-empresa.com / github.your-company.com                 |
| GitLab            | gitlab.com                                                      |
| Bitbucket         | bitbucket.org                                                   |
| Gitea o Forgejo   | git.tu-empresa.com / git.your-company.com                       |
| Azure DevOps      | dev.azure.com/tu-organizacion / dev.azure.com/your-organization |

| Clave            | ES                                          | EN                                  | Color      |
| ---------------- | ------------------------------------------- | ----------------------------------- | ---------- |
| secretSaved      | Guardado. Escribe uno nuevo para cambiarlo. | Saved. Type a new one to change it. |            |
| accChecked       | Comprobada {ago}                            | Checked {ago}                       | `--ok`     |
| accExpires_one   | Caduca en {count} día                       | Expires in {count} day              | `--warn`   |
| accExpires_other | Caduca en {count} días                      | Expires in {count} days             | `--warn`   |
| accExpired       | Token caducado · Actualizar credenciales    | Token expired · Update credentials  | `--danger` |
| accCheckFail     | No se pudo comprobar                        | Couldn't verify                     | `--danger` |
| accNotChecked    | Sin comprobar                               | Not checked yet                     | `--faint`  |

### 8. Textos e i18n (B-41 a B-50)

| Id   | Donde                   | Tratamiento    | Que cambia                                                                                                                                                                   |
| ---- | ----------------------- | -------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| B-41 | Desktop                 | Regla          | Una clave por texto (`copyName`, `copyModel`...); nada de editar otra traduccion.                                                                                            |
| B-42 | git-rudder-data.js      | Regla          | Plurales con claves `_one` y `_other`. Los textos nuevos de estos briefs ya vienen asi.                                                                                      |
| B-43 | git-rudder-data.js      | Regla          | Glifos y atajos fuera de las cadenas; el atajo se formatea segun el sistema.                                                                                                 |
| B-44 | git-rudder-data.js      | Regla          | Marcadores con nombre (`{count}`, `{branch}`, `{model}`).                                                                                                                    |
| B-45 | git-rudder-data.js, Web | Regla          | Espacios de nombres en `@git-rudder/i18n` en lugar de dos diccionarios.                                                                                                      |
| B-46 | Todos                   | Diseno         | Ningun texto dice "simulado" ni lleva datos fijos: `signedOutT`, `planRenew`, `W.dlStarted`, `W.version` pasan a los textos de abajo; `W.resetSent` lo sustituye W-01 (#69). |
| B-47 | VS Code                 | Diseno + Regla | La prop `vsLanguage` solo ofrece `en` y `es`; cualquier otro idioma cae a ingles. Falta la clave `commitV`.                                                                  |
| B-48 | Desktop, Web            | Regla          | Tiempo relativo con `Intl.RelativeTimeFormat`. Muestras del diseno: "ahora mismo", "hace 1 min", "hace 23 h", "ayer", "hace 2 semanas".                                      |
| B-49 | Desktop                 | Regla          | Zona horaria con todos los guiones bajos sustituidos.                                                                                                                        |
| B-50 | Web                     | Diseno         | La etiqueta del rol owner es "Titular" en Desktop y en la web (`r_owner`, `W.owner`).                                                                                        |

| Clave      | ES                                      | EN                               |
| ---------- | --------------------------------------- | -------------------------------- |
| signedOutT | Sesión cerrada                          | Signed out                       |
| planRenew  | Se renueva el {date} · {seats} asientos | Renews on {date} · {seats} seats |
| dlStarted  | Descarga iniciada: {file}               | Download started: {file}         |
| version    | v{version} · {date}                     | v{version} · {date}              |
| roleOwner  | Titular                                 | Owner                            |

### 9. Web y modales (B-51 a B-62)

| Id   | Donde        | Tratamiento    | Que cambia                                                                                                                                                                                      |
| ---- | ------------ | -------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| B-51 | Web          | #70            | W-15 y W-18. En el prototipo base, "Añadir asientos" deja de sumar uno en silencio.                                                                                                             |
| B-52 | Web, Desktop | Diseno         | Contadores separados de miembros, invitaciones pendientes y asientos (las invitaciones ocupan asiento).                                                                                         |
| B-53 | Web          | Diseno         | Al iniciar sesion no hay pista ni error de 8 caracteres; solo "Escribe tu contraseña." si esta vacia. El minimo sigue en el registro.                                                           |
| B-54 | Web          | #69            | W-04: el nombre de organizacion del registro se guarda y OAuth pasa por los terminos.                                                                                                           |
| B-55 | Web          | #70            | W-30: movil y sistema desconocido sin tarjeta destacada.                                                                                                                                        |
| B-56 | Web          | Diseno (#70)   | Cerrar sesion en otro dispositivo confirma (seccion 10). Datos reales y marca "Este navegador": W-22 (#70).                                                                                     |
| B-57 | Web          | Diseno         | "Configuracion en la nube" sin "Guardar" global: guardado inmediato por control con el estado de C-06.                                                                                          |
| B-58 | Web          | Diseno + Regla | "Descargar" del panel abre las descargas dentro del panel, sin salir. Con sesion, login y registro redirigen al panel y la landing ofrece "Abrir panel" (ya existe).                            |
| B-59 | Web          | Diseno         | La confirmacion de borrar workspace usa su propio texto; el buscador de repos busca tambien por usuario de la cuenta y lo dice; toasts de exito al crear workspace, renombrar y cambiar un rol. |
| B-60 | Todos        | #62            | Errores en linea (C-03) y saltos de linea respetados en las cajas informativas.                                                                                                                 |
| B-61 | Desktop      | #62            | Toast `info` en `--info`. Comprobar que `Git Rudder.dc.html` ya lo tiene.                                                                                                                       |
| B-62 | VS Code      | #68            | Atajo del grafo sin choques con VS Code.                                                                                                                                                        |

| Clave            | ES                                                    | EN                                                |
| ---------------- | ----------------------------------------------------- | ------------------------------------------------- |
| members_one      | {count} miembro                                       | {count} member                                    |
| members_other    | {count} miembros                                      | {count} members                                   |
| pendingInv_one   | {count} invitación pendiente                          | {count} pending invitation                        |
| pendingInv_other | {count} invitaciones pendientes                       | {count} pending invitations                       |
| seatsUsed        | {used} de {seats} asientos                            | {used} of {seats} seats                           |
| pwRequired       | Escribe tu contraseña.                                | Enter your password.                              |
| searchRepos      | Buscar repositorio, usuario de la cuenta o workspace… | Search repository, account username or workspace… |
| wsCreated        | Workspace {name} creado                               | Workspace {name} created                          |
| roleChanged      | {name} ahora es {role}                                | {name} is now {role}                              |
| renamed          | Nombre cambiado                                       | Name changed                                      |

Ejemplo de la linea de B-52: "4 miembros y 1 invitación pendiente · 5 de 5 asientos". Se compone en el
componente con las tres claves, sin trocear traducciones.

### 10. Confirmaciones de peligro (B-29, B-27, B-22, B-56 y el resto)

Patron unico: modal generico con `danger: true`; titulo en pregunta; caja info con la consecuencia;
boton de accion en `--danger` con texto blanco; foco inicial en "Cancelar"; Enter no confirma. En VS Code,
aviso modal nativo (`showWarningMessage` con `modal: true`) con el mismo texto y el boton de accion. Las
filas con "ya existe" solo se comprueban; las demas se dibujan.

| Accion                                      | Donde (estado hoy)                                | Titulo ES / EN                                                                      | Texto ES / EN                                                                                                                                                                                                                                                                                                        | Boton ES / EN                            |
| ------------------------------------------- | ------------------------------------------------- | ----------------------------------------------------------------------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ---------------------------------------- |
| Reset --hard                                | Desktop (ya existe), VS Code (falta)              | ¿Reset --hard a {sha}? / Reset --hard to {sha}?                                     | Se perderán los cambios sin commit y los commits posteriores en la rama actual. / Uncommitted changes and later commits on the current branch will be lost.                                                                                                                                                          | Reset --hard                             |
| Descartar cambios                           | Los dos (ya existe)                               | ¿Descartar cambios? / Discard changes?                                              | {file}. Esta acción no se puede deshacer. / {file}. This can't be undone.                                                                                                                                                                                                                                            | Descartar / Discard                      |
| Eliminar stash                              | Desktop (ya existe), VS Code (falta)              | ¿Eliminar {stash}? / Drop {stash}?                                                  | {message}. Esta acción no se puede deshacer. / {message}. This can't be undone.                                                                                                                                                                                                                                      | Eliminar / Drop                          |
| Eliminar tag                                | Los dos (falta)                                   | ¿Eliminar el tag {tag}? / Delete tag {tag}?                                         | Se borra en este repositorio. Si ya lo publicaste, sigue en el remoto. / It's deleted in this repository. If you pushed it, it stays on the remote.                                                                                                                                                                  | Eliminar / Delete                        |
| Eliminar rama local                         | Los dos (ya existe)                               | ¿Eliminar {branch}? / Delete {branch}?                                              | Puede tener commits que no están en otra rama. / It may have commits that aren't on another branch.                                                                                                                                                                                                                  | Eliminar / Delete                        |
| Eliminar rama remota (B-27)                 | Los dos (falta)                                   | ¿Eliminar {remote}/{branch} del remoto? / Delete {remote}/{branch} from the remote? | Se borra en {remote} para todo el equipo. Tu rama local no se toca. / It's deleted on {remote} for the whole team. Your local branch isn't touched.                                                                                                                                                                  | Eliminar del remoto / Delete from remote |
| Abortar merge, rebase, cherry-pick o revert | Los dos (falta); solo si hay trabajo que perder   | ¿Abortar el {operation}? / Abort the {operation}?                                   | Se descartan las resoluciones que hiciste y el repositorio vuelve a como estaba antes. / The resolutions you made are discarded and the repository goes back to how it was before.                                                                                                                                   | Abortar / Abort                          |
| Descartar edicion manual (B-22)             | Desktop (falta)                                   | ver seccion 4                                                                       | ver seccion 4                                                                                                                                                                                                                                                                                                        | ver seccion 4                            |
| Quitar repo de la lista                     | Desktop (ya existe)                               | ¿Quitar {name} de la lista? / Remove {name} from the list?                          | El repositorio seguirá en tu disco y en el remoto. / The repository stays on your disk and remote.                                                                                                                                                                                                                   | Quitar / Remove                          |
| Eliminar workspace                          | Desktop (ya existe), Web (texto equivocado, B-59) | ¿Eliminar el workspace {name}? / Delete workspace {name}?                           | Los repositorios no se eliminan; quedan sin workspace. Si está compartido, desaparece también para {org}. / Repositories are kept, without a workspace. If it's shared, it also disappears for {org}.                                                                                                                | Eliminar / Delete                        |
| Desconectar cuenta                          | Desktop (ya existe, texto nuevo), VS Code (falta) | ¿Desconectar @{user} de {platform}? / Disconnect @{user} from {platform}?           | Git Rudder borra el token de este equipo y, si la cuenta se sincroniza, de la nube. El token sigue siendo válido en {platform} hasta que lo revoques allí. / Git Rudder deletes the token from this device and, if the account syncs, from the cloud. The token stays valid on {platform} until you revoke it there. | Desconectar / Disconnect                 |
| Eliminar llave de IA                        | Desktop (ya existe, texto nuevo), VS Code (falta) | ¿Eliminar la llave {label}? / Delete key {label}?                                   | {provider} · {mask}. Las funciones que la usan pasan a la llave predeterminada. La llave sigue activa en {provider} hasta que la revoques allí. / {provider} · {mask}. Features using it switch to the default key. The key stays active at {provider} until you revoke it there.                                    | Eliminar / Delete                        |
| Quitar miembro                              | Desktop y Web (ya existe)                         | ¿Quitar a {name} de {org}? / Remove {name} from {org}?                              | {email}. Pierde el acceso a los workspaces compartidos y a las llaves de la organización. / {email}. They lose access to shared workspaces and organization keys.                                                                                                                                                    | Quitar / Remove                          |
| Cancelar invitacion                         | Web (hoy es "Quitar")                             | ¿Cancelar la invitación a {email}? / Cancel the invitation to {email}?              | El enlace del correo dejará de funcionar y el asiento queda libre. / The email link will stop working and the seat is freed.                                                                                                                                                                                         | Cancelar invitación / Cancel invitation  |
| Cerrar sesion en otro dispositivo           | Desktop y Web (falta)                             | ¿Cerrar la sesión de {device}? / Sign out {device}?                                 | Texto de X-03 del brief #63 (no promete borrado remoto). / X-03 text from brief #63 (no remote wipe promised).                                                                                                                                                                                                       | Cerrar sesión / Sign out                 |
| Quitar repo de la cuenta                    | Web (ya existe, texto nuevo)                      | ¿Quitar {name} de tu cuenta? / Remove {name} from your account?                     | Desaparece de tu cuenta y de tus workspaces en todos los equipos. Los clones en disco no se tocan. / It disappears from your account and workspaces on every device. Clones on disk aren't touched.                                                                                                                  | Quitar / Remove                          |
| Cerrar sesion en este equipo                | Desktop y VS Code                                 | K-09, brief #63                                                                     | K-09, brief #63                                                                                                                                                                                                                                                                                                      | K-09, brief #63                          |
| Desactivar sincronizacion de secretos       | Web y Desktop                                     | X-04, brief #63                                                                     | X-04, brief #63                                                                                                                                                                                                                                                                                                      | X-04, brief #63                          |
| Restablecer la boveda                       | Web                                               | K-10, brief #63                                                                     | K-10, brief #63                                                                                                                                                                                                                                                                                                      | K-10, brief #63                          |
| Eliminar cuenta de Git Rudder               | Web                                               | W-24, brief #70                                                                     | W-24, brief #70                                                                                                                                                                                                                                                                                                      | W-24, brief #70                          |

Nota: `{operation}` en "Abortar" no se compone en la traduccion: son cuatro claves (merge, rebase,
cherry-pick, revert) con el mismo patron.

### 11. Ademas: datos de muestra que se copiarian mal

No son filas B, pero salen de decisiones y estan dibujados en los prototipos base:

- Sesiones activas de Desktop sin ciudad (decision #53): "{device} · {app} · {ago}" y la marca "Este
  dispositivo". Lo mismo en los dispositivos de la web.
- Los correos de muestra van con `example.com` (`marta@example.com`) y cualquier URL del producto
  con `git-rudder.com`.
- Bitbucket "App password" pasa a "API token" en los datos de muestra (X-08).
- Los modelos por defecto fijos de `git-rudder-data.js` y el "Compartir con el equipo" de las cuentas web los
  quitan #67 (X-10) y #70 (X-01); aqui no se tocan para no pisarse.

## Reglas

- "Propietaria" pasa a "Titular" en todos los sitios (B-50).
- Toda accion destructiva confirma con el patron de la seccion 10. Generar o cambiar algo que no borra
  datos no usa el estilo de peligro.
- Toasts informativos en `--info` (B-61); ningun texto con "simulado" (B-46).
- No se anade ninguna pantalla nueva aqui: lo que falta lo dibujan los briefs #62 a #70.
- No se copian al prototipo los bugs que quedan como regla: la tabla del README los lista para quien
  implementa.
- Textos nuevos en `git-rudder-data.js` con claves `_one` y `_other` y marcadores con nombre (ADR 0017).

## Entregable

- Un zip `git-rudder-handoff-71-correcciones.zip` con la estructura del handoff original y los mismos nombres
  de fichero, para que sustituya a los base:
  - `Git Rudder.dc.html`, `Git Rudder VS Code.dc.html` y `Git Rudder Web.dc.html` con las correcciones "Diseno".
  - `git-rudder-data.js` con los textos nuevos y corregidos.
  - `support.js` sin cambios.
  - `README.md` del handoff con una seccion nueva "Bugs del prototipo": tabla de 62 filas con `Id`,
    `Estado` ("Corregido en el diseño", "Regla de implementación" o "En el brief #6x") y una nota corta.
    Las filas son las de este brief.
- Para revisarlo rapido, una lamina por grupo (secciones 1 a 11) en un `Git Rudder Correcciones.dc.html`
  con el antes y el despues lado a lado.

## Hecho cuando

- [ ] Cada bug marcado como corregido en el diseño o como regla de implementación
- [ ] "Propietaria" pasa a una etiqueta neutra
- [ ] Toda acción destructiva tiene confirmación de peligro
