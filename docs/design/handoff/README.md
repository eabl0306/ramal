# Handoff de diseno de Ramal

Prototipos de Claude Design de los tres productos: Ramal Desktop, Ramal for VS Code y Ramal Web.
Son la fuente de verdad visual. Donde el texto de este README y un prototipo no coinciden, manda el
prototipo.

Esta carpeta es el handoff consolidado que llego el 2026-10-01 ("Cliente Git estilo GitKraken.zip").
Sustituye al handoff original de tres prototipos, que sigue en el historial de git (commit
`df1666a`). Trae los tres prototipos base corregidos (#71) y once prototipos nuevos, uno o varios
por cada issue de diseno de la epica #2 (#62 a #71). El porque de cada pantalla esta en
[`../gaps.md`](../gaps.md) y lo que se pidio, en [`../briefs/`](../briefs/README.md).

## Indice

| Fichero                        | Producto               | Issue          | Props y valores                                                                                                                                                                                                            | Pantallas o estados                                       |
| ------------------------------ | ---------------------- | -------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | --------------------------------------------------------- |
| `Ramal.dc.html`                | Desktop                | base, #71      | `theme` (dark, light), `lang` (es, en), `graphNodes` (avatar, dot)                                                                                                                                                         | App interactiva sin prop de pantalla: se navega con clics |
| `Ramal VS Code.dc.html`        | VS Code                | base, #68, #71 | `vsTheme` (dark, light), `vsLanguage` (en, es), `state` (14 valores)                                                                                                                                                       | 14 estados                                                |
| `Ramal Web.dc.html`            | Web                    | base, #70, #71 | `theme`, `lang`, `startPage` (landing, login, signup), `os` (mac, win, linux, mobile, unknown), `downloads` (normal, api-caida, sin-version)                                                                               | 3 paginas de entrada                                      |
| `Ramal Estados.dc.html`        | Los tres               | #62            | `product` (desktop, vscode, web), `theme`, `lang`, `state` (all, C-01 a C-10)                                                                                                                                              | 10 laminas; `all` las apila                               |
| `Ramal Boveda.dc.html`         | Desktop, VS Code y web | #63            | `product` (desktop, vscode, web), `theme`, `lang`, `screen` (K-01 a K-10, X-04)                                                                                                                                            | 11 pantallas mas la rotacion (boton ROT)                  |
| `Ramal Entrada.dc.html`        | Desktop                | #64            | `theme`, `lang`, `os` (macos, windows, linux), `session` (signedOut, signedIn), `screen` (28 valores)                                                                                                                      | 28 pantallas                                              |
| `Ramal Git.dc.html`            | Desktop                | #65            | `theme`, `lang`, `density` (desktop, vscode), `screen` (33 valores)                                                                                                                                                        | 33 pantallas                                              |
| `Ramal Vistas.dc.html`         | Desktop                | #66            | `theme`, `lang`, `density` (desktop, vscode), `screen` (26 valores)                                                                                                                                                        | 26 pantallas                                              |
| `Ramal Ajustes.dc.html`        | Desktop                | #67            | `theme`, `lang`, `os` (macos, windows, linux), `session` (signedIn, signedOut), `role` (personal, member, admin, owner), `install` (dmg, nsis, msi, appimage, deb, rpm), `screen` (25 valores)                             | 25 pantallas                                              |
| `Ramal Correcciones.dc.html`   | Los tres               | #71            | Ninguna. Solo en castellano y pensada en oscuro: abrela con `?theme=dark`                                                                                                                                                  | 1 pagina con 11 grupos                                    |
| `Ramal Web Auth.dc.html`       | Web                    | #69            | `theme`, `lang`, `screen` (15 valores), `state` (texto libre, el subestado), `client` (desktop, vscode)                                                                                                                    | 15 pantallas y 92 subestados                              |
| `Ramal Web Panel.dc.html`      | Web                    | #70            | `theme`, `lang`, `screen` (15 valores), `context` (personal, org), `role` (owner, admin, member), `plan` (none, trialing, active, past_due, cancel_scheduled, canceled, inactive), `state` (normal, empty, loading, error) | 15 pantallas, mas variantes por chips                     |
| `Ramal Web Paginas.dc.html`    | Web                    | #70            | `theme`, `lang`, `screen` (10 valores), `state` (texto libre, el subestado)                                                                                                                                                | 10 pantallas y 35 subestados                              |
| `Ramal Local Callback.dc.html` | Motor (pagina local)   | #69            | `kind` (signin, platform), `client` (desktop, vscode), `result` (success, cancelled, error), `platform` (GitLab, Bitbucket), `lang`. El tema sale de `prefers-color-scheme`                                                | 12 combinaciones                                          |

En total, 204 pantallas o estados en el eje principal y 351 contando los subestados de Web Auth, Web
Paginas y Web Panel. Ademas:

- `ramal-data.js`: datos de muestra y todos los textos en castellano e ingles. Al diccionario `D`
  original se suman las claves nuevas de Desktop (prefijos `e_`, `g_`, `v_` y `a_`) y cinco objetos:
  `STATES` (#62), `VAULT` (#63), `VS` (#68, claves `vscode.*`), `WAUTH` (#69, claves `auth.*`) y
  `WEB70` (#70, espacios `dashboard`, `team`, `orgAi`, `billing`, `pricing`, `account`, `legal` y
  `download`). No se borro ninguna clave original.
- `support.js`: el runtime que pinta los prototipos. Igual byte a byte que el original.
- `local-callback.template.html`: la pagina estatica que sirve el receptor local del motor, sin
  JavaScript ni recursos externos (CSP `default-src 'none'`).
- `img/`: capturas de la landing (`landing-{desktop,vscode}-{dark,light}.png`, 924x540).
- `media/`: imagenes del recorrido de bienvenida de VS Code
  (`walkthrough-{1-grafo,2-plataforma,3-ia,4-cuenta}-{dark,light}.svg`, 480x300, sin texto). Van con
  los pasos `vscode.walkthrough.step1` a `step4` de `ramal-data.js`.

## Como abrirlos

1. Sirve la carpeta por HTTP y abre el fichero en el navegador:

   ```bash
   cd docs/design/handoff && python3 -m http.server 8000
   # http://localhost:8000/Ramal%20Estados.dc.html
   ```

   Con `file://` tambien abren, pero Chromium bloquea la lectura que `support.js` hace del propio
   fichero y el resultado puede variar.

2. Hace falta red: `support.js` carga React y Babel desde unpkg, casi todos cargan IBM Plex desde
   Google Fonts y `Ramal VS Code.dc.html` carga los Codicons desde jsDelivr.

3. Los props. `support.js` no los lee de la URL: cada prototipo arranca con los valores por defecto
   de su atributo `data-props`. De la query solo lee `theme`, y solo para el color del lienzo. Para
   cambiar los props:
   - En Claude Design, con el panel de props.
   - En el navegador, desde la consola:

     ```js
     __dcSetProps(__dcRootName(), { screen: 'K-03', theme: 'light', lang: 'en' });
     ```

   - Con la query: abre `...dc.html?screen=K-03&theme=light&lang=en` y pega en la consola
     `__dcSetProps(__dcRootName(), Object.fromEntries(new URLSearchParams(location.search)))`.

   `Ramal Web Auth.dc.html` y `Ramal Web Paginas.dc.html` leen ademas la pantalla del hash
   (`#W-01`, `#W-27-terminos`), y asi se enlazan entre ellos y con `Ramal Web.dc.html`.

4. Muchos prototipos traen arriba una barra propia con chips de pantalla, tema, idioma y variante, y
   los flujos se recorren con clics. Las variantes de `Ramal Web Panel.dc.html` (por ejemplo
   `email-sin-confirmar` en W-12 o `checkout-202` en W-18) solo se eligen con esos chips.
   `Ramal.dc.html` no tiene prop de pantalla: el resolvedor se abre con el merge del menu de la rama
   `hotfix/login` y los ajustes con el engranaje.

5. Algunos estados avanzan solos con un temporizador (1 a 4 s): los de carga de Web Paginas y Web
   Panel y el progreso de `d17-progreso` en Git. Para capturarlos, haz la captura enseguida.

## Que se diseno en cada issue

El zip traia un README de indice que remite a los README de cada handoff (`ramal-handoff-NN-*`), con
el detalle de pantallas, estados y claves. Esos README no venian en el zip entregado. Esta seccion
los sustituye con lo que se lee en los prototipos y en la revision del 2026-10-02 (cada pantalla y
estado renderizado en claro y oscuro, castellano e ingles, sin errores de pagina). El estado fila a
fila esta en la columna "Diseno" de [`../gaps.md`](../gaps.md). Los valores de `screen` empiezan por
el id de `gaps.md` en minusculas: `d12-token` es D-12.

### #62 Estados comunes: `Ramal Estados.dc.html`

Una lamina por estado, con `product` para la densidad (Desktop y web 40 px, VS Code 28 px) y, en VS
Code, una caja "Nativo de VS Code: se especifica, no se dibuja" con la notificacion, el quick input o
el `viewsWelcome` que toca.

| Valor  | Id   | Que ensena                                                                                           |
| ------ | ---- | ---------------------------------------------------------------------------------------------------- |
| `C-01` | C-01 | Esqueletos de repos, grafo, PRs, filas de ajustes y detalle; "Cargando más…" y el aviso de que tarda |
| `C-02` | C-02 | Toasts ok, info, aviso y error; dialogo de error plegado y desplegado con "Copiar detalle"           |
| `C-03` | C-03 | Validacion en linea: vacio, ayuda, error de campo, varios errores, comprobando y error del servidor  |
| `C-04` | C-04 | Vacios en barra lateral, panel y area grande, y busquedas sin resultados                             |
| `C-05` | C-05 | Sin conexion y API caida: chip en la barra, copia guardada de la nube, banner web y login            |
| `C-06` | C-06 | Indicador de sincronizacion, cambio desde otro dispositivo y guardado por control                    |
| `C-07` | C-07 | Sesion caducada (`session_ended` con limpieza de secretos) y otro 401 sin limpieza                   |
| `C-08` | C-08 | Solo lectura por rol, 403 y 404                                                                      |
| `C-09` | C-09 | Tema Sistema, Claro u Oscuro, conmutador rapido y aviso de idioma detectado                          |
| `C-10` | C-10 | Matriz de 14 controles en reposo, hover, foco, pulsado y deshabilitado                               |

Tambien corrige B-61 en `Ramal.dc.html` (toast de informacion en `--info`).

Pendiente: el error en textarea y en select de C-03; los bordes de las cajas de error y aviso mezclan
el color con `--border` en oklch y salen violeta y verdoso (pasa tambien en Boveda, Git, Ajustes,
Entrada y Vistas); tokens nuevos fuera de la tabla (`--sk-hi`, `--danger-ink`).

### #63 Boveda: `Ramal Boveda.dc.html`

| Valor  | Id   | Que ensena                                                                                            |
| ------ | ---- | ----------------------------------------------------------------------------------------------------- |
| `K-01` | K-01 | Crear la frase: medidor, reglas, "Ahora no", sin conexion y 409                                       |
| `K-02` | K-02 | Codigo de recuperacion: copiar, descargar, casilla, error al subir y exito                            |
| `K-03` | K-03 | Desbloquear con la frase o con el codigo, con errores, 429 y llave cambiada                           |
| `K-04` | K-04 | Listas con cuentas y llaves bloqueadas, "Probar conexión" desactivado y estados de la tarjeta Boveda  |
| `K-05` | K-05 | "Solo en este equipo", sin sesion, otra cuenta de Ramal y subida pendiente de decidir (ADR 0012)      |
| `K-06` | K-06 | Cambiar la frase, con la pregunta de si alguien la conoce                                             |
| `K-07` | K-07 | Regenerar el codigo                                                                                   |
| `K-08` | K-08 | "Olvidé la frase": otro equipo, codigo y reinicio en la web                                           |
| `K-09` | K-09 | Cerrar sesion: que se borra y que se queda, con la casilla apagada                                    |
| `K-10` | K-10 | Web: tarjeta Boveda y restablecer con confirmacion literal                                            |
| `X-04` | X-04 | Interruptores de secretos en web y Desktop con el estado de la boveda y el aviso de peligro al apagar |

El boton ROT de la barra recorre la rotacion de la llave. Los textos X-03 son `VAULT.x3_1` a `x3_9`.
En VS Code (`product=vscode`) los modales son paneles con "← Volver".

Pendiente: en K-05, "Sin bóveda" y "Categoría apagada" pintan una boveda bloqueada; K-04 sin la fila
de token rotado, permisos ni "Usada en N repos"; en VS Code, el aviso nativo de K-01, la notificacion
de K-04 y las filas de entidad.

### #64 Entrada y ciclo de vida del repo: `Ramal Entrada.dc.html`

| Ids  | Valores de `screen`                                                      |
| ---- | ------------------------------------------------------------------------ |
| D-01 | `d01-cuenta`, `d01-identidad`, `d01-repos`, `d01-plataformas`, `d01-ia`  |
| D-02 | `d02-ausente`, `d02-antiguo` (con `os` cambia el comando de instalacion) |
| D-03 | `d03-navegador`, `d03-codigo`, `d03-error`                               |
| D-04 | `d04-sin-sesion`                                                         |
| D-05 | `d05-vacio`, `d05-solo-nube`                                             |
| D-06 | `d06-lista`, `d06-abrir-todos`                                           |
| D-07 | `d07-tarjeta`, `d07-vista`                                               |
| D-08 | `d08-formulario`, `d08-progreso`, `d08-error`                            |
| D-09 | `d09-no-repo`, `d09-confianza`, `d09-solo-lectura`, `d09-iniciar`        |
| D-10 | `d10-vacio`                                                              |
| D-11 | `d11-formulario`, `d11-progreso`, `d11-error`                            |

Cada pantalla tiene chips de variante (por ejemplo los siete errores de clonado de `d08-error`).
`session=signedIn` cambia la barra de titulo y las zonas de nube.

Pendiente: D-03 dentro de la tarjeta de D-01; D-05 a 1024 px (la variante es igual a la de 1280); la
tarjeta de rejilla de D-06.

### #65 Credenciales y operaciones de Git: `Ramal Git.dc.html`

| Ids  | Valores de `screen`                                                                                                                                    |
| ---- | ------------------------------------------------------------------------------------------------------------------------------------------------------ |
| D-12 | `d12-plataforma`, `d12-github-codigo`, `d12-navegador`, `d12-token`, `d12-azure`, `d12-exito`, `d12-error-permisos`, `d12-error-sso`, `d12-actualizar` |
| D-13 | `d13-https`, `d13-ssh` (con la variante GPG, que no dibuja dialogo), `d13-huella`, `d13-huella-cambiada`                                               |
| D-14 | `d14-tarjetas`, `d14-banner`                                                                                                                           |
| D-15 | `d15-submenu`                                                                                                                                          |
| D-17 | `d17-progreso`                                                                                                                                         |
| D-18 | `d18-push-rechazado`, `d18-index-lock`, `d18-hook`, `d18-arbol-sucio`, `d18-firma`                                                                     |
| D-19 | `d19-pull`, `d19-rebase`, `d19-cherry-pick`, `d19-revert`, `d19-stash` (con `density=vscode`, el resolvedor en filas de 28 px)                         |
| D-20 | `d20-edit`                                                                                                                                             |
| D-21 | `d21-squash`                                                                                                                                           |
| D-22 | `d22-merge`                                                                                                                                            |
| D-23 | `d23-head-separado`                                                                                                                                    |
| D-24 | `d24-upstream`                                                                                                                                         |
| D-25 | `d25-historial`                                                                                                                                        |

Pendiente: los botones "Abortar" no piden confirmacion de peligro; los titulos de confirmacion se
arman concatenando texto (en ingles sale "¿Remove lock and retry?"); detalles de D-15 (menu de la
tarjeta), D-17 ("cancelando"), D-20 (anillo del commit) y D-23 (checkout desde el menu de commit).

### #66 Vistas de los menus y diff real: `Ramal Vistas.dc.html`

| Ids  | Valores de `screen`                                                                                    |
| ---- | ------------------------------------------------------------------------------------------------------ |
| D-27 | `d27-historial`                                                                                        |
| D-28 | `d28-blame`                                                                                            |
| D-29 | `d29-comparar`                                                                                         |
| D-30 | `d30-pr-abierto`, `d30-pr-conflictos`, `d30-pr-fusionado`                                              |
| D-31 | `d31-nuevo-pr`, `d31-sin-publicar`, `d31-exito`                                                        |
| D-32 | `d32-remotos`                                                                                          |
| D-33 | `d33-stash`                                                                                            |
| D-35 | `d35-lineas`                                                                                           |
| D-36 | `d36-bloques`, `d36-renombrado`, `d36-binario`, `d36-imagen`, `d36-enorme`, `d36-lfs`, `d36-submodulo` |
| D-37 | `d37-staging`, `d37-grafo`                                                                             |
| D-38 | `d38-binario`, `d38-borrado`                                                                           |
| D-40 | `d40-paginado`                                                                                         |
| D-41 | `d41-filtros`, `d41-busqueda`                                                                          |

`density=vscode` cambia D-31, D-38, D-40 y D-41, las vistas que VS Code reutiliza como webview. Los
modales (anadir y quitar remoto, eliminar stash, descartar lineas) y el modo lado a lado se abren con
clics.

Resaltado de sintaxis (funcion `tok`): comentarios en `--faint`, cadenas en `--ok`, numeros y
constantes en `--warn`, palabras clave en `--info`, tipos (con mayuscula) en `--accent`, llamadas a
funcion en `--fn` y el resto en `--text`. El cambio por palabra es el color de la linea al 28 % con
`color-mix` sobre `transparent`. `--fn` es nuevo: en oscuro es el carril 2 (`oklch(0.68 0.2 300)`) y
en claro `oklch(0.5 0.2 300)`, que no esta en la tabla de tokens. El contraste sobre `--add-bg` y
`--del-bg` no esta comprobado y en claro baja de 4,5:1.

Pendiente: los estados de carga, vacio, error y sin cuenta de D-27 a D-31; el modal de stash de los
seleccionados; varios casos de D-32, D-35, D-36 y D-38; primera carga, error, "Cargar más" y aristas
en D-40; filtros y 14 carriles en D-41; un diff distinto por archivo; plurales; el repaso de
formularios de B-04.

### #67 Ventana, ajustes y equipo: `Ramal Ajustes.dc.html`

| Ids  | Valores de `screen`                                                                            |
| ---- | ---------------------------------------------------------------------------------------------- |
| D-42 | `d42-barra` (con `os`), `d42-minimo`                                                           |
| D-43 | `d43-git`                                                                                      |
| D-44 | `d44-herramientas`                                                                             |
| D-45 | `d45-al-dia`, `d45-nueva`, `d45-lista`, `d45-paquete`, `d45-msi`, `d45-minima` (con `install`) |
| D-46 | `d46-atajos`, `d46-paleta`                                                                     |
| D-47 | `d47-cuenta`, `d47-equipo`, `d47-sin-asientos`, `d47-crear-org` (con `role` y `session`)       |
| D-48 | `d48-editar`, `d48-fetch`                                                                      |
| D-49 | `d49-clonar`, `d49-varias`                                                                     |
| D-51 | `d51-sync`                                                                                     |
| X-05 | `x05-ia`, `x05-primera-vez`                                                                    |
| X-10 | `x10-formulario`, `x10-selector`                                                               |

Trae la navegacion nueva de Ajustes: Usuario (Perfil, Cuenta, Equipo), Conexiones (Plataformas Git,
IA) y Aplicacion (Git, Herramientas externas, Apariencia, Atajos de teclado, Actualizaciones). La
barra de titulo de `d42-barra` es el bloque que copian Entrada y Git.

Pendiente: D-42 a 1024 px; casos de D-44 a D-49 y D-51 (ver `gaps.md`); la pastilla beta de arm64;
`ramal-data.js` conserva los modelos por defecto de cada proveedor y `Ramal.dc.html` el formulario
viejo de llaves con alcance (X-10).

### #68 Contribuciones de VS Code: `Ramal VS Code.dc.html`

| Valor de `state`                       | Ids                                           |
| -------------------------------------- | --------------------------------------------- |
| `normal`                               | V-11, V-12, V-14, V-18 y V-21 desde los menus |
| `V-01-sin-sesion`                      | V-01                                          |
| `V-05-reiniciando`, `V-05-detenido`    | V-05                                          |
| `V-06`                                 | V-06                                          |
| `V-07-rebase`, `V-07-cherry-pick`      | V-07                                          |
| `V-08`                                 | V-08                                          |
| `V-09-vacio`, `V-09-separado`          | V-09                                          |
| `V-15`                                 | V-15                                          |
| `V-16-ia`                              | V-16                                          |
| `V-22-sin-sesion`, `V-22-nube-apagada` | V-22                                          |

Solo se dibujan los webviews: Cambios, el grafo con su detalle, el editor de merge, Crear pull request
y Configuracion de Ramal. Lo nativo (paleta, QuickInput, modales, notificaciones, menus) sale como
tarjetas en el registro "Llamadas nativas" de la derecha, con la API que lo pinta. Todos los textos
nativos estan en `VS` de `ramal-data.js`: comandos (`vscode.command.*`, categoria Ramal), vistas,
menus, barra de estado, QuickInput, confirmaciones, recorrido y manifiesto. El atajo del grafo es
Ctrl+K Ctrl+G (Cmd+K Cmd+G en macOS), sin choque en VS Code 1.138.

Pendiente: la especificacion de contribuciones pasada a limpio (icono y `when` de cada comando,
acciones de titulo de cada vista, elementos de los arboles, cuando se ve cada elemento de la barra de
estado, esquema de `contributes.configuration` y condiciones de `viewsWelcome`). Hoy esa estructura
solo esta en el brief #68. V-02, V-03, V-04, V-10 y V-17 existen solo como textos, y sin sesion se ve
una llave de la organizacion.

### #69 Autenticacion web: `Ramal Web Auth.dc.html` y `Ramal Local Callback.dc.html`

| Valor de `screen`                                  | Id   | Subestados (`state`)                                                                                         |
| -------------------------------------------------- | ---- | ------------------------------------------------------------------------------------------------------------ |
| `W-01`                                             | W-01 | formulario, enviando, enviado, reenviar-bloqueado, email-no-valido, demasiadas, sin-conexion                 |
| `W-02`                                             | W-02 | formulario, no-coinciden, no-cumple, guardando, guardada, enlace-no-valido                                   |
| `W-03`, `W-03-cambio-email`, `W-03-deshacer-email` | W-03 | banner, reenviado, verificando, verificado, enlace-no-valido; confirmar cambio y deshacer cambio de correo   |
| `W-04-error`, `W-04-completar`                     | W-04 | conectando, cancelado, fallo, cuenta-existente, email-sin-verificar; formulario, terminos, creando, caducado |
| `W-05`                                             | W-05 | sin-sesion, reautenticar, validando, lista, autorizando, peticion-no-valida, vscode-vuelve, vscode-cancelado |
| `W-06`                                             | W-06 | codigo, formato-no-valido, buscando, encontrado, no-existe, demasiados, aprobado, rechazado                  |
| `W-07`                                             | W-07 | login-enviando, registro-enviando, credenciales, email-en-uso, demasiados, campos-422, error-servidor        |
| `W-08`                                             | W-08 | caducado, desconocido                                                                                        |
| `W-09`                                             | W-09 | caducada, cerrada-otro-dispositivo, con-sesion                                                               |
| `W-33`                                             | W-33 | con-contrasena, sin-contrasena, modal, actual-incorrecta, no-cumple, guardando, cambiada, anadida            |
| `W-34`                                             | W-34 | vinculadas, vinculada, ya-usada, email-sin-verificar, quitar, unica-forma                                    |
| `W-35`                                             | W-35 | metodos, metodos-fuertes, contrasena-incorrecta, no-vinculado, esperando, reenviar, pagina-confirmada        |

Las rutas de cada pantalla estan en la constante `ROUTES` y en la barra del prototipo
(`/forgot-password`, `/reset-password`, `/verify-email`, `/activate`, `/reauthenticate`...).
`client=vscode` cambia W-05 y W-06. `Ramal Local Callback.dc.html` pinta las paginas que sirve el
receptor local del motor al volver del navegador (inicio de sesion y plataforma, con exito, cancelado
o error) y `local-callback.template.html` es su version estatica.

Pendiente: a 375 px la barra superior desborda; W-35 ofrece la contrasena a una cuenta que no la
tiene.

### #70 Panel, sitio publico y equipo: `Ramal Web Panel.dc.html`, `Ramal Web Paginas.dc.html` y `Ramal Web.dc.html`

| Fichero y valor                                                          | Ids                    |
| ------------------------------------------------------------------------ | ---------------------- |
| Web Panel `W-11`, `W-12`, `W-13`, `W-14`, `W-15`, `W-16`, `W-36`, `W-18` | W-11 a W-18, W-36      |
| Web Panel `W-19` y `state=empty`, `loading`, `error` en cada seccion     | W-19, W-20             |
| Web Panel `W-21`, `W-22`, `W-23`, `W-24`, `W-25` (W-26 en W-21)          | W-21 a W-26            |
| Web Panel `X-02`                                                         | X-01, X-02             |
| Web Paginas `W-17`, `W-18`, `W-11`                                       | W-17, W-18, W-11       |
| Web Paginas `W-27-terminos`, `W-27-privacidad`, `W-27-reembolsos`        | W-27                   |
| Web Paginas `W-28`, `W-29-docs`, `W-29-releases`, `goodbye`              | W-28, W-29, W-24       |
| Ramal Web (landing): Precios, descargas con `os` y `downloads`, `img/`   | W-17, W-30, X-09, X-12 |

Los subestados de Web Paginas son: W-17 sin-sesion, con-sesion, con-organizaciones, con-plan; W-18
cargando, formulario, paddle-bloqueado, activando, activado, activacion-lenta, no-es-titular; W-11
cargando, sin-sesion, mismo-email, uniendote, unido, otro-email, caducada, no-existe, ya-miembro; W-28
404-publica, 404-panel, 500, sin-conexion; W-29-docs indice, articulo; W-29-releases lista, detalle,
cargando, vacia, error. Los tres ficheros se pueden ver a 768 y 375 px (W-30).

Pendiente: `Ramal Web.dc.html` conserva el panel antiguo, al que se llega tras iniciar sesion, con
"Compartir con el equipo", "Probar conexión" y el token a la vista (X-01, X-02, B-37, B-38, B-58);
tres textos rotos en el panel (el selector de organizacion dice `newOrg`, el aviso de correo sin
confirmar sale con un texto de relleno y la etiqueta del proveedor como `{serviceProvider}`); W-18 sin
plan mensual ni aviso del ultimo dia; W-22 sin la banda de rotar la llave; W-28 sin barra lateral;
capturas de la landing a 1x y con anotaciones del prototipo.

### #71 Revision de bugs: `Ramal Correcciones.dc.html` y los tres prototipos base

`Ramal Correcciones.dc.html` lista en 11 grupos los 30 bugs que se corrigieron en el dibujo, con el
antes, el despues y donde verlo. Las correcciones estan en `Ramal.dc.html`, `Ramal VS Code.dc.html`,
`Ramal Web.dc.html` y `ramal-data.js`: "Titular" en vez de "Propietaria", confirmaciones de peligro,
resolvedor, resumen de IA bajo demanda, mascaras y campos de secreto.

La tabla de 62 filas que pedia el brief (cada bug corregido en el diseno o regla de implementacion)
no venia en el zip. El estado de cada bug esta ahora en la columna "Estado en el diseno" de la
seccion 3 de [`../gaps.md`](../gaps.md).

Pendiente: en VS Code, B-10 (error del resumen), B-23, B-26 y B-27; "Cerrar sesión" de
`Ramal.dc.html` sin confirmar y "Abortar" de Git sin confirmar (B-29); `marta@ramal.dev` en
`Ramal.dc.html`; el panel antiguo de `Ramal Web.dc.html` (B-37, B-38, B-58); el septimo carril de
B-14.

## Notas para implementar

- Los textos de `ramal-data.js` son la base, no el resultado: unas 30 claves nuevas con contador no
  tienen `_one` y `_other`, algunas llevan glifos o atajos dentro, ocho claves de `STATES` y `D` se
  llaman igual con otro texto, y varios prototipos recortan o pegan traducciones para fabricar otras.
  En `@ramal/i18n` se arregla al copiarlas (ADR 0017, issue #522).
- La paleta de VS Code no coincide entre `Ramal VS Code.dc.html` y el modo `product=vscode` de
  Boveda y Estados. En la extension se usan las variables `--vscode-*`, asi que solo afecta a los
  prototipos.
- Lo que no se debe copiar de los prototipos sigue en la seccion 3 de [`../gaps.md`](../gaps.md).

## Especificacion base (handoff original)

Lo que sigue es el README del handoff original, en ingles, con los titulos bajados dos niveles. Sus
tablas de tokens, medidas y componentes siguen valiendo. Lo que ha cambiado desde entonces:

- **Stack recomendado.** No aplica: Ramal usa Tauri 2, un motor en Rust con el git del sistema y la
  web con React Router y Vite ([`../../architecture.md`](../../architecture.md)).
- **VS Code.** Lo nativo (paleta, quick input, notificaciones con "Origen: Ramal", barra de estado,
  menus) ya no se dibuja: lo especifica `Ramal VS Code.dc.html` (X-11). `vsLanguage` solo admite `en`
  y `es` y el atajo del grafo es Ctrl+K Ctrl+G.
- **Panel web.** Lo sustituye `Ramal Web Panel.dc.html`. La web ya no conecta, prueba ni comparte
  cuentas de plataforma (X-01, X-02). El panel que aun queda dentro de `Ramal Web.dc.html` no es
  referencia. `startPage` ya no tiene `dashboard`.
- **Landing y descargas.** La vista previa son las imagenes de `img/`, no el prototipo en un iframe
  (X-12). Sin Windows portable, macOS Universal en `.dmg`, un VSIX por plataforma y Linux desde Ubuntu
  22.04 y Debian 12 (X-09, ADR 0016).
- **Ajustes de Desktop.** La navegacion es la de `Ramal Ajustes.dc.html`. Las llaves de IA ya no
  tienen alcance ni modelo libre: las de la organizacion son de solo lectura y el modelo se elige de la
  lista del proveedor (X-05, X-10). En el modelo de datos salen `Account.team` y `AIKey.scope`.
- **Menus de rama.** Nombran la rama sobre la que actuan y separan borrar la local y la remota (B-26,
  B-27).
- **Tokens.** En claro, los ficheros web usan `--surface2` 0.945, `--border` 0.9 y una sombra mayor
  que la de la tabla; manda el prototipo. Los prototipos nuevos anaden `--canvas`, `--danger-ink`,
  `--scrim`, `--sk-hi` y `--fn`, que la tabla no recoge.
- **Assets y ficheros.** Ya hay imagenes (`img/`, `media/`) y catorce prototipos: ver el indice de
  arriba.

### Handoff: Ramal — cliente Git (Desktop, extensión VS Code y web)

#### Overview
Ramal is a Git client (functionally inspired by GitKraken) made of three products that share one account system:

1. **Ramal Desktop** (Electron/Tauri): repository list with search and filters, workspaces with aggregated PRs, a commit graph of every branch, staging and commits, side-by-side diff, a line-by-line merge conflict resolver, interactive rebase, context menus for everything, and settings (profile, Ramal account and team, Git platform accounts, AI keys, Git options, appearance).
2. **Ramal for VS Code** (extension): the same repo view without the repo list or workspaces, since the repo is the open folder. It follows the VS Code theme and language (falls back to English).
3. **Ramal Web**: marketing site with per-platform downloads, sign-up and login, and a cloud dashboard (overview and devices, workspaces, repositories, linked accounts, team, cloud-sync settings).

The UI is bilingual (ES/EN) everywhere and supports dark and light themes.

#### About the Design Files
The files in this bundle are **design references built in HTML**: working prototypes that show the intended look and behavior. They are not production code to copy directly. The task is to **recreate these designs in the target codebase** with its own patterns and libraries. If there is no codebase yet, pick the most suitable stack. Recommended:
- **Desktop:** Electron or Tauri + React + TypeScript. Git via `isomorphic-git`/`simple-git` (Electron) or `git2-rs` (Tauri).
- **VS Code extension:** extension host for Git and commands, with webviews for the graph, merge editor and settings, using `--vscode-*` CSS variables.
- **Web:** Next.js (or similar) with auth (email plus GitHub/GitLab/Google OAuth) and an API for syncing settings.

The prototypes are "Design Components" (`*.dc.html`). Each file has a template (markup with `{{ }}` holes, `<sc-for>` for loops and `<sc-if>` for conditionals) and a `class Component` holding all the state and logic in `renderVals()`. `support.js` is the runtime that renders them. Open any `.dc.html` in a browser, from the same folder, to try it. All data is fake (`ramal-data.js`).

#### Fidelity
**High fidelity.** Colors, typography, spacing, states and interactions are final. Recreate them pixel-perfect with the target stack's libraries.

---

#### Design Tokens

##### Typography
- UI: **IBM Plex Sans** 400/500/600/700 (Google Fonts). Base 13px (desktop app), 14px (web).
- Code, SHAs, paths, commands: **IBM Plex Mono** 400/500/600.
- VS Code extension: system font `-apple-system, BlinkMacSystemFont, 'Segoe UI'` 13px; code `Menlo/Consolas`.
- Scale in use: 10, 11 (uppercase section labels, `letter-spacing:.06em`, weight 600), 12, 12.5 (code), 13, 14, 15, 16, 17, 18, 20, 24 (H1 `letter-spacing:-0.02em`), 28–40 (web H2), `clamp(38px,6vw,66px)` (web hero, `letter-spacing:-0.035em`, line-height 1.04).

##### Colors — Desktop and Web (CSS variables on `body`)
| Token | Dark | Light |
|---|---|---|
| --bg | oklch(0.185 0.008 230) | oklch(0.965 0.004 230) (web 0.975) |
| --panel | oklch(0.21 0.009 230) | oklch(0.985 0.003 230) |
| --surface | oklch(0.24 0.01 230) | oklch(1 0 0) |
| --surface2 (hover) | oklch(0.285 0.012 230) | oklch(0.94 0.006 230) |
| --border | oklch(0.315 0.012 230) | oklch(0.89 0.008 230) |
| --text | oklch(0.95 0.005 230) | oklch(0.24 0.012 230) |
| --muted | oklch(0.76 0.012 230) | oklch(0.44 0.012 230) |
| --faint | oklch(0.62 0.012 230) | oklch(0.54 0.01 230) |
| --accent (teal) | oklch(0.74 0.12 190) | oklch(0.54 0.1 190) |
| --accent-ink | oklch(0.2 0.03 190) | oklch(0.99 0.005 190) |
| --accent-soft | accent / 0.14 | accent / 0.10 |
| --ok | oklch(0.76 0.15 150) | oklch(0.52 0.13 150) |
| --warn | oklch(0.8 0.14 80) | oklch(0.58 0.13 70) |
| --danger | oklch(0.7 0.17 25) | oklch(0.55 0.18 25) |
| --info | oklch(0.74 0.12 245) | oklch(0.52 0.13 250) |
| --add-bg / --del-bg | ok / danger at 13–14 % alpha | same, 12–13 % |

**Graph lane colors** (vivid, same in both themes):
`oklch(0.72 0.15 190)`, `oklch(0.68 0.2 300)`, `oklch(0.76 0.16 60)`, `oklch(0.68 0.21 355)`, `oklch(0.74 0.18 140)`, `oklch(0.7 0.16 250)`.

**Platform badges:** GitHub `oklch(0.5 0.03 260)` "GH", GitLab `oklch(0.64 0.17 40)` "GL", Bitbucket `oklch(0.56 0.15 255)` "BB", Gitea `oklch(0.6 0.14 145)` "GT", Azure DevOps `oklch(0.58 0.14 235)` "AZ".

**PR states:** open → info, draft → faint, approved → ok, changes requested → warn, merged → `oklch(0.68 0.18 300)`, closed → danger. Badge background = the color at 14 % (`color-mix(in oklch, C 14%, transparent)`).

**File status:** M → warn, A → ok, D → danger, R → info.

##### Colors — VS Code extension
Uses VS Code Dark Modern / Light Modern values: editor `#1f1f1f`/`#ffffff`, sidebar `#181818`/`#f8f8f8`, border `#2b2b2b`/`#e5e5e5`, fg `#cccccc`/`#3b3b3b`, accent `#0078d4`/`#005fb8`, list selection `#04395e`, git decorations M `#e2c08d`, A `#73c991`, D `#c74e39`, conflict `#e4676b`. **In the real extension, use the `--vscode-*` variables** (e.g. `--vscode-editor-background`, `--vscode-gitDecoration-modifiedResourceForeground`, `--vscode-button-background`) and not the hex values.

##### Radius, spacing, shadows
- Radius: 5–7 (chips, small buttons, menu items), 8–9 (inputs, buttons), 10–12 (cards, menus), 14 (large panels), 16–18 (modals, auth card).
- Heights: buttons 30/34/36/38/42/48; inputs 32–42; menu items 32 (desktop) / 24 (VS Code); graph rows 40 (desktop) / 28 (VS Code); sidebar items 30 / 22 (VS Code).
- Content padding: 32px × 36px, max width 1120–1200 (desktop home), 900 (settings), 1060 (web dashboard), 1160 (web landing).
- Shadow: `0 12px 40px oklch(0.05 0.01 230 / .5), 0 2px 6px oklch(0.05 0.01 230 / .3)` (dark); light uses oklch(0.3 0.02 230 / .16).

---

#### Screens — Ramal Desktop (`Ramal.dc.html`)

##### Global layout
- **Title bar / tabs (46px, --panel):** "R" logo (24px, teal, radius 8) + "Ramal". "Inicio" tab, a "⚙︎ Configuración" tab (when open), one tab per open repo (platform badge + name + ×, max 220px), "+". Active tab: `background:var(--bg)` + `box-shadow: inset 0 2px 0 var(--accent)`. On the right: ES/EN segmented control, ☀︎/☾ icon theme switch, ⚙︎, avatar (opens Profile).
- **Context menu (whole app):** 236px min, radius 12, items 32px. Submenus open on hover with "›". Danger items use --danger. Items with a check (✓) can stay open (`keep`) for multi-select. Opens with right-click and with the `⋯` buttons. Esc closes it.
- **Generic modal:** 460px (580 wide), radius 16. Field types: info, text, textarea, select, segmented, checkbox, action (AI button). Primary button uses accent, or danger for destructive actions.
- **Toasts:** bottom-right, colored dot (ok/warn/info), 3.2 s.

##### 1. Home › Repositories
- Sidebar 232px: "Repositorios (n)", WORKSPACES section with a color dot and count, "+ Nuevo workspace".
- Header: H1 + subtitle + buttons Abrir / Iniciar / **Clonar** (primary).
- **Filter bar on a single line** (card with 8px padding): search (flex:1, min 140px) plus 5 **dropdown** buttons: Tipo (Todos/Local/Remoto), Plataforma (multi-select), Cuenta, Workspace (including "Sin workspace"), Agrupar (Nada/Workspace/Plataforma/Cuenta). Each button shows a 10px caption and the value in 600 below it (max 120px with ellipsis). Active filter: accent border + accent-soft. Below: "N repositorios · Limpiar filtros".
- **Repo card** (grid `34px | 1.5fr | 1.2fr | 1fr | 224px`, gap 14, padding 14/12/14/16, radius 12, hover border accent): platform badge 34px (or "⌂" if local) · **alias** (600, 14px) + real name in mono + path in mono · "GitHub · @account" + host · account label · "⎇ branch" in accent mono + "↑2 ↓0 · N cambios" · workspace chip (first one + "+n") + "hace 5 min" + `⋯`.
- Menu: Abrir, Asignar alias…, Renombrar…, **Workspaces ›** (multiple checkboxes, stays open, "Quitar de todos", "+ Nuevo workspace"), Fetch, Mostrar en carpeta, Abrir terminal, Copiar ruta, Quitar de la lista… (confirmation with "mover carpeta a la papelera" option).
- **A repo can belong to several workspaces** (`wss: string[]`).

##### 2. Home › Workspace
- Header with color, name, "N repos · M PRs abiertos", Fetch de todos, Abrir todos, ⋯ (rename/delete).
- Repo cards grid (auto-fill minmax 250).
- **Aggregated PRs** across every repo in the workspace: state chips with counts. Table columns: repo (badge) | #num (GitLab uses "!num") + title + `from → to` | @author | state badge | checks (✓ Correctos / × Fallidos / ● En curso) | reviews n/m | updated. Clicking a row opens the repo.

##### 3. Repository view
- **Toolbar (62px):** repo selector ▾ and branch ▾ (accent mono) · buttons with glyph + label: Deshacer ↶, Rehacer ↷ | Fetch ⟳, Pull ↓ (▾ fetch all / ff / ff-only / rebase), Push ↑ (▾ force-with-lease, upstream; "ahead" badge) | Rama ⎇, Stash ⊟, Pop ⊞ (count badge) | PR ⇄ · commit search (dims the rows that don't match).
- **Left sidebar** (`clamp(200px,20vw,264px)`): filter box + collapsible sections LOCAL (✓ HEAD, ↑ahead), REMOTOS (grouped by remote), PULL REQUESTS, STASHES, TAGS (◆), SUBMÓDULOS. Click = select commit, double-click = checkout, right-click = menu.
- **Center — graph:** columns `200px refs | 130px graph | 1fr message | 150 author | 110 date | 76 sha`, **rows 40px**. Ref pills (lane color, 14 % background, 32 % if HEAD; icons ✓ HEAD, ⇅ remote, ◆ tag), max 2 + "+n". **WIP** row at the top ("// WIP · N archivos", dashed node). **Graph drawn in SVG:** X = 16 + lane·20, Y = row·40 + 20. Same lane → straight line. Branch-off (parent in a lower lane) → vertical, then a curve on the last row. Merge → curve on the first row, then vertical. Nodes r=9 with author initials (7.5px, 700), merges r=5, selected node gets an outer ring. Tweak `graphNodes: avatar|dot`.
- **Right panel** (`clamp(280px,26vw,360px)`):
  - *WIP:* "Sin preparar (n)" / "Preparados (n)" with Preparar todo / Quitar todo, rows with status letter + name + dir + [+/−] button. Commit box: "✦ Generar con IA" button, summary with a 72-character counter, description, amend checkbox, "Commit de N archivos en <rama>" (disabled until a file is staged and a summary written).
  - *Commit:* SHA + Copiar, message, **AI summary** (optional), author, parents, Cherry-pick / Revert / Crear rama, files with +/−.
- **Diff:** "← Grafo", status, path, +/−, **Lado a lado** (default) / Unificada, stage/unstage button. Side by side: `48px num | 18px sign | code` ×2, deleted lines on --del-bg, added on --add-bg, missing side shown as a diagonal hatch.
- **Conflict resolver (key feature):**
  - Header: "← Grafo" (you can **go back to the graph at any time**), ⚠, "Conflicto en <file>", "A: current ← B: incoming", Todo de A / Todo de B.
  - Top half: **A (Tu rama)** | **B (Entrante)** side by side, full file with line numbers. Each conflict block has a header ("Conflicto n" + "Tomar bloque"/"Quitar bloque") and every line has a **+** (add to output) / **−** (remove) button, with the lane color on a 3px inset left bar.
  - Bottom half: **Resultado** ("así quedará <file> al completar el merge"). Updates live. Each line shows a bar for its origin and a **−** button to remove it. An unresolved block is shown as a dashed warn box. "✎ Editar a mano" turns it into a textarea (Listo / Restablecer).
  - Right panel: "Merge en curso", list of files (resolved ✓ / in conflict !), "Completar merge" (enabled when everything is resolved; creates a two-parent merge commit), "Abortar merge".
- **Interactive rebase:** list with drag and drop (plus ↑↓), action select `pick/reword/edit/squash/fixup/drop` with its own color, input when the action is reword, strikethrough when drop, summary and Iniciar rebase.
- **Commit menu:** Checkout, Crear rama aquí, Crear tag aquí | Cherry-pick, Revert, Reset › soft/mixed/hard (hard asks for confirmation), Rebase interactivo | Merge (if it's a branch) | Copiar SHA / mensaje.
- **Branch menu:** Checkout, Merge X en actual, Rebase actual sobre X, Rebase interactivo | Push, Pull, Upstream | Crear PR, Comparar | Renombrar, Copiar nombre, Eliminar.
- Other menus: file (stage, view diff, discard, history, blame, .gitignore, editor, copy path), stash (apply/pop/drop), tag, remote, submodule, PR.

##### 4. Settings (its own tab, left nav with groups)
- **Usuario:** *Perfil* (64px avatar, name, username, email, time zone, Guardar). *Cuenta y equipo* (Ramal Pro plan + Gestionar suscripción, session + Cerrar sesión, "ACME Studio" organization with members, roles and pending invitations, Invitar, active sessions).
- **Conexiones:** *Plataformas Git* (groups per platform, several accounts each: @user, label, host, method OAuth/Token/SSH, scopes, "Usada en N repos", Predeterminada / Hacer predeterminada, menu to test/edit/disconnect). *IA*: keys (provider: Anthropic, OpenAI, Groq, Google Gemini, Mistral, OpenRouter, Ollama local, any OpenAI-compatible; label; masked key; model; **belongs to: Personal / Organización**; default). "Dónde usar la IA": toggle + key select for Mensajes de commit, Pull requests, Resumen de commits. Preferences: language, Conventional Commits, exclude sensitive files, extra instructions.
- **Aplicación:** *Git* (user.name/email, default branch, sign commits, auto fetch, default pull, prune, SSH keys). *Apariencia* (theme, language, graph nodes).
- Generic settings row: label + description on the left, control (input / select / segmented / toggle 38×22 / button / textarea) on the right, wrapping on narrow widths.

#### Screens — Ramal for VS Code (`Ramal VS Code.dc.html`)
- Title bar with the command center (opens the **command palette**, ⇧⌘P). Activity bar 48px with the Ramal "R" icon active (inset 2px accent). ⚙︎ at the bottom opens settings.
- **"RAMAL" view** (`clamp(230px,24vw,300px)`) with 22px section headers: CAMBIOS (commit textarea with ✦ IA, Commit button ▾ Commit/Commit y push/Amend; groups "Cambios de merge" (C), "Cambios preparados", "Cambios", each with inline +/−), RAMAS, REMOTOS, PULL REQUESTS, STASHES, TAGS.
- **Editor tabs:** "Ramal Graph" (pinned; compact toolbar with icon-only buttons, 28px rows, detail pane on the right), "Merge: <file>" (same resolver as desktop), "<file> (Working Tree)" diff, "Ramal Settings" (styled like the VS Code settings editor: "Ramal › AI: **Commit Messages**" + checkbox/select; sections Cuenta de Ramal and sync, Plataformas Git, IA, Git). No appearance or language settings: they **follow VS Code**.
- Status bar: remote indicator, "⎇ branch*", "⟳ 0↓ 2↑" (push), "⚠ N conflictos", "✦ <AI provider>", "Ramal: @account".
- Dialogs use the quick-input style (centered at the top, 600px). Notifications bottom-right with "Origen: Ramal".
- Language: `vsLanguage` es → Spanish, **anything else → English**.

#### Screens — Ramal Web (`Ramal Web.dc.html`)
- Sticky nav 64px with blur: logo, Funciones / Descargar / VS Code, ES/EN, ☀︎/☾, Iniciar sesión, **Crear cuenta** (or "Abrir panel" when logged in).
- **Landing:** v2.4 pill, hero H1 + subtitle, "Descargar para <OS detectado>" + "Todas las plataformas" + link to VS Code. Product preview (the real app in a window frame). Six features (grid with 1px borders, numbered 01–06). VS Code section with a preview and `code --install-extension ramal.ramal`. **Downloads:** macOS (Apple Silicon/Intel/Universal, `brew install --cask ramal`), Windows (x64/ARM64/.msi/portable, `winget install Ramal.Ramal`), Linux (.deb/.rpm/AppImage/Flathub/ARM64, `flatpak install flathub io.ramal.Ramal`), VS Code (Marketplace, Open VSX, .vsix). The card for the user's OS is highlighted ("Tu sistema"). Account CTA and footer.
- **Auth** (420px card): OAuth GitHub/GitLab/Google, "o con email", fields. Login: email, password + "¿La olvidaste?". Sign-up: name, email, password with an 8+ character hint, organization (optional), terms. Validation: required name, email regex, password ≥ 8, terms accepted. Error shown in a danger box.
- **Dashboard:** top bar 56px (organization switcher, Descargar, ES/EN, theme, avatar menu → Cerrar sesión). Nav: Resumen (counter tiles + synced devices), Workspaces (cards with repo chips, **Compartido con equipo / Solo tú** toggle, rename/delete), Repositorios (search; table repo | origin (platform + @account) | workspaces (multiple) | updated | ⋯ with workspace checkboxes and unlink), Cuentas vinculadas (per platform, owner Tuya/Equipo, test/share/disconnect), Equipo (used seats with a bar, members with a role select, invite, resend, remove), Configuración en la nube (profile; what syncs: workspaces and repos, accounts (encrypted tokens), AI preferences, AI keys (**off by default: they stay on each device**), Git, appearance; export JSON; delete account).

---

#### State Management (main models)
```ts
Account   { id, platform: 'github'|'gitlab'|'bitbucket'|'gitea'|'azure', user, name, host, method, scopes, def: boolean, team?: boolean }
Workspace { id, name, color, shared?: boolean }
Repo      { id, name, alias, path, acc: Account.id|null, wss: Workspace.id[], branch, ahead, behind, changes, updatedAt }
PR        { repo, num, title, author, from, to, state: 'open'|'draft'|'approved'|'changes'|'merged'|'closed', checks: 'pass'|'fail'|'pending', reviews }
Commit    { id, sha, lane, parents: id[], msg, author, date, refs: {name, type:'local'|'remote'|'tag', head?}[] }
Conflict  { src, file, choices: { [fileIdx]: { ours: boolean[], theirs: boolean[] }[] /* per hunk, per line */ }, manual: {[fileIdx]: string} }
AIKey     { id, prov, label, scope: 'personal'|'org', masked, model, def }
AIConfig  { commit, pr, summary: boolean; commitKey, prKey, summaryKey; lang: 'app'|'es'|'en'; conventional; instructions; exclude }
Member    { name, email, role: 'owner'|'admin'|'member', pending? }
```
- A file counts as resolved when every hunk has at least one selected line, or when it has a manual edit with no `<<<<<<<`/`=======`/`>>>>>>>` markers left.
- The resolver output is the context lines plus, for each hunk, the selected lines of A followed by the selected lines of B, in their original order.
- Real lane assignment for the graph: see the `DIFF` sample in `ramal-data.js` (the reuse-free-lanes algorithm).
- Web ↔ app sync: account + workspaces + repos (without local paths from other devices) + accounts (encrypted tokens) + preferences. AI keys are not synced unless the user turns it on.

#### Interactions to keep
- Right-click on every entity. `⋯` opens the same menu. Esc closes menus, modals and the palette.
- Workspaces menu with checkboxes that stays open.
- Merge with conflicts → resolver; you can leave it with "← Grafo" and come back through the WIP row or the file list.
- AI: disabled feature or no key → warning toast. Generation runs ~800 ms with "Generando…" and fills the summary/description or the PR title/description in the language set in AI preferences.
- Toggle knob `transition: transform .15s`. Toasts `toastIn .2s ease-out`.

#### Assets
No images. Icons are Unicode glyphs (⎇ ⟳ ↓ ↑ ⊟ ⊞ ⇄ ◆ ⇅ ✦ ⚙︎ ☀︎ ☾ ⋯). **In production, replace them with a consistent icon set** (e.g. Lucide or Codicons in VS Code). The "R" logo is a placeholder.

#### Files
- `Ramal.dc.html` — desktop app (home, workspaces, repo, diff, conflicts, rebase, settings).
- `Ramal VS Code.dc.html` — VS Code extension (props `vsTheme`, `vsLanguage`).
- `Ramal Web.dc.html` — landing, auth and dashboard (props `theme`, `lang`, `startPage`).
- `ramal-data.js` — fake data and the complete ES/EN dictionary (`D`). Reuse the keys as the base for real i18n.
- `support.js` — runtime needed to open the `.dc.html` files in a browser (not part of the product).
