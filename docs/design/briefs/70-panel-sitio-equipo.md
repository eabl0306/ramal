# Brief #70: panel, sitio publico y equipo (W-11 a W-30, W-36, X-01, X-02, X-09, X-12)

Issue: [eabl0306/git-rudder#70](https://github.com/eabl0306/git-rudder/issues/70). Fuente: `docs/design/gaps.md`
(seccion 1.5, X-01, X-02, X-03, X-04, X-09, X-12 y bugs B-06, B-50 a B-59), `docs/api-surface.md`
(Perfil, Dispositivos, Organizaciones, Miembros, Invitaciones, Asientos, Suscripcion, Llaves de IA de
organizacion, Publicaciones), ADR 0013 y 0016, y las decisiones del 2026-10-01: equipos, llaves de IA de
organizacion y facturacion entran en la primera version publica (#49); planes y precios (#51); Paddle
(#50); sin ubicacion en sesiones (#53); modelos recomendados (#54); matriz de descargas (#52, #55, #56).

## Contexto

- Producto: Git Rudder Web (`handoff/Git Rudder Web.dc.html`), en `https://git-rudder.com`.
- Landing: hero, marco de vista previa (barra de 34 px con tres puntos), rejilla de funciones numeradas,
  seccion VS Code, **tarjetas de descarga** (`platforms`, con filas `alts` y caja de comando), CTA y pie.
- Panel: barra superior de 56 px con **selector de organizacion** (`onOrg`) y menu de usuario, barra
  lateral de 232 px (`dNav`, con contadores), cabecera de seccion (`dh`, titulo, subtitulo y boton),
  **mosaicos** (`ov.tiles`), **lista de dispositivos** (`ov.devices`), **tarjetas de workspace**
  (`wsCards`, con interruptor de compartir), **tabla de repositorios** (`repoRows`), **grupos de cuentas**
  (`accGroups`), **tarjeta de asientos y tabla de miembros** (`tm`), **tarjetas con filas** (`cards`,
  `rw`), modal generico de 460 px (580 de ancho cuando hay mas campos), menu contextual y toasts.
- Estados comunes (esqueleto, error, vacio, sin conexion, sincronizacion, solo lectura por rol): patrones
  C-01 a C-08 del brief #62. La tarjeta de boveda de la web (K-10) es del brief #63. Autenticacion,
  contrasena e identidades (W-33, W-34, W-35): brief #69.

## Pantallas y estados

Todo en claro y oscuro, a 1280 px, 768 px y 375 px. El panel se dibuja con tres roles (titular, admin,
miembro) y dos contextos (Personal y una organizacion).

**Navegacion del panel (aplica a todas las secciones).** Selector de organizacion: "Personal", cada
organizacion con sus iniciales y rol, separador, "Crear organización…". Barra lateral, en este orden:

| Seccion                  | ES                       | EN              | Se ve                       |
| ------------------------ | ------------------------ | --------------- | --------------------------- |
| Resumen                  | Resumen                  | Overview        | siempre                     |
| Workspaces               | Workspaces               | Workspaces      | siempre                     |
| Repositorios             | Repositorios             | Repositories    | siempre                     |
| Cuentas vinculadas       | Cuentas vinculadas       | Linked accounts | siempre                     |
| Equipo                   | Equipo                   | Team            | siempre (en Personal, W-13) |
| IA de la organizacion    | IA de la organización    | Organization AI | contexto de organizacion    |
| Facturacion              | Facturación              | Billing         | contexto de organizacion    |
| Configuracion en la nube | Configuración en la nube | Cloud settings  | siempre                     |
| Cuenta                   | Cuenta                   | Account         | siempre                     |

"Descargar" de la barra superior abre las descargas dentro del panel, sin salir (B-58).

### 1. W-11 Aceptar invitacion

- **Proposito.** Que las invitaciones se puedan aceptar.
- **Muestra.** `/invitations#token=...`: pagina publica con una tarjeta de 420 px: cuadrado con las
  iniciales de la organizacion, "{inviter} te invita a unirte a {org}", chip de rol, "Invitación para
  {email} · caduca el {date}" y botones segun el estado. En el Resumen del panel, tarjeta "Invitaciones
  pendientes" (solo con el email confirmado) con "Aceptar" y "Rechazar" por fila.
- **Estados.** Cargando; sin sesion ("Crear cuenta con {email}", que abre el registro con el email
  prellenado y bloqueado, y "Ya tengo cuenta"); sesion con el mismo email ("Unirme a {org}", ocupado y
  exito: panel en el contexto de la organizacion con toast); sesion con otro email (409); caducada (410);
  no existe o retirada (404); ya miembro (como exito); rechazada desde el panel (toast).

| Clave       | ES                                                                                 | EN                                                                            |
| ----------- | ---------------------------------------------------------------------------------- | ----------------------------------------------------------------------------- |
| invTitle    | {inviter} te invita a unirte a {org}                                               | {inviter} invited you to join {org}                                           |
| invMeta     | Invitación para {email} · caduca el {date}                                         | Invitation for {email} · expires {date}                                       |
| invSignup   | Crear cuenta con {email}                                                           | Sign up with {email}                                                          |
| invHaveAcc  | Ya tengo cuenta: iniciar sesión                                                    | I have an account: log in                                                     |
| invLocked   | La invitación es para este email.                                                  | The invitation is for this email.                                             |
| invJoin     | Unirme a {org}                                                                     | Join {org}                                                                    |
| invJoining  | Uniéndote…                                                                         | Joining…                                                                      |
| invJoined   | Te uniste a {org}                                                                  | You joined {org}                                                              |
| invMismatch | Esta invitación es para {email} y has iniciado sesión como {current}.              | This invitation is for {email} and you're logged in as {current}.             |
| invSwitch   | Cerrar sesión y cambiar de cuenta                                                  | Log out and switch account                                                    |
| invExpired  | La invitación caducó. Pide a quien te invitó que te envíe otra.                    | The invitation expired. Ask whoever invited you to send another.              |
| invGone     | Esta invitación ya no existe. Puede que la hayan retirado o que ya la hayas usado. | This invitation no longer exists. It may have been withdrawn or already used. |
| invPendingT | Invitaciones pendientes                                                            | Pending invitations                                                           |
| invRow      | {org} · {role} · te invitó {inviter}                                               | {org} · {role} · invited by {inviter}                                         |
| invAccept   | Aceptar                                                                            | Accept                                                                        |
| invDecline  | Rechazar                                                                           | Decline                                                                       |
| invDeclined | Invitación rechazada                                                               | Invitation declined                                                           |

### 2. W-12 Crear organizacion y ajustes de la organizacion

- **Crear.** Desde el selector y desde W-13: modal con "Nombre de la organización" (1 a 60 caracteres) y
  caja informativa con la prueba y el precio. Exito: el panel cambia al contexto nuevo y sale un toast.
- **Ajustes.** Tarjeta "Organización" al final de Equipo, con filas: Nombre (campo y "Guardar", admins),
  Titular (nombre; el titular ve "Transferir titularidad…"), Salir de la organizacion, Eliminar
  organizacion (titular, peligro). Sin avatar: el contrato no lo tiene, se usan las iniciales.
- **Transferir.** Selector de miembro, aviso de lo que cambia, W-35 y boton de peligro.
- **Salir.** Confirmacion de peligro. El titular la ve deshabilitada con tooltip.
- **Eliminar.** W-35, modal de peligro con el nombre escrito para confirmar. Bloqueada si hay una
  suscripcion activa, con salida a Facturacion.
- **Estados.** Creando, creada, email sin confirmar (boton deshabilitado con `vNeedTip` del brief #69),
  limite de 5, renombrada (toast), transferida (toast, el titular pasa a admin), salir, eliminar,
  bloqueada por suscripcion.

| Clave        | ES                                                                                                                                          | EN                                                                                                                                      |
| ------------ | ------------------------------------------------------------------------------------------------------------------------------------------- | --------------------------------------------------------------------------------------------------------------------------------------- |
| orgNewTitle  | Crear organización                                                                                                                          | Create organization                                                                                                                     |
| orgName      | Nombre de la organización                                                                                                                   | Organization name                                                                                                                       |
| orgTrialInfo | Empiezas con 14 días de prueba de Git Rudder Pro, sin tarjeta. Después: 8 USD por asiento al mes con pago anual, o 10 USD con pago mensual. | You start with a 14-day Git Rudder Pro trial, no card needed. After that: $8 per seat per month billed annually, or $10 billed monthly. |
| orgCreate    | Crear organización                                                                                                                          | Create organization                                                                                                                     |
| orgCreated   | Organización {org} creada. Tu prueba termina el {date}.                                                                                     | Organization {org} created. Your trial ends on {date}.                                                                                  |
| orgLimit     | Ya creaste 5 organizaciones, el máximo.                                                                                                     | You've already created 5 organizations, the maximum.                                                                                    |
| orgCard      | Organización                                                                                                                                | Organization                                                                                                                            |
| orgRenamed   | Organización renombrada                                                                                                                     | Organization renamed                                                                                                                    |
| orgOwnerRow  | Titular                                                                                                                                     | Owner                                                                                                                                   |
| orgTransfer  | Transferir titularidad…                                                                                                                     | Transfer ownership…                                                                                                                     |
| orgTransferT | Transferir {org}                                                                                                                            | Transfer {org}                                                                                                                          |
| orgTransferI | Pasarás a ser admin. {name} gestionará la facturación y podrá eliminar la organización.                                                     | You'll become an admin. {name} will manage billing and can delete the organization.                                                     |
| orgTransferB | Transferir titularidad                                                                                                                      | Transfer ownership                                                                                                                      |
| orgLeave     | Salir de la organización…                                                                                                                   | Leave organization…                                                                                                                     |
| orgLeaveTip  | Transfiere la titularidad antes de salir.                                                                                                   | Transfer ownership before leaving.                                                                                                      |
| orgLeaveT    | ¿Salir de {org}?                                                                                                                            | Leave {org}?                                                                                                                            |
| orgLeaveI    | Dejarás de ver sus workspaces compartidos y de usar sus llaves de IA. Tus workspaces privados dentro de {org} se eliminan.                  | You'll stop seeing its shared workspaces and using its AI keys. Your private workspaces in {org} are deleted.                           |
| orgLeaveB    | Salir                                                                                                                                       | Leave                                                                                                                                   |
| orgDelete    | Eliminar organización…                                                                                                                      | Delete organization…                                                                                                                    |
| orgDeleteI   | Se borran sus miembros, invitaciones, workspaces de la organización, llaves de IA y su consumo. No se puede deshacer.                       | Its members, invitations, organization workspaces, AI keys and usage are deleted. This can't be undone.                                 |
| orgTypeName  | Escribe {org} para confirmar                                                                                                                | Type {org} to confirm                                                                                                                   |
| orgDeleteB   | Eliminar organización                                                                                                                       | Delete organization                                                                                                                     |
| orgHasSub    | Cancela la suscripción de {org} antes de eliminarla.                                                                                        | Cancel {org}'s subscription before deleting it.                                                                                         |
| goBilling    | Ir a Facturación                                                                                                                            | Go to Billing                                                                                                                           |

### 3. W-13 Contexto personal

- **Muestra.** Con "Personal" elegido: Equipo ensena un estado vacio con llamada a crear una organizacion
  (icono, titulo, texto, "Crear organización", linea "14 días de prueba, sin tarjeta" y enlace "Ver
  precios"); los workspaces no tienen interruptor de compartir; no aparecen IA de la organizacion ni
  Facturacion; el mosaico de miembros del Resumen se sustituye por "Equipo: crea una organización".

| Clave     | ES                                                                                                                                      | EN                                                                                                                    |
| --------- | --------------------------------------------------------------------------------------------------------------------------------------- | --------------------------------------------------------------------------------------------------------------------- |
| pcTitle   | Trabaja en equipo con Git Rudder Pro                                                                                                    | Work as a team with Git Rudder Pro                                                                                    |
| pcText    | En tu espacio personal no hay equipo. Crea una organización para invitar a tu equipo, compartir workspaces y usar llaves de IA comunes. | Your personal space has no team. Create an organization to invite your team, share workspaces and use shared AI keys. |
| pcTrial   | 14 días de prueba, sin tarjeta                                                                                                          | 14-day trial, no card needed                                                                                          |
| pcPricing | Ver precios                                                                                                                             | See pricing                                                                                                           |
| pcTile    | Equipo: crea una organización                                                                                                           | Team: create an organization                                                                                          |

### 4. W-14 Vistas por rol

- **Titular.** Todo, mas Facturacion y transferir o eliminar la organizacion.
- **Admin.** Invitar, reenviar y retirar invitaciones, cambiar roles entre admin y miembro, quitar
  miembros (nunca al titular), llaves y politica de IA, consumo, renombrar. Facturacion de solo lectura.
- **Miembro.** Equipo de solo lectura: rol como chip, sin selector, menu solo con "Copiar email", sin boton
  de invitar y con el banner `roTeam` del brief #62. IA de la organizacion: lista enmascarada y su consumo
  de hoy. Workspaces compartidos de otros: "Compartido por {name}" y menu sin editar ni borrar.
- **Contadores separados (B-52).** "4 miembros y 1 invitación pendiente · 5 de 5 asientos usados".
  Etiqueta del rol owner: "Titular" (B-50).
- **Invitar.** Modal con email (misma regla que el registro), rol en segmented (Miembro, Admin) y linea
  informativa. Filas pendientes con opacidad 0,5 y "Invitación pendiente · caduca el {date}" o
  "Caducada".
- **Estados.** Invitar, enviada (toast), ya es miembro, ya hay invitacion (ofrece reenviar), sin asientos
  (W-15), reenviada, reenvio demasiado pronto (429), retirar invitacion (confirmacion), rol cambiado
  (toast, B-59), quitar miembro (confirmacion de peligro), 403 inesperado (C-08).

| Clave         | ES                                                                                                                                                                          | EN                                                                                                                                              |
| ------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------- |
| tmCounts      | {members} miembros y {pending} invitación pendiente · {used} de {total} asientos usados                                                                                     | {members} members and {pending} pending invitation · {used} of {total} seats used                                                               |
| tmSharedBy    | Compartido por {name}                                                                                                                                                       | Shared by {name}                                                                                                                                |
| tmCopyEmail   | Copiar email                                                                                                                                                                | Copy email                                                                                                                                      |
| inviteTitle   | Invitar a {org}                                                                                                                                                             | Invite to {org}                                                                                                                                 |
| inviteInfo    | Ocupa un asiento mientras está pendiente. Caduca a los 7 días.                                                                                                              | It takes a seat while pending. It expires after 7 days.                                                                                         |
| invitePend    | Invitación pendiente · caduca el {date}                                                                                                                                     | Invitation pending · expires {date}                                                                                                             |
| inviteExpired | Caducada                                                                                                                                                                    | Expired                                                                                                                                         |
| inviteMember  | {email} ya es miembro.                                                                                                                                                      | {email} is already a member.                                                                                                                    |
| inviteDup     | Ya hay una invitación pendiente para {email}.                                                                                                                               | There's already a pending invitation for {email}.                                                                                               |
| resend        | Reenviar invitación                                                                                                                                                         | Resend invitation                                                                                                                               |
| resendSoon    | Ya la reenviaste hace poco. Espera unos minutos.                                                                                                                            | You resent it recently. Wait a few minutes.                                                                                                     |
| revokeInvT    | ¿Retirar la invitación a {email}?                                                                                                                                           | Withdraw the invitation to {email}?                                                                                                             |
| revokeInvI    | El enlace deja de funcionar y se libera el asiento.                                                                                                                         | The link stops working and the seat is freed.                                                                                                   |
| revokeInvB    | Retirar                                                                                                                                                                     | Withdraw                                                                                                                                        |
| roleChanged   | Rol de {name}: {role}                                                                                                                                                       | {name}'s role: {role}                                                                                                                           |
| rmMemberT     | ¿Quitar a {name} de {org}?                                                                                                                                                  | Remove {name} from {org}?                                                                                                                       |
| rmMemberI     | Pierde el acceso a los workspaces compartidos y a las llaves de IA en su próxima sincronización. Sus workspaces privados en {org} se eliminan; los que compartió se quedan. | They lose access to shared workspaces and AI keys on their next sync. Their private workspaces in {org} are deleted; the ones they shared stay. |
| rmMemberB     | Quitar                                                                                                                                                                      | Remove                                                                                                                                          |

### 5. W-15 Sin asientos libres al invitar

- **Muestra.** En el modal de invitar, caja de aviso (`--warn` al 10 %) con el motivo y "Invitar"
  deshabilitado. El titular tiene "Añadir asientos", que abre el modal de asientos de W-18 y vuelve al de
  invitar. Un admin ve a quien pedirselo. La tarjeta de asientos de Equipo pone la barra en `--warn` al
  llenarse. Nunca se suben asientos en silencio (B-51).
- **Estados.** Lleno (titular), lleno (admin), 409 `no_seats_available` al enviar (mismo aviso).

| Clave     | ES                                                                                                 | EN                                                                           |
| --------- | -------------------------------------------------------------------------------------------------- | ---------------------------------------------------------------------------- |
| seatsFull | No quedan asientos libres ({used} de {total}). Las invitaciones pendientes también ocupan asiento. | No free seats left ({used} of {total}). Pending invitations take a seat too. |
| addSeats  | Añadir asientos                                                                                    | Add seats                                                                    |
| askOwner  | Pide a {owner} que añada asientos.                                                                 | Ask {owner} to add seats.                                                    |

### 6. W-16 IA de la organizacion

- **Proposito.** Que un admin anada, rote y quite las llaves de IA de la organizacion; la llave nunca se
  vuelve a mostrar (X-05). Decision #54: el admin fija el modelo por funcion de cada llave.
- **Muestra.** Seccion con dos pestanas: "Llaves" y "Consumo" (W-36). Texto de cabecera de X-03. Lista de
  llaves en tarjeta con filas: insignia del proveedor (colores de `AIPROV` en `git-rudder-data.js`), etiqueta,
  mascara en monoespaciada calculada en el servidor, chip "Predeterminada", resumen de modelos por funcion
  en 12 px, "Añadida por {name} · último uso {ago}" y menu: Probar conexión, Editar…, Rotar llave…,
  Hacer predeterminada, Eliminar…. Debajo, tarjeta "Uso permitido" (politica).
- **Anadir y editar** (modal de 580): Proveedor (Anthropic, OpenAI, Google Gemini, Mistral, Groq,
  OpenRouter; "Compatible con OpenAI" solo si el servidor lo permite, con URL base `https`; nunca Ollama,
  con su explicacion), Etiqueta, API key (campo oculto), "Comprobar y cargar modelos" y, al cargar, el
  selector "Modelo predeterminado" y tres selectores por funcion (Mensajes de commit, Pull requests,
  Resumen de commits) cuya primera opcion es "El predeterminado de la llave" (ADR 0025). Todos se rellenan
  con la lista del proveedor y marcan la recomendada ("Recomendado · Rápido" en commits, "Recomendado ·
  Equilibrado" en PRs y resumenes). Despues, la casilla de predeterminada. Los miembros no pueden cambiar
  estos modelos. Al editar no se ve la llave: se rota aparte.
- **Politica** (cada control guarda solo, patron C-06): permitir el proxy, funciones permitidas,
  peticiones al dia por miembro, tope de tokens al mes con "Sin tope".
- **Vista de miembro.** Banner `roOrgKeys` del brief #62, lista sin menus y su consumo de hoy.
- **Estados.** Sin llaves (admin y miembro), comprobando, llave rechazada, sin cuota, sin red, modelos
  cargando, sin recomendacion (lista sin marcar), modelo retirado en una llave existente (chip de aviso),
  probar conexion bien o mal (toast), rotar, eliminar (peligro), politica guardando y guardada. Sin coste
  ni precios en ningun sitio.

| Clave         | ES                                                                                                                                                                                                                                                       | EN                                                                                                                                                                                                                                              |
| ------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| oaiSub        | Llaves de IA que paga tu organización y que el equipo usa sin verlas. Cifradas en los servidores de Git Rudder; Git Rudder las usa para generar en nombre de tu organización. Al usarlas, los cambios preparados pasan por los servidores de Git Rudder. | AI keys your organization pays for and the team uses without seeing them. Encrypted on Git Rudder's servers; Git Rudder uses them to generate on your organization's behalf. When they're used, staged changes go through Git Rudder's servers. |
| oaiTabKeys    | Llaves                                                                                                                                                                                                                                                   | Keys                                                                                                                                                                                                                                            |
| oaiTabUsage   | Consumo                                                                                                                                                                                                                                                  | Usage                                                                                                                                                                                                                                           |
| oaiAdd        | Añadir llave                                                                                                                                                                                                                                             | Add key                                                                                                                                                                                                                                         |
| oaiAddedBy    | Añadida por {name} · último uso {ago}                                                                                                                                                                                                                    | Added by {name} · last used {ago}                                                                                                                                                                                                               |
| oaiModels     | Commits: {commitModel} · PRs: {prModel} · Resúmenes: {summaryModel}                                                                                                                                                                                      | Commits: {commitModel} · PRs: {prModel} · Summaries: {summaryModel}                                                                                                                                                                             |
| oaiDefault    | Predeterminada                                                                                                                                                                                                                                           | Default                                                                                                                                                                                                                                         |
| oaiTest       | Probar conexión                                                                                                                                                                                                                                          | Test connection                                                                                                                                                                                                                                 |
| oaiEdit       | Editar…                                                                                                                                                                                                                                                  | Edit…                                                                                                                                                                                                                                           |
| oaiRotate     | Rotar llave…                                                                                                                                                                                                                                             | Rotate key…                                                                                                                                                                                                                                     |
| oaiMakeDef    | Hacer predeterminada                                                                                                                                                                                                                                     | Make default                                                                                                                                                                                                                                    |
| oaiDelete     | Eliminar…                                                                                                                                                                                                                                                | Delete…                                                                                                                                                                                                                                         |
| oaiOllama     | Ollama corre en tu equipo y el servidor de Git Rudder no puede llegar a él. Úsalo como llave personal en Git Rudder Desktop o VS Code.                                                                                                                   | Ollama runs on your machine and Git Rudder's server can't reach it. Use it as a personal key in Git Rudder Desktop or VS Code.                                                                                                                  |
| oaiNeverShown | La llave no se vuelve a mostrar nunca: solo su máscara.                                                                                                                                                                                                  | The key is never shown again: only its mask.                                                                                                                                                                                                    |
| oaiLoad       | Comprobar y cargar modelos                                                                                                                                                                                                                               | Check and load models                                                                                                                                                                                                                           |
| oaiChecking   | Comprobando la llave con {provider}…                                                                                                                                                                                                                     | Checking the key with {provider}…                                                                                                                                                                                                               |
| oaiBadKey     | {provider} rechazó la llave.                                                                                                                                                                                                                             | {provider} rejected the key.                                                                                                                                                                                                                    |
| oaiQuota      | La cuenta de {provider} no tiene saldo o llegó a su cuota.                                                                                                                                                                                               | The {provider} account is out of credit or hit its quota.                                                                                                                                                                                       |
| oaiNet        | Git Rudder no pudo conectar con {provider}. Inténtalo de nuevo.                                                                                                                                                                                          | Git Rudder couldn't reach {provider}. Try again.                                                                                                                                                                                                |
| oaiDefModel   | Modelo predeterminado                                                                                                                                                                                                                                    | Default model                                                                                                                                                                                                                                   |
| oaiSameAsDef  | El predeterminado de la llave ({model})                                                                                                                                                                                                                  | Key default ({model})                                                                                                                                                                                                                           |
| fCommit       | Mensajes de commit                                                                                                                                                                                                                                       | Commit messages                                                                                                                                                                                                                                 |
| fPR           | Pull requests                                                                                                                                                                                                                                            | Pull requests                                                                                                                                                                                                                                   |
| fSummary      | Resumen de commits                                                                                                                                                                                                                                       | Commit summaries                                                                                                                                                                                                                                |
| recFast       | Recomendado · Rápido                                                                                                                                                                                                                                     | Recommended · Fast                                                                                                                                                                                                                              |
| recBalanced   | Recomendado · Equilibrado                                                                                                                                                                                                                                | Recommended · Balanced                                                                                                                                                                                                                          |
| oaiUseDefault | Usar como predeterminada de la organización                                                                                                                                                                                                              | Use as the organization default                                                                                                                                                                                                                 |
| oaiSave       | Guardar llave                                                                                                                                                                                                                                            | Save key                                                                                                                                                                                                                                        |
| oaiModelGone  | El modelo {model} ya no está disponible. Elige otro.                                                                                                                                                                                                     | The model {model} is no longer available. Choose another.                                                                                                                                                                                       |
| oaiTestOk     | Conexión con {provider} correcta · {latency} ms                                                                                                                                                                                                          | Connected to {provider} · {latency} ms                                                                                                                                                                                                          |
| oaiRotateT    | Rotar «{label}»                                                                                                                                                                                                                                          | Rotate "{label}"                                                                                                                                                                                                                                |
| oaiRotateNew  | Llave nueva                                                                                                                                                                                                                                              | New key                                                                                                                                                                                                                                         |
| oaiRotateI    | La anterior deja de usarse en cuanto guardes. El equipo no tiene que hacer nada.                                                                                                                                                                         | The old one stops being used as soon as you save. The team doesn't need to do anything.                                                                                                                                                         |
| oaiDeleteT    | ¿Eliminar la llave «{label}»?                                                                                                                                                                                                                            | Delete the key "{label}"?                                                                                                                                                                                                                       |
| oaiDeleteI    | Quien la usaba pasa a la predeterminada de la organización, o a su llave personal si no queda otra.                                                                                                                                                      | Whoever used it switches to the organization default, or to their personal key if none is left.                                                                                                                                                 |
| oaiDeleteB    | Eliminar llave                                                                                                                                                                                                                                           | Delete key                                                                                                                                                                                                                                      |
| polTitle      | Uso permitido                                                                                                                                                                                                                                            | Allowed usage                                                                                                                                                                                                                                   |
| polProxy      | Permitir generar con las llaves de la organización                                                                                                                                                                                                       | Allow generating with organization keys                                                                                                                                                                                                         |
| polFeatures   | Funciones permitidas                                                                                                                                                                                                                                     | Allowed features                                                                                                                                                                                                                                |
| polDaily      | Peticiones al día por miembro                                                                                                                                                                                                                            | Requests per member per day                                                                                                                                                                                                                     |
| polBudget     | Tope de tokens al mes                                                                                                                                                                                                                                    | Monthly token cap                                                                                                                                                                                                                               |
| polNoCap      | Sin tope                                                                                                                                                                                                                                                 | No cap                                                                                                                                                                                                                                          |
| oaiMyUsage    | Hoy llevas {used} de {limit} peticiones.                                                                                                                                                                                                                 | You've used {used} of {limit} requests today.                                                                                                                                                                                                   |
| oaiEmptyAdm   | Tu organización aún no tiene llaves de IA. Añade una para que el equipo genere mensajes de commit y PRs sin poner su propia llave.                                                                                                                       | Your organization has no AI keys yet. Add one so the team can generate commit messages and PRs without their own key.                                                                                                                           |
| oaiEmptyMem   | Tu organización aún no tiene llaves de IA. Puedes usar tu llave personal en Git Rudder Desktop o VS Code.                                                                                                                                                | Your organization has no AI keys yet. You can use your personal key in Git Rudder Desktop or VS Code.                                                                                                                                           |

### 7. W-36 Consumo de IA de la organizacion

- **Muestra.** Pestana "Consumo" (solo admins): segmented de periodo, mosaicos (peticiones, tokens de
  entrada, tokens de salida, errores), barra del tope mensual si lo hay y tabla ordenable con columnas
  miembro, funcion, llave, peticiones, tokens de entrada y salida, errores y peticiones de hoy frente a
  su limite ("37 de 200"); el miembro cerca de su limite en `--warn`. Sin dinero.
- **Estados.** Cargando, con datos, sin consumo, error de la seccion.

| Clave       | ES                                  | EN                                 |
| ----------- | ----------------------------------- | ---------------------------------- |
| usThisMonth | Este mes                            | This month                         |
| usLastMonth | Mes pasado                          | Last month                         |
| usLast7     | Últimos 7 días                      | Last 7 days                        |
| usRequests  | Peticiones                          | Requests                           |
| usTokensIn  | Tokens de entrada                   | Input tokens                       |
| usTokensOut | Tokens de salida                    | Output tokens                      |
| usErrors    | Errores                             | Errors                             |
| usToday     | Hoy                                 | Today                              |
| usBudget    | Tokens del mes: {used} de {cap}     | Tokens this month: {used} of {cap} |
| usEmpty     | Aún no hay consumo en este periodo. | No usage in this period yet.       |

### 8. W-17 Precios en la landing y comparativa de planes

- **Muestra.** Seccion "Precios" en la landing, entre VS Code y Descargas, con enlace "Precios" en la
  barra; pagina `/pricing` con lo mismo mas la comparativa y las preguntas. Dos tarjetas (radio 14):
  - Gratis: "0 USD", "Para siempre, con o sin cuenta", lista y botones "Descargar Git Rudder" y "Crear cuenta".
  - Git Rudder Pro: segmented Anual (por defecto, con chip "Ahorra un 20 %") y Mensual; precio grande, linea de
    facturacion, lista, boton "Empezar prueba de 14 días" y linea "Sin tarjeta · mínimo 1 asiento".
  - Nota al pie: moneda, impuestos y Paddle.
- **Comparativa** (Gratis | Git Rudder Pro): Git local completo, repos privados incluidos (Sí, Sí); pull requests
  de las cinco plataformas (Sí, Sí); cuentas de plataforma ilimitadas (Sí, Sí); IA con tu llave, Ollama
  incluido (Sí, Sí); sincronizacion personal cifrada, dispositivos y panel web, con cuenta (Sí, Sí);
  organizacion, invitaciones, roles y asientos (No, Sí); workspaces compartidos (No, Sí); llaves de IA de
  organizacion con proxy, politica, cuotas y consumo (No, Sí); facturacion centralizada (No, Sí).
- **Boton de prueba.** Sin sesion: registro con vuelta a crear organizacion. Con sesion y sin
  organizaciones: modal de W-12. Con organizaciones: elegir una (solo las que no tienen plan) o crear otra.
  Si ya tiene plan: "Ver tu plan".
- **Landing coherente.** La funcion 06 dice que pide cuenta, no pago (decision #51).
- **Estados.** Anual, mensual, sin sesion, con sesion, organizacion con plan.

| Clave         | ES                                                                                                                                                                                                  | EN                                                                                                                                                                                 |
| ------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| prNav         | Precios                                                                                                                                                                                             | Pricing                                                                                                                                                                            |
| prKicker      | PRECIOS                                                                                                                                                                                             | PRICING                                                                                                                                                                            |
| prTitle       | Gratis para ti. Git Rudder Pro para tu equipo.                                                                                                                                                      | Free for you. Git Rudder Pro for your team.                                                                                                                                        |
| prFree        | Gratis                                                                                                                                                                                              | Free                                                                                                                                                                               |
| prFreePrice   | 0 USD                                                                                                                                                                                               | $0                                                                                                                                                                                 |
| prFreeSub     | Para siempre, con o sin cuenta                                                                                                                                                                      | Forever, with or without an account                                                                                                                                                |
| prF1          | Git local completo, repositorios privados incluidos                                                                                                                                                 | Full local Git, private repositories included                                                                                                                                      |
| prF2          | Pull requests de GitHub, GitLab, Bitbucket, Gitea y Azure DevOps                                                                                                                                    | Pull requests on GitHub, GitLab, Bitbucket, Gitea and Azure DevOps                                                                                                                 |
| prF3          | Cuentas de plataforma ilimitadas                                                                                                                                                                    | Unlimited platform accounts                                                                                                                                                        |
| prF4          | IA con tu propia llave, Ollama incluido                                                                                                                                                             | AI with your own key, Ollama included                                                                                                                                              |
| prF5          | Sincronización cifrada de extremo a extremo entre tus equipos (con cuenta)                                                                                                                          | End-to-end encrypted sync across your machines (with an account)                                                                                                                   |
| prPro         | Git Rudder Pro                                                                                                                                                                                      | Git Rudder Pro                                                                                                                                                                     |
| prAnnual      | Anual                                                                                                                                                                                               | Annual                                                                                                                                                                             |
| prMonthly     | Mensual                                                                                                                                                                                             | Monthly                                                                                                                                                                            |
| prSave        | Ahorra un 20 %                                                                                                                                                                                      | Save 20%                                                                                                                                                                           |
| prProAnnual   | 8 USD por asiento al mes                                                                                                                                                                            | $8 per seat per month                                                                                                                                                              |
| prProAnnualS  | 96 USD al año por asiento, facturado anualmente                                                                                                                                                     | $96 per seat per year, billed annually                                                                                                                                             |
| prProMonthly  | 10 USD por asiento al mes                                                                                                                                                                           | $10 per seat per month                                                                                                                                                             |
| prProMonthlyS | Facturado cada mes                                                                                                                                                                                  | Billed monthly                                                                                                                                                                     |
| prP1          | Todo lo de Gratis                                                                                                                                                                                   | Everything in Free                                                                                                                                                                 |
| prP2          | Organización con invitaciones, roles y asientos                                                                                                                                                     | Organization with invitations, roles and seats                                                                                                                                     |
| prP3          | Workspaces compartidos con el equipo                                                                                                                                                                | Workspaces shared with the team                                                                                                                                                    |
| prP4          | Llaves de IA de la organización: el equipo genera sin poner su llave, con cuotas y consumo                                                                                                          | Organization AI keys: the team generates without their own key, with quotas and usage                                                                                              |
| prP5          | Facturación centralizada, con factura e impuestos incluidos                                                                                                                                         | Centralized billing, with invoices and taxes handled                                                                                                                               |
| prTrial       | Empezar prueba de 14 días                                                                                                                                                                           | Start 14-day trial                                                                                                                                                                 |
| prTrialSub    | Sin tarjeta · mínimo 1 asiento                                                                                                                                                                      | No card needed · minimum 1 seat                                                                                                                                                    |
| prDownload    | Descargar Git Rudder                                                                                                                                                                                | Download Git Rudder                                                                                                                                                                |
| prSignup      | Crear cuenta                                                                                                                                                                                        | Sign up                                                                                                                                                                            |
| prSeePlan     | Ver tu plan                                                                                                                                                                                         | See your plan                                                                                                                                                                      |
| prPickOrg     | ¿En qué organización empiezas la prueba?                                                                                                                                                            | Which organization should start the trial?                                                                                                                                         |
| prFootnote    | Precios en dólares estadounidenses. Al pagar verás el importe en tu moneda y los impuestos de tu país (IVA o impuesto de ventas). Paddle gestiona el pago como revendedor autorizado de Git Rudder. | Prices in US dollars. At checkout you'll see the amount in your currency plus your country's taxes (VAT or sales tax). Paddle handles payment as Git Rudder's authorized reseller. |
| prCompare     | Comparar planes                                                                                                                                                                                     | Compare plans                                                                                                                                                                      |
| prYes         | Sí                                                                                                                                                                                                  | Yes                                                                                                                                                                                |
| prNo          | No                                                                                                                                                                                                  | No                                                                                                                                                                                 |
| f6t           | Con una cuenta gratuita, tus workspaces, cuentas vinculadas y preferencias te siguen a cada equipo y a VS Code.                                                                                     | With a free account, your workspaces, linked accounts and preferences follow you to every machine and to VS Code.                                                                  |

Preguntas de `/pricing` (pregunta y respuesta):

| Clave | ES                                                                                                                                                                     | EN                                                                                                                                            |
| ----- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------- |
| faq1Q | ¿Necesito cuenta para usar Git Rudder?                                                                                                                                 | Do I need an account to use Git Rudder?                                                                                                       |
| faq1A | No. Todo Git local funciona sin cuenta. La cuenta sirve para sincronizar tu configuración entre equipos.                                                               | No. All local Git works without an account. An account syncs your settings across machines.                                                   |
| faq2Q | ¿Qué es un asiento?                                                                                                                                                    | What is a seat?                                                                                                                               |
| faq2A | Un miembro de tu organización o una invitación pendiente. Puedes añadir o quitar asientos cuando quieras; el cambio se prorratea.                                      | A member of your organization or a pending invitation. You can add or remove seats any time; changes are prorated.                            |
| faq3Q | ¿Qué pasa al terminar la prueba?                                                                                                                                       | What happens when the trial ends?                                                                                                             |
| faq3A | Si no eliges plan, la organización pasa a solo lectura y sus datos se guardan 30 días.                                                                                 | If you don't choose a plan, the organization becomes read-only and its data is kept for 30 days.                                              |
| faq4Q | ¿Puedo cancelar?                                                                                                                                                       | Can I cancel?                                                                                                                                 |
| faq4A | Sí, cuando quieras desde Facturación. Sigues con Git Rudder Pro hasta el final del periodo pagado. Consulta la política de reembolsos.                                 | Yes, any time from Billing. You keep Git Rudder Pro until the end of the paid period. See the refund policy.                                  |
| faq5Q | ¿Quién ve mi código?                                                                                                                                                   | Who sees my code?                                                                                                                             |
| faq5A | Con tu llave personal, solo tu proveedor de IA. Con una llave de la organización, los cambios preparados pasan por los servidores de Git Rudder para generar el texto. | With your personal key, only your AI provider. With an organization key, staged changes go through Git Rudder's servers to generate the text. |

### 9. W-18 Facturacion: suscripcion, checkout con Paddle y avisos

- **Seccion Facturacion** (contexto de organizacion; el titular gestiona, los demas leen con `roBilling`).
  Tarjeta de plan: "Git Rudder Pro" con chip de estado (En prueba `--info`, Activa `--ok`, Pago pendiente
  `--warn`, Cancelación programada `--warn`, Cancelada `--danger`, Inactiva `--danger`), lineas de periodo,
  asientos (B-52) y
  precio, y botones del titular:
  - En prueba: "Elegir plan" (checkout).
  - Activa: "Cambiar asientos…" y "Gestionar pago y facturas" (portal de Paddle en una pestana nueva, icono
    de enlace externo: metodo de pago, facturas y cancelar).
  - Pago pendiente: "Actualizar pago" (portal).
- **Modal de asientos.** Contador con minimo en los usados (con su motivo), vista previa del prorrateo que
  devuelve el proveedor ("Hoy" y "Desde el {date}"), impuestos aparte y boton con el importe. Reducir
  explica el credito. Si el proveedor pide confirmar el pago (202), boton "Confirmar en Paddle". En prueba,
  sin cobro hasta elegir plan.
- **Checkout** `/billing/checkout?org=...`: solo el titular. A partir de 900 px, dos columnas: resumen a la
  izquierda (organizacion, "Git Rudder Pro", segmented Anual y Mensual, contador de asientos con minimo en los
  usados, "{seats} asientos × 96 USD", total "+ impuestos" y notas) y, a la derecha, el checkout en linea de
  Paddle.js, que se dibuja como un hueco rotulado "Paddle Checkout" con un esqueleto: no se disena el
  formulario de Paddle. En movil, apilado.
- **Avisos** en todas las secciones del contexto de organizacion (banda arriba del contenido): prueba que
  acaba (desde 3 dias antes; el ultimo dia en `--warn`), pago pendiente y suscripcion inactiva.
- **Inactiva.** Que pasa al acabar la prueba sin pagar o tras un pago fallido lo fija E30-01; la
  propuesta de ADR 0026 es solo lectura con los datos guardados 30 dias. Dibujala como estado aparte, facil
  de cambiar, igual que el brief #67 en Desktop; la respuesta 3 de las preguntas de `/pricing` depende de
  ella.
- **Estados.** Prueba (con dias que quedan), activa anual, activa mensual, cancelacion programada, pago
  pendiente, cancelada; vista de admin y de miembro; checkout cargando, Paddle bloqueado, pago hecho y
  activando, activado, activacion lenta, no es el titular; asientos por debajo de lo usado (409); avisos
  para titular y para el resto. Desktop "Gestionar suscripción" abre esta seccion.

| Clave         | ES                                                                                                                             | EN                                                                                                                            |
| ------------- | ------------------------------------------------------------------------------------------------------------------------------ | ----------------------------------------------------------------------------------------------------------------------------- |
| biTitle       | Facturación                                                                                                                    | Billing                                                                                                                       |
| biSub         | El plan de {org}, sus asientos y cómo se paga.                                                                                 | {org}'s plan, its seats and how it's paid.                                                                                    |
| stTrial       | En prueba                                                                                                                      | Trial                                                                                                                         |
| stActive      | Activa                                                                                                                         | Active                                                                                                                        |
| stPastDue     | Pago pendiente                                                                                                                 | Payment due                                                                                                                   |
| stCancelAt    | Se cancela el {date}                                                                                                           | Cancels on {date}                                                                                                             |
| stCanceled    | Cancelada                                                                                                                      | Canceled                                                                                                                      |
| biTrialLine   | Prueba gratis hasta el {date} (quedan {days} días)                                                                             | Free trial until {date} ({days} days left)                                                                                    |
| biAnnualLine  | Pago anual · {seats} asientos · se renueva el {date}                                                                           | Billed annually · {seats} seats · renews on {date}                                                                            |
| biMonthlyLine | Pago mensual · {seats} asientos · se renueva el {date}                                                                         | Billed monthly · {seats} seats · renews on {date}                                                                             |
| biPriceYear   | {total} USD al año + impuestos                                                                                                 | ${total} per year + taxes                                                                                                     |
| biPriceMonth  | {total} USD al mes + impuestos                                                                                                 | ${total} per month + taxes                                                                                                    |
| biChoose      | Elegir plan                                                                                                                    | Choose plan                                                                                                                   |
| biSeats       | Cambiar asientos…                                                                                                              | Change seats…                                                                                                                 |
| biPortal      | Gestionar pago y facturas                                                                                                      | Manage payment and invoices                                                                                                   |
| biUpdatePay   | Actualizar pago                                                                                                                | Update payment                                                                                                                |
| seTitle       | Asientos de {org}                                                                                                              | Seats for {org}                                                                                                               |
| seMin         | No puedes bajar de {used}: hay {members} miembros y {pending} invitaciones pendientes.                                         | You can't go below {used}: there are {members} members and {pending} pending invitations.                                     |
| seToday       | Hoy: {amount} USD, prorrateado hasta el {date}                                                                                 | Today: ${amount}, prorated until {date}                                                                                       |
| seFrom        | Desde el {date}: {total} USD por periodo                                                                                       | From {date}: ${total} per period                                                                                              |
| seTaxes       | Impuestos aparte, según tu país.                                                                                               | Taxes extra, depending on your country.                                                                                       |
| seConfirm     | Confirmar y pagar {amount} USD                                                                                                 | Confirm and pay ${amount}                                                                                                     |
| seReduce      | Reducir a {seats} asientos                                                                                                     | Reduce to {seats} seats                                                                                                       |
| seCredit      | El importe no usado se descuenta de la próxima factura.                                                                        | The unused amount is credited on the next invoice.                                                                            |
| seConfirmPad  | Paddle necesita que confirmes el pago.                                                                                         | Paddle needs you to confirm the payment.                                                                                      |
| seConfirmBtn  | Confirmar en Paddle                                                                                                            | Confirm in Paddle                                                                                                             |
| seTrial       | Durante la prueba no se cobra nada; los asientos cuentan al elegir plan.                                                       | Nothing is charged during the trial; seats count once you choose a plan.                                                      |
| seBelow       | No puedes bajar de {used} asientos: hay {used} en uso.                                                                         | You can't go below {used} seats: {used} are in use.                                                                           |
| coTitle       | Contratar Git Rudder Pro para {org}                                                                                            | Get Git Rudder Pro for {org}                                                                                                  |
| coLine        | {seats} asientos × {price} USD                                                                                                 | {seats} seats × ${price}                                                                                                      |
| coTotal       | Total: {total} USD + impuestos                                                                                                 | Total: ${total} + taxes                                                                                                       |
| coLoading     | Cargando el pago seguro…                                                                                                       | Loading secure checkout…                                                                                                      |
| coBlocked     | No se pudo cargar el formulario de pago. Si usas un bloqueador de anuncios, desactívalo en esta página y vuelve a intentarlo.  | The payment form couldn't load. If you use an ad blocker, turn it off on this page and try again.                             |
| coActivating  | Pago recibido. Activando Git Rudder Pro en {org}…                                                                              | Payment received. Activating Git Rudder Pro for {org}…                                                                        |
| coDone        | Git Rudder Pro está activo en {org}.                                                                                           | Git Rudder Pro is active for {org}.                                                                                           |
| coSlow        | El pago se hizo, pero la activación tarda más de lo normal. La verás en Facturación en unos minutos; no vuelvas a pagar.       | Payment went through, but activation is taking longer than usual. You'll see it in Billing in a few minutes; don't pay again. |
| coNotOwner    | Solo el titular de {org} puede contratar el plan.                                                                              | Only {org}'s owner can buy the plan.                                                                                          |
| bnTrialOwner  | La prueba de Git Rudder Pro de {org} termina en {days} días.                                                                   | {org}'s Git Rudder Pro trial ends in {days} days.                                                                             |
| bnTrialMember | La prueba de Git Rudder Pro de {org} termina en {days} días. Avisa a {owner} si quieres seguir usando las funciones de equipo. | {org}'s Git Rudder Pro trial ends in {days} days. Let {owner} know if you want to keep the team features.                     |
| bnPastOwner   | El último pago de {org} falló. Actualiza el método de pago para no perder las funciones de equipo.                             | {org}'s last payment failed. Update the payment method to keep the team features.                                             |
| bnPastMember  | Hay un problema con el pago de {org}. El titular ya está avisado.                                                              | There's a problem with {org}'s payment. The owner has been notified.                                                          |
| bnEnded       | La suscripción no está activa: {org} está en solo lectura.                                                                     | The subscription isn't active: {org} is read-only.                                                                            |
| stInactive    | Inactiva                                                                                                                       | Inactive                                                                                                                      |

### 10. W-19 Estados vacios del panel

- **Muestra.** Patron C-04 del brief #62 (icono, titulo, texto y una accion) dentro de cada seccion.

| Clave        | ES                                                                                                      | EN                                                                                                  |
| ------------ | ------------------------------------------------------------------------------------------------------- | --------------------------------------------------------------------------------------------------- |
| emWs         | Aún no tienes workspaces. Créalos aquí o en Git Rudder Desktop para agrupar repositorios.               | You have no workspaces yet. Create them here or in Git Rudder Desktop to group repositories.        |
| emWsBtn      | Nuevo workspace                                                                                         | New workspace                                                                                       |
| emRepos      | Tus repositorios aparecen aquí cuando los abres en Git Rudder Desktop o VS Code con la sesión iniciada. | Your repositories show up here when you open them in Git Rudder Desktop or VS Code while logged in. |
| emSearch     | Ningún repositorio coincide con «{query}».                                                              | No repositories match "{query}".                                                                    |
| emSearchBtn  | Limpiar búsqueda                                                                                        | Clear search                                                                                        |
| emDevices    | Aún no hay dispositivos. Inicia sesión en Git Rudder Desktop o en la extensión de VS Code.              | No devices yet. Log in to Git Rudder Desktop or the VS Code extension.                              |
| emGetDesktop | Descargar Git Rudder Desktop                                                                            | Download Git Rudder Desktop                                                                         |
| emGetVs      | Instalar la extensión                                                                                   | Install the extension                                                                               |
| emTeam       | Solo estás tú. Invita a tu equipo para compartir workspaces y llaves de IA.                             | It's just you. Invite your team to share workspaces and AI keys.                                    |
| emAccounts   | No hay cuentas vinculadas. Se añaden desde Git Rudder Desktop o VS Code.                                | No linked accounts. Add them from Git Rudder Desktop or VS Code.                                    |

Workspace sin repositorios: "Sin repositorios" (`workspaces.empty`); su confirmacion de borrado usa su
propio texto (`deleteWsInfo`), no esa clave (B-59).

### 11. W-20 Esqueletos y errores de la API por seccion

- **Muestra.** Cada seccion carga y falla por separado: esqueletos con la forma de mosaicos, filas de
  dispositivos, tarjetas de workspace, filas de repos, grupos de cuentas, filas de miembros, filas de
  llaves y tarjeta de plan (C-01). Error: tarjeta en linea en el sitio de la seccion con "No se pudo
  cargar {section}." y "Reintentar" (C-02). Sin conexion: banner de C-05 arriba del panel. 403 y 404:
  C-08.

| Clave    | ES                           | EN                       |
| -------- | ---------------------------- | ------------------------ |
| secError | No se pudo cargar {section}. | Couldn't load {section}. |

### 12. W-21 Cuenta: perfil, email y dispositivos y sesiones

- **Muestra.** Seccion "Cuenta" con tarjetas en este orden: Perfil, Email, Contraseña y Formas de entrar
  (brief #69), Dispositivos y sesiones, Datos y Zona de peligro (W-24).
  - Perfil (formulario con "Guardar cambios"): foto de 64 px con "Cambiar foto" y "Quitar foto", Nombre,
    Usuario, Zona horaria (selector con busqueda, nombres IANA legibles con todos los guiones bajos
    cambiados, B-49) e Idioma de los correos.
  - Email: direccion, chip "Confirmado" o "Sin confirmar", "Cambiar email…" (W-35, modal con el email
    nuevo y fila "Pendiente" hasta confirmar).
  - Dispositivos y sesiones: **una sola lista** (fin de "Sesiones activas" frente a "Dispositivos"). Fila:
    icono por tipo (escritorio, VS Code, navegador), nombre del equipo o "Firefox en Linux", linea
    "Git Rudder Desktop 2.4.0 · macOS 15.1 · arm64", "Última actividad hace 2 min", chips "Este navegador" o
    "Este dispositivo" y "Actualización disponible" en `--warn`. Clic abre W-22. Boton de cabecera "Cerrar
    las demás sesiones" (confirmacion). Sin ciudad ni pais (decision #53). El Resumen ensena solo los
    dispositivos de Git Rudder con "Ver todos".
  - Datos: "Exportar configuración" con la nota de que no incluye secretos.
- **Estados.** Perfil guardando y guardado, usuario cogido (409), foto subiendo, foto grande (413) o
  tipo no valido (415), cambio de email pendiente, email actual sin confirmar (403), lista cargando,
  cerrar las demas (confirmacion y toast).

| Clave         | ES                                                                                                      | EN                                                                                           |
| ------------- | ------------------------------------------------------------------------------------------------------- | -------------------------------------------------------------------------------------------- |
| acTitle       | Cuenta                                                                                                  | Account                                                                                      |
| acSub         | Tu perfil, cómo entras y dónde tienes la sesión iniciada.                                               | Your profile, how you log in and where you're logged in.                                     |
| prfPhoto      | Cambiar foto                                                                                            | Change photo                                                                                 |
| prfPhotoRm    | Quitar foto                                                                                             | Remove photo                                                                                 |
| prfPhotoHint  | PNG, JPG o WebP de hasta 2 MB.                                                                          | PNG, JPG or WebP up to 2 MB.                                                                 |
| prfTooBig     | La imagen pesa más de 2 MB.                                                                             | The image is larger than 2 MB.                                                               |
| prfBadType    | No se pudo leer la imagen. Usa PNG, JPG o WebP.                                                         | The image couldn't be read. Use PNG, JPG or WebP.                                            |
| prfUsername   | Usuario                                                                                                 | Username                                                                                     |
| prfUserHint   | De 3 a 39 caracteres: minúsculas, cifras y guiones.                                                     | 3 to 39 characters: lowercase letters, digits and hyphens.                                   |
| prfUserTaken  | Ese usuario ya está cogido.                                                                             | That username is taken.                                                                      |
| prfTz         | Zona horaria                                                                                            | Time zone                                                                                    |
| prfMailLang   | Idioma de los correos                                                                                   | Email language                                                                               |
| prfSave       | Guardar cambios                                                                                         | Save changes                                                                                 |
| emCard        | Email                                                                                                   | Email                                                                                        |
| emVerified    | Confirmado                                                                                              | Confirmed                                                                                    |
| emUnverified  | Sin confirmar                                                                                           | Not confirmed                                                                                |
| emChange      | Cambiar email…                                                                                          | Change email…                                                                                |
| emNew         | Email nuevo                                                                                             | New email                                                                                    |
| emPending     | Pendiente: {new}. Te enviamos un enlace para confirmarlo; hasta entonces sigues entrando con {current}. | Pending: {new}. We sent a link to confirm it; until then you keep logging in with {current}. |
| emNeedVerify  | Confirma tu email actual antes de cambiarlo.                                                            | Confirm your current email before changing it.                                               |
| dsTitle       | Dispositivos y sesiones                                                                                 | Devices and sessions                                                                         |
| dsLast        | Última actividad {ago}                                                                                  | Last active {ago}                                                                            |
| dsThisBrowser | Este navegador                                                                                          | This browser                                                                                 |
| dsThisDevice  | Este dispositivo                                                                                        | This device                                                                                  |
| dsUpdate      | Actualización disponible: {version}                                                                     | Update available: {version}                                                                  |
| dsOthersOut   | Cerrar las demás sesiones                                                                               | Log out other sessions                                                                       |
| dsOthersT     | ¿Cerrar todas las demás sesiones?                                                                       | Log out all other sessions?                                                                  |
| dsOthersI     | Se cierra la sesión en todos tus dispositivos y navegadores menos este. Te avisaremos por email.        | Every device and browser except this one is logged out. We'll email you about it.            |
| dsOthersDone  | Cerramos {count} sesiones.                                                                              | {count} sessions were logged out.                                                            |
| dsSeeAll      | Ver todos                                                                                               | See all                                                                                      |
| exNoSecrets   | No incluye secretos: ni tokens ni llaves, tampoco cifrados.                                             | It doesn't include secrets: no tokens or keys, not even encrypted.                           |

### 13. W-22 Detalle de dispositivo

- **Muestra.** Panel lateral de 420 px desde la derecha (pantalla completa en movil): nombre editable
  (1 a 80 caracteres), tipo, sistema y arquitectura, version de la app con chip de actualizacion, "Conectado
  desde {date}", "Última sincronización {ago}" y la marca de este navegador. Accion de peligro "Cerrar
  sesión en este dispositivo" con un texto que no promete borrado remoto (X-03). En la sesion actual, la
  accion es cerrar sesion.
- **Estados.** Renombrar (guardando, guardado), version desactualizada, confirmar cierre, cerrado (toast),
  rotacion recomendada de la boveda (banda del brief #63 con el texto de abajo).

| Clave      | ES                                                                                                                                                                                                                                                                   | EN                                                                                                                                                                                                                                                   |
| ---------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| dvSince    | Conectado desde el {date}                                                                                                                                                                                                                                            | Connected since {date}                                                                                                                                                                                                                               |
| dvLastSync | Última sincronización {ago}                                                                                                                                                                                                                                          | Last synced {ago}                                                                                                                                                                                                                                    |
| dvRename   | Nombre del dispositivo                                                                                                                                                                                                                                               | Device name                                                                                                                                                                                                                                          |
| dvSignOut  | Cerrar sesión en este dispositivo                                                                                                                                                                                                                                    | Log out on this device                                                                                                                                                                                                                               |
| dvOutT     | ¿Cerrar sesión en {device}?                                                                                                                                                                                                                                          | Log out on {device}?                                                                                                                                                                                                                                 |
| dvOutI     | Ese equipo deja de sincronizar. Borrará las cuentas y llaves que recibió de otros equipos la próxima vez que se conecte; si no vuelve a conectarse, conserva lo que ya tenía. Te recomendamos rotar la llave de tu bóveda y los tokens de las plataformas que usaba. | That device stops syncing. It will remove the accounts and keys it got from other devices the next time it connects; if it never connects again, it keeps what it already had. We recommend rotating your vault key and the platform tokens it used. |
| dvOutB     | Cerrar sesión                                                                                                                                                                                                                                                        | Log out                                                                                                                                                                                                                                              |
| dvOutDone  | Sesión cerrada en {device}                                                                                                                                                                                                                                           | Logged out on {device}                                                                                                                                                                                                                               |
| dvRotate   | Rota la llave de tu bóveda desde Git Rudder Desktop o VS Code: {device} la tenía descargada.                                                                                                                                                                         | Rotate your vault key from Git Rudder Desktop or VS Code: {device} had it downloaded.                                                                                                                                                                |

### 14. W-23 Detalle de repositorio

- **Muestra.** Panel lateral al pulsar una fila de la tabla: alias y nombre, URL del remoto en
  monoespaciada con "Copiar", plataforma y cuenta, chips de workspaces (editables con el menu que ya
  existe), "Actualizado {ago}", boton "Abrir en Git Rudder Desktop" (`git-rudder://open?repo=...`) y enlace "¿No
  tienes Git Rudder Desktop? Descárgalo". Repo sin remoto: "Local en {device}" y su nota. Sin lista de
  equipos donde esta clonado (la API no guarda presencia, a proposito) y sin rutas locales (B-06). El
  buscador busca tambien por usuario de la cuenta (B-59).
- **Estados.** Con remoto, local, Desktop no instalado (el enlace no responde: el texto de descarga
  queda visible siempre).

| Clave       | ES                                        | EN                                            |
| ----------- | ----------------------------------------- | --------------------------------------------- |
| rdOpen      | Abrir en Git Rudder Desktop               | Open in Git Rudder Desktop                    |
| rdNoDesktop | ¿No tienes Git Rudder Desktop? Descárgalo | Don't have Git Rudder Desktop? Download it    |
| rdCopyUrl   | Copiar URL                                | Copy URL                                      |
| rdLocal     | Local en {device}                         | Local on {device}                             |
| rdLocalNote | Solo existe en ese equipo; ábrelo allí.   | It only exists on that device; open it there. |

### 15. W-24 Eliminar cuenta

- **Muestra.** Tarjeta "Zona de peligro" al final de Cuenta. Flujo: "Eliminar cuenta…", W-35 fuerte (sin
  GitHub ni GitLab), bloqueo si es titular unica de una organizacion con otros miembros (lista con enlace a
  cada una), modal de peligro con lo que se borra y lo que se queda y el email escrito para confirmar, y
  pagina final `/goodbye`. Si una organizacion en la que solo esta ella tiene suscripcion activa, se trata
  como el bloqueo de W-12 (pendiente de confirmar en el contrato).
- **Estados.** Reautenticando, bloqueada, confirmar (boton deshabilitado hasta que el email coincide),
  eliminando, pagina final.

| Clave      | ES                                                                                                                                                                                                                  | EN                                                                                                                                                                                                 |
| ---------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| dzTitle    | Zona de peligro                                                                                                                                                                                                     | Danger zone                                                                                                                                                                                        |
| dzDelete   | Eliminar cuenta…                                                                                                                                                                                                    | Delete account…                                                                                                                                                                                    |
| dzBlockedT | Aún no puedes eliminar tu cuenta                                                                                                                                                                                    | You can't delete your account yet                                                                                                                                                                  |
| dzBlocked  | Eres titular de organizaciones con otros miembros. Transfiere la titularidad o elimina cada organización antes de borrar tu cuenta.                                                                                 | You own organizations with other members. Transfer ownership or delete each organization before deleting your account.                                                                             |
| dzGoOrg    | Ir a {org}                                                                                                                                                                                                          | Go to {org}                                                                                                                                                                                        |
| dzConfirmT | Eliminar tu cuenta para siempre                                                                                                                                                                                     | Delete your account forever                                                                                                                                                                        |
| dzGone     | Se borran tu perfil, sesiones y dispositivos, la bóveda y los secretos sincronizados, las cuentas vinculadas, las llaves de IA, tus repositorios y workspaces personales, y las organizaciones donde solo estás tú. | Your profile, sessions and devices, the vault and synced secrets, linked accounts, AI keys, your personal repositories and workspaces, and organizations where you're the only member are deleted. |
| dzStays    | Se quedan los workspaces que compartiste en organizaciones, que pasan a sus admins. Tus repositorios en disco no se tocan.                                                                                          | Workspaces you shared in organizations stay and pass to their admins. Repositories on disk aren't touched.                                                                                         |
| dzType     | Escribe tu email para confirmar                                                                                                                                                                                     | Type your email to confirm                                                                                                                                                                         |
| dzDeleteB  | Eliminar mi cuenta                                                                                                                                                                                                  | Delete my account                                                                                                                                                                                  |
| byeTitle   | Tu cuenta se eliminó                                                                                                                                                                                                | Your account was deleted                                                                                                                                                                           |
| byeText    | Git Rudder Desktop y la extensión de VS Code siguen funcionando sin cuenta en tus equipos. Las cuentas y llaves que añadiste en cada equipo siguen allí.                                                            | Git Rudder Desktop and the VS Code extension keep working without an account on your machines. The accounts and keys you added on each machine are still there.                                    |
| byeHome    | Ir al inicio                                                                                                                                                                                                        | Go to home                                                                                                                                                                                         |

### 16. W-25 Indicador de sincronizacion

- **Muestra.** En "Configuración en la nube": linea de estado arriba (componente C-06) con la ultima
  sincronizacion de cada dispositivo; el "Guardar" global desaparece y cada interruptor guarda solo con
  "Guardando…" y "Guardado" (B-57); reservar el hueco de la tarjeta de boveda (K-10, brief #63).
  - Apagar "Cuentas vinculadas" o "Llaves de IA" borra sus datos de la nube: confirmacion de peligro.
  - Encender una categoria de secretos sin boveda: descripcion de X-04 bajo el interruptor.
  - "Llaves de IA" encendido: descripcion de X-03.
  - Las otras cuatro categorias se pausan: descripcion propia al apagarlas.
  - Un cambio llegado de otro dispositivo (412): C-06.
- **Estados.** Todo sincronizado, un dispositivo atrasado (punto `--warn`), guardando, guardado, fallo
  al guardar, confirmar apagado de secretos, sin boveda, pausada.

| Clave       | ES                                                                                                                                                 | EN                                                                                                                                                |
| ----------- | -------------------------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------- |
| syDevices   | Última sincronización: {device} {ago}                                                                                                              | Last sync: {device} {ago}                                                                                                                         |
| syAccOffT   | ¿Dejar de sincronizar las cuentas vinculadas?                                                                                                      | Stop syncing linked accounts?                                                                                                                     |
| syAccOffI   | Se borran de la nube sus datos y sus tokens cifrados ({count} cuentas). Tus equipos conservan su copia local y la cuenta que usa cada repositorio. | Their data and encrypted tokens are deleted from the cloud ({count} accounts). Your machines keep their local copy and each repository's account. |
| syKeysOffT  | ¿Dejar de sincronizar las llaves de IA?                                                                                                            | Stop syncing AI keys?                                                                                                                             |
| syKeysOffI  | Se borran de la nube sus datos y sus llaves cifradas ({count} llaves). Tus equipos conservan su copia local.                                       | Their data and encrypted keys are deleted from the cloud ({count} keys). Your machines keep their local copy.                                     |
| syOffB      | Dejar de sincronizar                                                                                                                               | Stop syncing                                                                                                                                      |
| syNeedVault | Requiere tu frase de sincronización; se configura en Git Rudder Desktop o VS Code.                                                                 | Requires your sync passphrase; set it up in Git Rudder Desktop or VS Code.                                                                        |
| syKeysOn    | Cifradas de extremo a extremo con tu frase de sincronización                                                                                       | End-to-end encrypted with your sync passphrase                                                                                                    |
| syPaused    | En pausa: tus equipos dejan de aplicar estos cambios. Los datos se quedan.                                                                         | Paused: your machines stop applying these changes. The data stays.                                                                                |

### 17. W-26 Exportacion asincrona (P2, opcional)

- **Muestra.** Solo si la exportacion deja de ser instantanea: fila de Datos con "Preparando tu
  exportación…" y, al terminar, "Lista para descargar · caduca en 24 h" con boton.

| Clave       | ES                                    | EN                                  |
| ----------- | ------------------------------------- | ----------------------------------- |
| exPreparing | Preparando tu exportación…            | Preparing your export…              |
| exReady     | Lista para descargar · caduca en 24 h | Ready to download · expires in 24 h |

### 18. W-27 Terminos, privacidad y reembolsos

- **Muestra.** Rutas `/terms`, `/privacy` y `/refunds` (Paddle exige terminos con condiciones de pago y
  politica de reembolso para aprobar el dominio). Barra publica, columna de 720 px, indice lateral fijo a
  partir de 1024 px, cabecera con titulo y "Versión 2026-09 · en vigor desde el {date}", anclas en cada
  titulo y version para imprimir. Sin pagina ni banner de cookies: no hay analitica (solo cookies de
  sesion necesarias). El texto legal lo revisa un abogado: se disena la estructura con estos titulos y
  texto de relleno, y el titular del servicio queda como `{serviceProvider}` (el pais desde el que se factura no
  esta confirmado).
  - Terminos: Quién presta el servicio; Tu cuenta; Uso aceptable; Planes de pago, prueba y renovación;
    Cancelación y reembolsos; Licencia del software; Responsabilidad; Cambios en estos términos; Contacto.
  - Privacidad: Qué datos guardamos; Qué no podemos ver; Direcciones IP; Sin geolocalización; Proveedores
    (alojamiento en AWS us-east-1, correo con Resend, pagos con Paddle, proveedores de IA de las llaves de
    organización); Tus derechos; Cookies; Contacto.
  - Reembolsos: Plazo; Cómo pedirlo; Cambios de asientos y prorrateos.
- **Pie de pagina.** Documentación, Notas de la versión, Estado, Precios, Términos, Privacidad y
  Reembolsos.

| Clave     | ES                                           | EN                                   |
| --------- | -------------------------------------------- | ------------------------------------ |
| lgTerms   | Términos del servicio                        | Terms of service                     |
| lgPrivacy | Política de privacidad                       | Privacy policy                       |
| lgRefunds | Política de reembolsos                       | Refund policy                        |
| lgVersion | Versión {version} · en vigor desde el {date} | Version {version} · effective {date} |
| lgToc     | En esta página                               | On this page                         |
| ftTerms   | Términos                                     | Terms                                |
| ftRefunds | Reembolsos                                   | Refunds                              |
| ftPricing | Precios                                      | Pricing                              |

### 19. W-28 Paginas 404 y 500

- **Muestra.** "404" o "500" grande en monoespaciada, titulo, texto y botones. Dos variantes: publica
  (barra de 64 px) y dentro del panel (conserva barra superior y lateral).
- **Estados.** 404 publica, 404 en el panel, 500, sin conexion (C-05).

| Clave      | ES                                                                                                  | EN                                                                                         |
| ---------- | --------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------ |
| nfTitle    | Página no encontrada                                                                                | Page not found                                                                             |
| nfText     | Esta dirección no existe en Git Rudder.                                                             | This address doesn't exist on Git Rudder.                                                  |
| nfHome     | Ir al inicio                                                                                        | Go to home                                                                                 |
| nfDash     | Abrir panel                                                                                         | Open dashboard                                                                             |
| e500Title  | Algo falló en Git Rudder                                                                            | Something went wrong at Git Rudder                                                         |
| e500Text   | No es culpa tuya. Inténtalo de nuevo en un momento; si sigue fallando, mira el estado del servicio. | It's not your fault. Try again in a moment; if it keeps failing, check the service status. |
| e500Retry  | Reintentar                                                                                          | Retry                                                                                      |
| e500Status | Estado del servicio                                                                                 | Service status                                                                             |

### 20. W-29 Documentacion, notas de la version y estado

- **Notas de la version.** `/releases`: lista de la API con version, fecha, chip "Beta" en el canal beta,
  notas en Markdown saneado y enlace a descargas; `/releases/{version}`: detalle. Estados: cargando,
  lista, vacia (antes de la primera version publica), error.
- **Documentacion.** `/docs`: indice con tarjetas de seccion (Primeros pasos, Cuentas de plataforma,
  Bóveda y frase de sincronización, IA, Equipos y facturación, Solución de problemas) y plantilla de
  articulo (navegacion lateral de 240 px, contenido de 720 px, indice de la pagina a la derecha). El
  contenido no es parte de este brief.
- **Estado.** `/status` ya existe en codigo; no cambia. Solo se enlaza desde el pie y desde W-28.

| Clave      | ES                               | EN                         |
| ---------- | -------------------------------- | -------------------------- |
| rnTitle    | Notas de la versión              | Release notes              |
| rnBeta     | Beta                             | Beta                       |
| rnEmpty    | Aún no hay versiones publicadas. | No versions published yet. |
| rnDownload | Descargar esta versión           | Download this version      |
| dcIndex    | Documentación                    | Documentation              |
| dcStart    | Primeros pasos                   | Getting started            |
| dcAccounts | Cuentas de plataforma            | Platform accounts          |
| dcVault    | Bóveda y frase de sincronización | Vault and sync passphrase  |
| dcAi       | IA                               | AI                         |
| dcTeams    | Equipos y facturación            | Teams and billing          |
| dcTrouble  | Solución de problemas            | Troubleshooting            |

### 21. W-30 Diseno responsive

- **Cortes.** 1024 px o mas, escritorio; de 768 a 1023, tableta; menos de 768, movil (referencia 375).
- **Barra publica** por debajo de 900 px: boton de menu que abre una hoja con los enlaces, ES/EN, el
  conmutador de tema de tres estados (C-09) y "Iniciar sesión" y "Crear cuenta" (o "Abrir panel").
- **Panel** por debajo de 900 px: la barra lateral pasa a un cajon que abre un boton de menu en la barra
  superior; el selector de organizacion se queda; "Descargar" entra en el cajon. Tablas (repos, miembros,
  consumo) en tarjetas apiladas por debajo de 768. Modales a todo el ancho con 16 px de margen; paneles
  laterales a pantalla completa.
- **Visitantes desde movil (B-55).** Android e iOS: el boton del hero pasa a "Ver todas las plataformas",
  las descargas ensenan todas las tarjetas sin destacar ninguna y el aviso `download.mobile` que ya existe,
  con "Copiar enlace". Sistema desconocido: todas sin destacar.

| Clave      | ES            | EN         |
| ---------- | ------------- | ---------- |
| rsMenu     | Menú          | Menu       |
| rsClose    | Cerrar menú   | Close menu |
| rsCopyLink | Copiar enlace | Copy link  |

### 22. X-01 Fuera las cuentas de plataforma compartidas con el equipo

- **Cambio.** Se quitan la opcion "Compartir con el equipo" del menu, la casilla del modal "Conectar" (el
  modal entero desaparece por X-02) y los chips "Tuya" y "Equipo". Cada miembro vincula sus propias cuentas.
  Compartir workspaces se mantiene: son metadatos, no secretos.

### 23. X-02 La web solo ve y quita cuentas vinculadas

- **Muestra.** Subtitulo nuevo; sin boton "+ Conectar" en la cabecera ni en los grupos; arriba, tarjeta de
  llamada con "Abrir en Git Rudder Desktop" (`git-rudder://settings/platforms`) y "Descargar Git Rudder". Grupos por
  plataforma con insignia, nombre y numero; los vacios no se ensenan. Filas solo con metadatos: iniciales,
  "@usuario · etiqueta", linea en monoespaciada "servidor · método", chips "Predeterminada", estado que
  informa el dispositivo ("Comprobada hace 2 h", "Token rechazado" `--danger`, "Caduca en 5 días" `--warn`,
  "Caducada" `--danger`), "Sin token en la nube" (con tooltip), "Usada en {count} repos" y permisos en el
  tooltip. Menu: solo "Quitar…", con un texto que dice la verdad y enlaza a la pagina de tokens de la
  plataforma. Sin "Probar conexión": la web no puede descifrar el token.
- **Estados.** Con cuentas, vacio (W-19), sin token en la nube, caducada, quitar (confirmacion de
  peligro), quitada (toast).

| Metodo (`method`) | ES             | EN             |
| ----------------- | -------------- | -------------- |
| `oauth`           | OAuth          | OAuth          |
| `pat`             | Token personal | Personal token |
| `api_token`       | API token      | API token      |
| `ssh_token`       | SSH y token    | SSH and token  |

| Clave         | ES                                                                                                                                        | EN                                                                                                                        |
| ------------- | ----------------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------- |
| laSub         | Cuentas de plataformas Git que usas en Git Rudder. Los tokens están cifrados de extremo a extremo: el panel no puede verlos ni probarlos. | Git platform accounts you use in Git Rudder. Tokens are end-to-end encrypted: the dashboard can't see or test them.       |
| laCallT       | Añade cuentas desde Git Rudder Desktop o VS Code                                                                                          | Add accounts from Git Rudder Desktop or VS Code                                                                           |
| laCallText    | El token se queda en tus equipos y viaja cifrado con tu frase de sincronización.                                                          | The token stays on your machines and travels encrypted with your sync passphrase.                                         |
| laOpenDesk    | Abrir en Git Rudder Desktop                                                                                                               | Open in Git Rudder Desktop                                                                                                |
| laDownload    | Descargar Git Rudder                                                                                                                      | Download Git Rudder                                                                                                       |
| laChecked     | Comprobada {ago}                                                                                                                          | Checked {ago}                                                                                                             |
| laRejected    | Token rechazado                                                                                                                           | Token rejected                                                                                                            |
| laExpiresIn   | Caduca en {days} días                                                                                                                     | Expires in {days} days                                                                                                    |
| laExpired     | Caducada                                                                                                                                  | Expired                                                                                                                   |
| laNoSecret    | Sin token en la nube                                                                                                                      | No token in the cloud                                                                                                     |
| laNoSecretTip | Se sincroniza sin token. Actualiza las credenciales en un equipo para usarla en los demás.                                                | It syncs without a token. Update the credentials on one machine to use it on the others.                                  |
| laRemoveT     | ¿Quitar @{user} ({host}) de Git Rudder?                                                                                                   | Remove @{user} ({host}) from Git Rudder?                                                                                  |
| laRemoveI     | Se quita de todos tus dispositivos cuando sincronicen. No revoca el token en {platform}: hazlo tú en su página de tokens.                 | It's removed from all your devices when they sync. It doesn't revoke the token on {platform}: do that on its tokens page. |
| laTokensPage  | Abrir la página de tokens de {platform}                                                                                                   | Open {platform}'s tokens page                                                                                             |
| laRemoveB     | Quitar                                                                                                                                    | Remove                                                                                                                    |
| laRemoved     | Cuenta @{user} quitada                                                                                                                    | Account @{user} removed                                                                                                   |

Pagina de tokens por plataforma (el host sale de la cuenta): GitHub `https://{host}/settings/applications`
(OAuth) o `https://{host}/settings/personal-access-tokens` (token de grano fino); GitLab `https://{host}/-/user_settings/personal_access_tokens`;
Bitbucket `https://id.atlassian.com/manage-profile/security/api-tokens`; Azure DevOps
`https://dev.azure.com/{org}/_usersSettings/tokens`; Gitea y Forgejo
`https://{host}/user/settings/applications`.

### 24. X-09 Descargas

- **Muestra.** Las cuatro tarjetas `platforms` con version, tamanos y sha256 de la API (nada escrito a
  mano). Cabecera "v{version} · {date}". Tarjeta del sistema detectado con "Tu sistema"; si el navegador
  da la arquitectura y es ARM64 en Windows, el principal pasa a ARM64.

| Tarjeta | Requisito                                                         | Principal              | Alternativas                                                                                                                                                                  | Comando                                          |
| ------- | ----------------------------------------------------------------- | ---------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------ |
| macOS   | macOS 12 o posterior                                              | Apple Silicon (.dmg)   | Intel (.dmg) · Universal (.dmg)                                                                                                                                               | Homebrew del tap propio (por confirmar)          |
| Windows | Windows 10 y 11                                                   | x64 (.exe)             | ARM64 (.exe) con chip "Beta" · x64 (.msi), sin actualizaciones automáticas (con tooltip)                                                                                      | `winget install Git Rudder.Git Rudder`           |
| Linux   | Ubuntu 22.04+, Debian 12+. Los .deb y .rpm no se actualizan solos | Debian / Ubuntu (.deb) | Fedora / RHEL (.rpm) · AppImage · subtitulo "ARM64 · Beta" con .deb, .rpm y AppImage                                                                                          | sin caja de comando                              |
| VS Code | VS Code 1.138 o posterior. No disponible en vscode.dev            | Instalar extensión     | Open VSX (VSCodium, Cursor) · desplegable "Descargar .vsix por plataforma" con los 8: Windows x64 y ARM64, macOS Intel y Apple Silicon, Linux x64 y ARM64, Alpine x64 y ARM64 | `code --install-extension git-rudder.git-rudder` |

- **Fuera.** Windows Portable (.zip) y macOS Universal (.zip). Flathub no sale en el lanzamiento (ADR
  0016): no se dibuja.
- **Estados.** Valores del prerender, refresco en el cliente (sin parpadeo), API caida (se ensenan los del
  prerender y se ocultan los tamanos), sin version publica todavia, descarga iniciada (toast
  `download.started`, nunca "simulado").

| Clave      | ES                                                                                                                        | EN                                                                                                            |
| ---------- | ------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------- |
| dlBeta     | Beta                                                                                                                      | Beta                                                                                                          |
| dlArmBeta  | ARM64 · Beta                                                                                                              | ARM64 · Beta                                                                                                  |
| dlMsiTip   | Para equipos de IT que despliegan Git Rudder con sus herramientas. Se instala para todo el equipo y no se actualiza sola. | For IT teams that deploy Git Rudder with their own tools. It installs machine-wide and doesn't update itself. |
| dlAppImage | AppImage (se actualiza sola)                                                                                              | AppImage (updates itself)                                                                                     |
| dlVsixAll  | Descargar .vsix por plataforma                                                                                            | Download .vsix by platform                                                                                    |
| dlNone     | La primera versión pública llega pronto.                                                                                  | The first public version is coming soon.                                                                      |

Etiquetas de artefacto y requisitos: claves existentes de `download.artifact.*`, `download.requirement.*` y
`vscode.requirement` en `packages/i18n` (web).

### 25. X-12 La landing no embebe el prototipo

- **Cambio.** El marco de vista previa del hero y el de la seccion VS Code muestran imagenes estaticas,
  no `<iframe>` de los prototipos: `<picture>` con variante clara y oscura, ancho y alto fijos (sin salto
  de maquetacion) y carga diferida. Mas adelante podra ser una demo de solo lectura con `@git-rudder/repo-view`,
  fuera de este brief.
- **Imagenes.** Capturas a 2x desde los prototipos: el grafo de Desktop en un estado limpio (sin
  conflicto ni datos de demo rotos) y el grafo de VS Code, cada una en claro y oscuro.

## Reglas

- Equipos, invitaciones, asientos, facturacion y llaves de IA de organizacion son de la primera version
  publica (#49): W-11 a W-18 se disenan completos, no como "futuro".
- Precios cerrados: 8 USD por asiento al mes en anual (96 USD al ano) o 10 USD en mensual, prueba de 14
  dias sin tarjeta, minimo 1 asiento, sin plan gratis para organizaciones pequenas. No se ensenan
  descuento de lanzamiento, Pro gratis para open source ni precios regionales: no estan decididos.
- Paddle es el comerciante registrado; la pagina de checkout es la unica que carga Paddle.js y la unica con
  la CSP abierta a Paddle. La API nunca ve una tarjeta. No se escribe el pais desde el que se factura.
- Nunca se suben asientos sin confirmar y sin precio a la vista (B-51). Contadores separados (B-52).
  Etiqueta "Titular" (B-50).
- Sin cuentas de plataforma compartidas (X-01) y sin conectar ni probar desde la web (X-02).
- Llaves de organizacion: solo escritura, mascara del servidor, texto de X-03, sin Ollama, modelo por
  funcion elegido de la lista del proveedor con el nivel recomendado marcado, sin coste por generacion.
- Sin ciudad ni pais en dispositivos y sesiones (#53). Cerrar sesion en otro dispositivo no promete borrar
  nada a distancia (X-03).
- Cada control guarda solo; el unico formulario con boton es el perfil (B-57). Exito visible de cada accion
  (B-59).
- Banners y avisos informativos con `--info` (B-61); toda accion destructiva confirma con boton de peligro
  (B-29).
- Sin rutas locales de otros equipos (B-06). Sin "(simulado)" en ningun texto (B-46).
- Dominio: `git-rudder.com` y `api.git-rudder.com`.
- La landing no carga `support.js`, React desde unpkg ni ningun prototipo (X-12).
- Las claves con contador (`{count}`, `{days}`) llevan variantes `_one` y `_other` (ADR 0017); las
  tablas escriben la forma plural.
- Confirmaciones de peligro con el patron del brief #71 (seccion 10): titulo en pregunta, consecuencia en
  una caja, boton `--danger` y foco inicial en "Cancelar".
- Correos de muestra con `example.com`.

## Entregable

Un zip `git-rudder-handoff-70-panel.zip` con la estructura de `docs/design/handoff/`:

- `Git Rudder Web.dc.html` corregido: landing sin iframes, seccion Precios, descargas de X-09, barra publica
  responsive y pie nuevo. Props `theme`, `lang`, `startPage` y `os` (`mac`, `win`, `linux`, `mobile`,
  `unknown`).
- `Git Rudder Web Panel.dc.html` nuevo: el panel completo, con props `theme`, `lang`, `screen` (seccion con el
  id de `gaps.md` que toque: `W-11` a `W-16`, `W-18`, `W-19`, `W-21` a `W-25`, `W-36`, `X-02`),
  `context` (`personal`, `org`), `role` (`owner`, `admin`, `member`), `plan` (`none`, `trialing`,
  `active`, `past_due`, `cancel_scheduled`, `canceled`, `inactive`) y `state` (`normal`, `empty`,
  `loading`, `error`).
- `Git Rudder Web Paginas.dc.html` nuevo: `/pricing` (W-17), `/billing/checkout` (W-18), `/invitations`
  (W-11), `/terms`, `/privacy` y `/refunds` (W-27), 404 y 500 (W-28), `/docs` y `/releases` (W-29) y
  `/goodbye` (W-24), con props `theme`, `lang`, `screen` y `state`.
- `img/landing-desktop-dark.png`, `img/landing-desktop-light.png`, `img/landing-vscode-dark.png` e
  `img/landing-vscode-light.png`, a 2x (X-12).
- `git-rudder-data.js` (o el objeto `W`) con las claves nuevas en es y en, con espacios de nombres
  (`dashboard.*`, `team.*`, `orgAi.*`, `billing.*`, `pricing.*`, `account.*`, `legal.*`, `download.*`).
  `support.js` sin cambios.
- `README.md` solo con lo nuevo: navegacion del panel, pantallas y estados con sus ids, precios y
  descargas.

## Hecho cuando

- [ ] W-19 a W-30 diseñados, y W-11 a W-18 listos para cuando entren equipos y facturación
- [ ] Cuentas vinculadas solo se ven y se quitan, sin "Compartir con el equipo"
- [ ] Descargas con Linux 22.04+, sin Portable y con VSIX por plataforma
- [ ] La landing no embebe el prototipo
