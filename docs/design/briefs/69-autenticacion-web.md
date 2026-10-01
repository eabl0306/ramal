# Brief #69: vistas de autenticacion web (W-01 a W-09, W-33 a W-35, B-54)

Issue: [eabl0306/ramal#69](https://github.com/eabl0306/ramal/issues/69). Fuente: `docs/design/gaps.md`
(seccion 1.5, B-53, B-54, B-58), `docs/api-surface.md` (Autenticacion web, Inicio de sesion de las apps
nativas), ADR 0008 y las decisiones #53 y #57 del 2026-10-01.

## Contexto

- Producto: Ramal Web (`handoff/Ramal Web.dc.html`), en `https://ramal.bahoque.com`. Hoy solo existen
  login y registro; "¿La olvidaste?" es un toast y OAuth entra directo sin terminos.
- Paginas publicas: barra de navegacion de 64 px y la **tarjeta de autenticacion** (`au`: 420 px, radio
  18, botones OAuth de 42 px con insignia de plataforma, separador "o con email", campos de 42 px, caja de
  error `au.error` en `--danger` al 10 %, boton principal de 44 px y la linea "¿No tienes cuenta?").
  Todas las paginas nuevas de este brief reutilizan esa tarjeta.
- Panel: barra superior de 56 px, barra lateral de 232 px, **tarjetas con filas** (`cards` y `rw`, las de
  "Configuracion en la nube"), **modal generico** (460 px, radio 16, boton principal acento o peligro) y
  toasts. W-33 y W-34 viven en la seccion "Cuenta" del panel (la misma que define el brief #70 para perfil
  y dispositivos).
- Desktop y VS Code nunca tienen formulario de contrasena (ADR 0008): abren estas paginas. Validacion en
  linea, sin conexion y sesion caducada siguen los patrones C-03, C-05 y C-07 del brief #62.

## Pantallas y estados

Todo en claro y oscuro, a 1280 px y a 375 px (la tarjeta ocupa el ancho con 16 px de margen). Rutas
nuevas: `/forgot-password`, `/reset-password`, `/verify-email`, `/email-change/confirm`,
`/email-change/revert`, `/signup/complete`, `/authorize/app`, `/activate`, `/connect/error`,
`/reauthenticate`. Los tokens de los enlaces van en el fragmento (`#token=...`) y nunca se ensenan.

### 1. W-01 Olvide mi contrasena

- **Proposito.** Pedir el enlace de restablecimiento. Misma respuesta exista o no la cuenta.
- **Muestra.** Tarjeta con titulo, texto, campo Email (prellenado si se escribio en el login), boton y
  enlace de vuelta. Sin botones OAuth.
- **Estados.** Formulario, enviando, enviado (siempre igual), reenviar bloqueado 60 s con cuenta atras,
  email no valido (C-03), demasiadas peticiones (429), sin conexion (C-05).

| Clave        | ES                                                                                                              | EN                                                                                                                    |
| ------------ | --------------------------------------------------------------------------------------------------------------- | --------------------------------------------------------------------------------------------------------------------- |
| fpTitle      | Recupera tu contraseña                                                                                          | Reset your password                                                                                                   |
| fpSub        | Escribe el email de tu cuenta y te enviaremos un enlace para elegir una nueva.                                  | Enter your account email and we'll send you a link to choose a new one.                                               |
| fpSubmit     | Enviar enlace                                                                                                   | Send link                                                                                                             |
| fpBusy       | Enviando…                                                                                                       | Sending…                                                                                                              |
| fpBack       | Volver a iniciar sesión                                                                                         | Back to log in                                                                                                        |
| fpSentTitle  | Revisa tu correo                                                                                                | Check your email                                                                                                      |
| fpSent       | Si hay una cuenta con {email}, te llegará un enlace en unos minutos. Caduca en 30 minutos y solo sirve una vez. | If there's an account for {email}, a link will arrive in a few minutes. It expires in 30 minutes and works only once. |
| fpSpam       | ¿No llega? Mira en la carpeta de spam o pide otro enlace.                                                       | Not there? Check your spam folder or ask for another link.                                                            |
| fpResend     | Enviar otro enlace                                                                                              | Send another link                                                                                                     |
| fpResendWait | Enviar otro enlace ({time})                                                                                     | Send another link ({time})                                                                                            |
| fpTooMany    | Has pedido varios enlaces seguidos. Espera unos minutos.                                                        | You've asked for several links in a row. Wait a few minutes.                                                          |

### 2. W-02 Restablecer contrasena

- **Proposito.** Poner la contrasena nueva desde el enlace del correo.
- **Muestra.** Tarjeta con "Contraseña nueva" (pista de 8 caracteres y medidor visual Débil, Aceptable,
  Fuerte), "Repite la contraseña" y una **caja informativa** (`--info` al 10 %, icono) que dice que no
  cambia la frase de sincronizacion y que se cierran todas las sesiones.
- **Estados.** Formulario, no coinciden (en linea), no cumple (422, en linea), guardando, guardada (con
  boton "Iniciar sesión": restablecer no inicia sesion), enlace no valido (falta, caduco, usado o
  sustituido: mismo texto para todo).

| Clave       | ES                                                                                                                                                          | EN                                                                                                                                                         |
| ----------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------- |
| rpTitle     | Elige una contraseña nueva                                                                                                                                  | Choose a new password                                                                                                                                      |
| rpNew       | Contraseña nueva                                                                                                                                            | New password                                                                                                                                               |
| rpRepeat    | Repite la contraseña                                                                                                                                        | Repeat the password                                                                                                                                        |
| rpMismatch  | Las contraseñas no coinciden.                                                                                                                               | The passwords don't match.                                                                                                                                 |
| rpRules     | La contraseña debe tener entre 8 y 128 caracteres.                                                                                                          | The password must be 8 to 128 characters long.                                                                                                             |
| pwWeak      | Débil                                                                                                                                                       | Weak                                                                                                                                                       |
| pwFair      | Aceptable                                                                                                                                                   | Fair                                                                                                                                                       |
| pwStrong    | Fuerte                                                                                                                                                      | Strong                                                                                                                                                     |
| rpVault     | Esto cambia solo la contraseña para entrar en Ramal. No cambia tu frase de sincronización: tus cuentas y llaves cifradas siguen protegidas con ella.        | This only changes the password you use to sign in to Ramal. It doesn't change your sync passphrase: your encrypted accounts and keys stay protected by it. |
| rpSessions  | Al guardarla cerramos la sesión en todos tus dispositivos. Vuelve a entrar en cada uno; con tu frase de sincronización recuperas allí tus cuentas y llaves. | Saving it signs you out on every device. Sign in again on each one; your sync passphrase brings your accounts and keys back there.                         |
| rpSubmit    | Guardar contraseña                                                                                                                                          | Save password                                                                                                                                              |
| rpBusy      | Guardando…                                                                                                                                                  | Saving…                                                                                                                                                    |
| rpDoneTitle | Contraseña cambiada                                                                                                                                         | Password changed                                                                                                                                           |
| rpDone      | Ya puedes iniciar sesión con tu contraseña nueva. Cerramos la sesión en todos tus dispositivos.                                                             | You can now log in with your new password. We signed you out on every device.                                                                              |
| linkDeadT   | Este enlace ya no sirve                                                                                                                                     | This link no longer works                                                                                                                                  |
| rpDead      | Puede haber caducado (duran 30 minutos), haberse usado ya o haber sido sustituido por uno más reciente.                                                     | It may have expired (links last 30 minutes), already been used, or been replaced by a newer one.                                                           |
| rpAskAgain  | Pedir otro enlace                                                                                                                                           | Ask for another link                                                                                                                                       |

### 3. W-03 Verificacion de correo (y enlaces de cambio de correo)

- **Proposito.** Probar que el email es del usuario. No hace falta para entrar; hasta verificar no se
  pueden crear organizaciones ni invitar.
- **Muestra.**
  - Tras el registro: el usuario entra al panel, toast `ok` y **banner** fijo arriba del contenido
    (`--info` al 10 %) con "Reenviar enlace" y "Ocultar" (vuelve en la siguiente sesion). "Crear
    organización" e "Invitar miembro" deshabilitados con tooltip.
  - `/verify-email`: tarjeta que verifica sola al cargar.
  - `/email-change/confirm` y `/email-change/revert`: misma tarjeta. Deshacer pide confirmar antes,
    porque cierra todas las sesiones.
- **Estados.** Verificando, verificado (con o sin sesion), enlace no valido, reenviado, demasiados
  reenvios (429), ya verificado (409: el banner desaparece); cambio confirmado, direccion ya usada (409),
  deshacer (confirmar, hecho).

| Clave       | ES                                                                                                              | EN                                                                                                           |
| ----------- | --------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------ |
| vToast      | Cuenta creada. Te enviamos un enlace para confirmar tu email.                                                   | Account created. We sent you a link to confirm your email.                                                   |
| vBanner     | Confirma tu email: te enviamos un enlace a {email}. Hasta entonces no puedes crear organizaciones ni invitar.   | Confirm your email: we sent a link to {email}. Until then you can't create organizations or invite people.   |
| vResend     | Reenviar enlace                                                                                                 | Resend link                                                                                                  |
| vHide       | Ocultar                                                                                                         | Hide                                                                                                         |
| vNeedTip    | Confirma tu email para hacer esto.                                                                              | Confirm your email to do this.                                                                               |
| vChecking   | Confirmando tu email…                                                                                           | Confirming your email…                                                                                       |
| vDoneTitle  | Email confirmado                                                                                                | Email confirmed                                                                                              |
| vDone       | Ya puedes crear organizaciones e invitar a tu equipo.                                                           | You can now create organizations and invite your team.                                                       |
| goDash      | Ir al panel                                                                                                     | Go to dashboard                                                                                              |
| vDead       | Caduca a las 48 horas y solo sirve una vez.                                                                     | It expires after 48 hours and works only once.                                                               |
| vDeadNoSess | Inicia sesión para pedir otro enlace.                                                                           | Log in to ask for another link.                                                                              |
| vResent     | Te enviamos otro enlace a {email}.                                                                              | We sent another link to {email}.                                                                             |
| vTooMany    | Ya enviamos varios enlaces. Espera un rato antes de pedir otro.                                                 | We've already sent several links. Wait a while before asking for another.                                    |
| vAlready    | Tu email ya estaba confirmado.                                                                                  | Your email was already confirmed.                                                                            |
| ecDoneTitle | Email cambiado                                                                                                  | Email changed                                                                                                |
| ecDone      | A partir de ahora inicia sesión y recibe los avisos con tu email nuevo.                                         | From now on, log in and get notices with your new email.                                                     |
| ecTaken     | Esa dirección ya tiene una cuenta de Ramal. El cambio no se aplicó.                                             | That address already has a Ramal account. The change wasn't applied.                                         |
| erTitle     | ¿Deshacer el cambio de email?                                                                                   | Undo the email change?                                                                                       |
| erText      | Tu cuenta volverá a usar este email y cerraremos todas las sesiones. Hazlo si tú no pediste el cambio.          | Your account will go back to this email and we'll end every session. Do it if you didn't ask for the change. |
| erSubmit    | Deshacer el cambio                                                                                              | Undo the change                                                                                              |
| erDone      | Hecho. Tu cuenta vuelve a usar este email y cerramos todas las sesiones. Te recomendamos cambiar la contraseña. | Done. Your account uses this email again and every session was ended. We recommend changing your password.   |

### 4. W-04 Estados de OAuth y completar el alta (B-54)

- **Proposito.** OAuth de identidad (GitHub, GitLab.com, Google) con todos sus finales, y el paso de alta
  que el prototipo se saltaba.
- **Muestra.**
  - Boton OAuth pulsado: indicador giratorio y "Conectando con {provider}…"; los demas controles
    deshabilitados hasta que el navegador sale.
  - Vuelta con error a `/login?error=...`: caja `au.error` arriba del formulario. `oauth_denied` usa la
    caja informativa (lo decidio el usuario, no es un fallo).
  - `/signup/complete`: tarjeta "Completa tu cuenta" con fila de solo lectura (insignia del proveedor y
    email, que no se puede cambiar), Nombre prellenado, Organización (opcional) con pista de prueba,
    casilla de terminos obligatoria (enlaces a `/terms` y `/privacy`, brief #70), "Crear cuenta" y
    "Cancelar". Es el paso de terminos de B-54.
  - El registro con email tambien guarda la organizacion (B-54): misma pista bajo el campo.
- **Estados.** Conectando, cancelado, fallo, cuenta existente sin verificar, email del proveedor sin
  verificar, alta pendiente, terminos sin aceptar (en linea), creando, alta caducada (410).

| Clave        | ES                                                                                                                                                                                           | EN                                                                                                                                                                                      |
| ------------ | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| oaConnecting | Conectando con {provider}…                                                                                                                                                                   | Connecting to {provider}…                                                                                                                                                               |
| oaDenied     | Cancelaste el acceso en el proveedor. Puedes intentarlo de nuevo o entrar con tu email.                                                                                                      | You cancelled access at the provider. You can try again or log in with your email.                                                                                                      |
| oaFailed     | No se pudo completar el inicio de sesión con el proveedor. Inténtalo de nuevo.                                                                                                               | Signing in with the provider couldn't be completed. Try again.                                                                                                                          |
| oaExists     | Ya hay una cuenta de Ramal con ese email, pero aún no está confirmado. Entra con tu contraseña o usa «¿La olvidaste?» para recuperarla; después podrás vincular este proveedor desde Cuenta. | There's already a Ramal account with that email, but it isn't confirmed yet. Log in with your password or use "Forgot it?" to recover it; then you can link this provider from Account. |
| oaUnverified | Tu email no está verificado en el proveedor. Verifícalo allí o crea la cuenta con tu email.                                                                                                  | Your email isn't verified at the provider. Verify it there or sign up with your email.                                                                                                  |
| scTitle      | Completa tu cuenta                                                                                                                                                                           | Finish creating your account                                                                                                                                                            |
| scWith       | Con {provider} como {email}                                                                                                                                                                  | With {provider} as {email}                                                                                                                                                              |
| orgHint      | Creamos la organización con 14 días de prueba de Ramal Pro, sin tarjeta. Puedes dejarlo vacío.                                                                                               | We create the organization with a 14-day Ramal Pro trial, no card needed. You can leave it empty.                                                                                       |
| scSubmit     | Crear cuenta                                                                                                                                                                                 | Create account                                                                                                                                                                          |
| scBusy       | Creando cuenta…                                                                                                                                                                              | Creating account…                                                                                                                                                                       |
| scExpiredT   | El alta caducó                                                                                                                                                                               | Sign-up expired                                                                                                                                                                         |
| scExpired    | Tienes 15 minutos para completarla. Vuelve a empezar desde Crear cuenta.                                                                                                                     | You have 15 minutes to finish it. Start again from Sign up.                                                                                                                             |
| scRestart    | Volver a crear cuenta                                                                                                                                                                        | Sign up again                                                                                                                                                                           |

La casilla de terminos y su error reutilizan `terms` y `error.terms` de `packages/i18n` (web).

### 5. W-05 Autorizar Ramal Desktop o Ramal for VS Code

- **Proposito.** La mitad web del inicio de sesion con PKCE de D-03 (brief #64) y V-01 (brief #68).
- **Muestra.** Tarjeta de 420 px: logo "R" con un distintivo pequeno de escritorio o de VS Code, titulo,
  texto, fila "Cuenta" (nombre y email, enlace "¿No eres tú? Cambia de cuenta", que cierra la sesion web y
  vuelve aqui), fila "Equipo" con el nombre que declara el equipo y su aviso, que podra hacer la app,
  botones "Autorizar" y "Cancelar" (formulario de verdad, no JavaScript) y una nota al pie.
- **Estados.**
  - Sin sesion web: login con `returnTo` y banner informativo encima de la tarjeta.
  - Inicio de sesion de hace mas de 5 minutos: W-35 en linea dentro de la tarjeta, antes de ensenar nada.
  - Validando (esqueleto de la tarjeta), lista, autorizando (boton ocupado).
  - Peticion no valida (400): nunca redirige; error con el `code` en monoespaciada.
  - VS Code tras Autorizar o Cancelar: la pagina cambia a "Vuelve a VS Code" o "Cancelado" (el navegador
    se queda en esta pagina al abrir `vscode://`).
  - Desktop tras Autorizar o Cancelar: el navegador aterriza en la pagina del receptor local del motor
    (`127.0.0.1`). Se disena aqui: estatica, sin JavaScript ni recursos externos, CSS en linea, fuentes del
    sistema, mismos tokens y tema con `prefers-color-scheme`. Estados exito, cancelado y error.

| Clave         | ES                                                                                                                                              | EN                                                                                                                                             |
| ------------- | ----------------------------------------------------------------------------------------------------------------------------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------- |
| azNeedLogin   | Inicia sesión para autorizar {client} en este equipo.                                                                                           | Log in to authorize {client} on this device.                                                                                                   |
| azTitle       | Autorizar {client}                                                                                                                              | Authorize {client}                                                                                                                             |
| azText        | {client} pide acceso a tu cuenta de Ramal en este equipo.                                                                                       | {client} is asking for access to your Ramal account on this device.                                                                            |
| azAccount     | Cuenta                                                                                                                                          | Account                                                                                                                                        |
| azNotYou      | ¿No eres tú? Cambia de cuenta                                                                                                                   | Not you? Switch account                                                                                                                        |
| azDevice      | Equipo                                                                                                                                          | Device                                                                                                                                         |
| azDeviceNote  | Nombre que declara el equipo. Si no lo reconoces, cancela.                                                                                      | Name the device reports. If you don't recognize it, cancel.                                                                                    |
| azCan         | Podrá sincronizar con tu cuenta tus workspaces, repositorios, cuentas vinculadas y preferencias, y usar las llaves de IA de tus organizaciones. | It will be able to sync your workspaces, repositories, linked accounts and preferences with your account, and use your organizations' AI keys. |
| azApprove     | Autorizar                                                                                                                                       | Authorize                                                                                                                                      |
| azBusy        | Autorizando…                                                                                                                                    | Authorizing…                                                                                                                                   |
| azFoot        | Te avisaremos por email de este dispositivo nuevo. Puedes cerrarle la sesión cuando quieras en Cuenta › Dispositivos y sesiones.                | We'll email you about this new device. You can sign it out any time in Account › Devices and sessions.                                         |
| azBadTitle    | Esta petición de autorización no es válida                                                                                                      | This authorization request isn't valid                                                                                                         |
| azBad         | Vuelve a Ramal y empieza de nuevo. Si llegaste aquí desde un enlace que te enviaron, ciérralo: puede ser un intento de suplantación.            | Go back to Ramal and start again. If you got here from a link someone sent you, close it: it may be a phishing attempt.                        |
| azBackVs      | Vuelve a VS Code                                                                                                                                | Go back to VS Code                                                                                                                             |
| azBackVsText  | Si el navegador pregunta, permite abrir VS Code. El inicio de sesión termina allí; puedes cerrar esta pestaña.                                  | If your browser asks, allow it to open VS Code. Signing in finishes there; you can close this tab.                                             |
| azCancelled   | Cancelado. Puedes cerrar esta pestaña.                                                                                                          | Cancelled. You can close this tab.                                                                                                             |
| lcOkTitle     | Sesión iniciada en {client}                                                                                                                     | Signed in to {client}                                                                                                                          |
| lcOk          | Ya puedes volver a la app y cerrar esta pestaña.                                                                                                | You can go back to the app and close this tab.                                                                                                 |
| lcCancelTitle | Inicio de sesión cancelado                                                                                                                      | Sign-in cancelled                                                                                                                              |
| lcCancel      | Vuelve a {client} si quieres intentarlo de nuevo.                                                                                               | Go back to {client} if you want to try again.                                                                                                  |
| lcErrTitle    | No se pudo iniciar sesión                                                                                                                       | Couldn't sign in                                                                                                                               |
| lcErr         | {reason}. Vuelve a {client} e inténtalo de nuevo.                                                                                               | {reason}. Go back to {client} and try again.                                                                                                   |

`{client}` es "Ramal Desktop" o "Ramal for VS Code"; los nombres de producto no se traducen.

### 6. W-06 Codigo de dispositivo

- **Proposito.** Alternativa cuando la redireccion no funciona (Codespaces en el navegador, SSH remoto,
  navegador en otro equipo). El usuario escribe el codigo que ve en su equipo: nunca llega un enlace con el
  codigo puesto.
- **Muestra.** Tarjeta con un campo grande en monoespaciada (24 px, espaciado amplio), formato `XXXX-XXXX`
  en mayusculas, que acepta el codigo pegado con o sin guion. Tras "Continuar": que app pide acceso, el
  equipo con su aviso, "Pedido hace 42 segundos" y una **caja de aviso** (`--warn` al 10 %) antes de
  "Aprobar" y "Rechazar". Sin ciudad ni pais (decision #53).
- **Estados.** Sin sesion (login con banner), escribir codigo, formato no valido (en linea), buscando,
  codigo encontrado, reautenticar (W-35 en linea), codigo no existe o caducado, demasiados intentos,
  aprobado, rechazado.

| Clave       | ES                                                                                                                 | EN                                                                                                             |
| ----------- | ------------------------------------------------------------------------------------------------------------------ | -------------------------------------------------------------------------------------------------------------- |
| dcNeedLogin | Inicia sesión para conectar un equipo con un código.                                                               | Log in to connect a device with a code.                                                                        |
| dcTitle     | Conecta un equipo                                                                                                  | Connect a device                                                                                               |
| dcText      | Escribe el código que ves en Ramal for VS Code o en Ramal Desktop.                                                 | Enter the code shown in Ramal for VS Code or Ramal Desktop.                                                    |
| dcContinue  | Continuar                                                                                                          | Continue                                                                                                       |
| dcFormat    | El código tiene 8 letras, por ejemplo WDJB-MJHT.                                                                   | The code has 8 letters, for example WDJB-MJHT.                                                                 |
| dcAsks      | {client} pide acceso                                                                                               | {client} is asking for access                                                                                  |
| dcAgo       | Pedido hace {seconds} segundos                                                                                     | Requested {seconds} seconds ago                                                                                |
| dcWarn      | Aprueba solo si acabas de pedir este código en tu propio equipo. Si alguien te lo ha enviado o dictado, recházalo. | Only approve if you just requested this code on your own device. If someone sent or read it to you, reject it. |
| dcApprove   | Aprobar                                                                                                            | Approve                                                                                                        |
| dcReject    | Rechazar                                                                                                           | Reject                                                                                                         |
| dcNotFound  | Ese código no existe o ha caducado. Los códigos duran 10 minutos; pide uno nuevo en tu equipo.                     | That code doesn't exist or has expired. Codes last 10 minutes; get a new one on your device.                   |
| dcTooMany   | Demasiados intentos. Espera unos minutos antes de probar otro código.                                              | Too many attempts. Wait a few minutes before trying another code.                                              |
| dcApproved  | Listo. Vuelve a tu equipo: el inicio de sesión termina solo en unos segundos.                                      | Done. Go back to your device: signing in finishes on its own in a few seconds.                                 |
| dcRejected  | Rechazado. Ese equipo no tendrá acceso a tu cuenta.                                                                | Rejected. That device won't get access to your account.                                                        |

### 7. W-07 Formularios enviando y errores del servidor

- **Muestra.** En login, registro y todas las tarjetas de este brief: boton ocupado (indicador de 14 px y
  gerundio), campos y botones OAuth deshabilitados mientras se envia. Errores del servidor en la caja
  `au.error`; errores de campo (422) en linea bajo cada campo, todos a la vez.
- **Reglas.** El minimo de 8 caracteres solo en el registro; al entrar basta con que no este vacia
  (B-53). Mismo texto si el email no existe o la contrasena no casa.

| Clave        | ES                                                             | EN                                                    |
| ------------ | -------------------------------------------------------------- | ----------------------------------------------------- |
| busyLogin    | Entrando…                                                      | Logging in…                                           |
| busySignup   | Creando cuenta…                                                | Creating account…                                     |
| errCreds     | El email o la contraseña no son correctos.                     | The email or password is incorrect.                   |
| errTaken     | Ya existe una cuenta con este email.                           | An account with this email already exists.            |
| errTakenLink | Iniciar sesión                                                 | Log in                                                |
| errRate      | Demasiados intentos. Vuelve a intentarlo en {minutes} minutos. | Too many attempts. Try again in {minutes} minutes.    |
| errName      | Escribe tu nombre (máximo 80 caracteres).                      | Enter your name (80 characters max).                  |
| errServer    | Algo falló en Ramal. Inténtalo de nuevo en un momento.         | Something went wrong at Ramal. Try again in a moment. |

Sin conexion: clave `loginNoApi` del brief #62. Email no valido, contrasena vacia y terminos: claves
existentes `error.email`, `error.passwordRequired`, `error.passwordTooShort` y `error.terms`.

### 8. W-08 Vuelta del OAuth de plataforma intermediado

- **Proposito.** Conectar una cuenta de plataforma (GitLab con PKCE, Bitbucket con el broker) termina en el
  receptor local del motor, no en la web. La web solo pone la pagina de error cuando la API no sabe a
  donde redirigir.
- **Muestra.**
  - `/connect/error?reason=expired` (y razon desconocida): pagina publica con la tarjeta, titulo, texto y
    "Ir al inicio".
  - Paginas del receptor local para plataformas: la misma plantilla estatica de W-05, con exito y error.
- **Estados.** Caducado, desconocido; local exito y local error.

| Clave      | ES                                                                                                                               | EN                                                                                                                                  |
| ---------- | -------------------------------------------------------------------------------------------------------------------------------- | ----------------------------------------------------------------------------------------------------------------------------------- |
| ceTitle    | Esta conexión ya no es válida                                                                                                    | This connection is no longer valid                                                                                                  |
| ceExpired  | El enlace para conectar tu cuenta caducó o ya se usó. Vuelve a Ramal Desktop o a VS Code y empieza de nuevo desde Añadir cuenta. | The link to connect your account expired or was already used. Go back to Ramal Desktop or VS Code and start again from Add account. |
| ceUnknown  | Algo falló al conectar la cuenta. Vuelve a Ramal e inténtalo de nuevo.                                                           | Something went wrong connecting the account. Go back to Ramal and try again.                                                        |
| ceHome     | Ir al inicio                                                                                                                     | Go to home                                                                                                                          |
| lpOkTitle  | Cuenta de {platform} conectada                                                                                                   | {platform} account connected                                                                                                        |
| lpOk       | Vuelve a Ramal para terminar. Puedes cerrar esta pestaña.                                                                        | Go back to Ramal to finish. You can close this tab.                                                                                 |
| lpErrTitle | {platform} no autorizó la conexión                                                                                               | {platform} didn't authorize the connection                                                                                          |
| lpErr      | Motivo: {reason}. Vuelve a Ramal e inténtalo de nuevo.                                                                           | Reason: {reason}. Go back to Ramal and try again.                                                                                   |

### 9. W-09 Sesion caducada y redirecciones

- **Muestra.** Un 401 definitivo en el panel lleva a `/login?returnTo=...` con banner informativo encima de
  la tarjeta; al entrar vuelve a la pagina de origen (tambien `/authorize/app`, `/activate` y
  `/invitations`). Con sesion iniciada, `/login`, `/signup` y `/forgot-password` redirigen al panel sin
  pantalla, y la barra publica ensena "Abrir panel" (B-58).
- **Estados.** Caducada, cerrada desde otro dispositivo. Dibujar las dos variantes del login.

| Clave      | ES                                                                                 | EN                                                                    |
| ---------- | ---------------------------------------------------------------------------------- | --------------------------------------------------------------------- |
| webRevoked | Tu sesión se cerró desde otro dispositivo. Vuelve a iniciar sesión para continuar. | Your session was ended from another device. Log in again to continue. |

Caducada: clave `webExpired` del brief #62.

### 10. W-33 Contrasena

- **Proposito.** Cambiar la contrasena, o poner la primera en una cuenta creada con OAuth. Desktop y VS
  Code abren `/account#password` (la tarjeta se resalta 2 s).
- **Muestra.** En la seccion "Cuenta", tarjeta "Contraseña" con una fila y su boton. El cambio se hace en el
  modal generico con "Contraseña actual", "Contraseña nueva" (medidor) y "Repite la contraseña nueva", mas
  una linea informativa. Sin contrasena: la fila lo explica y "Añadir contraseña…" pide antes W-35.
- **Estados.** Con contrasena, sin contrasena, modal, actual incorrecta (401, en linea), no cumple (422),
  guardando, cambiada (toast `ok`), anadida (toast `ok`).

| Clave       | ES                                                                                                      | EN                                                                                                        |
| ----------- | ------------------------------------------------------------------------------------------------------- | --------------------------------------------------------------------------------------------------------- |
| pwCard      | Contraseña                                                                                              | Password                                                                                                  |
| pwHasD      | Para entrar con tu email.                                                                               | Used to log in with your email.                                                                           |
| pwChange    | Cambiar contraseña…                                                                                     | Change password…                                                                                          |
| pwNoneD     | Tu cuenta no tiene contraseña: entras con {provider}. Añade una para poder entrar también con tu email. | Your account has no password: you log in with {provider}. Add one so you can also log in with your email. |
| pwAdd       | Añadir contraseña…                                                                                      | Add password…                                                                                             |
| pwCurrent   | Contraseña actual                                                                                       | Current password                                                                                          |
| pwRepeatNew | Repite la contraseña nueva                                                                              | Repeat the new password                                                                                   |
| pwModalNote | No cambia tu frase de sincronización. Cerraremos la sesión en tus otros dispositivos y navegadores.     | It doesn't change your sync passphrase. We'll sign you out on your other devices and browsers.            |
| pwWrong     | La contraseña actual no es correcta.                                                                    | The current password is incorrect.                                                                        |
| pwChanged   | Contraseña cambiada. Cerramos la sesión en tus otros dispositivos.                                      | Password changed. We signed you out on your other devices.                                                |
| pwAdded     | Contraseña añadida. Ya puedes entrar también con tu email.                                              | Password added. You can now log in with your email too.                                                   |

### 11. W-34 Identidades vinculadas

- **Proposito.** Ver, vincular y quitar las formas de entrar. No son las cuentas de plataforma para Git:
  el subtitulo lo dice.
- **Muestra.** Tarjeta "Formas de entrar" en "Cuenta": fila "Email y contraseña" (si la hay), una fila por
  identidad vinculada (insignia, email, "Vinculada el 1 sep 2026 · último uso hace 2 días", menu con
  "Quitar…") y una fila por proveedor sin vincular con "Vincular".
- **Estados.** Vincular (W-35 y salida al proveedor), vinculada (toast), ya usada por otra cuenta (banner
  al volver), correo sin verificar (Vincular deshabilitado con tooltip), quitar (modal de peligro), unica
  forma de entrar (Quitar deshabilitado con tooltip, 409 `last_sign_in_method`).

| Clave        | ES                                                                                             | EN                                                                                     |
| ------------ | ---------------------------------------------------------------------------------------------- | -------------------------------------------------------------------------------------- |
| idCard       | Formas de entrar                                                                               | Sign-in methods                                                                        |
| idSub        | Sirven para entrar en Ramal. Las cuentas para clonar y hacer push están en Cuentas vinculadas. | They let you log in to Ramal. Accounts for cloning and pushing are in Linked accounts. |
| idEmailPw    | Email y contraseña                                                                             | Email and password                                                                     |
| idMeta       | Vinculada el {date} · último uso {ago}                                                         | Linked on {date} · last used {ago}                                                     |
| idLink       | Vincular                                                                                       | Link                                                                                   |
| idRemove     | Quitar…                                                                                        | Remove…                                                                                |
| idLinked     | {provider} vinculado. Te avisamos por email.                                                   | {provider} linked. We've emailed you about it.                                         |
| idInUse      | Esa cuenta de {provider} ya está vinculada a otra cuenta de Ramal.                             | That {provider} account is already linked to another Ramal account.                    |
| idNeedVerify | Confirma tu email para vincular otra forma de entrar.                                          | Confirm your email to link another sign-in method.                                     |
| idRmTitle    | ¿Quitar {provider}?                                                                            | Remove {provider}?                                                                     |
| idRmText     | Ya no podrás entrar con {provider}. Te avisaremos por email.                                   | You won't be able to log in with {provider} anymore. We'll email you about it.         |
| idRmBtn      | Quitar                                                                                         | Remove                                                                                 |
| idLast       | Es tu única forma de entrar. Añade una contraseña u otra identidad antes de quitarla.          | It's your only way to log in. Add a password or another identity before removing it.   |

### 12. W-35 Reautenticar antes de una accion sensible

- **Proposito.** Eliminar la cuenta, reiniciar la boveda, cambiar el email, vincular o quitar una forma de
  entrar, anadir contrasena, transferir o eliminar una organizacion y autorizar un equipo exigen un inicio
  de sesion reciente.
- **Muestra.** Modal generico (y variante en linea dentro de las tarjetas de W-05 y W-06) con titulo, la
  accion que lo pide y solo los metodos que acepta (`acceptedMethods`): campo de contrasena con
  "Confirmar"; "Continuar con Google"; "Continuar con GitHub" o "GitLab" solo si la accion admite
  reautenticacion debil; "Enviarme un enlace por email". Si GitHub y GitLab no valen, una linea lo
  explica. Al confirmar, el modal se cierra y la accion sigue sola.
- **Enlace por email.** Estado de espera dentro del modal, con email enmascarado y "Usar otro método"; la
  pagina `/reauthenticate` confirma o dice que el enlace no sirve.
- **Estados.** Metodos, contrasena incorrecta, proveedor no vinculado (`reauth_failed`), enlace enviado y
  esperando, reenviar (a los 60 s), confirmado, enlace no valido.

| Clave       | ES                                                                                                 | EN                                                                                               |
| ----------- | -------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------ |
| raTitle     | Confirma que eres tú                                                                               | Confirm it's you                                                                                 |
| raFor       | Para {action} necesitamos confirmar tu identidad.                                                  | To {action}, we need to confirm your identity.                                                   |
| raConfirm   | Confirmar                                                                                          | Confirm                                                                                          |
| raEmail     | Enviarme un enlace por email                                                                       | Email me a link                                                                                  |
| raStrong    | Para esta acción no sirven GitHub ni GitLab: no pueden pedirte que vuelvas a entrar.               | GitHub and GitLab don't work for this action: they can't ask you to log in again.                |
| raWaiting   | Te enviamos un enlace a {maskedEmail}. Ábrelo en cualquier dispositivo; esta ventana seguirá sola. | We sent a link to {maskedEmail}. Open it on any device; this window will continue on its own.    |
| raOther     | Usar otro método                                                                                   | Use another method                                                                               |
| raWrong     | La contraseña no es correcta.                                                                      | The password is incorrect.                                                                       |
| raNotLinked | Esa cuenta de {provider} no está vinculada a tu cuenta de Ramal.                                   | That {provider} account isn't linked to your Ramal account.                                      |
| raLinkOk    | Identidad confirmada. Vuelve a la pestaña donde lo pediste para continuar.                         | Identity confirmed. Go back to the tab where you asked for it to continue.                       |
| raLinkDead  | Este enlace ya no sirve. Dura 15 minutos y solo funciona una vez. Pídelo otra vez desde Ramal.     | This link no longer works. It lasts 15 minutes and works only once. Ask for it again from Ramal. |

Valores de `{action}`: "eliminar tu cuenta" / "delete your account", "reiniciar la bóveda" / "reset the
vault", "cambiar tu email" / "change your email", "vincular otra forma de entrar" / "link another sign-in
method", "quitar una forma de entrar" / "remove a sign-in method", "añadir una contraseña" / "add a
password", "transferir la organización" / "transfer the organization", "eliminar la organización" /
"delete the organization", "autorizar un equipo" / "authorize a device".

## Reglas

- Desktop y VS Code nunca dibujan un formulario de contrasena: abren estas paginas.
- "¿La olvidaste?" lleva a W-01, nunca a un toast. Ningun texto dice "simulado" (B-46).
- W-01 responde siempre igual. El login da el mismo texto para email desconocido y contrasena incorrecta.
- El minimo de 8 caracteres solo en el registro (B-53). El registro con email y el alta con OAuth guardan la
  organizacion y pasan por los terminos (B-54).
- `/activate` no ensena ciudad ni pais (decision #53) y no existe ningun enlace con el codigo ya puesto.
- Con un `redirect_uri` no valido, `/authorize/app` nunca redirige: ensena el error.
- Banners y cajas informativas con `--info` (B-61); errores en la caja `au.error`; quitar una identidad
  confirma con boton de peligro.
- Las identidades de entrada no son cuentas de plataforma ni se comparten con nadie.
- Textos de boveda: cambiar o restablecer la contrasena no toca la frase de sincronizacion.
- Dominio en los textos: `ramal.bahoque.com`; nunca `ramal.dev`. Rol de titular: "Titular" (B-50).
- Las paginas del receptor local no cargan nada externo (ni Google Fonts) y funcionan sin JavaScript.
- Las claves con contador (`{count}`, `{seconds}`, `{minutes}`) llevan variantes `_one` y `_other`
  (ADR 0017); las tablas escriben la forma plural.
- Confirmaciones de peligro con el patron del brief #71 (seccion 10): titulo en pregunta, consecuencia en
  una caja, boton `--danger` y foco inicial en "Cancelar".

## Entregable

Un zip `ramal-handoff-69-auth.zip` con la estructura de `docs/design/handoff/`:

- `Ramal Web Auth.dc.html` nuevo, con props `theme`, `lang`, `screen` (un valor por pagina con el id de
  `gaps.md`: `W-01`, `W-02`, `W-03`, `W-03-cambio-email`, `W-03-deshacer-email`, `W-04-error`,
  `W-04-completar`, `W-05`, `W-06`, `W-07`, `W-08`, `W-09`, `W-33`, `W-34`, `W-35`) y `state` (los estados
  de cada pagina). `W-33` y `W-34` se ensenan dentro del panel con la seccion "Cuenta" seleccionada.
- `Ramal Local Callback.dc.html` nuevo: las paginas del receptor local, con props `kind` (`signin`,
  `platform`), `client` (`desktop`, `vscode`), `result` (`success`, `cancelled`, `error`) y tema por
  `prefers-color-scheme`.
- `Ramal Web.dc.html` actualizado solo donde cambia lo existente: "¿La olvidaste?" navega a W-01, pista de
  organizacion en el registro, errores del servidor en la caja de error.
- `ramal-data.js` (o el objeto `W` del prototipo web) con las claves nuevas bajo `auth.*`, en es y en.
- `README.md` solo con lo nuevo: rutas, pantallas y estados con sus ids.

## Hecho cuando

- [ ] W-01 a W-09 y W-33 a W-35 diseñados
- [ ] La página de restablecer dice que no cambia la frase de sincronización
- [ ] El alta con OAuth pasa también por los términos
