# Brief #68: contribuciones nativas de VS Code (X-11, V-01 a V-22, B-62)

Issue: [eabl0306/git-rudder#68](https://github.com/eabl0306/git-rudder/issues/68). Fuente: `docs/design/gaps.md`
(seccion 1.4, X-11 y B-62), ADR 0008, 0010 y 0012, y la decision #58 del 2026-10-01.

## Contexto

- Producto: Git Rudder for VS Code. Base visual: `handoff/Git Rudder VS Code.dc.html` y la seccion "Screens - Git Rudder
  for VS Code" del `handoff/README.md`. Colores siempre con variables `--vscode-*`; en el prototipo, los
  valores Dark Modern y Light Modern del README.
- Hoy el prototipo redibuja en HTML la paleta (`pal`), los dialogos (`modal`, estilo quick input de 600 px
  arriba), las notificaciones con "Origen: Git Rudder" (`toasts`), la barra de estado (`statusL`, `statusR`),
  los menus (`menu`) y la pagina de ajustes. En el producto esas superficies son de VS Code: aqui se
  especifican como contribuciones, con su texto, y **no se dibujan**.
- Siguen siendo webview y se dibujan de verdad: la vista **Cambios** (`scm`: caja de mensaje con boton IA y
  boton Commit partido), la pestana **Grafo de Git Rudder** (barra `tools` de 34 px, filas `rows` de 28 px, panel
  de detalle `dt`), el **editor de merge** (`cf`), el formulario **Crear pull request** y la pagina
  **Configuracion de Git Rudder** (`setSections`, con filas `isEntity`, `isSetting` e `isButtons`).
- Las secciones con cabecera de 22 px (`panes`: RAMAS, REMOTOS, PULL REQUESTS, STASHES, TAGS) pasan a ser
  vistas de arbol nativas. Los estados comunes (carga, error con detalle, vacios, sin conexion) son del
  brief #62; la boveda (K-01 a K-09) es del brief #63. Aqui solo se dice donde caen en VS Code.

## Pantallas y estados

Densidad compacta en todo: filas del grafo 28 px, filas de arbol 22 px, menus 24 px, botones de webview
26 px. Todo en Dark Modern y Light Modern. Iconos: Codicons (nombre entre comillas invertidas), nunca
glifos Unicode.

### 1. X-11 Especificacion de contribuciones

**Que es nativo y que es webview.**

| Superficie                                     | Como se hace                                                                        |
| ---------------------------------------------- | ----------------------------------------------------------------------------------- |
| Paleta de comandos                             | `contributes.commands` con categoria "Git Rudder" y `commandPalette` con `when`     |
| Crear rama, stash, merge, tag, checkout, reset | `showQuickPick` e `showInputBox` (QuickInput), con paso "1/3" cuando hay varios     |
| Anadir cuenta y anadir llave                   | QuickInput de varios pasos, secreto con `password: true`                            |
| Confirmaciones destructivas                    | `showWarningMessage` modal con el verbo en el boton                                 |
| Avisos y errores                               | Notificaciones nativas; VS Code pone el origen solo                                 |
| Exito de operaciones rapidas                   | `setStatusBarMessage` 4 s con `check`, sin notificacion                             |
| Progreso                                       | Barra de la vista (`viewId`) y, en clonar e iniciar sesion, notificacion cancelable |
| Ramas, remotos, PRs, stashes, tags             | TreeViews con Codicons; clic o accion en linea, nunca doble clic                    |
| Menus contextuales                             | `view/item/context`, `webview/context`, `editor/context`, `explorer/context`        |
| Barra de estado                                | Elementos nativos (tabla mas abajo)                                                 |
| Iniciar sesion                                 | `AuthenticationProvider` "Git Rudder" en el menu de Cuentas                         |
| Ajustes locales de la maquina                  | `contributes.configuration`, solo ambito `machine` (V-22)                           |
| Git, IA, cuentas y llaves                      | Webview "Configuracion de Git Rudder" (V-22)                                        |
| Cambios, grafo, merge, crear PR                | Webview                                                                             |
| Diff de un archivo                             | Editor de diff nativo (`vscode.diff`)                                               |
| Bienvenida y primeros pasos                    | `viewsWelcome` y `contributes.walkthroughs`                                         |

**Comandos.** Categoria "Git Rudder" en los dos idiomas: la paleta ensena "Git Rudder: Abrir grafo". Los que
abren un formulario terminan en "…". "Paleta" dice cuando aparecen; los comandos de un elemento concreto
(checkout de una rama del arbol) llevan `when: false` en la paleta.

| Id                                | ES                                            | EN                              | Icono                     | Paleta                        |
| --------------------------------- | --------------------------------------------- | ------------------------------- | ------------------------- | ----------------------------- |
| `gitRudder.openGraph`             | Abrir grafo                                   | Open Graph                      | `git-commit`              | con repo                      |
| `gitRudder.selectRepository`      | Elegir repositorio…                           | Select Repository…              | `repo`                    | mas de un repo                |
| `gitRudder.fetch`                 | Fetch                                         | Fetch                           | `repo-fetch`              | con repo                      |
| `gitRudder.pull`                  | Pull                                          | Pull                            | `repo-pull`               | con repo                      |
| `gitRudder.pullWithMode`          | Pull (elegir modo)…                           | Pull (Choose Mode)…             |                           | con repo                      |
| `gitRudder.push`                  | Push                                          | Push                            | `repo-push`               | con repo                      |
| `gitRudder.forcePush`             | Force push (with lease)…                      | Force Push (with Lease)…        |                           | rama con upstream             |
| `gitRudder.publishBranch`         | Publicar rama…                                | Publish Branch…                 | `cloud-upload`            | rama sin upstream             |
| `gitRudder.commit`                | Commit                                        | Commit                          | `check`                   | con repo                      |
| `gitRudder.commitAndPush`         | Commit y push                                 | Commit & Push                   |                           | con repo                      |
| `gitRudder.commitAmend`           | Commit (amend)                                | Commit (Amend)                  |                           | con repo                      |
| `gitRudder.generateCommitMessage` | Generar mensaje de commit con IA              | Generate Commit Message with AI | `sparkle`                 | IA de commits activa          |
| `gitRudder.checkout`              | Checkout a…                                   | Checkout to…                    | `git-branch`              | con repo                      |
| `gitRudder.createBranch`          | Crear rama…                                   | Create Branch…                  | `git-branch-create`       | con repo                      |
| `gitRudder.merge`                 | Merge en la rama actual…                      | Merge into Current Branch…      | `git-merge`               | con repo                      |
| `gitRudder.rebase`                | Rebase de la rama actual sobre…               | Rebase Current Branch onto…     |                           | con repo                      |
| `gitRudder.continueOperation`     | Continuar la operación en curso               | Continue Current Operation      | `debug-continue`          | operacion en curso            |
| `gitRudder.skipCommit`            | Omitir este commit                            | Skip This Commit                | `debug-step-over`         | rebase o cherry-pick en curso |
| `gitRudder.abortOperation`        | Abortar la operación en curso…                | Abort Current Operation…        | `debug-stop`              | operacion en curso            |
| `gitRudder.stash`                 | Stash…                                        | Stash…                          | `git-stash`               | con cambios                   |
| `gitRudder.stashPop`              | Pop del último stash                          | Pop Latest Stash                | `git-stash-pop`           | hay stashes                   |
| `gitRudder.stashApply`            | Aplicar stash…                                | Apply Stash…                    | `git-stash-apply`         | hay stashes                   |
| `gitRudder.createTag`             | Crear tag…                                    | Create Tag…                     | `tag`                     | con repo                      |
| `gitRudder.createPullRequest`     | Crear pull request…                           | Create Pull Request…            | `git-pull-request-create` | con repo y remoto             |
| `gitRudder.fileHistory`           | Ver historial del archivo                     | Show File History               | `history`                 | archivo del repo activo       |
| `gitRudder.stageSelectedRanges`   | Preparar las líneas seleccionadas             | Stage Selected Lines            |                           | diff de Git Rudder            |
| `gitRudder.unstageSelectedRanges` | Quitar de preparados las líneas seleccionadas | Unstage Selected Lines          |                           | diff de Git Rudder            |
| `gitRudder.discardSelectedRanges` | Descartar las líneas seleccionadas…           | Discard Selected Lines…         |                           | diff de Git Rudder            |
| `gitRudder.clone`                 | Clonar repositorio…                           | Clone Repository…               | `repo-clone`              | siempre                       |
| `gitRudder.initRepository`        | Iniciar repositorio                           | Initialize Repository           |                           | carpeta sin repo              |
| `gitRudder.openInDesktop`         | Abrir en Git Rudder Desktop                   | Open in Git Rudder Desktop      | `link-external`           | con repo y sin `remoteName`   |
| `gitRudder.signIn`                | Iniciar sesión                                | Sign In                         | `account`                 | sin sesion                    |
| `gitRudder.signOut`               | Cerrar sesión…                                | Sign Out…                       |                           | con sesion                    |
| `gitRudder.openSettings`          | Abrir configuración de Git Rudder             | Open Git Rudder Settings        | `gear`                    | siempre                       |
| `gitRudder.openDashboard`         | Abrir panel web                               | Open Web Dashboard              | `link-external`           | con sesion                    |
| `gitRudder.addPlatformAccount`    | Añadir cuenta de plataforma…                  | Add Platform Account…           | `add`                     | siempre                       |
| `gitRudder.addAiKey`              | Añadir llave de IA…                           | Add AI Key…                     | `add`                     | siempre                       |
| `gitRudder.checkGit`              | Volver a comprobar Git                        | Check Git Again                 | `refresh`                 | git ausente o antiguo         |
| `gitRudder.restartEngine`         | Reiniciar el motor                            | Restart Engine                  | `debug-restart`           | siempre                       |
| `gitRudder.showEngineLog`         | Mostrar el registro del motor                 | Show Engine Log                 | `output`                  | siempre                       |
| `gitRudder.refresh`               | Actualizar                                    | Refresh                         | `refresh`                 | nunca (solo titulo de vista)  |

Claves de contexto que usan los `when`: `gitRudder.hasRepository`, `gitRudder.repositoryCount`,
`gitRudder.operation` (`merge`, `rebase`, `cherryPick`, `revert`, `stashApply` o vacio), `gitRudder.signedIn`,
`gitRudder.engineState` (`starting`, `ready`, `restarting`, `stopped`), `gitRudder.gitState` (`ok`, `missing`,
`outdated`), `gitRudder.prState` (`ok`, `noAccount`, `unknownHost`, `noRemote`).

**Vistas del contenedor "Git Rudder"** (barra de actividad, icono `media/git-rudder.svg`):

| Id                       | Tipo    | ES            | EN            | Visible cuando       | Acciones del titulo                                                             |
| ------------------------ | ------- | ------------- | ------------- | -------------------- | ------------------------------------------------------------------------------- |
| `gitRudder.welcome`      | arbol   | Git Rudder    | Git Rudder    | sin repo o sin motor | ninguna (solo `viewsWelcome`)                                                   |
| `gitRudder.changes`      | webview | Cambios       | Changes       | con repo             | `refresh`, `git-commit` (Abrir grafo), `gear`; en "…": Fetch, Pull, Push, Stash |
| `gitRudder.branches`     | arbol   | Ramas         | Branches      | con repo             | `git-branch-create` (Crear rama…)                                               |
| `gitRudder.remotes`      | arbol   | Remotos       | Remotes       | con repo             | `repo-fetch` (Fetch de todos los remotos)                                       |
| `gitRudder.pullRequests` | arbol   | Pull requests | Pull Requests | con repo             | `git-pull-request-create`, `refresh`                                            |
| `gitRudder.stashes`      | arbol   | Stashes       | Stashes       | con repo             | `add` (Stash…)                                                                  |
| `gitRudder.tags`         | arbol   | Tags          | Tags          | con repo             | `add` (Crear tag…)                                                              |

Con varios repos, cada vista pone el nombre del repo en `TreeView.description` (texto gris junto al
titulo).

**Elementos de los arboles.** Clic: selecciona y revela el commit en el grafo. Checkout y el resto,
por accion en linea (al pasar el raton) o menu contextual.

| Elemento       | Icono                                           | Descripcion (gris)  | En linea                            |
| -------------- | ----------------------------------------------- | ------------------- | ----------------------------------- |
| Rama local     | `git-branch` (color del carril)                 | `2↑ 1↓` si aplica   | `check` Checkout                    |
| Rama actual    | `check` (`--vscode-charts-blue`)                | `2↑ 1↓`, en negrita | `repo-push` Push si va por delante  |
| Remoto (grupo) | `cloud`                                         | URL corta           | `repo-fetch` Fetch                  |
| Rama remota    | `git-branch`                                    |                     | `check` Checkout (crea local)       |
| PR abierto     | `git-pull-request` (`charts.blue`)              | estado, checks      | `link-external` Abrir en {platform} |
| PR borrador    | `git-pull-request-draft` (`disabledForeground`) |                     | igual                               |
| PR fusionado   | `git-merge` (`charts.purple`)                   |                     | igual                               |
| PR cerrado     | `git-pull-request-closed` (`charts.red`)        |                     | igual                               |
| Stash          | `git-stash`                                     | `stash@{0}` y fecha | `git-stash-apply`, `git-stash-pop`  |
| Tag            | `tag` (`charts.yellow`)                         | sha corto           | `repo-push` Push del tag            |

Tooltip de un PR (Markdown): "#142 Grafo de commits con zoom · feature/graph-ui → develop · @marta-dev ·
Borrador · Checks: 2 correctos, 1 en curso · Revisiones: 1 de 2".

**Menus contextuales.** Las etiquetas son fijas (un comando no puede tener titulo dinamico). Grupos en
este orden; lo destructivo, al final y con "…" (abre confirmacion).

En un menu nativo el titulo de cada comando es fijo (sale de `package.json`), asi que "Push" o "Pull"
no pueden llevar el nombre de la rama. El comando recibe el elemento del arbol y actua sobre esa rama,
no sobre la actual (B-26). Borrar la rama local y la remota son entradas distintas, cada una con su
confirmacion (B-27).

| Donde                          | Entradas                                                                                                                                                                                                              |
| ------------------------------ | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Rama local                     | Checkout · Merge en la rama actual… · Rebase de la actual sobre esta… · Push · Pull · Configurar upstream… · Crear pull request… · Copiar nombre · Renombrar rama… · Eliminar rama…                                   |
| Rama actual                    | Push · Pull · Configurar upstream… · Crear pull request… · Copiar nombre · Renombrar rama…                                                                                                                            |
| Rama remota                    | Checkout · Merge en la rama actual… · Crear pull request… · Copiar nombre · Eliminar rama remota…                                                                                                                     |
| Remoto                         | Fetch · Copiar URL                                                                                                                                                                                                    |
| PR                             | Abrir en {platform} · Checkout de la rama · Copiar enlace                                                                                                                                                             |
| Stash                          | Aplicar · Pop (aplicar y eliminar) · Ver cambios · Eliminar stash…                                                                                                                                                    |
| Tag                            | Checkout del tag · Push del tag · Copiar nombre · Eliminar tag…                                                                                                                                                       |
| Commit en el grafo (webview)   | Checkout · Crear rama aquí… · Crear tag aquí… · Cherry-pick en {branch}… · Revertir commit · Reset de {branch} a este commit (submenu Soft, Mixed, Hard…) · Copiar SHA · Copiar mensaje · Abrir en Git Rudder Desktop |
| Pastilla de rama en el grafo   | El menu de su rama. Pastilla de tag: el menu de tag (B-28)                                                                                                                                                            |
| Archivo en Cambios (webview)   | Ver diff · Abrir archivo · Preparar / Quitar de preparados · Ver historial del archivo · Copiar ruta · Descartar cambios…                                                                                             |
| Explorador y pestana de editor | Ver historial del archivo                                                                                                                                                                                             |
| Diff de Git Rudder (editor)    | Preparar las líneas seleccionadas · Quitar de preparados las líneas seleccionadas · Descartar las líneas seleccionadas…                                                                                               |

En los webviews, el menu es nativo (`webview/context` con `data-vscode-context` por fila): el submenu
de reset es un submenu de VS Code. "Abrir en Git Rudder Desktop" no aparece con `remoteName`.

Las etiquetas que ya existen en `git-rudder-data.js` se reutilizan con su ingles (`mCheckout`, `mPush`,
`mPull`, `mUpstream`, `mCreatePR`, `mDelete`, `mDeleteRemote`, `mPrOpen`, `mPrCheckout`, `mCopyLink`,
`mApply`, `mPop`, `mDrop`, `mPushTag`, `mDeleteTag`, `mBranchHere`, `mTagHere`, `mCherry`, `mRevert`,
`mReset`, `rSoft`, `rMixed`, `rHard`, `mCopySha`, `mCopyMsg`, `openDesktop`, `mViewDiff`, `mStage`,
`mUnstage`, `mCopyPath`, `mDiscard`, `mFetchAllR`). Nuevas:

| Clave         | ES                              | EN                         |
| ------------- | ------------------------------- | -------------------------- |
| mMergeCurrent | Merge en la rama actual…        | Merge into Current Branch… |
| mRebaseOnThis | Rebase de la actual sobre esta… | Rebase Current onto This…  |
| mRenameBranch | Renombrar rama…                 | Rename Branch…             |
| mCopyName     | Copiar nombre                   | Copy Name                  |
| mCopyUrl      | Copiar URL                      | Copy URL                   |
| mStashView    | Ver cambios                     | View Changes               |
| mTagCheckout  | Checkout del tag                | Check Out Tag              |
| mOpenFile     | Abrir archivo                   | Open File                  |
| mFileHistory  | Ver historial del archivo       | Show File History          |

**Barra de estado.** Valores reales siempre (B-33). La identidad es la de Git Rudder, nunca un usuario de
plataforma (B-40).

| Elemento    | Lado | Texto                                                 | Tooltip                                                                       | Clic                                | Visible cuando                                              |
| ----------- | ---- | ----------------------------------------------------- | ----------------------------------------------------------------------------- | ----------------------------------- | ----------------------------------------------------------- |
| Repo        | izq. | `$(repo) web-app`                                     | "Repositorio de Git Rudder: web-app (2 de 3)"                                 | Elegir repositorio…                 | mas de un repo                                              |
| Rama        | izq. | `$(git-branch) feature/graph-ui*`                     | "Rama actual · 3 cambios sin commit"                                          | Checkout a…                         | solo si el Git integrado esta apagado (V-21)                |
| Sincronizar | izq. | `$(sync) 1↓ 2↑` o `$(cloud-upload) Publicar`          | "1 commit por traer y 2 por subir"                                            | Pull y push / Publicar rama…        | igual que Rama                                              |
| Operacion   | izq. | `$(warning) 2 conflictos` o `$(git-merge) Rebase 3/7` | "Rebase en curso: commit 3 de 7"                                              | Editor de merge o grafo             | operacion en curso, fondo `statusBarItem.warningBackground` |
| IA          | der. | `$(sparkle) Anthropic` o `$(sparkle) Sin llave`       | "IA para mensajes de commit: Anthropic · claude-haiku-4-5 (llave «Personal»)" | Configuración de Git Rudder, IA     | alguna funcion de IA activa                                 |
| Cuenta      | der. | ver V-01                                              | ver V-01                                                                      | Configuración de Git Rudder, cuenta | siempre                                                     |
| Motor       | der. | ver V-05                                              | ver V-05                                                                      | ver V-05                            | motor reiniciando o detenido                                |

| Clave       | ES                                                                                              | EN                                                                                           |
| ----------- | ----------------------------------------------------------------------------------------------- | -------------------------------------------------------------------------------------------- |
| sbRepoTip   | Repositorio de Git Rudder: {repo} ({index} de {total})                                          | Git Rudder repository: {repo} ({index} of {total})                                           |
| sbBranchTip | Rama actual · {count} cambios sin commit                                                        | Current branch · {count} uncommitted changes                                                 |
| sbSyncTip   | {behind} commits por traer y {ahead} por subir                                                  | {behind} commits to pull and {ahead} to push                                                 |
| sbPublish   | Publicar                                                                                        | Publish                                                                                      |
| sbOpTip     | Rebase en curso: commit {index} de {total}                                                      | Rebase in progress: commit {index} of {total}                                                |
| sbConflicts | {count} conflictos                                                                              | {count} conflicts                                                                            |
| sbAiTip     | IA para mensajes de commit: {provider} · {model} (llave «{label}»)                              | AI for commit messages: {provider} · {model} ("{label}" key)                                 |
| sbAiNoKey   | Sin llave                                                                                       | No Key                                                                                       |
| prTooltip   | #{num} {title} · {from} → {to} · @{author} · {state} · Checks: {checks} · Revisiones: {reviews} | #{num} {title} · {from} → {to} · @{author} · {state} · Checks: {checks} · Reviews: {reviews} |
| committedSb | Commit {sha} creado                                                                             | Commit {sha} created                                                                         |

**Notificaciones y progreso.** Exito rapido (commit, fetch, push, stash): mensaje de barra de estado de
4 s, por ejemplo "$(check) Commit a41f9c2 creado". Error: notificacion de error con acciones concretas y
"Ver detalle", que abre un modal de error con la salida de git en `detail` y el boton "Copiar detalle"
(patron C-02 del brief #62). Operacion larga: barra de progreso de la vista Git Rudder y, si pasa de 2 s,
elemento de barra de estado `$(sync~spin) Push…`. Clonar e iniciar sesion: notificacion de progreso con
Cancelar.

**QuickInput.** Titulo arriba, paso "1/3" a la derecha, `placeholder` dentro y `prompt` debajo. Errores
de validacion en linea (`validationMessage`), nunca un modal que se cierra sin decir por que (B-60).

| Flujo                       | Pasos                                                                                                                                                                                                      |
| --------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Crear rama                  | 1. InputBox "Crear rama desde a41f9c2", placeholder "feature/…", validacion de `check-ref-format` (B-31). 2. QuickPick "Crear y hacer checkout" (por defecto) / "Solo crear".                              |
| Checkout a…                 | QuickPick con separadores "Locales" y "Remotas"; cada fila `$(git-branch) nombre`, descripcion `2↑ 1↓ · hace 3 h`; primera fila `$(git-branch-create) Crear rama…`.                                        |
| Merge en la rama actual     | 1. QuickPick de rama. 2. QuickPick de estrategia: "Fast-forward si es posible", "Crear commit de merge", "Squash". 3. Si la estrategia crea commit: InputBox con el mensaje propuesto (B-04).              |
| Rebase sobre…               | QuickPick de rama; si el arbol esta sucio, V-11.                                                                                                                                                           |
| Pull (elegir modo)          | QuickPick: "Fast-forward si es posible", "Solo fast-forward", "Rebase".                                                                                                                                    |
| Stash                       | 1. QuickPick: "Todos los cambios", "Todos, con archivos sin seguimiento", "Solo lo preparado". 2. InputBox "Mensaje (opcional)", placeholder "WIP en feature/graph-ui".                                    |
| Crear tag                   | 1. InputBox nombre (validacion de refs). 2. InputBox "Mensaje (opcional; vacío crea un tag ligero)". 3. QuickPick "Hacer push del tag a origin" / "Solo en local".                                         |
| Reset desde el grafo        | QuickPick: "Soft · conserva todo preparado", "Mixed · conserva cambios", "Hard · descarta todo…"; Hard pide el modal de peligro.                                                                           |
| Publicar rama               | QuickPick de remoto (se salta con uno solo), InputBox con el nombre remoto propuesto.                                                                                                                      |
| Clonar                      | 1. InputBox "URL del repositorio o owner/repo". 2. Con `owner/repo`, QuickPick de cuenta (solo las del host que corresponde). 3. Dialogo nativo de carpeta. Progreso en notificacion con fases y Cancelar. |
| Anadir cuenta de plataforma | Ver V-15.                                                                                                                                                                                                  |
| Anadir llave de IA          | Ver V-16.                                                                                                                                                                                                  |

Reutilizan claves existentes `sFF`, `sNoFF`, `sSquash`, `rSoft`, `rMixed`, `rHard`, `mPullFF`, `mPullOnly`,
`mPullRebase` y `cloneUrl`. Nuevas:

| Clave         | ES                                            | EN                                       |
| ------------- | --------------------------------------------- | ---------------------------------------- |
| qiBranchFrom  | Crear rama desde {sha}                        | Create branch from {sha}                 |
| qiBranchBad   | Nombre no válido: {reason}                    | Invalid name: {reason}                   |
| qiCreateCo    | Crear y hacer checkout                        | Create and Check Out                     |
| qiCreateOnly  | Solo crear                                    | Create Only                              |
| qiLocal       | Locales                                       | Local                                    |
| qiRemote      | Remotas                                       | Remote                                   |
| qiStrategy    | ¿Cómo hacer el merge de {source} en {target}? | How should {source} merge into {target}? |
| qiMergeMsg    | Mensaje del commit de merge                   | Merge commit message                     |
| qiStashAll    | Todos los cambios                             | All Changes                              |
| qiStashUntr   | Todos, con archivos sin seguimiento           | All, Including Untracked Files           |
| qiStashStaged | Solo lo preparado                             | Staged Changes Only                      |
| qiStashMsg    | Mensaje (opcional)                            | Message (optional)                       |
| qiPickRemote  | ¿En qué remoto publicar {branch}?             | Which remote should {branch} go to?      |
| qiRemoteName  | Nombre de la rama en el remoto                | Branch name on the remote                |
| qiCloneAcc    | ¿Con qué cuenta clonar?                       | Which account should clone it?           |
| qiCloneDest   | Elegir carpeta de destino                     | Choose Destination Folder                |
| qiCloning     | Clonando {repo}: {phase} {percent}%           | Cloning {repo}: {phase} {percent}%       |
| qiCloned      | Repositorio clonado en {path}.                | Repository cloned to {path}.             |
| qiOpen        | Abrir                                         | Open                                     |
| qiOpenNew     | Abrir en una ventana nueva                    | Open in New Window                       |
| qiAddWs       | Añadir al espacio de trabajo                  | Add to Workspace                         |

**Modales de confirmacion** (`showWarningMessage` con `modal: true`; VS Code anade Cancelar). VS Code no
pinta un boton de peligro: el texto del boton nombra la accion.

| Accion               | Mensaje ES                                  | Detalle ES                                                                                  | Boton ES                              | Mensaje EN                              | Boton EN                       |
| -------------------- | ------------------------------------------- | ------------------------------------------------------------------------------------------- | ------------------------------------- | --------------------------------------- | ------------------------------ |
| Descartar cambios    | ¿Descartar los cambios de {file}?           | Esta acción no se puede deshacer.                                                           | Descartar                             | Discard changes to {file}?              | Discard                        |
| Eliminar rama        | ¿Eliminar la rama {branch}?                 | Tiene commits que no están en otra rama.                                                    | Eliminar / Forzar (-D)                | Delete branch {branch}?                 | Delete / Force (-D)            |
| Eliminar rama remota | ¿Eliminar {remote}/{branch} en el servidor? | Se borra para todo el equipo.                                                               | Eliminar en {remote}                  | Delete {remote}/{branch} on the server? | Delete on {remote}             |
| Force push           | ¿Force push a origin/{branch}?              | Se usa --force-with-lease: falla si alguien ha hecho push desde tu último fetch.            | Force push                            | Force push to origin/{branch}?          | Force push                     |
| Reset --hard         | ¿Reset --hard a {sha}?                      | Se perderán los cambios sin commit y los commits posteriores en la rama actual.             | Reset --hard                          | Reset --hard to {sha}?                  | Reset --hard                   |
| Eliminar stash       | ¿Eliminar {stash}?                          | «{message}». Esta acción no se puede deshacer.                                              | Eliminar                              | Drop {stash}?                           | Drop                           |
| Eliminar tag         | ¿Eliminar el tag {tag}?                     | Si ya está en origin, también puedes borrarlo allí.                                         | Eliminar / Eliminar también en origin | Delete tag {tag}?                       | Delete / Also delete on origin |
| Desconectar cuenta   | ¿Desconectar @{user} ({host})?              | Git Rudder deja de usarla en este y en tus otros equipos. No revoca el token en {platform}. | Desconectar                           | Disconnect @{user} ({host})?            | Disconnect                     |
| Eliminar llave de IA | ¿Eliminar la llave «{label}»?               | Las funciones que la usaban pasan a la llave predeterminada.                                | Eliminar                              | Delete the key "{label}"?               | Delete                         |
| Abortar operacion    | ¿Abortar el {operation}?                    | Se descartan los conflictos resueltos y la rama vuelve a como estaba antes de empezar.      | Abortar                               | Abort the {operation}?                  | Abort                          |

Detalles EN de la tabla: "This can't be undone." · "It has commits that aren't on any other branch." ·
"It is deleted for the whole team." · "Uses --force-with-lease: fails if someone pushed since your last
fetch." · "Uncommitted changes and later commits on the current branch will be lost." · "\"{message}\".
This can't be undone." · "If it's already on origin, you can delete it there too." · "Git Rudder stops using it
on this and your other devices. It doesn't revoke the token on {platform}." · "Features that used it
switch to the default key." · "Resolved conflicts are discarded and the branch goes back to how it was
before you started."

### 2. V-01 Iniciar sesion en Git Rudder

- **Proposito.** Entrar en Git Rudder desde VS Code sin formulario de contrasena (ADR 0008): PKCE con el
  `UriHandler` (`vscode://git-rudder.git-rudder/auth-callback`) y device flow como alternativa. Git local funciona
  sin cuenta.
- **Muestra.** Proveedor "Git Rudder" en el menu de Cuentas (abajo en la barra de actividad). La extension no
  pide sesion al arrancar (`getSession` silencioso): sin insignia en Cuentas. Elemento Cuenta de la barra
  de estado y tarjeta de cuenta en Configuracion de Git Rudder.
- **Acciones.** Iniciar sesion (comando, tarjeta, menu de Cuentas), Usar un codigo, Cancelar, Cerrar
  sesion.
- **Estados.**
  - Sin sesion: barra `$(account) Git Rudder: sin sesión`; tarjeta con texto y boton "Iniciar sesión".
  - En curso: se abre el navegador en la pagina "Autorizar Git Rudder for VS Code" (W-05, brief #69) y sale la
    notificacion de progreso con Cancelar. A los 20 s sin respuesta, notificacion con "Usar un código" y
    "Abrir de nuevo".
  - Device flow (Codespaces en el navegador, o si se pide): modal informativo con el codigo en grande en
    `detail` y boton "Copiar código y abrir el navegador"; despues, progreso "Esperando la aprobación…".
  - Exito: mensaje de barra de estado. Si hay categorias de secretos activas y no hay boveda, sigue K-01
    (brief #63).
  - Fallido: notificacion de error con el motivo y "Reintentar" / "Usar un código".
  - Caducada o revocada (C-07): notificacion de aviso con "Iniciar sesión"; barra con fondo de aviso. Si
    el 401 trae el `code` de sesion revocada, el texto incluye la limpieza de secretos de C-07.
  - Cerrar sesion (K-09 y decision #57): modal con dos botones, porque un modal nativo no tiene casilla.
    Tambien desde el menu de Cuentas: VS Code pregunta primero y despues sale este modal. Las demas
    ventanas de VS Code sueltan la llave y ensenan un mensaje de barra de estado.
  - Barra con sesion: `$(cloud) Git Rudder`; sincronizando `$(sync~spin) Git Rudder`; sin conexion
    `$(debug-disconnect) Git Rudder`; caducada `$(warning) Git Rudder: inicia sesión`; nube apagada en esta
    instalacion `$(account) Git Rudder` con tooltip propio.

| Clave           | ES                                                                                                                                                                  | EN                                                                                                                                                     |
| --------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------ |
| sbSignedOut     | Git Rudder: sin sesión                                                                                                                                              | Git Rudder: Signed Out                                                                                                                                 |
| sbSignedOutTip  | Sin sesión. Git local funciona igual. Inicia sesión para sincronizar cuentas, llaves y preferencias.                                                                | Signed out. Local Git works the same. Sign in to sync accounts, keys and preferences.                                                                  |
| sbSignedInTip   | Sesión iniciada como {name} ({email}) · Sincronizado {ago}                                                                                                          | Signed in as {name} ({email}) · Synced {ago}                                                                                                           |
| sbCloudOffTip   | La nube de Git Rudder está desactivada en este VS Code.                                                                                                             | Git Rudder cloud is turned off in this VS Code.                                                                                                        |
| sbExpired       | Git Rudder: inicia sesión                                                                                                                                           | Git Rudder: Sign In                                                                                                                                    |
| cardSignedOut   | Inicia sesión para usar en este VS Code las cuentas, llaves y preferencias de tu cuenta de Git Rudder. Git funciona igual sin cuenta.                               | Sign in to use your Git Rudder account's accounts, keys and preferences in this VS Code. Git works the same without an account.                        |
| progressBrowser | Iniciando sesión en Git Rudder: termina en el navegador.                                                                                                            | Signing in to Git Rudder: finish in your browser.                                                                                                      |
| slowBrowser     | ¿No se abrió el navegador o no vuelve a VS Code?                                                                                                                    | Browser didn't open or didn't come back to VS Code?                                                                                                    |
| useCode         | Usar un código                                                                                                                                                      | Use a Code                                                                                                                                             |
| openAgain       | Abrir de nuevo                                                                                                                                                      | Open Again                                                                                                                                             |
| codeTitle       | Escribe este código en git-rudder.com/activate                                                                                                                      | Enter this code at git-rudder.com/activate                                                                                                             |
| codeDetail      | {code}. Caduca en 10 minutos. Apruébalo solo si lo pediste tú en este equipo.                                                                                       | {code}. It expires in 10 minutes. Only approve it if you requested it on this device.                                                                  |
| copyAndOpen     | Copiar código y abrir el navegador                                                                                                                                  | Copy Code and Open Browser                                                                                                                             |
| waitingApproval | Esperando la aprobación en el navegador…                                                                                                                            | Waiting for approval in the browser…                                                                                                                   |
| signedInMsg     | Sesión iniciada como {name}                                                                                                                                         | Signed in as {name}                                                                                                                                    |
| signInFailed    | No se pudo iniciar sesión: {reason}                                                                                                                                 | Couldn't sign in: {reason}                                                                                                                             |
| reasonDenied    | cancelaste la autorización en el navegador.                                                                                                                         | you cancelled the authorization in the browser.                                                                                                        |
| reasonExpired   | el código caducó.                                                                                                                                                   | the code expired.                                                                                                                                      |
| reasonNetwork   | no hay conexión con Git Rudder. Git local sigue funcionando.                                                                                                        | Git Rudder can't be reached. Local Git keeps working.                                                                                                  |
| signOutTitle    | ¿Cerrar sesión en Git Rudder?                                                                                                                                       | Sign out of Git Rudder?                                                                                                                                |
| signOutDetail   | Se borran de este equipo la llave de tu bóveda y las cuentas y llaves que llegaron de otros equipos. Las que añadiste aquí se quedan para que Git siga funcionando. | Your vault key and the accounts and keys that came from other devices are removed from this device. The ones you added here stay so Git keeps working. |
| signOutKeep     | Cerrar sesión                                                                                                                                                       | Sign Out                                                                                                                                               |
| signOutWipe     | Cerrar sesión y borrar también las añadidas aquí                                                                                                                    | Sign Out and Also Remove the Ones Added Here                                                                                                           |
| otherWindowOut  | Se cerró la sesión de Git Rudder en otra ventana.                                                                                                                   | Git Rudder was signed out in another window.                                                                                                           |

### 3. V-02 Sin carpeta abierta

- **Proposito.** Que el contenedor Git Rudder no aparezca vacio.
- **Muestra.** `viewsWelcome` en `gitRudder.welcome` con `when: workbenchState == empty`. Botones del texto:
  `[Abrir carpeta](command:vscode.openFolder)` y `[Clonar repositorio](command:gitRudder.clone)`.
- **Estados.** Uno solo. Variante remota: igual (Abrir carpeta abre el dialogo remoto).

| Clave      | ES                                                                                               | EN                                                                                      |
| ---------- | ------------------------------------------------------------------------------------------------ | --------------------------------------------------------------------------------------- |
| noFolder   | Abre una carpeta con un repositorio de Git, o clona uno, para ver su grafo. No necesitas cuenta. | Open a folder with a Git repository, or clone one, to see its graph. No account needed. |
| openFolder | Abrir carpeta                                                                                    | Open Folder                                                                             |
| cloneRepo  | Clonar repositorio                                                                               | Clone Repository                                                                        |

### 4. V-03 Carpeta que no es un repositorio

- **Muestra.** `viewsWelcome` con `when: workbenchState != empty && !gitRudder.hasRepository &&
gitRudder.gitState == ok && gitRudder.engineState == ready`. Botones "Iniciar repositorio" y "Clonar
  repositorio". En multi-root, "Iniciar repositorio" abre un QuickPick de carpetas.
- **Estados.** Una carpeta, varias carpetas, iniciando (progreso de la vista), iniciado (el contenedor
  pasa a las vistas normales y el grafo muestra V-09 vacio).

| Clave        | ES                                                         | EN                                                |
| ------------ | ---------------------------------------------------------- | ------------------------------------------------- |
| notRepo      | La carpeta abierta no es un repositorio de Git.            | The open folder isn't a Git repository.           |
| notRepoMulti | Ninguna de las carpetas abiertas es un repositorio de Git. | None of the open folders is a Git repository.     |
| initRepo     | Iniciar repositorio                                        | Initialize Repository                             |
| pickFolder   | Elige la carpeta donde iniciar el repositorio              | Choose the folder to initialize the repository in |

### 5. V-04 Git no instalado o anterior a 2.34

- **Muestra.** `viewsWelcome` con `when: gitRudder.gitState == missing` o `outdated`. El motor lee la ruta de
  git del ajuste `git.path` de VS Code (ADR 0010), y el texto lo dice. En remoto, git tiene que estar en
  la maquina remota: variante con `when: remoteName`.
- **Acciones.** `[Instalar Git](https://git-scm.com/downloads)`, `[Volver a comprobar](command:gitRudder.checkGit)`.
- **Estados.** Ausente, antiguo (con la version detectada en el detalle del grafo y en el tooltip),
  comprobando (progreso de la vista), remoto.

| Clave      | ES                                                                                                                                 | EN                                                                                                                              |
| ---------- | ---------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------- |
| gitMissing | Git Rudder necesita Git 2.34 o posterior y no lo encuentra en este equipo. Si está en otra ruta, indícala en el ajuste `git.path`. | Git Rudder needs Git 2.34 or later and can't find it on this machine. If it's somewhere else, set it in the `git.path` setting. |
| gitOld     | Git Rudder necesita Git 2.34 o posterior. La versión que encuentra es más antigua.                                                 | Git Rudder needs Git 2.34 or later. The version it found is older.                                                              |
| gitRemote  | Git Rudder corre en la máquina remota y necesita Git 2.34 o posterior instalado allí.                                              | Git Rudder runs on the remote machine and needs Git 2.34 or later installed there.                                              |
| installGit | Instalar Git                                                                                                                       | Install Git                                                                                                                     |
| checkAgain | Volver a comprobar                                                                                                                 | Check Again                                                                                                                     |

### 6. V-05 Motor no disponible

- **Proposito.** El motor es un proceso aparte (ADR 0010); su caida es un estado real.
- **Estados y donde se ven.**
  - Arrancando: barra de progreso de las vistas; los webviews muestran esqueletos (C-01).
  - Reiniciando tras una caida (1, 2, 4, 8 y 16 s): barra `$(sync~spin) Git Rudder: reiniciando el motor…`; en
    cada webview, banner fino arriba con las acciones deshabilitadas.
  - Detenido tras 5 caidas en 3 minutos: barra `$(error) Git Rudder: motor detenido` con fondo de error,
    notificacion de error con "Reintentar" y "Ver registro" (canal "Git Rudder Engine"), `viewsWelcome` en el
    contenedor y estado de panel completo en los webviews abiertos.
  - Version de protocolo distinta: es un fallo de empaquetado y no se reintenta.
  - Sin binario para la plataforma (VSIX equivocado instalado a mano).
  - Error de `engine/info` por git: no es caida; se ve V-04.

| Clave           | ES                                                                                                                                                                 | EN                                                                                                                                                           |
| --------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------ | ------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| sbRestarting    | Git Rudder: reiniciando el motor…                                                                                                                                  | Git Rudder: Restarting Engine…                                                                                                                               |
| bannerReconnect | Reconectando con el motor de Git Rudder…                                                                                                                           | Reconnecting to the Git Rudder engine…                                                                                                                       |
| sbStopped       | Git Rudder: motor detenido                                                                                                                                         | Git Rudder: Engine Stopped                                                                                                                                   |
| engineStopped   | El motor de Git Rudder se detuvo 5 veces en 3 minutos y no se volverá a arrancar solo.                                                                             | The Git Rudder engine stopped 5 times in 3 minutes and won't restart on its own.                                                                             |
| retry           | Reintentar                                                                                                                                                         | Retry                                                                                                                                                        |
| showLog         | Ver registro                                                                                                                                                       | Show Log                                                                                                                                                     |
| protocolBroken  | La extensión está dañada: el motor y la interfaz no son de la misma versión. Reinstala Git Rudder.                                                                 | The extension is damaged: the engine and the interface aren't the same version. Reinstall Git Rudder.                                                        |
| noBinary        | Esta instalación de Git Rudder no trae el motor para {os} {arch}. Instala la extensión desde el Marketplace o Open VSX para que VS Code elija la versión correcta. | This Git Rudder install doesn't include the engine for {os} {arch}. Install the extension from the Marketplace or Open VSX so VS Code picks the right build. |

### 7. V-06 Multi-root y varios repos en una carpeta

- **Muestra.** Repos detectados: carpetas del workspace, repos anidados y submodulos. Selector en tres
  sitios: elemento Repo de la barra de estado, `TreeView.description` de cada vista y un desplegable a la
  izquierda de la barra `tools` del grafo ("web-app ▾" en lugar de "⎇ rama"). Cada repo abre su propia
  pestana de grafo, "Grafo: web-app".
- **QuickPick "Elegir repositorio…".** Fila `$(repo) web-app`, descripcion con la ruta relativa al
  workspace, detalle con etiqueta "Submódulo de api" o "Anidado en monorepo" cuando aplica; el actual con
  `check`.
- **Estados.** Un repo (sin selector), varios, repo que desaparece (se cierra su pestana con aviso,
  ADR 0010).

| Clave       | ES                                                         | EN                                                      |
| ----------- | ---------------------------------------------------------- | ------------------------------------------------------- |
| pickRepo    | Elige el repositorio que muestran las vistas de Git Rudder | Choose the repository Git Rudder's views show           |
| graphTab    | Grafo: {repo}                                              | Graph: {repo}                                           |
| submoduleOf | Submódulo de {repo}                                        | Submodule of {repo}                                     |
| nestedIn    | Anidado en {repo}                                          | Nested in {repo}                                        |
| repoGone    | {repo} ya no es un repositorio. Cerramos su grafo.         | {repo} is no longer a repository. Its graph was closed. |

### 8. V-07 Conflictos de pull, rebase, cherry-pick y revert

- **Muestra.** El editor de merge (`cf`) gana una banda de operacion encima de la cabecera actual:
  icono, "Rebase en curso · commit 3 de 7: «Validar tokens OAuth»" y botones "Continuar", "Omitir commit"
  (rebase y cherry-pick) y "Abortar rebase". Etiquetas A y B segun la operacion (B-23). La vista Cambios
  muestra el grupo "Cambios de merge" y la misma banda en pequeno. La fila WIP del grafo dice que
  operacion hay.
- **"Editar a mano".** Abre el archivo en el editor nativo con los marcadores; al guardar sin marcadores
  cuenta como resuelto. Boton "Marcar como resuelto igualmente" para lineas que empiezan por `=======`
  (B-21).
- **Estados.** Con conflictos (Continuar deshabilitado con tooltip), todo resuelto, continuando
  (progreso), siguiente commit con conflictos (la banda avanza a "4 de 7"), terminado (mensaje de barra),
  abortando (modal de confirmacion de la seccion 1).

| Operacion     | Etiqueta A ES / EN                      | Etiqueta B ES / EN                                              |
| ------------- | --------------------------------------- | --------------------------------------------------------------- |
| Merge y pull  | Tu rama (actual) / Yours (current)      | Entrante ({branch}) / Incoming ({branch})                       |
| Rebase        | Base ({upstream}) / Base ({upstream})   | Commit que se reaplica ({sha}) / Commit being reapplied ({sha}) |
| Cherry-pick   | Tu rama (actual) / Yours (current)      | Cherry-pick de {sha} / Cherry-pick of {sha}                     |
| Revert        | Tu rama (actual) / Yours (current)      | Revert de {sha} / Revert of {sha}                               |
| Aplicar stash | Tu árbol de trabajo / Your working tree | Stash {index} / Stash {index}                                   |

| Clave        | ES                                                       | EN                                                          |
| ------------ | -------------------------------------------------------- | ----------------------------------------------------------- |
| opRebase     | Rebase en curso · commit {index} de {total}: «{subject}» | Rebase in progress · commit {index} of {total}: "{subject}" |
| opCherry     | Cherry-pick de {sha} en curso                            | Cherry-pick of {sha} in progress                            |
| opRevert     | Revert de {sha} en curso                                 | Revert of {sha} in progress                                 |
| opPull       | Pull de {remote}/{branch} con conflictos                 | Pull from {remote}/{branch} has conflicts                   |
| continue     | Continuar                                                | Continue                                                    |
| skip         | Omitir commit                                            | Skip Commit                                                 |
| abortOp      | Abortar {operation}                                      | Abort {operation}                                           |
| continueTip  | Resuelve todos los conflictos para continuar             | Resolve every conflict to continue                          |
| editInEditor | Editar a mano                                            | Edit Manually                                               |
| markResolved | Marcar como resuelto igualmente                          | Mark as Resolved Anyway                                     |
| wipOp        | // {operation} en curso · resuelve los conflictos        | // {operation} in progress · resolve conflicts              |

### 9. V-08 Repositorios grandes

- **Muestra.** El grafo carga por ventanas (`graph/window`), con filas virtualizadas de 28 px. Al final,
  fila "Cargar más" y, mientras llega, fila de esqueleto (C-01). Si las refs cambiaron entre paginas
  (`stale_window`), vuelve a la fila 0 con un mensaje de barra.
- **Estados.** Primera carga (esqueleto de 16 filas), cargando mas, todo cargado (sin fila final),
  historial cambiado.

| Clave       | ES                                                           | EN                                                        |
| ----------- | ------------------------------------------------------------ | --------------------------------------------------------- |
| loadMore    | Cargar más commits                                           | Load More Commits                                         |
| staleWindow | El historial cambió. Recargamos el grafo desde el principio. | The history changed. The graph was reloaded from the top. |

### 10. V-09 Repositorio vacio y HEAD separado

- **Repo vacio.** El grafo muestra un estado vacio centrado (icono `git-commit`, texto y boton "Ir a
  Cambios"); la vista Cambios sigue normal y el boton dice "Commit inicial en main".
- **HEAD separado.** Banda de aviso arriba del grafo y encima de la caja de mensaje de Cambios, con "Crear
  rama aquí…". Si Git Rudder muestra la rama en la barra de estado: `$(git-commit) a41f9c2 (separado)`.

| Clave       | ES                                                                                          | EN                                                                                     |
| ----------- | ------------------------------------------------------------------------------------------- | -------------------------------------------------------------------------------------- |
| emptyRepo   | Este repositorio aún no tiene commits. Prepara archivos y haz el primer commit en {branch}. | This repository has no commits yet. Stage files and make the first commit on {branch}. |
| goChanges   | Ir a Cambios                                                                                | Go to Changes                                                                          |
| firstCommit | Commit inicial en {branch}                                                                  | Initial commit on {branch}                                                             |
| detached    | HEAD separado en {sha}. Los commits que hagas aquí no pertenecen a ninguna rama.            | Detached HEAD at {sha}. Commits you make here don't belong to any branch.              |
| branchHere  | Crear rama aquí…                                                                            | Create Branch Here…                                                                    |
| sbDetached  | {sha} (separado)                                                                            | {sha} (detached)                                                                       |

### 11. V-10 Push rechazado, sin upstream y fallo de autenticacion

- **Rechazado.** Notificacion de error con "Pull y reintentar", "Force push (with lease)…" (modal de la
  seccion 1) y "Ver detalle".
- **Sin upstream.** Push sobre una rama sin upstream abre directamente "Publicar rama" (QuickInput de la
  seccion 1). La barra muestra `$(cloud-upload) Publicar`.
- **Autenticacion.** Notificacion de error con "Conectar una cuenta de {platform}" (V-15) o, si la
  cuenta existe, "Actualizar credenciales…".
- **Estados.** Los tres, mas red caida (notificacion con "Reintentar") y hook fallido (C-02 con su
  salida).

| Clave        | ES                                                           | EN                                                         |
| ------------ | ------------------------------------------------------------ | ---------------------------------------------------------- |
| pushRejected | Push rechazado: origin/{branch} tiene commits que no tienes. | Push rejected: origin/{branch} has commits you don't have. |
| pullRetry    | Pull y reintentar                                            | Pull and Retry                                             |
| viewDetail   | Ver detalle                                                  | View Details                                               |
| copyDetail   | Copiar detalle                                               | Copy Details                                               |
| noUpstream   | La rama {branch} todavía no está en ningún remoto.           | The branch {branch} isn't on any remote yet.               |
| publish      | Publicar rama                                                | Publish Branch                                             |
| authFailed   | No se pudo autenticar con {host}.                            | Couldn't authenticate with {host}.                         |
| connectAcc   | Conectar una cuenta de {platform}                            | Connect a {platform} Account                               |
| updateCreds  | Actualizar credenciales…                                     | Update Credentials…                                        |

### 12. V-11 Arbol sucio en checkout, merge o rebase

- **Muestra.** Solo cuando git se niega (si los cambios no chocan, git los lleva consigo y no se
  pregunta). Modal de aviso con la lista corta de archivos en `detail` y botones segun la operacion.
  Despues de un stash automatico, notificacion "Tus cambios están en stash@{0}" con "Aplicar ahora".

| Clave         | ES                                                                                          | EN                                                                            |
| ------------- | ------------------------------------------------------------------------------------------- | ----------------------------------------------------------------------------- |
| dirtyCheckout | No se puede hacer checkout de {branch}: tus cambios en {count} archivos se sobrescribirían. | Can't check out {branch}: your changes to {count} files would be overwritten. |
| stashAndGo    | Guardar en stash y continuar                                                                | Stash and Continue                                                            |
| autostash     | Rebase con autostash                                                                        | Rebase with Autostash                                                         |
| stashedNow    | Tus cambios están en stash@{0}.                                                             | Your changes are in stash@{0}.                                                |
| applyNow      | Aplicar ahora                                                                               | Apply Now                                                                     |

### 13. V-12 Staging parcial desde el diff nativo

- **Muestra.** Git Rudder abre `vscode.diff` con la izquierda en el indice y la derecha en el arbol de trabajo.
  Titulos de pestana (B-18): "{file} (Índice ↔ Árbol de trabajo)", "{file} (HEAD ↔ Índice)" para lo
  preparado y "{file} ({sha})" para un archivo de commit. Comandos de la seccion 1 en el menu
  contextual del editor y en la barra de titulo del editor (`editor/title`, iconos `add`, `remove`,
  `discard`). Sin seleccion actua sobre el hunk del cursor. Sin atajos propios: los del Git integrado
  (`Ctrl+K Ctrl+Alt+S`) ya existen y chocarian.
- **Estados.** Lineas preparadas (el diff se refresca), nada seleccionado (hunk del cursor), archivo
  binario (comandos ocultos), descartar (modal).

| Clave       | ES                                 | EN                            |
| ----------- | ---------------------------------- | ----------------------------- |
| diffWorking | {file} (Índice ↔ Árbol de trabajo) | {file} (Index ↔ Working Tree) |
| diffStaged  | {file} (HEAD ↔ Índice)             | {file} (HEAD ↔ Index)         |
| diffCommit  | {file} ({sha})                     | {file} ({sha})                |
| linesStaged | {count} líneas preparadas          | {count} lines staged          |

### 14. V-13 Historial de archivo

- **Muestra.** Pestana de grafo filtrada por ruta, "Historial: {file}", con las mismas filas de 28 px y
  el panel de detalle mostrando solo ese archivo. Sigue renombrados. No se usa `TimelineProvider`: es API
  propuesta y no se puede publicar en el Marketplace.
- **Estados.** Cargando, con commits, archivo sin historial (sin seguimiento o nuevo).

| Clave          | ES                                                 | EN                                          |
| -------------- | -------------------------------------------------- | ------------------------------------------- |
| historyTab     | Historial: {file}                                  | History: {file}                             |
| followsRenames | Incluye los commits anteriores a cambiar de nombre | Includes commits from before it was renamed |
| noHistory      | Este archivo aún no está en el historial.          | This file isn't in the history yet.         |

### 15. V-14 Crear tag y menu de tag

- **Muestra.** QuickInput "Crear tag" de la seccion 1 (desde la paleta, el titulo de Tags y el menu de
  commit "Crear tag aquí…"), arbol Tags y menu de tag (tambien en la pastilla del grafo, B-28). Checkout de
  un tag lleva a HEAD separado (V-09).
- **Estados.** Creado (mensaje de barra), creado y publicado, nombre ya existe (validacion en linea),
  borrar (modal con "Eliminar también en origin").

| Clave      | ES                                           | EN                                                  |
| ---------- | -------------------------------------------- | --------------------------------------------------- |
| tagName    | Nombre del tag                               | Tag name                                            |
| tagMsg     | Mensaje (opcional; vacío crea un tag ligero) | Message (optional; empty creates a lightweight tag) |
| tagPushQ   | Hacer push del tag a origin                  | Push the Tag to origin                              |
| tagLocal   | Solo en local                                | Local Only                                          |
| tagExists  | Ya existe un tag con ese nombre.             | A tag with that name already exists.                |
| tagCreated | Tag {tag} creado                             | Tag {tag} created                                   |

### 16. V-15 Sin cuenta de plataforma o host desconocido

- **Muestra.** El arbol Pull requests vacio con `viewsWelcome` segun `gitRudder.prState`. El texto de
  `viewsWelcome` es fijo; el host concreto va en `TreeView.message` encima.
- **QuickInput "Añadir cuenta de plataforma…"** (OAuth primero, X-07; el usuario y los permisos se leen,
  no se teclean):
  1. QuickPick de plataforma: GitHub, GitLab.com, GitLab propio, Gitea o Forgejo, GitHub Enterprise
     Server, Bitbucket Cloud, Azure DevOps. Nunca "App password" (X-08).
  2. Metodo: en GitHub "Iniciar sesión con GitHub (recomendado)" o "Token de grano fino"; en GitLab.com
     "Iniciar sesión con GitLab (recomendado)" o "Token personal"; en Bitbucket "Iniciar sesión con
     Bitbucket (recomendado)" o "API token". Los demas, solo token.
  3. Servidor (propios) o organizacion (Azure DevOps, con el aviso de alcance de organizacion).
  4. Token en InputBox oculto; mientras se prueba, `busy` y "Probando el token…".
  5. Resultado: "@{user} · permisos: repo, read:org" y InputBox "Etiqueta (opcional)".
  - GitHub OAuth: modal con el codigo de dispositivo y "Copiar código y abrir GitHub", despues progreso.
    GitLab y Bitbucket: navegador y progreso "Esperando a {platform} en el navegador…".
  - Con sesion, "Cuentas vinculadas" activo y sin boveda: sigue K-01; "Ahora no" la deja "Solo en este
    equipo" (K-05, brief #63).
- **Estados.** Sin cuenta, host desconocido, sin remoto, conectando, token rechazado, permisos
  insuficientes, SSO requerido, conectada.

| Clave          | ES                                                                                                                                        | EN                                                                                                                                      |
| -------------- | ----------------------------------------------------------------------------------------------------------------------------------------- | --------------------------------------------------------------------------------------------------------------------------------------- |
| prNoAccount    | Para ver y crear pull requests de este repositorio, conecta una cuenta de su plataforma.                                                  | To see and create pull requests for this repository, connect an account on its platform.                                                |
| prNoAccountMsg | El remoto está en {host}. No hay ninguna cuenta conectada para ese servidor.                                                              | The remote is on {host}. No account is connected for that host.                                                                         |
| prUnknownHost  | Git Rudder no reconoce la plataforma de este remoto. Si es GitLab, Gitea, Forgejo o GitHub Enterprise, añade una cuenta con ese servidor. | Git Rudder doesn't recognize this remote's platform. If it's GitLab, Gitea, Forgejo or GitHub Enterprise, add an account for that host. |
| prNoRemote     | Este repositorio no tiene remoto. Los pull requests necesitan uno.                                                                        | This repository has no remote. Pull requests need one.                                                                                  |
| addAccount     | Añadir cuenta…                                                                                                                            | Add Account…                                                                                                                            |
| pickPlatform   | ¿En qué plataforma está la cuenta?                                                                                                        | Which platform is the account on?                                                                                                       |
| oauthRec       | Iniciar sesión con {platform} (recomendado)                                                                                               | Sign In with {platform} (Recommended)                                                                                                   |
| tokenFine      | Token de grano fino                                                                                                                       | Fine-Grained Token                                                                                                                      |
| tokenPersonal  | Token personal                                                                                                                            | Personal Access Token                                                                                                                   |
| apiToken       | API token                                                                                                                                 | API Token                                                                                                                               |
| hostPrompt     | Servidor, por ejemplo gitlab.empresa.com                                                                                                  | Host, for example gitlab.company.com                                                                                                    |
| azureOrg       | Organización de Azure DevOps. El token tiene que tener alcance de esa organización.                                                       | Azure DevOps organization. The token must be scoped to that organization.                                                               |
| tokenPrompt    | Pega el token. Git Rudder lo guarda en el almacén seguro de VS Code y nunca lo vuelve a mostrar.                                          | Paste the token. Git Rudder stores it in VS Code's secret storage and never shows it again.                                             |
| testingToken   | Probando el token…                                                                                                                        | Testing the token…                                                                                                                      |
| tokenResult    | @{user} · permisos: {scopes}                                                                                                              | @{user} · scopes: {scopes}                                                                                                              |
| labelOpt       | Etiqueta (opcional): Trabajo, Personal…                                                                                                   | Label (optional): Work, Personal…                                                                                                       |
| ghCodeTitle    | Escribe este código en github.com/login/device                                                                                            | Enter this code at github.com/login/device                                                                                              |
| copyOpenGh     | Copiar código y abrir GitHub                                                                                                              | Copy Code and Open GitHub                                                                                                               |
| waitPlatform   | Esperando a {platform} en el navegador…                                                                                                   | Waiting for {platform} in the browser…                                                                                                  |
| tokenRejected  | {platform} rechazó el token.                                                                                                              | {platform} rejected the token.                                                                                                          |
| scopesMissing  | Al token le faltan permisos: {scopes}.                                                                                                    | The token is missing scopes: {scopes}.                                                                                                  |
| accConnected   | Cuenta @{user} conectada                                                                                                                  | Account @{user} connected                                                                                                               |

### 17. V-16 Errores de la API de la plataforma y de la IA

- **Plataforma.** En el arbol Pull requests, `TreeView.message` con el error y una fila de accion; en el
  formulario Crear PR, caja de error en linea.
- **IA.** En la vista Cambios, debajo de la caja de mensaje, linea de error en `--vscode-errorForeground`
  con su accion; si la generacion la lanzo la paleta, notificacion. La primera vez que un miembro genera
  con una llave de la organizacion, modal informativo (X-03).
- **QuickInput "Añadir llave de IA…"** (X-05 y X-10, decision #54): 1. proveedor (Anthropic, OpenAI,
  Google Gemini, Mistral, Groq, OpenRouter, Ollama local, Compatible con OpenAI); 2. URL base (OpenRouter
  la trae puesta, Ollama `http://localhost:11434`, compatible obligatoria); 3. llave oculta (no en Ollama),
  con "Comprobando la llave…"; 4. modelo: QuickPick con la lista del proveedor, la recomendada arriba con
  descripcion "Recomendado · Rápido" (texto libre solo en compatible con OpenAI); 5. etiqueta. Sin
  selector de alcance: lo que se anade aqui es siempre personal.
- **Estados.** Token de plataforma rechazado, limite de peticiones, SSO requerido; IA sin red, llave
  invalida, sin cuota, modelo retirado, limite diario de la organizacion, aviso de proxy; lista de modelos
  cargando, sin red (lista sin recomendacion), sin lista (texto libre o reintentar).

| Clave          | ES                                                                                                                                                  | EN                                                                                                                                   |
| -------------- | --------------------------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------ |
| plat401        | {platform} rechazó el token de @{user}.                                                                                                             | {platform} rejected @{user}'s token.                                                                                                 |
| platRate       | Límite de peticiones de {platform}. Lo volvemos a intentar a las {time}.                                                                            | {platform} rate limit reached. We'll try again at {time}.                                                                            |
| platSso        | La organización {org} exige autorizar el token con SSO.                                                                                             | The {org} organization requires SSO authorization for the token.                                                                     |
| authorizeSso   | Autorizar en {platform}                                                                                                                             | Authorize on {platform}                                                                                                              |
| aiNetwork      | No hay conexión con {provider}.                                                                                                                     | Can't reach {provider}.                                                                                                              |
| aiBadKey       | {provider} rechazó la llave «{label}».                                                                                                              | {provider} rejected the key "{label}".                                                                                               |
| aiQuota        | Tu cuenta de {provider} no tiene saldo o llegó a su cuota.                                                                                          | Your {provider} account is out of credit or hit its quota.                                                                           |
| aiModelGone    | El modelo {model} ya no está disponible.                                                                                                            | The model {model} is no longer available.                                                                                            |
| pickModel      | Elegir otro modelo                                                                                                                                  | Choose Another Model                                                                                                                 |
| reviewKey      | Revisar llave                                                                                                                                       | Review Key                                                                                                                           |
| aiOrgLimit     | Llegaste a tu límite diario de IA en {org} ({count} peticiones). Se renueva mañana.                                                                 | You reached your daily AI limit in {org} ({count} requests). It resets tomorrow.                                                     |
| orgProxyTitle  | Esta llave es de tu organización                                                                                                                    | This key belongs to your organization                                                                                                |
| orgProxyDetail | Para generar el mensaje, los cambios preparados pasan por los servidores de Git Rudder, que usan la llave de {org}. Con una llave personal no pasa. | To generate the message, your staged changes go through Git Rudder's servers, which use {org}'s key. With a personal key they don't. |
| generate       | Generar                                                                                                                                             | Generate                                                                                                                             |
| useMyKey       | Usar mi llave                                                                                                                                       | Use My Key                                                                                                                           |
| checkingKey    | Comprobando la llave…                                                                                                                               | Checking the key…                                                                                                                    |
| recFast        | Recomendado · Rápido                                                                                                                                | Recommended · Fast                                                                                                                   |
| recBalanced    | Recomendado · Equilibrado                                                                                                                           | Recommended · Balanced                                                                                                               |
| modelsFail     | No se pudo cargar la lista de modelos de {provider}.                                                                                                | Couldn't load {provider}'s model list.                                                                                               |

### 18. V-17 Detalle de PR

- **Muestra.** Minimo de la primera version: clic en el PR revela su commit en el grafo si esta
  descargado; accion en linea `link-external` abre la plataforma; tooltip Markdown de la seccion 1. El
  detalle completo en webview queda para despues.
- **Estados.** PR con rama local, sin rama local (Checkout de la rama la crea), PR de un fork.

### 19. V-18 Git Rudder Desktop no instalado

- **Muestra.** "Abrir en Git Rudder Desktop" lanza `git-rudder://open?repo=...`. Si `openExternal` devuelve falso,
  notificacion con "Descargar Git Rudder Desktop" (`https://git-rudder.com/#download`). En remoto el comando
  no aparece: Desktop no puede abrir una carpeta de otra maquina.

| Clave      | ES                                                           | EN                                                               |
| ---------- | ------------------------------------------------------------ | ---------------------------------------------------------------- |
| noDesktop  | Git Rudder Desktop no parece estar instalado en este equipo. | Git Rudder Desktop doesn't seem to be installed on this machine. |
| getDesktop | Descargar Git Rudder Desktop                                 | Download Git Rudder Desktop                                      |

### 20. V-19 Recorrido de bienvenida

- **Muestra.** `contributes.walkthroughs` con id `gitRudder.start`. Cuatro pasos con imagen SVG a la derecha
  (estilo Dark y Light Modern) y el boton del paso; se completan con `onCommand`. El orden pone primero lo
  que funciona sin cuenta.

| Paso | Titulo ES / EN                                                          | Descripcion ES                                                                             | Descripcion EN                                                                            | Boton ES / EN               |
| ---- | ----------------------------------------------------------------------- | ------------------------------------------------------------------------------------------ | ----------------------------------------------------------------------------------------- | --------------------------- |
| -    | Empieza con Git Rudder / Get Started with Git Rudder                    | El grafo de tu repositorio, conflictos línea a línea e IA con tu llave, dentro de VS Code. | Your repository's graph, line-by-line conflicts and AI with your own key, inside VS Code. | -                           |
| 1    | Abre el grafo / Open the graph                                          | Todas las ramas, tags y stashes del repositorio abierto. Atajo: {shortcut}.                | Every branch, tag and stash in the open repository. Shortcut: {shortcut}.                 | Abrir grafo / Open Graph    |
| 2    | Conecta tu plataforma / Connect your platform                           | Para ver y crear pull requests en GitHub, GitLab, Bitbucket, Gitea o Azure DevOps.         | To see and create pull requests on GitHub, GitLab, Bitbucket, Gitea or Azure DevOps.      | Añadir cuenta / Add Account |
| 3    | Añade una llave de IA (opcional) / Add an AI key (optional)             | Genera mensajes de commit y descripciones de PR. Cada generación la paga tu llave.         | Generate commit messages and PR descriptions. Your key pays for each generation.          | Añadir llave / Add Key      |
| 4    | Sincroniza con tu cuenta (opcional) / Sync with your account (optional) | Usa las mismas cuentas, llaves y preferencias que en Git Rudder Desktop.                   | Use the same accounts, keys and preferences as in Git Rudder Desktop.                     | Iniciar sesión / Sign In    |

### 21. V-20 Confianza, remoto y entornos no soportados

- **Modo restringido.** `untrustedWorkspaces.supported: false` (ADR 0010): VS Code desactiva Git Rudder y
  ensena la descripcion del manifiesto. Git Rudder no dibuja nada propio.
- **vscode.dev y github.dev.** `virtualWorkspaces` con `supported: false` y descripcion.
- **Remoto (SSH, WSL, contenedores, Codespaces).** La extension y el motor corren en el remoto; los
  textos de V-04 usan la variante remota. En Codespaces en el navegador, el inicio de sesion va directo
  al device flow (V-01).

| Clave         | ES                                                                                                                                                  | EN                                                                                                                                         |
| ------------- | --------------------------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------ |
| untrustedDesc | Git Rudder ejecuta Git en la carpeta, y Git puede ejecutar hooks y configuración del propio repositorio. Confía en la carpeta para usar Git Rudder. | Git Rudder runs Git in the folder, and Git can run hooks and configuration from the repository itself. Trust the folder to use Git Rudder. |
| virtualDesc   | Git Rudder necesita Git y archivos locales. No funciona en vscode.dev ni en github.dev.                                                             | Git Rudder needs Git and local files. It doesn't work on vscode.dev or github.dev.                                                         |

### 22. V-21 Convivencia con el Git integrado y con GitLens

- **Reglas.** Git Rudder no registra un proveedor SCM: no hay segundo cuadro de commit en la
  vista de control de codigo. La rama y la sincronizacion de la barra de estado solo aparecen si
  `git.enabled` es falso. Los comandos de lineas seleccionadas solo salen en diffs abiertos por Git Rudder.
  Git Rudder no contribuye blame ni anotaciones en el margen (V-23 es P2), asi que no choca con GitLens.
- **Estados.** Git integrado activo (barra sin rama de Git Rudder) y Git integrado apagado (barra con rama y
  sincronizacion de Git Rudder). Se especifican las dos barras; no se dibujan.

### 23. V-22 Ajustes: Settings Sync de VS Code o nube de Git Rudder (decision #58)

- **Regla.** Lo compartido con Desktop vive en la nube de Git Rudder, no en `settings.json`. VS Code nunca
  sincroniza `SecretStorage` ni los ajustes de ambito `machine`, asi que no hay dos sincronizaciones sobre
  la misma clave. Tema e idioma siguen a VS Code.

| Ajuste                                                                                              | Donde vive                        | Quien lo sincroniza           |
| --------------------------------------------------------------------------------------------------- | --------------------------------- | ----------------------------- |
| `gitRudder.cloud.enabled` (usar la nube de Git Rudder)                                              | `settings.json`, ambito `machine` | Nadie: es de esta instalacion |
| `gitRudder.engine.logLevel`                                                                         | `settings.json`, ambito `machine` | Nadie                         |
| `gitRudder.engine.path` (solo desarrollo)                                                           | `settings.json`, ambito `machine` | Nadie                         |
| Git: fetch automatico, pull, firmar, podar                                                          | Motor, documento `git`; webview   | Nube de Git Rudder            |
| IA: funciones, llave y modelo por funcion, idioma, Conventional Commits, exclusiones, instrucciones | Motor, documento `ai`; webview    | Nube de Git Rudder            |
| Cuentas de plataforma y llaves de IA                                                                | `SecretStorage` y boveda; webview | Nube de Git Rudder (boveda)   |
| Tema e idioma                                                                                       | VS Code                           | VS Code                       |

Hoy `package.json` declara `engine.path` y `engine.logLevel` con ambito `application`, y Settings Sync si
los sincroniza: queda como arreglo de implementacion, no de diseno.

- **Editor de ajustes nativo.** Solo tres entradas bajo "Git Rudder" (se especifican, no se dibujan).
- **Webview "Configuración de Git Rudder".** Secciones `setSections` en este orden, con la etiqueta de
  categoria gris ("Git Rudder › IA: ") del prototipo:
  1. Cuenta de Git Rudder: tarjeta `isEntity` (nombre, correo, organizacion y plan), casilla "Usar la
     configuración de mi cuenta de Git Rudder" (escribe `gitRudder.cloud.enabled`), botones "Abrir panel web" y
     "Cerrar sesión…". Sin sesion: la tarjeta de V-01.
  2. Plataformas Git: filas `isEntity` con estado real (B-39: "Comprobada hace 2 h", "Caduca en 3 días",
     "Token rechazado") y menu Probar conexión, Hacer predeterminada, Actualizar credenciales…,
     Desconectar…; boton "Añadir cuenta…" que abre el QuickInput de V-15.
  3. IA: llaves personales y, de solo lectura, las de la organizacion con insignia "De la organización"
     (sin menu salvo "Probar conexión" para admins); boton "Añadir llave…". Debajo, por cada funcion
     (Mensajes de commit, Pull requests, Resumen de commits): casilla, selector de llave y selector de
     modelo. El selector de modelo se rellena con la lista del proveedor y marca la recomendada
     ("Recomendado · Rápido" en commits, "Recomendado · Equilibrado" en PRs y resumenes); primera opcion
     "Modelo de la llave ({model})". Con llave de la organizacion, el modelo va fijo y deshabilitado. Sin
     red ni cache, la lista sin recomendacion. Despues: idioma del texto generado ("Igual que VS Code",
     English, Español), Conventional Commits, patrones excluidos e instrucciones adicionales. Sin coste por
     generacion.
  4. Git: fetch automatico, pull por defecto, firmar commits, podar.
- **Estados.** Con sesion y nube activa (linea C-06 "Se sincroniza con tu cuenta de Git Rudder · hace 2 min"),
  sin sesion o nube apagada ("Solo en este VS Code"), cambio llegado de otro dispositivo (C-06), boveda
  bloqueada (K-04, brief #63).

| Clave         | ES                                                                                                                                                                                                           | EN                                                                                                                                                                                                        |
| ------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| cfgCloud      | Usar la configuración de mi cuenta de Git Rudder                                                                                                                                                             | Use my Git Rudder account settings                                                                                                                                                                        |
| cfgCloudD     | Este VS Code usa las cuentas, las llaves de IA y las preferencias de Git e IA de tu cuenta. Qué se guarda en la nube se elige en tu cuenta. El tema, el idioma y el resto del editor los sincroniza VS Code. | This VS Code uses your account's linked accounts, AI keys and Git and AI preferences. What's stored in the cloud is chosen in your account. VS Code syncs the theme, language and the rest of the editor. |
| cfgLogLevel   | Cuánto escribe el motor de Git Rudder en el canal de salida "Git Rudder Engine". Solo en este equipo.                                                                                                        | How much the Git Rudder engine writes to the "Git Rudder Engine" output channel. This machine only.                                                                                                       |
| cfgHeader     | Estas preferencias las comparte Git Rudder Desktop y se guardan en tu cuenta de Git Rudder. No aparecen en la configuración de VS Code.                                                                      | Git Rudder Desktop shares these preferences and they're stored in your Git Rudder account. They don't appear in VS Code's settings.                                                                       |
| cfgLocal      | Solo en este VS Code                                                                                                                                                                                         | This VS Code only                                                                                                                                                                                         |
| fromOrg       | De la organización                                                                                                                                                                                           | From organization                                                                                                                                                                                         |
| keyModel      | Modelo de la llave ({model})                                                                                                                                                                                 | Key's model ({model})                                                                                                                                                                                     |
| orgModelFixed | Fijado por {org}: {model}                                                                                                                                                                                    | Set by {org}: {model}                                                                                                                                                                                     |
| aiHint        | Para los mensajes de commit te recomendamos un modelo rápido y barato. Para PRs y resúmenes, uno equilibrado. Cada generación la paga tu llave.                                                              | For commit messages we recommend a fast, low-cost model. For PRs and summaries, a balanced one. Your key pays for every generation.                                                                       |
| excludeLabel  | Archivos que nunca se envían a la IA                                                                                                                                                                         | Files never sent to AI                                                                                                                                                                                    |
| excludeD      | Además de .env, *.pem, *.key y otros que Git Rudder excluye siempre.                                                                                                                                         | On top of .env, *.pem, *.key and others Git Rudder always excludes.                                                                                                                                       |
| matchVs       | Igual que VS Code                                                                                                                                                                                            | Match VS Code                                                                                                                                                                                             |

### 24. B-62 Atajo del grafo

- **Elegido:** `Ctrl+K Ctrl+G` en Windows y Linux, `Cmd+K Cmd+G` en macOS, sin `when` (global). Sustituye
  al `⇧⌘G` del prototipo, que en Windows y Linux abre la vista de control de codigo (`Ctrl+Shift+G`) y en
  macOS es "Buscar anterior".
- **Comprobado** contra las combinaciones por defecto de VS Code 1.138 con sus extensiones integradas
  (volcado del fichero de atajos por defecto de Linux, que comparte tabla con Windows, y busqueda del
  acorde en el paquete del workbench, donde estan tambien las variantes de macOS): ningun comando usa
  `Ctrl+K Ctrl+G` ni `Cmd+K Cmd+G`. Descartados: `Ctrl+Alt+G` (secuencia del terminal), `Ctrl+Shift+Alt+G`
  (accesibilidad), `Alt+G` (en macOS escribe un caracter). Extensiones de terceros (GitLens) no se
  comprobaron; el usuario puede reasignarlo.
- **Donde se ve.** En la paleta junto a "Git Rudder: Abrir grafo", en el tooltip del boton de titulo y en el
  paso 1 del recorrido.

## Reglas

- La paleta, los QuickInput, las notificaciones, la barra de estado, los arboles, los menus, el editor de
  ajustes y los `viewsWelcome` son de VS Code: se especifican y no se dibujan; nada de cromo propio que los
  imite en los webviews. Las notificaciones no llevan "Origen: Git Rudder": VS Code pone el origen.
- Las claves con contador (`{count}`, `{days}`, `{seconds}`) llevan variantes `_one` y `_other`
  (ADR 0017); las tablas escriben la forma plural.
- Codicons siempre; `cloud-offline` no existe en Codicons, usar `debug-disconnect` para sin conexion.
  Colores solo `--vscode-*` en los webviews. Boton de peligro en webview con el color de error del tema
  (token de peligro que fije el brief #62), nunca el hex `#c72e0f` del prototipo.
- Toda accion destructiva confirma (B-29): modal nativo con el verbo en el boton, o boton de peligro en
  webview.
- Mensaje de commit completo, resumen, linea en blanco y cuerpo (B-07); contador de 72 en grafemas (B-09).
  Diff etiquetado `(Índice)` o con el sha (B-18). Valores reales en la barra (B-33). Identidad de Git Rudder,
  no de plataforma (B-40). Idiomas: solo es y en, con ingles de respaldo; nada de frances (B-47).
- Cuentas de plataforma: nunca compartidas con el equipo. Llaves de la organizacion: solo lectura, con
  "De la organización" y el aviso de proxy (X-03, X-05); sin selector de alcance al anadir; Ollama nunca es
  de organizacion.
- Textos de boveda: nadie puede recuperar los secretos sin la frase o el codigo (brief #63).
- Modelos: el selector se rellena con la lista del proveedor y marca el nivel recomendado; nunca una
  constante en el binario; sin precio por generacion (decision #54).
- Git e IA no van a `settings.json`; solo ajustes de ambito `machine`. Nada en `globalState.setKeysForSync`.
- Dominio en los textos: `git-rudder.com` (web) y `api.git-rudder.com` (API).
- Sin ubicacion de sesiones en ningun sitio (decision #53); VS Code no lista sesiones: eso es del panel.

## Entregable

Un zip `git-rudder-handoff-68-vscode.zip` con la misma estructura que `docs/design/handoff/`:

- `Git Rudder VS Code.dc.html` corregido: sin paleta, notificaciones, quick input ni barra de estado dibujados
  en HTML como si fueran de Git Rudder; solo los webviews (vista Cambios, grafo con detalle, editor de merge,
  crear PR y Configuracion de Git Rudder) con sus estados. Props `vsTheme` y `vsLanguage`, y un prop `state`
  con un valor por estado y el id de `gaps.md` delante: `V-01-sin-sesion`, `V-05-reiniciando`,
  `V-05-detenido`, `V-06`, `V-07-rebase`, `V-07-cherry-pick`, `V-08`, `V-09-vacio`, `V-09-separado`,
  `V-15`, `V-16-ia`, `V-22-sin-sesion`, `V-22-nube-apagada`, mas `normal`.
- Cuatro SVG del recorrido (V-19), en claro y oscuro: `walkthrough-1-grafo`, `walkthrough-2-plataforma`,
  `walkthrough-3-ia` y `walkthrough-4-cuenta`, con sufijos `-light.svg` y `-dark.svg`.
- `git-rudder-data.js` con todas las claves de este brief en un objeto `VS` con espacios de nombres
  (`vscode.welcome.noFolder`, `vscode.status.signedOut`...), en es y en. `support.js` sin cambios.
- `README.md` solo con lo nuevo: la especificacion de contribuciones de este brief pasada a limpio
  (comandos, atajo, vistas y arboles, menus, barra de estado, notificaciones, QuickInput, modales,
  ajustes, recorrido y textos del manifiesto) y los estados de los webviews con sus ids.

Lo nativo no se dibuja: se especifica con su texto en el README (regla comun de los briefs).

## Hecho cuando

- [ ] Especificación de contribuciones escrita
- [ ] Estados V-01 a V-22 definidos
- [ ] Atajo del grafo elegido sin choques con VS Code
