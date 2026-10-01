# Huecos del diseno

- Fecha: 2026-09-30
- Revisado: 2026-10-01, con las decisiones que Ezequiel tomo ese dia: la
  [seccion 4](#4-decisiones-abiertas-para-ezequiel) las resume y enlaza su ADR, y las filas que
  cambiaban estan corregidas
- Base: el handoff en [`handoff/`](handoff/README.md), es decir el README y los tres prototipos
  [`Ramal.dc.html`](handoff/Ramal.dc.html) (Desktop), [`Ramal VS Code.dc.html`](handoff/Ramal%20VS%20Code.dc.html)
  y [`Ramal Web.dc.html`](handoff/Ramal%20Web.dc.html), mas los datos de `ramal-data.js`. Donde el README y un
  prototipo no coinciden, manda el prototipo.

El handoff dibuja muy bien el camino feliz: el grafo, el resolvedor de conflictos, el rebase interactivo,
los ajustes y el panel web estan casi completos. Lo que le falta es todo lo que rodea a ese camino: que
pasa mientras una operacion tarda, cuando falla, cuando no hay sesion, cuando no hay repositorio, cuando
git ni siquiera esta instalado. Y hay un segundo grupo de huecos que no son olvidos del diseno sino
consecuencia de decisiones de arquitectura tomadas despues: la boveda cifrada de extremo a extremo, el
motor git unico en Rust, el piso de Linux de Tauri 2.

Este documento sirve para dos cosas:

1. **Backlog.** Cada fila tiene un identificador estable para convertirla en ticket sin reescribirla.
2. **Lista de cambios para quien disena.** La seccion 2 entera y las filas de la seccion 1 son trabajo de
   diseno; la seccion 3 es lo que no hay que copiar del prototipo al implementar.

Es un documento tecnico y entra en detalle a proposito. El inventario completo de cada producto, con cada
accion clasificada, vive en los analisis previos; aqui solo queda lo que falta o esta mal.

## Como leerlo

Prefijos de los identificadores:

| Prefijo | Que es                                                  |
| ------- | ------------------------------------------------------- |
| C       | Estado o vista comun a los productos, se disena una vez |
| K       | Boveda E2E (frase de sincronizacion y recuperacion)     |
| D       | Ramal Desktop                                           |
| V       | Ramal for VS Code                                       |
| W       | Ramal Web (landing, autenticacion y panel)              |
| X       | Cambio de diseno obligado por una decision              |
| B       | Bug del prototipo que no hay que copiar                 |
| Q       | Decision abierta para Ezequiel                          |

Prioridades:

| Prioridad | Significado                                                                        |
| --------- | ---------------------------------------------------------------------------------- |
| P0        | Sin esto no hay una primera version usable, o el producto promete algo que no hace |
| P1        | Necesario para la primera version publica, pero no bloquea empezar a construir     |
| P2        | Despues de la primera version publica                                              |

Dos supuestos que condicionan las prioridades:

- **Equipos, IA de organizacion y facturacion entran en la primera version publica**
  ([Q-01](#4-decisiones-abiertas-para-ezequiel), resuelta el 2026-10-01 por
  [ADR 0026](../adr/0026-planes-y-precios.md)). Las filas de equipo y facturacion (W-11 a W-18) y
  D-47 son del MVP con la prioridad que tienen.
- **La web no edita valores de ajustes sincronizados** (preferencias de IA, de Git, apariencia). Solo decide
  que se sincroniza, como en el prototipo. Los valores se editan en Desktop o VS Code.

---

## 1. Vistas y estados que faltan

### 1.1 Comunes (se disenan una vez en `@ramal/ui`)

El prototipo no tiene ni un estado de carga, error o sin conexion. El unico estado asincrono es
"Generando..." de la IA, que dura 800 ms fijos. El tipo de toast `err` existe y nunca se emite. Esto se
disena una vez y se reutiliza en los tres productos.

| Id   | Vista o estado                                                                                                 | Prioridad | Por que                                                                                                        |
| ---- | -------------------------------------------------------------------------------------------------------------- | --------- | -------------------------------------------------------------------------------------------------------------- |
| C-01 | Esqueletos de carga para listas y paneles (repos, PRs, miembros, cuentas, llaves, sesiones, detalle de commit) | P0        | Todo dato real llega con retraso; sin esqueleto la UI parece rota o vacia.                                     |
| C-02 | Toast de error y dialogo de error con detalle y "Copiar detalle"                                               | P0        | Los errores de git traen salida larga (hooks, rechazos); un toast de 3 s no la puede mostrar.                  |
| C-03 | Errores de validacion en linea dentro de los modales                                                           | P0        | Hoy los modales fallan en silencio: `onSubmit` devuelve `false` y el modal se queda abierto sin decir por que. |
| C-04 | Estados vacios: primer arranque, sin stashes, tags, remotos, PRs, cuentas, llaves, resultados de busqueda      | P0        | Es lo primero que ve un usuario nuevo y hoy solo algunos estan dibujados.                                      |
| C-05 | Sin conexion o API de Ramal caida                                                                              | P0        | Git local tiene que seguir funcionando sin red (decision 4); el estado debe decir que se degrada solo la nube. |
| C-06 | Estado de sincronizacion ("sincronizado hace N", error, cambiado en otro dispositivo)                          | P1        | Los ajustes sincronizados usan concurrencia optimista; un 412 necesita una salida visible, no un fallo mudo.   |
| C-07 | Sesion caducada o revocada desde otro dispositivo: volver a iniciar sesion                                     | P1        | El refresh token puede rotar, caducar o revocarse desde el panel; hoy no hay forma de enterarse.               |
| C-08 | Permiso denegado y vistas de solo lectura por rol (miembro frente a admin)                                     | P1        | Todo el diseno asume que quien mira es propietario de la organizacion.                                         |
| C-09 | Tema "sistema" y deteccion de idioma del sistema en el primer arranque                                         | P1        | Solo existen claro y oscuro, y el idioma arranca fijo en espanol.                                              |
| C-10 | Foco de teclado visible y estados de foco en todos los controles                                               | P1        | El prototipo no dibuja foco; sin el, la app no se puede usar con teclado.                                      |

### 1.2 Boveda E2E (Desktop y VS Code; la web muestra el estado y es el unico sitio con reinicio)

La decision 2 hace que los secretos sincronizados (tokens de plataformas y, si se activa, llaves de IA
personales) viajen cifrados con una llave que el servidor nunca ve. Esa llave se protege con una **frase
de sincronizacion** distinta de la contrasena de inicio de sesion, y con un **codigo de recuperacion**.

La frase no puede ser la contrasena por dos motivos: la contrasena la recibe el servidor para verificarla,
asi que no protege nada frente al servidor; y quien entra con GitHub, GitLab o Google no tiene contrasena.
Ninguna de estas pantallas existe en el diseno.

| Id   | Vista o estado                                                                                                           | Prioridad | Por que                                                                                                                                                                                                                     |
| ---- | ------------------------------------------------------------------------------------------------------------------------ | --------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| K-01 | Crear la frase de sincronizacion, con explicacion de por que no es la contrasena                                         | P0        | Sin frase no hay boveda, y sin boveda no se sincroniza ninguna cuenta de plataforma.                                                                                                                                        |
| K-02 | Mostrar el codigo de recuperacion una sola vez: copiar, descargar y confirmar que se guardo                              | P0        | Es la unica salida si se olvida la frase; si el usuario no lo guarda, pierde los secretos.                                                                                                                                  |
| K-03 | Desbloquear en un dispositivo nuevo con la frase o con el codigo de recuperacion                                         | P0        | Cada equipo nuevo necesita la llave de la boveda antes de usar las cuentas sincronizadas.                                                                                                                                   |
| K-04 | Boveda bloqueada en este equipo: cuentas sincronizadas visibles como metadatos con "Desbloquear para usar"               | P0        | Los metadatos llegan sin frase, los tokens no; hay que explicar por que una cuenta visible no funciona.                                                                                                                     |
| K-05 | Cuenta "Solo en este equipo" (secreto que no se sincroniza)                                                              | P0        | Quien dice "Ahora no" a la boveda, o no tiene cuenta de Ramal, sigue pudiendo anadir cuentas.                                                                                                                               |
| K-06 | Cambiar la frase de sincronizacion                                                                                       | P1        | Solo reenvuelve la llave; los demas dispositivos siguen funcionando y conviene decirlo.                                                                                                                                     |
| K-07 | Regenerar el codigo de recuperacion (invalida el anterior)                                                               | P1        | Si el codigo se pierde o se filtra, tiene que poder sustituirse sin rehacer la boveda.                                                                                                                                      |
| K-08 | Olvide la frase y el codigo: en Desktop y VS Code, "Olvide la frase" explica la situacion y enlaza al reinicio de la web | P1        | El reinicio solo existe en la web (ver K-10); el texto debe decir que Ramal no puede recuperar los secretos.                                                                                                                |
| K-09 | Efecto de cerrar sesion sobre los secretos de este equipo                                                                | P1        | **Resuelta el 2026-10-01** ([ADR 0012](../adr/0012-boveda-de-extremo-a-extremo.md)): se borra lo que llego de otros equipos y lo creado aqui se queda, con la casilla "Borrar también las añadidas en este equipo" apagada. |
| K-10 | Web: estado de la boveda (configurada, desde cuando) y reinicio; es el unico sitio con reinicio                          | P1        | La web no puede descifrar nada, pero si puede mostrar si existe y borrarla, y es la salida de K-08.                                                                                                                         |

Comportamiento propuesto para estas pantallas, para que el disenador no tenga que adivinarlo:

- **La boveda se crea cuando hace falta, no al iniciar sesion.** Se pide la primera vez que un secreto se
  iba a sincronizar: al anadir una cuenta de plataforma con "Cuentas vinculadas" activado, o al activar la
  sincronizacion de llaves de IA. "Ahora no" guarda el secreto solo en este equipo (K-05). Asi quien no
  sincroniza secretos nunca ve una frase.
- **Crear y desbloquear la boveda ocurre solo en Desktop y VS Code.** La web no tiene secretos que cifrar
  (decision 2: los tokens se anaden en los clientes), asi que no necesita criptografia en el navegador.
- **Si la sincronizacion de secretos se enciende desde la web**, que no puede crear la boveda, el
  siguiente Desktop o VS Code que sincroniza ve la categoria activa sin boveda y propone K-01.
- **Olvidar la frase no siempre es perder los datos.** Si otro equipo sigue desbloqueado, desde el se
  cambia la frase (K-06) sin perder nada. Restablecer solo hace falta cuando no queda ningun equipo
  desbloqueado ni codigo de recuperacion, y se hace en la web (K-10). La pantalla de "olvide la frase"
  (K-08) debe ofrecer primero la salida de otro equipo y despues el enlace al reinicio web.
- **Cambiar la frase o regenerar el codigo no protege si alguien los conoce**: quien guardo la envoltura
  anterior la sigue abriendo. K-06 y K-07 preguntan si el usuario cree que alguien los conoce y, si dice
  que si, encadenan una rotacion de la llave, que pide la frase y ensena un codigo de recuperacion nuevo.
- **Revocar un dispositivo recomienda rotar la llave.** Hasta que alguien rote, todos los equipos
  ensenan "Rotar la llave de la boveda".
- **Restablecer la contrasena de inicio de sesion no toca la boveda.** El texto de la pagina de
  restablecer contrasena (W-02) debe decirlo, porque el usuario espera lo contrario.
- **Cerrar sesion borra de este equipo la llave de la boveda y las copias de los tokens y las llaves
  que llegaron de otros equipos; lo creado en este equipo se queda**, este sincronizado o no, porque
  sin cuenta Ramal sigue siendo un cliente Git que lo necesita. Es lo que decide ADR 0012 para K-09,
  ratificado el 2026-10-01. Para un ordenador compartido, el dialogo ofrece la casilla "Borrar
  también las añadidas en este equipo", apagada por defecto. Texto propuesto: "Al cerrar sesión se
  borran de este equipo la llave de tu bóveda y las cuentas y llaves que llegaron de otros equipos.
  Las que añadiste aquí se quedan para que Git siga funcionando." Lo mismo pasa cuando la sesion se
  revoca desde otro sitio y el equipo lo descubre al refrescar (un 401 con el `code`
  `session_ended`).

### 1.3 Ramal Desktop

#### Entrada, sesion y sincronizacion

| Id   | Vista o estado                                                                                                                                                                                       | Prioridad | Por que                                                                                                           |
| ---- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | --------- | ----------------------------------------------------------------------------------------------------------------- |
| D-01 | Primer arranque: bienvenida con "Continuar sin cuenta" e "Iniciar sesion", identidad de Git (prellenada desde `git config --global`), buscar repos en carpetas, conectar una plataforma, IA opcional | P0        | La app arranca hoy dentro de un conflicto; no existe primera vez, y Ramal debe funcionar sin cuenta (decision 4). |
| D-02 | Git no instalado o anterior a 2.34: bloqueo con instrucciones por sistema operativo y "Volver a comprobar"                                                                                           | P0        | El motor usa el git del sistema para todo (decision 1); sin git no hay producto.                                  |
| D-03 | Iniciar sesion desde el escritorio: abre el navegador (PKCE con redireccion loopback), pantalla de espera, cancelar, error y codigo de dispositivo como alternativa                                  | P0        | Es la unica forma de entrar desde el escritorio y no esta dibujada.                                               |
| D-04 | Estado sin sesion en toda la app: avatar de la barra de titulo, "Cuenta y equipo", sincronizacion                                                                                                    | P0        | Todo el diseno asume sesion iniciada; sin cuenta, esas zonas tienen que invitar a entrar, no romperse.            |
| D-05 | Sin repositorio abierto y lista de repos vacia en el primer arranque                                                                                                                                 | P0        | Primera pantalla de cualquier usuario nuevo.                                                                      |
| D-06 | Repos de la nube que no estan en este equipo: "No clonado en este equipo" y "Clonar"; "Abrir todos" de un workspace con repos que faltan                                                             | P1        | Los workspaces se sincronizan, los clones no; el segundo equipo del usuario los vera sin carpeta.                 |
| D-07 | Ruta desaparecida (repo movido o borrado en disco): "Localizar..." y "Quitar"                                                                                                                        | P1        | Pasa en cuanto el usuario reorganiza carpetas; hoy el repo quedaria roto sin explicacion.                         |

#### Ciclo de vida del repositorio

| Id   | Vista o estado                                                                                                                                                                                                  | Prioridad | Por que                                                                 |
| ---- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | --------- | ----------------------------------------------------------------------- |
| D-08 | Clonar con progreso: fases (contando, recibiendo, resolviendo), porcentaje, cancelar, errores (autenticacion, no existe, destino ocupado, red), autocompletar `owner/repo` desde la cuenta elegida, SSH o HTTPS | P0        | Clonar un repo grande tarda minutos; hoy es un toast inmediato.         |
| D-09 | Selector de carpeta nativo para Abrir, Iniciar y destino de Clonar; error "No es un repositorio" con "Iniciar aqui"                                                                                             | P0        | Las tres entradas al producto dependen de el.                           |
| D-10 | Repositorio vacio (rama sin nacer, sin commits) en el grafo y en el staging                                                                                                                                     | P1        | Es el estado de todo repo recien iniciado y el grafo no sabe dibujarlo. |
| D-11 | Publicar al iniciar: nombre remoto, propietario u organizacion, visibilidad, descripcion, `.gitignore`, licencia, README, progreso y errores                                                                    | P1        | El flujo "Iniciar" termina hoy sin remoto.                              |

#### Credenciales y cuentas de plataforma

| Id   | Vista o estado                                                                                                                                                                                                                | Prioridad | Por que                                                                                                                    |
| ---- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | --------- | -------------------------------------------------------------------------------------------------------------------------- |
| D-12 | Conectar cuenta con OAuth sin secreto de cliente: codigo de dispositivo o navegador con PKCE, espera, exito con usuario y permisos detectados, fallo, permisos insuficientes, autorizacion SAML/SSO de organizacion en GitHub | P0        | El modal de hoy solo pide usuario y token; la decision 2 prefiere OAuth publico para que el token no pase por el servidor. |
| D-13 | Peticiones de credenciales durante fetch, pull, push y clone cuando ninguna cuenta encaja: usuario y token HTTPS, frase de la llave SSH, confirmar la huella de un host desconocido                                           | P0        | El git del sistema las pide por `GIT_ASKPASS`/`SSH_ASKPASS`; si Ramal no las dibuja, la operacion se cuelga.               |
| D-14 | Token caducado o revocado: aviso y "Actualizar credenciales" desde el propio aviso                                                                                                                                            | P1        | Los tokens OAuth de GitHub pueden caducar (8 h, con refresh) y los de Bitbucket y Azure tienen fecha de fin.               |
| D-15 | Elegir la cuenta que usa cada repositorio (tarjeta del repo o ajustes del repo)                                                                                                                                               | P1        | "Varias cuentas por plataforma" es una promesa de la landing y solo se elige de forma implicita al clonar.                 |
| D-16 | Generar llave SSH (tipo, comentario, frase) y subirla a la plataforma; elegir llave GPG o SSH para firmar                                                                                                                     | P2        | "Generar llave SSH" y "Firmar commits" existen en ajustes sin pantalla detras.                                             |

#### Operaciones de Git

| Id   | Vista o estado                                                                                                                                                                                     | Prioridad | Por que                                                                                                     |
| ---- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | --------- | ----------------------------------------------------------------------------------------------------------- |
| D-17 | Progreso de operaciones largas (fetch, pull, push, rebase, checkout en repos grandes) con cancelar                                                                                                 | P0        | Hoy todas terminan al instante; en un repo real duran segundos o minutos.                                   |
| D-18 | Errores de operacion: push rechazado (ofrecer pull o force-with-lease), autenticacion, red, `index.lock` presente, hook fallido con su salida, arbol sucio que bloquea el checkout (ofrecer stash) | P0        | Son los errores diarios de cualquier cliente Git y ninguno esta dibujado.                                   |
| D-19 | Conflictos fuera del merge: pull (merge o rebase), rebase, cherry-pick, revert, aplicar stash; con continuar, omitir y abortar segun la operacion                                                  | P0        | Solo el merge tiene conflictos en el diseno; en la practica los de rebase y pull son igual de comunes.      |
| D-20 | Rebase en curso: parado en `edit` (modificar y continuar), progreso "3 de 7", banner con continuar, omitir y abortar                                                                               | P0        | El editor de rebase ofrece `edit`, pero no existe la pantalla de "rebase detenido".                         |
| D-21 | Editor del mensaje de `squash` durante el rebase (por defecto, los mensajes concatenados)                                                                                                          | P0        | git pide ese mensaje; sin editor, el squash del editor de rebase no se puede completar bien.                |
| D-22 | Editor del mensaje de merge y de squash merge                                                                                                                                                      | P1        | "Editar el mensaje antes de confirmar" existe como casilla y no abre nada.                                  |
| D-23 | HEAD separado: banner con "Crear rama aqui"                                                                                                                                                        | P1        | Hoy es un toast de aviso y el usuario puede perder commits sin enterarse.                                   |
| D-24 | Establecer upstream (remoto y nombre de rama) y primer push de una rama nueva                                                                                                                      | P1        | El menu lo ofrece sin dialogo; el primer push de cada rama lo necesita.                                     |
| D-25 | Deshacer y rehacer: que operaciones se pueden deshacer, confirmacion de lo deshecho, historial                                                                                                     | P1        | Los botones existen; detras hace falta un diario de operaciones en el motor y la UI debe decir sus limites. |
| D-26 | Registro de operaciones o consola de git con comandos y salidas                                                                                                                                    | P2        | Diagnostico para usuarios avanzados y soporte.                                                              |

#### Vistas que los menus prometen y no existen

| Id   | Vista o estado                                                                                                                                                                    | Prioridad | Por que                                                              |
| ---- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | --------- | -------------------------------------------------------------------- |
| D-27 | Historial de un archivo (commits que lo tocan, con el diff de cada uno)                                                                                                           | P0        | Esta en el menu de archivo y hoy es un toast.                        |
| D-28 | Blame (autor y commit por linea, saltar al commit)                                                                                                                                | P0        | Igual: menu sin vista.                                               |
| D-29 | Comparar ramas (commits por delante y por detras, archivos cambiados, diff)                                                                                                       | P1        | "Comparar con" abre hoy un diff fijo de un solo archivo.             |
| D-30 | Detalle de PR: descripcion, checks con enlaces, revisiones y comentarios, aprobar, fusionar, checkout                                                                             | P1        | Pulsar un PR hoy solo abre el repo.                                  |
| D-31 | Crear PR completo: rama sin publicar (publicar primero), plantillas, etiquetas, asignados, destino en un fork, vista previa de commits, autocompletar revisores, exito con enlace | P1        | El modal actual cubre el caso minimo y falla con ramas sin publicar. |
| D-32 | Gestion de remotos: anadir, quitar, renombrar                                                                                                                                     | P1        | Solo existe editar la URL.                                           |
| D-33 | Vista previa del contenido de un stash antes de aplicarlo; stash de archivos sueltos                                                                                              | P1        | Aplicar un stash a ciegas es la forma habitual de liarla.            |
| D-34 | Submodulos completos (anadir, init, sync, update recursivo) y tags remotos (borrar, publicar todos)                                                                               | P2        | Hoy solo hay acciones sueltas; borrar tag ni siquiera confirma.      |

#### Diff, staging y grafo

| Id   | Vista o estado                                                                                                                                                                                                 | Prioridad | Por que                                                                                         |
| ---- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | --------- | ----------------------------------------------------------------------------------------------- |
| D-35 | Staging y descarte por hunk y por linea                                                                                                                                                                        | P0        | Es la funcion por la que se elige un cliente como este; el diccionario tiene "Parcial" sin uso. |
| D-36 | Diff real: cabeceras de hunk, regiones sin cambios plegadas, resaltado de sintaxis, cambios por palabra, opciones de espacios, renombrados, binarios e imagenes, archivos enormes, no rastreados, punteros LFS | P1        | El prototipo usa el mismo diff de muestra para todos los archivos.                              |
| D-37 | Multiseleccion en staging y en el grafo                                                                                                                                                                        | P1        | Esperado en cualquier lista; en el grafo habilita squash de commits seleccionados.              |
| D-38 | Conflictos binarios, borrado frente a modificado y renombrado: elegir a nivel de archivo (nuestro, suyo, mantener borrado)                                                                                     | P1        | El resolvedor asume hunks de texto; con estos conflictos no hay forma de terminar el merge.     |
| D-39 | Resolvedor con vista base (diff3), herramienta de merge externa y resolucion parcial que sobrevive a un reinicio                                                                                               | P2        | Mejora de calidad sobre el resolvedor de dos vias.                                              |
| D-40 | Grafo con carga paginada, filas virtualizadas y "cargar mas"                                                                                                                                                   | P0        | El prototipo carga 19 commits; un repo real tiene cientos de miles.                             |
| D-41 | Grafo: ancho de columna dinamico con mas de cinco carriles, ocultar o aislar ramas, filtros de autor y fecha, buscar en todo el historial, tooltips de nodo y ref                                              | P1        | La columna es fija de 130 px y la busqueda solo mira las filas cargadas.                        |

#### Ajustes y ventana

| Id   | Vista o estado                                                                                                                                                                                                | Prioridad | Por que                                                                                                                                           |
| ---- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | --------- | ------------------------------------------------------------------------------------------------------------------------------------------------- |
| D-42 | Barra de titulo de Tauri: zona de arrastre, controles de ventana en Windows y Linux, hueco de semaforo en macOS, tamano minimo                                                                                | P0        | La ventana es sin marco; sin esto no se puede mover ni cerrar.                                                                                    |
| D-43 | Ajustes de Git: ruta y version del ejecutable git detectado, "Cambiar ruta"                                                                                                                                   | P1        | Con git del sistema (decision 1), el usuario necesita ver cual se usa y cambiarlo.                                                                |
| D-44 | Herramientas externas: editor, terminal, herramienta de diff y de merge                                                                                                                                       | P1        | "Abrir en el editor" y "Abrir terminal" existen en menus sin ajuste detras.                                                                       |
| D-45 | Actualizaciones: version, canal, comprobar, "Hay una version nueva", notas; y el estado para formatos sin actualizacion automatica (`.deb`, `.rpm`, Flatpak)                                                  | P1        | Sin esto la app firmada nunca avisa de nada; ver X-09.                                                                                            |
| D-46 | Atajos de teclado y paleta de comandos del escritorio                                                                                                                                                         | P1        | Los menus muestran atajos que no hacen nada.                                                                                                      |
| D-47 | Cuenta y equipo: vista de miembro sin permisos de admin, cambiar de organizacion, crear o salir de una organizacion, transferir la propiedad, sin asientos libres al invitar, invitaciones pendientes para mi | P1        | La web tiene selector de organizaciones y el escritorio no; ambos asumen propietario.                                                             |
| D-48 | Workspace: color, orden, "Compartido con el equipo", progreso por repo en "Fetch de todos"                                                                                                                    | P1        | La web permite compartir y el escritorio no lo muestra.                                                                                           |
| D-49 | Enlaces profundos: `ramal://open?repo=...`, "Abrir en Ramal Desktop" desde web y VS Code, enlaces de clonar                                                                                                   | P1        | La clave `openDesktop` existe en VS Code y no lleva a ninguna parte.                                                                              |
| D-50 | Red y proxy, telemetria con consentimiento, notificaciones, ajustes por repositorio, plantillas de commit, consentimiento de informes de fallos, "novedades" tras actualizar                                  | P2        | Necesario a medio plazo, no para la primera version.                                                                                              |
| D-51 | "Que se sincroniza" en Ajustes > Cuenta: los seis interruptores de la web, con los de secretos que crean la boveda (K-01) si no existe                                                                        | P1        | Hoy los interruptores solo estan en la web, que no puede crear la boveda; encender ahi los secretos deja la categoria activa sin forma de cifrar. |
| D-52 | Pestanas y lista de repos: ordenar y fijar repos, desbordamiento de pestanas, arrastrar para reordenar y "Cerrar las demas"                                                                                   | P2        | El inventario lo pide; con muchos repos abiertos la barra de pestanas no cabe.                                                                    |

### 1.4 Ramal for VS Code

| Id   | Vista o estado                                                                                                                                          | Prioridad | Por que                                                                                                                                                                                                                                                        |
| ---- | ------------------------------------------------------------------------------------------------------------------------------------------------------- | --------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| V-01 | Iniciar sesion en Ramal: proveedor de autenticacion, estado sin sesion en Ajustes y en la barra de estado, `viewsWelcome`; en curso, fallido y caducado | P0        | El diseno solo muestra la sesion iniciada; el flujo usa `UriHandler` y `asExternalUri`, y Git local funciona sin cuenta.                                                                                                                                       |
| V-02 | Sin carpeta abierta: `viewsWelcome` con "Abrir carpeta" y "Clonar repositorio"                                                                          | P0        | Sin esto las vistas de Ramal aparecen vacias.                                                                                                                                                                                                                  |
| V-03 | Carpeta que no es un repositorio: "Iniciar repositorio" y "Clonar"                                                                                      | P0        | Mismo motivo.                                                                                                                                                                                                                                                  |
| V-04 | Git no instalado o anterior a 2.34                                                                                                                      | P0        | La extension no puede hacer nada; necesita vista de bienvenida con enlace de instalacion.                                                                                                                                                                      |
| V-05 | Motor no disponible: el binario `ramal-vscode-engine` no arranca, se cae y se reinicia, o la plataforma no tiene VSIX propio                            | P0        | Con el motor como proceso aparte (decision 1), su caida es un estado real que el usuario tiene que ver.                                                                                                                                                        |
| V-06 | Multi-root y varios repos en una carpeta (repos anidados, submodulos): selector de repo en el grafo, el titulo de la vista y la barra de estado         | P0        | El diseno asume un solo repo; en VS Code los espacios multi-root son habituales.                                                                                                                                                                               |
| V-07 | Conflictos de pull, rebase, cherry-pick y revert en el editor de merge, con continuar, omitir y abortar; "Abortar merge"; "Editar a mano"               | P0        | Solo el merge esta dibujado, y en VS Code ni siquiera se puede abortar ni editar a mano.                                                                                                                                                                       |
| V-08 | Carga y repos grandes: log paginado, filas virtualizadas, "cargar mas"                                                                                  | P0        | El prototipo carga todos los commits de golpe.                                                                                                                                                                                                                 |
| V-09 | Repositorio vacio y HEAD separado                                                                                                                       | P1        | El grafo vacio no tiene estado y HEAD separado es solo un toast.                                                                                                                                                                                               |
| V-10 | Push rechazado, sin upstream y fallo de autenticacion: ofrecer pull, force-with-lease con confirmacion, establecer upstream, conectar cuenta            | P1        | Solo el camino feliz esta dibujado; Desktop ya tiene estas claves en el diccionario.                                                                                                                                                                           |
| V-11 | Arbol sucio en checkout, merge o rebase: ofrecer stash o autostash                                                                                      | P1        | Hoy no se contempla.                                                                                                                                                                                                                                           |
| V-12 | Staging parcial desde el diff nativo (comando sobre la seleccion)                                                                                       | P1        | En Desktop es P0; en VS Code el diff es nativo y hay que decidir como se ofrece.                                                                                                                                                                               |
| V-13 | Historial de archivo (grafo filtrado por ruta o `TimelineProvider`)                                                                                     | P1        | El menu lo ofrece y es un stub.                                                                                                                                                                                                                                |
| V-14 | Crear tag y menu de tag                                                                                                                                 | P1        | Falta en VS Code; se reutiliza el formulario de Desktop.                                                                                                                                                                                                       |
| V-15 | Sin cuenta de plataforma para el remoto, o host desconocido: "Conectar una cuenta de X" en la vista de PRs                                              | P1        | Sin cuenta, la vista de PRs y la creacion de PR no funcionan y no dicen por que.                                                                                                                                                                               |
| V-16 | Errores de la API de la plataforma (401, limite de peticiones, SSO requerido) y de la IA (red, llave invalida, cuota)                                   | P1        | Solo estan dibujados "IA desactivada", "sin llave" y "nada en staging".                                                                                                                                                                                        |
| V-17 | Detalle de PR (o, como minimo, abrirlo en la plataforma)                                                                                                | P1        | Pulsar un PR no hace nada.                                                                                                                                                                                                                                     |
| V-18 | Ramal Desktop no instalado cuando se pulsa "Abrir en Ramal Desktop": ofrecer la descarga                                                                | P1        | El enlace profundo falla en silencio.                                                                                                                                                                                                                          |
| V-19 | Recorrido de bienvenida (`contributes.walkthroughs`): iniciar sesion, conectar plataforma, anadir llave, abrir grafo                                    | P1        | Primera vez del usuario en la extension.                                                                                                                                                                                                                       |
| V-20 | Confianza del espacio de trabajo (modo restringido), Remote Development, WSL y Codespaces; vscode.dev declarado no soportado                            | P1        | Ejecutar git en carpetas no confiables es un riesgo; en remoto el motor corre en el host remoto.                                                                                                                                                               |
| V-21 | Convivencia con el Git integrado y con GitLens: ocultar duplicados de la barra de estado y del SCM                                                      | P1        | Sin esto el usuario ve dos ramas actuales y dos botones de commit.                                                                                                                                                                                             |
| V-22 | Conflicto entre Settings Sync de VS Code y la nube de Ramal para los mismos ajustes                                                                     | P1        | **Resuelta el 2026-10-01** ([ADR 0010](../adr/0010-un-motor-para-escritorio-y-vscode.md#ajustes-y-sincronizacion)): Git e IA viven en la nube de Ramal y se editan en el webview de ajustes; en `settings.json` solo lo de la maquina, con `scope: "machine"`. |
| V-23 | Blame                                                                                                                                                   | P2        | Se solapa con GitLens; no es prioritario en VS Code.                                                                                                                                                                                                           |
| V-24 | Rebase interactivo, deshacer, submodulos y gestion de remotos en VS Code (o derivar a Desktop)                                                          | P2        | Hoy son solo de Desktop.                                                                                                                                                                                                                                       |
| V-25 | Temas de alto contraste                                                                                                                                 | P2        | No estan disenados; basta con bordes de contraste en el mapeo de tokens.                                                                                                                                                                                       |

### 1.5 Ramal Web

#### Autenticacion e identidad

| Id   | Vista o estado                                                                                                                                                                                                              | Prioridad | Por que                                                                                                                                               |
| ---- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | --------- | ----------------------------------------------------------------------------------------------------------------------------------------------------- |
| W-01 | "Olvide mi contrasena": pedir email y estado "revisa tu correo" (misma respuesta exista o no la cuenta)                                                                                                                     | P0        | Hoy "La olvidaste?" es un toast que ni pide el email.                                                                                                 |
| W-02 | Restablecer contrasena: token del correo, contrasena nueva y confirmacion, token caducado; aviso de que no cambia la frase de sincronizacion                                                                                | P0        | Sin esta pagina el correo de W-01 no lleva a ningun sitio.                                                                                            |
| W-03 | Verificacion de email: "revisa tu correo" tras el registro, pagina de verificacion, reenviar, banner de no verificado                                                                                                       | P0        | Invitaciones y recuperacion dependen de que el email sea de verdad del usuario.                                                                       |
| W-04 | Estados de OAuth: volviendo del proveedor, acceso denegado, email ya usado con otro metodo (vincular), completar perfil la primera vez (terminos y organizacion opcional)                                                   | P0        | Hoy OAuth entra directo, sin terminos y sin ningun error posible.                                                                                     |
| W-05 | Pagina "Autorizar Ramal Desktop" o "Autorizar Ramal for VS Code": que app pide acceso, con que cuenta, Autorizar o Cancelar; pagina de exito que devuelve a la app                                                          | P0        | Es la mitad web del inicio de sesion con PKCE de D-03 y V-01; sin ella ningun cliente puede entrar.                                                   |
| W-06 | Pagina de codigo de dispositivo: introducir el codigo, confirmar, exito                                                                                                                                                     | P0        | Alternativa cuando la redireccion al cliente no funciona (SSH remoto, contenedores, navegador en otro equipo).                                        |
| W-07 | Formularios enviando y errores del servidor (credenciales invalidas, email ya registrado, limite de intentos)                                                                                                               | P0        | Hoy solo existen errores de validacion del cliente.                                                                                                   |
| W-08 | Vuelta del OAuth de plataforma intermediado: la pagina "Cuenta conectada, vuelve a Ramal" (y su error) la sirve el receptor de loopback del motor; la web solo pone la pagina de error para un flujo desconocido o caducado | P1        | Con Bitbucket la API redirige al loopback del motor sin canjear; el navegador nunca aterriza en la web salvo cuando la API no sabe a donde redirigir. |
| W-09 | Sesion caducada (401) y redireccion de usuarios con sesion fuera de login y registro                                                                                                                                        | P1        | Hoy las paginas de autenticacion siguen accesibles con sesion iniciada.                                                                               |
| W-10 | Autenticacion en dos pasos (alta y desafio)                                                                                                                                                                                 | P2        | Con la boveda E2E, robar la sesion ya no da los tokens; baja la urgencia, sigue siendo esperable.                                                     |
| W-33 | Contrasena: cambiarla, o establecer la primera en una cuenta creada con OAuth (con reautenticacion)                                                                                                                         | P1        | La API lo permite y no hay pantalla; Desktop y VS Code abren esta pagina, porque desde ellos no se puede.                                             |
| W-34 | Identidades vinculadas: listar, vincular otra (con reautenticacion) y quitar una                                                                                                                                            | P1        | Quitar GitHub sin perder la cuenta, o anadir Google, no tiene donde hacerse.                                                                          |
| W-35 | Reautenticar antes de una accion sensible: contrasena, OAuth o enlace al correo, y vuelta a la accion                                                                                                                       | P1        | Eliminar la cuenta, reiniciar la boveda o cambiar el correo exigen un inicio de sesion reciente.                                                      |

#### Organizacion, equipo e IA de la organizacion

| Id   | Vista o estado                                                                                                                           | Prioridad | Por que                                                                                               |
| ---- | ---------------------------------------------------------------------------------------------------------------------------------------- | --------- | ----------------------------------------------------------------------------------------------------- |
| W-11 | Aceptar invitacion: unirse a la organizacion con un rol, entrar o registrarse con el email invitado, invitacion caducada o revocada      | P0        | Sin esta pagina las invitaciones que ya estan dibujadas no se pueden aceptar.                         |
| W-12 | Crear organizacion y ajustes de la organizacion (renombrar, avatar, borrar, transferir propiedad, salir)                                 | P1        | "Crear organizacion" es hoy un toast de relleno.                                                      |
| W-13 | Contexto personal: que muestran Equipo y los workspaces compartidos cuando se elige "Personal"                                           | P1        | La seccion Equipo se muestra igual en contexto personal.                                              |
| W-14 | Vistas por rol: admin y miembro (sin invitar, sin quitar, roles de solo lectura)                                                         | P1        | El usuario conectado siempre es propietario en el prototipo.                                          |
| W-15 | Sin asientos libres al invitar, con salida para comprar asientos                                                                         | P1        | Hoy invitar por encima del limite sube los asientos en silencio (B-51).                               |
| W-16 | IA de la organizacion (solo admins): anadir, rotar y quitar llaves de la organizacion, sin volver a mostrarlas nunca                     | P1        | La decision 3 guarda esas llaves en el servidor y el diseno no tiene donde darlas de alta (ver X-05). |
| W-36 | Consumo de IA de la organizacion (solo admins): peticiones y tokens por miembro, funcion y llave, y la cuota que le queda a cada miembro | P2        | La API ya lo sirve; sin pantalla, las llaves de organizacion son un cheque en blanco.                 |

#### Facturacion

| Id   | Vista o estado                                                                                       | Prioridad | Por que                                                                                                                                                                                                                    |
| ---- | ---------------------------------------------------------------------------------------------------- | --------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| W-17 | Precios en la landing y comparativa de planes                                                        | P1        | Desktop muestra "Ramal Pro" y la landing no tiene seccion de precios. Planes y precios en [ADR 0026](../adr/0026-planes-y-precios.md); mensaje principal: "Gratis para siempre: todo Git local, repos privados incluidos". |
| W-18 | Gestion de la suscripcion: plan, renovacion, metodo de pago, facturas, cancelar, asientos con precio | P1        | "Gestionar suscripcion" (Desktop) y "Anadir asientos" (web) no llevan a nada. Se cobra con Paddle ([ADR 0022](../adr/0022-proveedor-de-pagos.md)): checkout en una pagina de la web y portal del proveedor.                |

#### Contenido del panel

| Id   | Vista o estado                                                                                                                                                            | Prioridad | Por que                                                                                                                                                                                |
| ---- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | --------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| W-19 | Estados vacios: sin workspaces, sin repos, busqueda sin resultados, sin dispositivos (llamada a descargar la app o instalar la extension), equipo solo con el propietario | P0        | Un usuario recien registrado ve todo el panel vacio.                                                                                                                                   |
| W-20 | Esqueletos y errores de la API por seccion                                                                                                                                | P0        | Cada seccion carga por separado y puede fallar por separado.                                                                                                                           |
| W-21 | Perfil completo (avatar, usuario, zona horaria) y un unico concepto de sesiones y dispositivos                                                                            | P1        | Desktop tiene "Sesiones activas" con ciudad y la web "Dispositivos"; es la misma cosa con dos nombres. Sin ubicacion (Q-05): dispositivo, sistema, app o navegador y ultima actividad. |
| W-22 | Detalle de dispositivo: marca de "este navegador", version desactualizada, renombrar, cerrar sesion con confirmacion                                                      | P1        | Hoy no se distingue el dispositivo actual y cerrar sesion en otro no confirma.                                                                                                         |
| W-23 | Detalle de repositorio: "Abrir en Ramal Desktop" (y la descarga si no esta instalado)                                                                                     | P1        | La web lista repos pero no deja hacer nada con ellos. En que dispositivos esta clonado no se sabe: la API no guarda presencia por dispositivo, a proposito.                            |
| W-24 | Eliminar cuenta: volver a autenticarse, bloqueo si es propietario de una organizacion con miembros, pagina final                                                          | P1        | Hoy basta un clic y una confirmacion.                                                                                                                                                  |
| W-25 | Indicador de sincronizacion (ultima vez, errores)                                                                                                                         | P1        | Los toggles de sincronizacion no dicen si algo se ha sincronizado nunca.                                                                                                               |
| W-26 | Exportacion asincrona (en curso, lista para descargar)                                                                                                                    | P2        | Solo si la exportacion deja de ser instantanea.                                                                                                                                        |

#### Sitio publico

| Id   | Vista o estado                                                                                                                                        | Prioridad | Por que                                                                                   |
| ---- | ----------------------------------------------------------------------------------------------------------------------------------------------------- | --------- | ----------------------------------------------------------------------------------------- |
| W-27 | Terminos del servicio y politica de privacidad (y de cookies si hay analitica)                                                                        | P0        | La casilla del registro los acepta y no existen; no se puede abrir el registro sin ellos. |
| W-28 | Paginas 404 y 500                                                                                                                                     | P0        | Cualquier enlace roto o fallo del servidor cae hoy en nada.                               |
| W-29 | Documentacion, notas de la version y pagina de estado                                                                                                 | P1        | Son enlaces del pie que hoy muestran "Listo: ...".                                        |
| W-30 | Diseno responsive: navegacion publica plegable, barra lateral del panel en pantallas estrechas, visitantes desde movil ("disponible para escritorio") | P1        | Android e iOS se detectan hoy como macOS y se les ofrece un `.dmg`.                       |
| W-31 | Pagina "Gracias por descargar" con primeros pasos, sha256 y firma                                                                                     | P2        | Patron habitual; da la suma de verificacion que hoy no aparece en ningun sitio.           |
| W-32 | Contacto y soporte                                                                                                                                    | P2        | No hay forma de contactar.                                                                |

---

## 2. Cambios de diseno obligados por decisiones

Cada cambio dice que muestra hoy el diseno, en que cambia y que decision lo obliga. Las decisiones
numeradas son las del usuario: (1) motor git unico en Rust con git CLI, (2) secretos de plataforma
cifrados de extremo a extremo, (3) llaves de IA de organizacion solo en el servidor, (4) Ramal funciona
sin cuenta para todo lo local.

### X-01 Fuera las cuentas de plataforma compartidas con el equipo

- **Hoy:** en la web, cada cuenta vinculada tiene el menu "Compartir con el equipo", el modal "Conectar"
  tiene la casilla del mismo nombre y cada fila lleva la etiqueta "Tuya" o "Equipo".
- **Cambio:** se quitan la casilla, la opcion de menu y la etiqueta. Cada miembro vincula sus propias
  cuentas. Compartir workspaces con el equipo se mantiene: son metadatos, no secretos.
- **Por que:** decision 2. Con cifrado de extremo a extremo el servidor no puede entregar un token a otro
  miembro, y copiarlo a sus equipos significa que revocar el acceso no deshace la fuga.

### X-02 La web solo ve y quita cuentas vinculadas

- **Hoy:** la web tiene "+ Conectar" por plataforma, un modal con host, usuario y token en texto plano, y
  "Probar conexion" en el menu de cada cuenta.
- **Cambio:**
  - Se quitan el modal "Conectar" y "Probar conexion". En su lugar, cada grupo de plataforma muestra una
    llamada "Anade cuentas desde Ramal Desktop o VS Code", con enlace profundo a los ajustes de
    plataformas de Desktop y enlace a descargas si no esta instalado.
  - Cada fila muestra solo metadatos: plataforma, host, usuario, etiqueta, metodo, permisos, en cuantos
    repos se usa y, si el proveedor lo informa, la fecha de caducidad declarada.
  - "Quitar" se mantiene, con un texto que diga la verdad: quita la cuenta de Ramal en todos los
    dispositivos cuando sincronicen, pero **no revoca el token en la plataforma**, porque el servidor no
    lo tiene. El dialogo enlaza a la pagina de tokens de esa plataforma.
- **Por que:** decision 2. Probar una conexion exige usar el token, y la web no puede descifrarlo.
  "Probar conexion" se queda en Desktop y VS Code, donde el token existe.

### X-03 Los textos sobre cifrado tienen que seguir siendo verdad

"Tokens cifrados de extremo a extremo" (`W.sAccD`) pasa a ser cierto para cuentas vinculadas y llaves de IA
personales sincronizadas. El resto de textos tiene que acompanarlo:

- **Llaves de IA de la organizacion no son E2E.** Se cifran en reposo en el servidor y el servidor las usa.
  Su texto debe decirlo: "Cifradas en los servidores de Ramal; Ramal las usa para generar en nombre de tu
  organizacion".
- **Usar una llave de la organizacion envia el diff por los servidores de Ramal.** El proxy de la decision
  3 recibe los cambios en staging para generar el mensaje. La primera vez que un miembro genera con una
  llave de la organizacion, y en la fila de la llave, hay que decirlo. Con llave personal no pasa.
- **Nadie puede recuperar los secretos**, tampoco el soporte de Ramal, si se pierden la frase y el codigo
  de recuperacion. Debe estar en K-01, K-02 y K-08, y en cualquier texto de ayuda.
- **Cerrar sesion en otro dispositivo no garantiza borrar sus secretos.** El dispositivo pierde acceso a
  la API y borra la boveda la proxima vez que conecte; si nunca vuelve a conectar, conserva lo que ya
  tenia descifrado. El dialogo de W-22 no debe prometer un borrado remoto.
- **VS Code:** `syncD` dice que se sincronizan las llaves de IA. Debe decir que es opcional y esta
  desactivado por defecto, igual que en la web.
- **Web:** `W.sAiKeysD` ("Desactivado: cada dispositivo guarda sus propias llaves") no cambia al activar
  el toggle. Necesita texto para el estado activado ("Cifradas de extremo a extremo con tu frase de
  sincronizacion").

### X-04 Pantallas de la boveda y toggles de sincronizacion de secretos

- **Hoy:** "Cuentas vinculadas" y "Llaves de IA" son toggles simples en la web y en VS Code.
- **Cambio:** las pantallas K-01 a K-10 de la seccion 1.2. Los toggles de secretos en la web muestran el
  estado de la boveda ("Requiere tu frase de sincronizacion; se configura en Ramal Desktop o VS Code").
  Desactivar la sincronizacion de cuentas o de llaves **borra de la nube los metadatos y las copias
  cifradas**, y la interfaz lo confirma con un aviso de peligro; los equipos conservan su copia local y su
  eleccion de cuenta por repositorio. Desktop tiene los mismos interruptores (D-51).
- **Por que:** decision 2. Sin frase no hay forma de cifrar para varios equipos a la vez.

### X-05 Llaves de IA de la organizacion

- **Hoy:** en Ajustes > IA de Desktop, al anadir una llave se elige el alcance "Personal" u
  "Organizacion - ACME", y la llave queda en el equipo como cualquier otra.
- **Cambio:**
  - El alta, la rotacion y la baja de llaves de la organizacion se disenan en la web, para admins (W-16).
    El formulario solo escribe: la llave nunca se vuelve a mostrar, solo su mascara calculada en el
    servidor.
  - Desktop y VS Code las listan de solo lectura con la etiqueta "De la organizacion", y se pueden elegir
    por funcion (mensajes de commit, PRs, resumenes). "Probar conexion" pasa por la API y solo lo ven los
    admins.
  - El modelo de cada funcion con una llave de la organizacion lo fija el admin, y el miembro lo ve de
    solo lectura ([ADR 0025](../adr/0025-recomendacion-de-modelos-de-ia.md)).
  - El selector de alcance desaparece del formulario de llaves de Desktop: lo que se anade ahi es
    siempre personal.
  - Ollama y los endpoints compatibles con OpenAI en redes privadas no pueden ser llave de organizacion:
    el servidor no llega al `localhost` ni a la red del usuario.
- **Por que:** decision 3. La llave vive en el servidor y el servidor hace de intermediario.

### X-06 Ramal sin cuenta

- **Hoy:** todo el diseno asume sesion iniciada.
- **Cambio:** estado sin sesion en los tres productos (D-01, D-04, V-01). Las zonas de nube (cuenta y
  equipo, llaves de organizacion, sincronizacion) muestran una invitacion a iniciar sesion en vez de
  desaparecer. Los workspaces de Desktop funcionan sin sesion como workspaces locales, y se suben al
  iniciar sesion si la nube no tiene ninguno (ADR 0014); lo que pide sesion es que se sincronicen. Las cuentas de plataforma y las llaves personales se pueden anadir sin cuenta de Ramal
  y quedan "Solo en este equipo". En la landing, la funcion 06 ("Tu configuracion en la nube") dice que
  requiere cuenta (no pago: la sincronizacion personal es gratis, ADR 0026), y el resto no.
- **Por que:** decision 4.

### X-07 Alta de cuentas de plataforma: OAuth publico primero, usuario detectado

- **Hoy:** el modal pide plataforma, host, usuario y token. El metodo es un texto libre y el usuario se
  teclea.
- **Cambio:** el formulario ofrece primero OAuth sin secreto de cliente y el token como alternativa. El
  usuario y los permisos se leen de la plataforma al validar, no se teclean. Metodo por plataforma:

  | Plataforma                                                   | Metodo por defecto                                    | Alternativa                                         |
  | ------------------------------------------------------------ | ----------------------------------------------------- | --------------------------------------------------- |
  | GitHub                                                       | OAuth con codigo de dispositivo                       | Token de grano fino                                 |
  | GitLab.com                                                   | OAuth con PKCE                                        | Token personal                                      |
  | GitLab self-managed, Gitea/Forgejo, GitHub Enterprise Server | Token personal                                        | OAuth si el admin registro una app, despues del MVP |
  | Bitbucket Cloud                                              | OAuth intermediado por la API sin guardar el token    | API token                                           |
  | Azure DevOps                                                 | Token personal de alcance organizacion                | Entra ID mas adelante                               |
  | SSH                                                          | Solo transporte, siempre junto a un token para la API | Ninguna                                             |

- **Por que:** decision 2. Con clientes publicos el token nunca pasa por el servidor; solo Bitbucket, que
  exige secreto de cliente, pasa por la API de forma transitoria. El navegador vuelve al receptor de
  loopback del motor, que es quien sirve la pagina de exito (W-08).

### X-08 Bitbucket y Azure DevOps

- **Bitbucket, hoy:** los datos de ejemplo usan el metodo "App password".
  **Cambio:** "App password" desaparece de todos los selectores y textos; se usa "API token" (con alcance
  y caducidad) u OAuth. **Por que:** Atlassian dejo de crear app passwords el 2025-09-09 y desactivo las
  que quedaban el 2026-06-09.
- **Azure DevOps, hoy:** formulario generico con host `dev.azure.com` y token.
  **Cambio:** el formulario pide la organizacion y explica que el token debe tener alcance de esa
  organizacion; no se ofrece OAuth. **Por que:** los tokens globales dejan de funcionar el 2026-12-01,
  dos meses despues de hoy, y el OAuth de Azure DevOps se retira en 2026.

### X-09 Descargas: piso de Linux, formatos y actualizaciones

- **Linux, hoy:** "Ubuntu 20.04+, Fedora 38+, Debian 11+".
  **Cambio:** "Ubuntu 22.04+, Debian 12+", y en Fedora solo versiones con soporte vigente que CI pruebe
  (Fedora 38 ya no tiene soporte). **Por que:** Tauri 2 necesita `webkit2gtk-4.1`, que no existe en
  Ubuntu 20.04 ni en Debian 11.
  **Estado (2026-10-01):** hecho. La landing dice "Ubuntu 22.04+, Debian 12+" y no nombra Fedora
  hasta que CI pruebe una version con soporte vigente.
- **Windows Portable (.zip), hoy:** tarjeta con 121 MB.
  **Cambio:** se quita (ADR 0016). **Por que:** el empaquetador de Tauri no lo produce, no es portable de
  verdad y nunca se actualizaria solo.
- **Windows MSI:** se mantiene para despliegues gestionados, solo x64, con la etiqueta "sin
  actualizaciones automáticas", y sale en el lanzamiento. El instalador que se actualiza solo es el NSIS.
  No es un limite de Tauri (desde `tauri-plugin-updater` 2.10 sabe instalar MSI): es una decision, porque
  el MSI lo gestiona IT y actualizarlo pide administrador.
- **Windows arm64 y Linux arm64:** salen con la etiqueta beta hasta que su E2E pase en runners arm.
  Windows arm64 solo tiene NSIS: el MSI de Tauri no se compila para arm64.
- **macOS Universal (.zip), hoy:** alternativa en la tarjeta de macOS.
  **Cambio:** "Universal (.dmg)", que Tauri si produce y que ADR 0016 anade a la matriz.
- **`.deb`, `.rpm` y Flathub:** no se actualizan con el actualizador de Tauri (en `.deb` y `.rpm` porque
  pediria la contrasena de root; Flathub, que llega despues del lanzamiento, lo actualiza Flatpak). La
  tarjeta lo indica y
  Desktop, en esos formatos, muestra "Hay una version nueva" con enlace de descarga (o nada en Flatpak,
  que se actualiza por su cuenta) en vez de "Actualizar y reiniciar" (D-45).
- **VS Code:** la extension lleva el binario del motor, asi que el `.vsix` es uno por plataforma
  (`win32-x64`, `darwin-arm64`, `linux-x64`...) y pesa bastante mas que los 6 MB del diseno (el analisis
  estima de 10 a 15 MB). La alternativa ".vsix" de la tarjeta pasa a ser una lista por plataforma y el
  tamano sale del pipeline. Los requisitos anaden "no disponible en vscode.dev".
- **Version, tamanos y sha256** salen del endpoint de releases de la API, no del HTML (hoy "v2.4.0, 30
  sep 2026" esta escrito a mano).
- **Por que:** decision 1 (VSIX por plataforma) y los hechos de Tauri 2.
- **Estado (2026-10-01):** resuelta y ratificada con Q-04 y Q-07; el detalle esta en
  [ADR 0016](../adr/0016-actualizaciones-y-descargas.md).

### X-10 Modelos de IA: de la lista del proveedor, nunca constantes

- **Hoy:** cada proveedor trae un modelo por defecto fijo en `ramal-data.js` (`claude-sonnet-4-5`,
  `gpt-5-mini`, `gemini-2.5-flash`...) y el campo modelo es texto libre.
- **Cambio:** el campo modelo es un selector que se rellena con la lista de modelos del proveedor despues
  de validar la llave (en Ollama, los modelos instalados en local), con texto libre solo para endpoints
  compatibles con OpenAI. Necesita estados de cargando, error y "este modelo ya no esta disponible". El
  campo URL base (OpenRouter, Ollama, personalizado) se guarda de verdad (hoy se recoge y se descarta) y
  aparece tambien en VS Code, que no lo tiene.
- **Por que:** los modelos cambian cada pocos meses; una constante en una app instalada se queda vieja
  sin poder corregirla.
- **Estado (2026-10-01):** resuelta con Q-06 por
  [ADR 0025](../adr/0025-recomendacion-de-modelos-de-ia.md). La insignia "Recomendado" sale de un
  fichero remoto que sirve la API, cruzado con la lista viva del proveedor, con dos niveles ("Rápido"
  para commits, "Equilibrado" para PRs y resumenes). Cada funcion puede tener su modelo; con una llave de
  la organizacion lo fija el admin. Sin red ni copia en cache, no hay recomendacion. Los modelos por
  defecto de `ramal-data.js` no se copian.

### X-11 VS Code: lo nativo se hace nativo

- **Hoy:** el prototipo redibuja en HTML la paleta de comandos, los dialogos de entrada, las
  notificaciones con "Origen: Ramal", la barra de estado, los menus contextuales y la pagina de ajustes.
- **Cambio:** el entregable de diseno para VS Code deja de ser una maqueta pixel a pixel de esas
  superficies y pasa a ser una especificacion de contribuciones:

  | Superficie                                                                       | Como se hace                                                             |
  | -------------------------------------------------------------------------------- | ------------------------------------------------------------------------ |
  | Paleta de comandos                                                               | Comandos con categoria "Ramal"; el diseno define nombres y agrupacion    |
  | Dialogos (nombre de rama, mensaje de stash, estrategia de merge, confirmaciones) | `showInputBox`, `showQuickPick`, avisos modales                          |
  | Anadir cuenta y anadir llave                                                     | QuickInput de varios pasos, con campo de secreto oculto                  |
  | Notificaciones                                                                   | Nativas; VS Code anade el origen solo                                    |
  | Barra de estado                                                                  | Elementos nativos                                                        |
  | Ramas, remotos, PRs, stashes, tags                                               | TreeViews con Codicons; no hay doble clic, se usa clic o accion en linea |
  | Menus contextuales                                                               | Contribuciones de menus, tambien dentro de los webviews                  |
  | Ajustes de esta maquina (ruta y log del motor, usar la nube)                     | Editor de ajustes de VS Code, con `scope: "machine"` (V-22)              |
  | Iniciar sesion                                                                   | Menu de cuentas mediante un proveedor de autenticacion                   |

  Quedan como webview: el grafo con su panel de detalle, el editor de merge, el formulario de crear PR y
  la pagina de ajustes de Ramal, con las cuentas, las llaves y las preferencias de Git e IA que comparte
  con Desktop (V-22). El atajo `Mayus+Cmd+G` choca con el de la vista
  de control de codigo de VS Code y hay que elegir otro.

- **Por que:** coherencia con el editor, teclado y accesibilidad gratis, y menos codigo que mantener.
  Decision 1 ademas deja el host de la extension delgado.

### X-12 La landing no embebe el prototipo

- **Hoy:** la landing mete `Ramal.dc.html` y `Ramal VS Code.dc.html` en `<iframe>` escalados.
- **Cambio:** captura o video, o una demo de solo lectura hecha con `@ramal/repo-view` y datos de ejemplo,
  cargada en diferido.
- **Por que:** el runtime del prototipo (`support.js`) carga React y Babel desde unpkg y evalua codigo con
  `new Function`, lo que choca con cualquier politica de seguridad de contenido, pesa y no es codigo de
  produccion.

### X-13 Git del sistema como requisito visible

- **Hoy:** el diseno no menciona git en ningun sitio.
- **Cambio:** estados de git ausente o antiguo (D-02, V-04), ruta y version en ajustes (D-43), y
  dialogos propios de credenciales (D-13). La frase de una llave GPG la pide `pinentry` del sistema, no
  Ramal: el diseno no debe dibujar ese dialogo como propio.
- **Por que:** decision 1. Todo pasa por el git del sistema, minimo 2.34, con deteccion de funciones.

---

## 3. Bugs del prototipo que NO hay que copiar

Son atajos del prototipo o errores de logica. Al implementar se hace lo de la ultima columna.

### Estado y datos

| Id   | Bug                                                                                                                                                | Donde            | Comportamiento correcto                                                             |
| ---- | -------------------------------------------------------------------------------------------------------------------------------------------------- | ---------------- | ----------------------------------------------------------------------------------- |
| B-01 | Estado de repositorio global: todas las pestanas muestran el mismo grafo, staging, conflicto y stashes                                             | Desktop          | Una sesion por repositorio (por pestana); cambiar de pestana cambia todo el estado. |
| B-02 | Arranca dentro de un conflicto; el siguiente merge falla o no segun un interruptor de demo (`conflictUsed`); en VS Code el merge simula conflictos | Desktop, VS Code | Estado inicial leido del repo; el resultado del merge lo decide git.                |
| B-03 | Datos fijos: el mismo diff para todos los archivos, archivos de commit elegidos por indice de fila, URLs y enlaces de PR escritos a mano           | Desktop, VS Code | Todo sale del motor.                                                                |
| B-04 | Estrategia de merge (ff, no-ff, squash) y "editar mensaje" se recogen y se ignoran; la opcion de publicar tag y "mover a la papelera" tambien      | Desktop          | Cada opcion del formulario tiene efecto o no se muestra.                            |
| B-05 | Filtro "Local / Remoto" calculado segun la cuenta de Ramal enlazada                                                                                | Desktop          | Segun si el repo tiene remoto en git.                                               |
| B-06 | El panel web muestra rutas locales de repos de otros dispositivos                                                                                  | Web              | Sin rutas de otros equipos: identidad remota y nombres de dispositivo.              |

### Commit y mensajes

| Id   | Bug                                                                                                                                  | Donde            | Comportamiento correcto                                                                                 |
| ---- | ------------------------------------------------------------------------------------------------------------------------------------ | ---------------- | ------------------------------------------------------------------------------------------------------- |
| B-07 | El commit usa solo la primera linea del mensaje (`msg.split('\n')[0]`) y pierde el cuerpo que genera la IA                           | VS Code          | Mensaje completo: resumen, linea en blanco, cuerpo.                                                     |
| B-08 | El commit ignora la descripcion                                                                                                      | Desktop          | Igual que B-07.                                                                                         |
| B-09 | El contador de 72 caracteres cuenta unidades UTF-16                                                                                  | Ambos            | Contar grafemas (`Intl.Segmenter`): una tilde o un emoji cuentan uno.                                   |
| B-10 | El resumen de IA se "genera" al instante para cada commit seleccionado, sin carga ni error                                           | Desktop, VS Code | Bajo demanda, cacheado por sha, con estados de cargando y error; cada llamada cuesta dinero al usuario. |
| B-11 | Rama destino del PR adivinada con una heuristica de gitflow (`develop` o `main`); "Merge..." de la paleta fija `develop` como origen | Desktop, VS Code | Destino = rama por defecto del repo en la plataforma; origen elegido por el usuario.                    |
| B-12 | Mensaje de cherry-pick `-x` en la misma linea (`(cherry picked from <sha>)`)                                                         | Desktop          | El formato de git: linea nueva `(cherry picked from commit <sha completo>)`.                            |

### Grafo y diff

| Id   | Bug                                                                                                                                                                        | Donde            | Comportamiento correcto                                                                                                                                                             |
| ---- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ---------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| B-13 | Carriles escritos a mano en los datos; el algoritmo de la muestra `lanes.ts` pierde carriles cuando dos hijos comparten padre y siempre anade los padres de merge al final | ramal-data.js    | `assignLanes` puro en el motor: libera ranuras convergentes, reutiliza ranuras libres para padres de merge, recorta, y fija la punta de la rama por defecto; otras puntas, despues. |
| B-14 | Color `LANE[lane]` sin modulo: a partir del carril 6 no hay color                                                                                                          | Desktop, VS Code | Color = carril modulo el tamano de la paleta.                                                                                                                                       |
| B-15 | Las aristas de merge pueden cruzar nodos (tramo vertical en el carril final del padre)                                                                                     | Desktop, VS Code | Enrutar por el carril donde vivia la reserva (`travelLane`): curva, vertical, curva.                                                                                                |
| B-16 | Aristas a padres fuera de las filas cargadas se omiten                                                                                                                     | Desktop, VS Code | Se dibujan hasta el borde inferior, como una rama que sigue.                                                                                                                        |
| B-17 | Diff unificado que intercala `-` y `+` linea a linea                                                                                                                       | Desktop          | Hunks como git: bloque de lineas quitadas y despues bloque de lineas anadidas.                                                                                                      |
| B-18 | Diff del staging etiquetado `(HEAD)` y diff de un archivo de commit etiquetado `(Working Tree)`                                                                            | VS Code          | `(Indice)` para lo que esta en staging; sha corto del commit para archivos de commit.                                                                                               |

### Resolvedor de conflictos

| Id   | Bug                                                                                                                                            | Donde   | Comportamiento correcto                                                                                                        |
| ---- | ---------------------------------------------------------------------------------------------------------------------------------------------- | ------- | ------------------------------------------------------------------------------------------------------------------------------ |
| B-19 | Elegir la misma linea en los dos lados la duplica en el resultado                                                                              | Ambos   | Pedir a git marcadores `zdiff3` para que las lineas comunes salgan del hunk, y avisar si aun asi hay duplicados.               |
| B-20 | Un hunk no se puede resolver a vacio: con un lado vacio el boton dice "Quitar bloque" y no hace nada                                           | Ambos   | Decision explicita por hunk (`ninguna`, `lineas`, `vacio`); "vacio" cuenta como resuelto.                                      |
| B-21 | La deteccion de marcadores ignora `\|\|\|\|\|\|\|` y marca como conflicto lineas legitimas que empiezan por `=======` (subrayados de Markdown) | Ambos   | Incluir el marcador base y permitir "Marcar como resuelto igualmente".                                                         |
| B-22 | El texto editado a mano se pierde al cambiar de archivo; tocar una eleccion descarta la edicion sin avisar                                     | Desktop | Borrador por archivo; pedir confirmacion antes de descartar una edicion manual.                                                |
| B-23 | Etiquetas fijas: A = tu rama (ours), B = entrante (theirs)                                                                                     | Ambos   | Etiquetas segun la operacion: en un rebase "ours" es la base sobre la que se reconstruye y "theirs" el commit que se reaplica. |

### Rebase interactivo

| Id   | Bug                                                                                                                        | Donde   | Comportamiento correcto                                                                                                     |
| ---- | -------------------------------------------------------------------------------------------------------------------------- | ------- | --------------------------------------------------------------------------------------------------------------------------- |
| B-24 | La lista de commits se arma siguiendo primeros padres mientras el carril no cambia, maximo 8                               | Desktop | Rango `merge-base(onto, HEAD)..HEAD` sin merges ni parches ya en upstream.                                                  |
| B-25 | "Iniciar rebase" solo aplica drop y reword; ignora el orden, squash, fixup y edit; drop borra commits sin reenlazar padres | Desktop | `git rebase -i` real con el todo generado; validar antes (el primero no puede ser squash o fixup, reword necesita mensaje). |

### Acciones de Git

| Id   | Bug                                                                                                                              | Donde            | Comportamiento correcto                                                                   |
| ---- | -------------------------------------------------------------------------------------------------------------------------------- | ---------------- | ----------------------------------------------------------------------------------------- |
| B-26 | "Push" del menu de una rama empuja siempre la rama actual; "Pull" de rama solo muestra un toast; "Comparar" abre un archivo fijo | Desktop, VS Code | Actuar sobre la rama del menu.                                                            |
| B-27 | Borrar rama borra refs de cualquier tipo con ese nombre; el borrado de rama remota usa la etiqueta de borrado local              | Desktop, VS Code | Borrar solo el tipo de ref elegido; el borrado remoto se llama y se confirma como remoto. |
| B-28 | Las pildoras de tag abren el menu de rama                                                                                        | VS Code          | Menu de tag.                                                                              |
| B-29 | Reset --hard, descartar stash, borrar tag, desconectar cuenta y borrar llave sin confirmacion                                    | Desktop, VS Code | Confirmacion de peligro en toda accion destructiva.                                       |
| B-30 | El nombre del repo al clonar falla con URLs `git@host:repo.git` y con barra final                                                | Desktop          | Analizar la URL remota (https, ssh, estilo scp, `owner/repo`, rutas de Azure).            |
| B-31 | Nombre de rama nueva: solo `trim` y espacios a guiones                                                                           | Desktop          | Validar con las reglas de refs de git (`check-ref-format`).                               |
| B-32 | El contador de la seccion Remotos ignora el filtro                                                                               | Desktop          | Contar lo filtrado.                                                                       |
| B-33 | La barra de estado muestra `0` detras fijo y el `*` ignora cambios solo en staging                                               | VS Code          | Valores reales de ahead, behind y cambios.                                                |

### Cuentas y llaves

| Id   | Bug                                                                                                                  | Donde            | Comportamiento correcto                                                                                                                                                                  |
| ---- | -------------------------------------------------------------------------------------------------------------------- | ---------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| B-34 | La mascara muestra entera cualquier llave de menos de 11 caracteres; el prefijo es de 7 en Desktop y de 6 en VS Code | Desktop, VS Code | Menos de 12 caracteres: mascara completa. Si no: prefijo conocido (`sk-ant-`, `sk-proj-`...) mas los 4 ultimos. Se calcula en el motor o la API; la llave en claro nunca vuelve a la UI. |
| B-35 | Editar una cuenta resetea el metodo a OAuth e ignora el token nuevo                                                  | Desktop          | Conservar el metodo y actualizar el token.                                                                                                                                               |
| B-36 | Quitar la cuenta predeterminada no promueve otra                                                                     | Desktop, VS Code | Promover la primera que quede de esa plataforma.                                                                                                                                         |
| B-37 | Campo de token con `type=text`                                                                                       | Web, Desktop     | Campo de secreto oculto, que nunca vuelve a mostrar el valor guardado.                                                                                                                   |
| B-38 | Hosts por defecto distintos en Desktop, web y VS Code; el placeholder de host siempre dice `github.com`              | Los tres         | Una sola tabla de plataformas en `@ramal/domain`.                                                                                                                                        |
| B-39 | Punto de estado de la cuenta siempre verde                                                                           | Desktop          | Estado real: ultima comprobacion, caducidad, error.                                                                                                                                      |
| B-40 | La barra de estado dice `Ramal: @marta-dev`, que es un usuario de GitHub                                             | VS Code          | Identidad de Ramal.                                                                                                                                                                      |

### Textos e i18n

| Id   | Bug                                                                                                                | Donde              | Comportamiento correcto                                                               |
| ---- | ------------------------------------------------------------------------------------------------------------------ | ------------------ | ------------------------------------------------------------------------------------- |
| B-41 | Cadenas construidas editando otras traducciones (`tr('mCopySha').replace('SHA', tr('name'))` da "Copiar Nombre")   | Desktop            | Claves propias: `copyName`, `copyEmail`, `copyModel` y claves `*Title` separadas.     |
| B-42 | Sin plurales: 13 claves con contador producen "1 archivos", "1 conflictos"; `conflictsT` fija "2 archivos"         | ramal-data.js      | Formas plurales con `Intl.PluralRules` en `@ramal/i18n`.                              |
| B-43 | Glifos y atajos de Mac dentro de las cadenas (la tecla Cmd de Mac en `msgPh`, marcas de check y la estrella de IA) | ramal-data.js      | Glifos en los componentes; el atajo como parametro segun el sistema operativo.        |
| B-44 | Marcadores ambiguos: `{m}` es total, modo o modelo segun la clave                                                  | ramal-data.js      | Marcadores con nombre y tipados.                                                      |
| B-45 | Dos diccionarios (`D` y `W`) que chocan en 27 claves, 8 con texto distinto                                         | ramal-data.js, Web | Espacios de nombres en `@ramal/i18n`.                                                 |
| B-46 | Datos fijos o "(simulado)" dentro de textos (`planRenew`, `signedOutT`, `W.dlStarted`, `W.resetSent`, `W.version`) | Todos              | Parametros; ningun texto de producto dice "simulado".                                 |
| B-47 | Falta la clave `commitV`; VS Code ofrece frances sin traduccion                                                    | VS Code            | Anadir la clave; los idiomas ofrecidos son los disponibles, con ingles como respaldo. |
| B-48 | Tiempo relativo: "hace 0 min", 1439 minutos se muestra "24 h", sin semanas ni meses, fuera del diccionario         | Desktop, Web       | `Intl.RelativeTimeFormat` con umbrales probados en los bordes.                        |
| B-49 | La zona horaria solo sustituye el primer guion bajo (`replace` en vez de `replaceAll`)                             | Desktop            | Sustituir todos.                                                                      |
| B-50 | "Propietaria" como etiqueta del rol owner                                                                          | Web                | Etiqueta neutra ("Titular").                                                          |

### Web y modales

| Id   | Bug                                                                                                                                                                                                               | Donde   | Comportamiento correcto                                                                                                 |
| ---- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ------- | ----------------------------------------------------------------------------------------------------------------------- |
| B-51 | Invitar por encima de los asientos los sube en silencio; "Anadir asientos" suma uno sin precio ni confirmacion                                                                                                    | Web     | Bloquear con W-15; comprar asientos con precio y confirmacion.                                                          |
| B-52 | Las invitaciones pendientes cuentan como miembros en los contadores                                                                                                                                               | Web     | Contadores separados: "4 miembros y 1 invitacion pendiente, 5 de 5 asientos".                                           |
| B-53 | El minimo de 8 caracteres se aplica tambien al iniciar sesion                                                                                                                                                     | Web     | Solo en el registro; al entrar basta con que no este vacia.                                                             |
| B-54 | El nombre de organizacion del registro se descarta; el registro con OAuth se salta los terminos                                                                                                                   | Web     | Guardarlo; paso de terminos tambien con OAuth (W-04).                                                                   |
| B-55 | Android e iOS detectados como macOS; la tarjeta de VS Code nunca se destaca; sistema desconocido pasa a macOS                                                                                                     | Web     | Estado movil (W-30); sistema desconocido muestra todas las tarjetas sin destacar ninguna.                               |
| B-56 | "Local, MacBook Pro" fijo; sin marca de dispositivo actual; cerrar sesion en un dispositivo sin confirmar                                                                                                         | Web     | Datos reales, marca "este navegador", confirmacion (W-22).                                                              |
| B-57 | Ajustes que mezclan toggles instantaneos con un "Guardar" global                                                                                                                                                  | Web     | Un solo modelo: guardado inmediato por control, con la version del documento para detectar cambios de otro dispositivo. |
| B-58 | "Crear cuenta" y las paginas de autenticacion siguen visibles con sesion iniciada; "Descargar" del panel saca del panel                                                                                           | Web     | Redirigir al panel; "Abrir panel" en la landing; descargas sin salir del panel.                                         |
| B-59 | La confirmacion de borrar workspace muestra "Sin repositorios" (clave equivocada); el buscador de repos promete buscar por cuenta y no lo hace; faltan toasts de exito (crear workspace, rol cambiado, renombrar) | Web     | Clave correcta, busqueda por usuario de la cuenta, confirmacion visible de cada accion.                                 |
| B-60 | Los modales validan en silencio y el texto informativo con `\n` se junta en una linea                                                                                                                             | Todos   | Errores en linea (C-03) y saltos de linea respetados.                                                                   |
| B-61 | El toast `info` usa el color `--accent` mientras el README dice `--info`                                                                                                                                          | Desktop | `--info` para informacion; `--accent` queda para la accion principal y la marca.                                        |
| B-62 | El atajo `Mayus+Cmd+G` choca con VS Code                                                                                                                                                                          | VS Code | Atajo que no choque (X-11).                                                                                             |

---

## 4. Decisiones abiertas para Ezequiel

Solo lo que bloquea diseno o backlog y no se puede decidir desde el codigo. Ezequiel resolvio las
ocho el 2026-10-01; quedan aqui con su resumen y el ADR que las desarrolla.

| Id   | Decision                                                                                                               | Resuelta                                                                                                                          | Resumen                                                                                                                                                                                                                                                 |
| ---- | ---------------------------------------------------------------------------------------------------------------------- | --------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Q-01 | Equipos y facturacion en la primera version publica, o despues                                                         | 2026-10-01, [ADR 0026](../adr/0026-planes-y-precios.md)                                                                           | Todo dentro: organizaciones, equipo, invitaciones, asientos, facturacion y llaves de IA de organizacion con el proxy. E27 a E30 pasan al MVP, y W-11 a W-18 y D-47 son del MVP.                                                                         |
| Q-02 | Proveedor de pagos                                                                                                     | 2026-10-01, [ADR 0022](../adr/0022-proveedor-de-pagos.md)                                                                         | Paddle como comerciante registrado (plan B: Polar), detras de un puerto `BillingProvider`. Checkout en una pagina de la web con Paddle.js y webhook en `/v1/billing/webhooks/paddle`.                                                                   |
| Q-03 | Que es gratis y que es Ramal Pro, y precio por asiento                                                                 | 2026-10-01, [ADR 0026](../adr/0026-planes-y-precios.md)                                                                           | Pagan los equipos. Gratis para siempre: todo Git local, PRs, IA con llave propia y la sincronizacion personal. Ramal Pro es el plan de una organizacion: 8 USD por asiento al mes en anual o 10 en mensual, 14 dias de prueba sin tarjeta.              |
| Q-04 | Windows ARM64 y Linux ARM64 en el lanzamiento                                                                          | 2026-10-01, [ADR 0016](../adr/0016-actualizaciones-y-descargas.md) y [ADR 0010](../adr/0010-un-motor-para-escritorio-y-vscode.md) | Ratificada la matriz entera y los ocho VSIX. Windows arm64 y Linux arm64 salen como beta hasta que su E2E pase en runners arm; Windows arm64 solo lleva NSIS.                                                                                           |
| Q-05 | Ciudad en la lista de sesiones (GeoIP) o solo dispositivo y ultima actividad                                           | 2026-10-01, [api-surface.md](../api-surface.md#sesiones-activas) y [ADR 0008](../adr/0008-autenticacion-con-tokens.md)            | Sin ubicacion: dispositivo, sistema, app o navegador y ultima actividad. Sin GeoIP, y la pagina del device flow pierde el aviso de pais en la v1.                                                                                                       |
| Q-06 | Modelos de IA recomendados por defecto y nivel de coste para mensajes de commit                                        | 2026-10-01, [ADR 0025](../adr/0025-recomendacion-de-modelos-de-ia.md)                                                             | Fichero remoto publico servido por la API, cruzado con la lista viva del proveedor. Rapido para commits, equilibrado para PRs y resumenes, razonamiento al minimo. Modelo por funcion; con llaves de organizacion lo fija el admin. Sin coste en la v1. |
| Q-07 | Windows Portable: quitarlo o mantenerlo sin actualizaciones automaticas                                                | 2026-10-01, [ADR 0016](../adr/0016-actualizaciones-y-descargas.md)                                                                | Ratificada: sin Portable. NSIS se actualiza solo; el MSI x64 para IT sale en el lanzamiento con "sin actualizaciones automáticas", por decision y no por un limite de Tauri.                                                                            |
| Q-08 | Repositorio de releases publico (`ramal-releases`) o almacenamiento de objetos, y si el repo del producto sera publico | 2026-10-01, [ADR 0016](../adr/0016-actualizaciones-y-descargas.md)                                                                | Sin repositorio aparte ni organizacion: se publica en `eabl0306/ramal`, privado hasta la beta y publico desde entonces. El manifiesto lo sirve la API, con `{{bundle_type}}` en la URL.                                                                 |

El mismo dia se decidieron dos cosas que tocan a este documento sin ser filas suyas: el despliegue y
el dominio (`ramal.bahoque.com` y `api.ramal.bahoque.com`,
[ADR 0023](../adr/0023-despliegue-en-apps-01-y-dominio.md)) y el correo
([ADR 0024](../adr/0024-correo-transaccional-con-resend.md)).

### Lo que sigue abierto

- **Descuento de lanzamiento, Ramal Pro gratis para codigo abierto y educacion, y precios
  regionales.** No bloquean el MVP.
- **Que pasa al acabar la prueba sin pagar, o cuando falla un pago.** La propuesta es que la
  organizacion quede en solo lectura y sus datos se conserven 30 dias. Lo fija E30-01 antes de
  construir los limites del plan.
- **Pais y forma (persona fisica o empresa) desde la que factura Ezequiel.** Se confirma al abrir la
  cuenta de Paddle. De ahi dependen tambien la via de firma de Windows y el nombre que ensena el
  certificado de macOS.
- **Que buzon real recibe las respuestas al correo y los informes de DMARC.** Sin el, `Reply-To` no
  se fija y DMARC no pasa de `p=none`.
- **Otra cuenta de Ramal en el mismo equipo**: si se le pregunta antes de subir lo creado alli o no
  se le ofrece nunca. Lo seguro (no subirlo solo) ya esta decidido.
- **Como llegan las builds a la beta privada** mientras el repositorio sea privado: las descargas de
  GitHub Releases de un repositorio privado no son publicas, y el actualizador no puede leerlas.
- **El diseno de lo que falta** (issues #62 a #71). Ezequiel lo disena en Claude Design, en el mismo
  proyecto del handoff original, y trae un handoff nuevo.

### Issues de seguimiento que salen de estas decisiones

- Hecho el 2026-10-01: el codigo ya no usa `ramal.dev`. El prefijo de los `type` de los errores
  (`apps/api/crates/http/src/problem.rs`), `apps/api/openapi.json`, sus tests y
  `@ramal/api-client` usan `https://ramal.bahoque.com/problems/`.
- Aceptar en el arranque de la API una base por delante del binario (`set_ignore_missing`), para que
  una vuelta atras que cruce una migracion no deje la API reintentando en bucle
  ([ADR 0023](../adr/0023-despliegue-en-apps-01-y-dominio.md)). Issue #517 (E26-16).
- Pasar `ramal.engine.path` y `ramal.engine.logLevel` a `scope: "machine"` en
  `apps/vscode/package.json` ([ADR 0010](../adr/0010-un-motor-para-escritorio-y-vscode.md)). Va en
  el issue #301.
