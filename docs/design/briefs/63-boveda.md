# Brief #63: pantallas de la boveda (K-01 a K-10, X-03, X-04)

Issue: [eabl0306/git-rudder#63](https://github.com/eabl0306/git-rudder/issues/63). Fuentes: `docs/design/gaps.md`
(seccion 1.2, X-03 y X-04), ADR 0012 (boveda de extremo a extremo) y la decision #57 (cierre de sesion).

## Contexto

- Productos: Desktop y VS Code para K-01 a K-09 (K-08 enlaza al reinicio de la web); la web solo K-10.
  La web no cifra ni descifra nada: solo sabe si la boveda existe y desde cuando, y es el unico sitio
  donde se restablece.
- Que es: los tokens de plataforma y, si el usuario lo activa, sus llaves de IA personales viajan
  cifrados con una llave que el servidor nunca ve. Esa llave se protege con una **frase de
  sincronizacion** (no es la contrasena de Git Rudder) y con un **codigo de recuperacion** de 32 caracteres.
- Donde se engancha: Desktop en Ajustes > Cuenta y equipo (tarjeta nueva "Boveda"), Ajustes >
  Plataformas Git (tarjetas de cuenta) y Ajustes > IA (filas de llave); VS Code en el webview
  "Configuracion de Git Rudder" (secciones Cuenta de Git Rudder, Plataformas Git e IA, y un panel de boveda dentro
  de ese mismo webview); la web en el panel, "Configuracion en la nube".
- Componentes a reutilizar: Desktop el **modal generico** en su variante ancha (580), la **caja info**,
  el **campo de texto**, el **checkbox**, los botones principal y de peligro, la **tarjeta** de ajustes
  con sus **filas**, la **tarjeta de cuenta** de Plataformas Git, la **fila de llave** de IA, el chip
  "Predeterminada" y el chip de alcance (fondo `--bg` con borde), y el **toast**. VS Code: el **modal
  quick input** (600 px arriba), la **fila de entidad** y la **fila de ajuste** del webview de ajustes, y
  la **notificacion**. Web: la **tarjeta** con **filas** de "Configuracion en la nube", el **toggle**
  38x22 y el **modal** de confirmacion. Estados de carga, error y validacion: los del brief #62.

## Pantallas y estados

Variantes para todo: claro y oscuro; ES y EN; Desktop (densidad de escritorio) y VS Code (tokens y
tipografia de VS Code, botones de 26 px y radio 2, filas compactas). En VS Code, K-01, K-02, K-03, K-06,
K-07 y K-08 se abren como un panel dentro del webview "Configuracion de Git Rudder" (que sustituye el
contenido de la pagina, con "Volver" arriba y el mismo ancho maximo de 820), porque el quick input
nativo no puede ensenar una explicacion larga, un medidor de fortaleza ni un codigo con "Descargar".
Si el flujo empieza fuera (al anadir una cuenta), se abre ese webview directamente en el panel. Asi no
hace falta un webview nuevo ademas de los que fija el brief #68. En Desktop son modales anchos con
indicador "Paso 1 de 2".

### 1. K-01 Crear la frase de sincronizacion

- **Cuando sale.** La primera vez que un secreto se iba a sincronizar: al anadir una cuenta de
  plataforma con "Cuentas vinculadas" activo, al activar "Llaves de IA" en la sincronizacion, o cuando un
  equipo ve una categoria de secretos activa sin boveda (se encendio desde la web). Nunca al iniciar
  sesion: quien no sincroniza secretos nunca ve una frase. En VS Code, al final del quick input "Anadir
  cuenta" sale primero un aviso modal nativo con "Configurar la boveda" y "Ahora no".
- **Muestra.** Titulo; dos frases que dicen para que sirve (variante cuentas o variante llaves de IA,
  textos separados); un parrafo de por que no es la contrasena; campo "Frase de sincronizacion" con boton
  de mostrar u ocultar; medidor de fortaleza (tres tramos de 4 px en `--danger`, `--warn` y `--ok`, con
  la etiqueta a la derecha); campo "Repite la frase"; caja de aviso en `--warn` que dice que nadie puede
  recuperar los secretos; nota bajo los botones sobre "Ahora no".
- **Acciones.** "Ahora no" (secundario): guarda el secreto solo en este equipo (K-05) y cierra.
  "Continuar" (principal): pasa a K-02. Deshabilitado hasta que la frase tenga 12 caracteres, la
  fortaleza minima y las dos coincidan.
- **Estados.** Vacio, escribiendo con medidor debil, aceptable y fuerte, frases distintas, demasiado
  corta, sin conexion (todo deshabilitado con el texto de C-05), ya existe una boveda (409
  `vault_exists`, otro equipo la creo mientras tanto: el modal pasa a K-03 con un aviso).

| Clave        | ES                                                                                                                                                                            | EN                                                                                                                                                                            |
| ------------ | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| k1Title      | Protege tus secretos con una frase de sincronización                                                                                                                          | Protect your secrets with a sync passphrase                                                                                                                                   |
| k1BodyAcc    | Para que tus cuentas de plataforma lleguen a tus otros equipos, Git Rudder cifra sus tokens en este equipo antes de subirlos. Solo tú tienes la llave.                        | So your platform accounts reach your other devices, Git Rudder encrypts their tokens on this device before uploading them. Only you hold the key.                             |
| k1BodyAi     | Para que tus llaves de IA lleguen a tus otros equipos, Git Rudder las cifra en este equipo antes de subirlas. Solo tú tienes la llave.                                        | So your AI keys reach your other devices, Git Rudder encrypts them on this device before uploading them. Only you hold the key.                                               |
| k1WhyNot     | No es tu contraseña de Git Rudder. La contraseña la comprueba el servidor; esta frase nunca sale de tus equipos. Si entras con GitHub, GitLab o Google, también la necesitas. | It isn't your Git Rudder password. The server checks your password; this passphrase never leaves your devices. You need it even if you sign in with GitHub, GitLab or Google. |
| passLabel    | Frase de sincronización                                                                                                                                                       | Sync passphrase                                                                                                                                                               |
| passRepeat   | Repite la frase                                                                                                                                                               | Repeat the passphrase                                                                                                                                                         |
| passHint     | Al menos 12 caracteres. Una frase de varias palabras es fácil de recordar y difícil de adivinar.                                                                              | At least 12 characters. A phrase of several words is easy to remember and hard to guess.                                                                                      |
| strWeak      | Débil                                                                                                                                                                         | Weak                                                                                                                                                                          |
| strOk        | Aceptable                                                                                                                                                                     | Acceptable                                                                                                                                                                    |
| strStrong    | Fuerte                                                                                                                                                                        | Strong                                                                                                                                                                        |
| errShort     | La frase necesita al menos 12 caracteres.                                                                                                                                     | The passphrase needs at least 12 characters.                                                                                                                                  |
| errGuessable | Es fácil de adivinar. Añade más palabras.                                                                                                                                     | It's easy to guess. Add more words.                                                                                                                                           |
| errMismatch  | Las frases no coinciden.                                                                                                                                                      | The passphrases don't match.                                                                                                                                                  |
| noRecovery   | Si olvidas la frase y pierdes el código de recuperación, nadie podrá recuperar tus secretos, tampoco el soporte de Git Rudder. Tendrías que volver a conectar tus cuentas.    | If you forget the passphrase and lose the recovery code, nobody can recover your secrets, not even Git Rudder support. You'd have to reconnect your accounts.                 |
| notNow       | Ahora no                                                                                                                                                                      | Not now                                                                                                                                                                       |
| notNowNote   | Con «Ahora no», lo que añadas se guarda solo en este equipo.                                                                                                                  | With "Not now", what you add is saved on this device only.                                                                                                                    |
| continue     | Continuar                                                                                                                                                                     | Continue                                                                                                                                                                      |
| show / hide  | Mostrar / Ocultar                                                                                                                                                             | Show / Hide                                                                                                                                                                   |
| stepOf       | Paso {step} de {total}                                                                                                                                                        | Step {step} of {total}                                                                                                                                                        |
| vaultExists  | Ya tienes una bóveda creada en otro equipo. Desbloquéala con tu frase.                                                                                                        | You already have a vault created on another device. Unlock it with your passphrase.                                                                                           |
| vsSyncPrompt | ¿Sincronizar esta cuenta con tus otros equipos? Necesitas una frase de sincronización que solo conoces tú.                                                                    | Sync this account with your other devices? You need a sync passphrase that only you know.                                                                                     |
| setUpVault   | Configurar la bóveda                                                                                                                                                          | Set up vault                                                                                                                                                                  |

### 2. K-02 Mostrar el codigo de recuperacion una sola vez

- **Cuando sale.** Despues de K-01, y en sus variantes de K-07 (codigo nuevo) y de rotacion de la llave.
  Segun ADR 0012, el codigo se ensena y se confirma **antes** de subir nada.
- **Muestra.** Titulo; texto que dice que es la unica salida si se olvida la frase y que solo se ve
  ahora; el codigo en IBM Plex Mono de 15 px con espaciado `.08em`, 8 grupos de 4 caracteres en una
  rejilla de 4x2 dentro de una caja `--bg` con borde y radio 9 (ejemplo:
  `7KQ2-M9XD-4TPA-HV3N-Q8WE-ZJ6R-B1CF-5YGS`, base32 de Crockford, sin I, L, O ni U); botones "Copiar" y
  "Descargar .txt"; checkbox de confirmacion; aviso de que nadie mas puede recuperarlo.
- **Acciones.** "Copiar" (toast `info`), "Descargar .txt" (fichero con titulo, correo de la cuenta,
  fecha, codigo y la misma advertencia), "Atras" (vuelve a K-01 con la misma frase y el mismo codigo) y
  "Crear boveda" (principal, deshabilitado hasta marcar la casilla). Cerrar con Esc o x pide
  confirmacion.
- **Estados.** Sin marcar, marcado, copiado, descargado, creando (boton ocupado "Creando tu boveda..."),
  error al subir (caja de error del brief #62 con "Reintentar": el codigo en pantalla sigue valiendo y
  reintentar no genera otro), exito, confirmacion de salida sin terminar.
- **Exito.** Ultimo paso del modal: titulo "Boveda creada", texto, boton "Listo"; toast `ok`.

| Clave        | ES                                                                                                      | EN                                                                                                                      |
| ------------ | ------------------------------------------------------------------------------------------------------- | ----------------------------------------------------------------------------------------------------------------------- |
| k2Title      | Guarda tu código de recuperación                                                                        | Save your recovery code                                                                                                 |
| k2Body       | Es la única forma de abrir tu bóveda si olvidas la frase. Solo lo verás ahora: Git Rudder no lo guarda. | It's the only way to open your vault if you forget the passphrase. You'll only see it now: Git Rudder doesn't store it. |
| k2NoOne      | Nadie más puede recuperarlo, tampoco el soporte de Git Rudder. Guárdalo fuera de este equipo.           | Nobody else can recover it, not even Git Rudder support. Keep it somewhere other than this device.                      |
| copy         | Copiar                                                                                                  | Copy                                                                                                                    |
| codeCopied   | Código copiado                                                                                          | Code copied                                                                                                             |
| download     | Descargar .txt                                                                                          | Download .txt                                                                                                           |
| fileName     | git-rudder-codigo-de-recuperacion.txt                                                                   | git-rudder-recovery-code.txt                                                                                            |
| k2Check      | He guardado el código en un lugar seguro                                                                | I've saved the code somewhere safe                                                                                      |
| back         | Atrás                                                                                                   | Back                                                                                                                    |
| createVault  | Crear bóveda                                                                                            | Create vault                                                                                                            |
| creating     | Creando tu bóveda…                                                                                      | Creating your vault…                                                                                                    |
| uploadFail   | No se pudo crear la bóveda. El código que guardaste sigue valiendo.                                     | The vault couldn't be created. The code you saved is still valid.                                                       |
| leaveTitle   | ¿Salir sin crear la bóveda?                                                                             | Leave without creating the vault?                                                                                       |
| leaveBody    | Lo que añadiste se guardará solo en este equipo.                                                        | What you added will be saved on this device only.                                                                       |
| stayHere     | Seguir aquí                                                                                             | Stay                                                                                                                    |
| leave        | Salir                                                                                                   | Leave                                                                                                                   |
| doneTitle    | Bóveda creada                                                                                           | Vault created                                                                                                           |
| doneBody     | Tus secretos se sincronizan cifrados. En cada equipo nuevo te pediremos la frase una vez.               | Your secrets now sync encrypted. Each new device will ask for the passphrase once.                                      |
| done         | Listo                                                                                                   | Done                                                                                                                    |
| newCodeTitle | Tu nuevo código de recuperación                                                                         | Your new recovery code                                                                                                  |
| oldCodeVoid  | El código anterior ya no sirve.                                                                         | The previous code no longer works.                                                                                      |

### 3. K-03 Desbloquear en un dispositivo nuevo

- **Cuando sale.** Tras la primera sincronizacion de un equipo con sesion, si existe boveda y hay una
  categoria de secretos activa (una vez, con "Ahora no"; despues quedan los avisos de K-04). Tambien
  desde "Desbloquear" en K-04 o en la tarjeta Boveda, al intentar usar una cuenta sincronizada, despues
  de que otro equipo rote la llave y despues de un 409 `vault_exists` en K-01.
- **Muestra.** Titulo, texto, campo de frase (o de codigo, con un enlace que alterna), nota de que llega
  un correo de aviso, enlace "Olvide la frase" (K-08). El campo de codigo acepta minusculas, espacios y
  guiones y agrupa de cuatro en cuatro mientras se escribe.
- **Acciones.** "Ahora no", "Desbloquear" (principal), "Usar el codigo de recuperacion" o "Usar la
  frase", "Olvide la frase".
- **Estados.** Vacio, desbloqueando (Argon2id tarda alrededor de un segundo: boton ocupado), descargando
  secretos (texto con el numero), frase incorrecta, cinco fallos seguidos (aparecen las dos salidas),
  codigo incompleto, codigo incorrecto, limite diario alcanzado (429, 20 al dia), sin conexion, exito
  (toast `ok` con lo que quedo listo). Si se entro con el codigo, despues del exito se ofrece cambiar la
  frase (K-06). Variante "la llave cambio en otro equipo" con su propio texto.

| Clave          | ES                                                                                                          | EN                                                                                                  |
| -------------- | ----------------------------------------------------------------------------------------------------------- | --------------------------------------------------------------------------------------------------- |
| k3Title        | Desbloquea tu bóveda en este equipo                                                                         | Unlock your vault on this device                                                                    |
| k3Body         | Escribe tu frase de sincronización para usar aquí las cuentas y llaves que añadiste en otros equipos.       | Enter your sync passphrase to use the accounts and keys you added on other devices.                 |
| k3Rotated      | La llave de tu bóveda cambió en otro equipo. Escribe tu frase para seguir usando tus cuentas sincronizadas. | Your vault key changed on another device. Enter your passphrase to keep using your synced accounts. |
| k3Email        | Te enviaremos un correo para avisarte de que este equipo tiene acceso.                                      | We'll email you to let you know this device now has access.                                         |
| useCode        | Usar el código de recuperación                                                                              | Use the recovery code                                                                               |
| usePass        | Usar la frase                                                                                               | Use the passphrase                                                                                  |
| codeLabel      | Código de recuperación                                                                                      | Recovery code                                                                                       |
| forgot         | Olvidé la frase                                                                                             | I forgot the passphrase                                                                             |
| unlock         | Desbloquear                                                                                                 | Unlock                                                                                              |
| unlocking      | Desbloqueando…                                                                                              | Unlocking…                                                                                          |
| fetching       | Descargando tus secretos…                                                                                   | Downloading your secrets…                                                                           |
| errPass        | La frase no es correcta.                                                                                    | That passphrase isn't correct.                                                                      |
| errPassMany    | ¿No la recuerdas? Usa el código de recuperación o mira qué hacer si la olvidaste.                           | Can't remember it? Use the recovery code or see what to do if you forgot it.                        |
| errCodeShort   | El código tiene 32 caracteres. Revisa que esté completo.                                                    | The code has 32 characters. Check that it's complete.                                               |
| errCode        | El código no es correcto.                                                                                   | That code isn't correct.                                                                            |
| errDaily       | Has pedido tu bóveda demasiadas veces hoy. Vuelve a intentarlo mañana.                                      | You've requested your vault too many times today. Try again tomorrow.                               |
| unlocked_one   | Bóveda desbloqueada · {count} secreto listo                                                                 | Vault unlocked · {count} secret ready                                                               |
| unlocked_other | Bóveda desbloqueada · {count} secretos listos                                                               | Vault unlocked · {count} secrets ready                                                              |
| afterCode      | Has entrado con el código de recuperación. ¿Quieres poner una frase nueva ahora?                            | You used the recovery code. Do you want to set a new passphrase now?                                |
| changePass     | Cambiar la frase                                                                                            | Change passphrase                                                                                   |
| later          | Más tarde                                                                                                   | Later                                                                                               |

### 4. K-04 Boveda bloqueada en este equipo, y la tarjeta Boveda

- **Proposito.** Los metadatos de las cuentas llegan sin frase y los tokens no: hay que explicar por que
  una cuenta visible no funciona.
- **En las listas** (Desktop Plataformas Git e IA; VS Code secciones Plataformas Git e IA del webview):
  banner informativo arriba de la seccion con "Desbloquear". Cada cuenta sincronizada sin desbloquear
  ensena sus metadatos (plataforma, usuario, host, metodo, scopes, "Usada en N repos") con un candado en
  lugar del punto de estado y la linea "Bloqueada · Desbloquear para usar". "Probar conexion" queda
  deshabilitado con tooltip. "Desconectar cuenta" sigue disponible. Las cuentas "Solo en este equipo"
  (K-05) funcionan con normalidad.
- **Otros estados de fila** (mismo sitio): "Sin token en este equipo" (la cuenta se sincronizo sin
  secreto, por ejemplo tras un reinicio de la boveda) con "Actualizar credenciales"; "Las credenciales
  no coinciden con la cuenta" (`--danger`, la copia sellada no coincide con la fila y Git Rudder no la usa)
  con "Reconectar"; token rotado en otro equipo con "Reconectar" (reutiliza D-14, brief #65).
- **Al usarla.** Si una operacion necesita una cuenta bloqueada (push, PRs): toast `warn` con accion
  "Desbloquear" (Desktop) o notificacion nativa con el mismo boton (VS Code).
- **Tarjeta "Boveda"** en Desktop Ajustes > Cuenta y equipo, despues de la tarjeta de sesion (en VS Code,
  bloque dentro de "Cuenta de Git Rudder"). Es la entrada a K-06, K-07, la rotacion y K-08. Estados:
  - Sin configurar: texto y "Configurar la boveda" (abre K-01).
  - Desbloqueada: "Desbloqueada en este equipo · creada el {date}", botones "Cambiar la frase" y "Nuevo
    codigo de recuperacion", menu con "Rotar la llave..." y "Olvide la frase".
  - Bloqueada: "Bloqueada en este equipo", "Desbloquear" (principal) y "Olvide la frase".
  - Rotacion recomendada (se revoco un equipo que tenia la boveda): caja `--warn` dentro de la tarjeta
    con "Rotar la llave". Sale en todos los equipos hasta que uno rote.
  - Restablecida desde la web: caja `--info` con "Configurar" (K-01).
  - Sin sesion: la tarjeta no aparece. Sin conexion: acciones deshabilitadas (C-05).
- **Rotar la llave** (desde la tarjeta, desde el aviso o encadenado desde K-06 y K-07): modal de
  confirmacion con la frase actual (si viene de K-06 se usa la frase nueva y no se pide), despues K-02 en
  su variante de codigo nuevo y despues progreso sin cancelar. Errores: frase incorrecta, 409
  `vault_secrets_mismatch` (se reintenta una vez solo; si vuelve a fallar, error), 412 (cambio en otro
  equipo).

| Clave          | ES                                                                                                                                                           | EN                                                                                                                                                  |
| -------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------ | --------------------------------------------------------------------------------------------------------------------------------------------------- |
| lockedBanner   | Tu bóveda está bloqueada en este equipo. Ves tus cuentas sincronizadas, pero sus tokens no se pueden usar hasta que la desbloquees.                          | Your vault is locked on this device. You can see your synced accounts, but their tokens can't be used until you unlock it.                          |
| lockedRow      | Bloqueada · Desbloquear para usar                                                                                                                            | Locked · Unlock to use                                                                                                                              |
| testLocked     | Desbloquea la bóveda para probarla                                                                                                                           | Unlock the vault to test it                                                                                                                         |
| noTokenHere    | Sin token en este equipo                                                                                                                                     | No token on this device                                                                                                                             |
| updateCreds    | Actualizar credenciales                                                                                                                                      | Update credentials                                                                                                                                  |
| credsMismatch  | Las credenciales no coinciden con la cuenta. Git Rudder no las usará.                                                                                        | The credentials don't match the account. Git Rudder won't use them.                                                                                 |
| reconnect      | Reconectar                                                                                                                                                   | Reconnect                                                                                                                                           |
| useLocked      | Esta cuenta está en tu bóveda, que está bloqueada en este equipo.                                                                                            | This account is in your vault, which is locked on this device.                                                                                      |
| vaultCard      | Bóveda                                                                                                                                                       | Vault                                                                                                                                               |
| vaultNone      | Sin configurar. Se crea la primera vez que sincronizas una cuenta o una llave.                                                                               | Not set up. It's created the first time you sync an account or a key.                                                                               |
| vaultOpen      | Desbloqueada en este equipo · creada el {date}                                                                                                               | Unlocked on this device · created on {date}                                                                                                         |
| vaultLocked    | Bloqueada en este equipo                                                                                                                                     | Locked on this device                                                                                                                               |
| newCode        | Nuevo código de recuperación                                                                                                                                 | New recovery code                                                                                                                                   |
| rotateMenu     | Rotar la llave…                                                                                                                                              | Rotate key…                                                                                                                                         |
| rotateRec      | Se cerró la sesión de un equipo que tenía tu bóveda. Rota la llave para que ese equipo no pueda leer lo que subas a partir de ahora.                         | A device that had your vault was signed out. Rotate the key so that device can't read anything you upload from now on.                              |
| rotateBtn      | Rotar la llave                                                                                                                                               | Rotate key                                                                                                                                          |
| resetFromWeb   | Tu bóveda se restableció desde el panel web. Configúrala de nuevo para sincronizar tus secretos.                                                             | Your vault was reset from the web dashboard. Set it up again to sync your secrets.                                                                  |
| rotateTitle    | Rotar la llave de la bóveda                                                                                                                                  | Rotate the vault key                                                                                                                                |
| rotateBody     | Git Rudder crea una llave nueva, vuelve a cifrar tus secretos y te da un código de recuperación nuevo. Tus otros equipos te pedirán la frase la próxima vez. | Git Rudder creates a new key, re-encrypts your secrets and gives you a new recovery code. Your other devices will ask for the passphrase next time. |
| rotating_one   | Volviendo a cifrar {count} secreto… No cierres Git Rudder.                                                                                                   | Re-encrypting {count} secret… Don't close Git Rudder.                                                                                               |
| rotating_other | Volviendo a cifrar {count} secretos… No cierres Git Rudder.                                                                                                  | Re-encrypting {count} secrets… Don't close Git Rudder.                                                                                              |
| rotated        | Llave rotada. Tus otros equipos te pedirán la frase.                                                                                                         | Key rotated. Your other devices will ask for the passphrase.                                                                                        |
| errChanged     | La bóveda cambió en otro equipo. Vuelve a intentarlo.                                                                                                        | The vault changed on another device. Try again.                                                                                                     |

### 5. K-05 Cuenta "Solo en este equipo"

- **Proposito.** Quien dice "Ahora no", no tiene cuenta de Git Rudder o tiene la categoria apagada sigue
  pudiendo anadir cuentas y llaves.
- **Muestra.** Chip "Solo en este equipo" con el estilo del chip de alcance (fondo `--bg` y borde) en la
  tarjeta de cuenta, en la fila de llave y en la fila de entidad de VS Code. Tooltip con la explicacion.
- **Acciones del menu.** "Sincronizar esta cuenta" (o llave): con sesion, categoria activa y boveda, la
  sube; sin boveda, abre K-01; sin sesion, el elemento dice "Inicia sesion para sincronizarla" y abre el
  inicio de sesion. Variante: secreto creado con otra cuenta de Git Rudder en este equipo. ADR 0012 fija que
  nunca se sube solo a la nube de otra cuenta; si se le ofrece subirlo o no se ofrece nunca **esta sin
  decidir**. Dibuja el chip "Añadida con otra cuenta de Git Rudder" (va en los dos casos) y, como lamina
  aparte y marcada "pendiente de decidir", la accion "Subir a mi cuenta" con su confirmacion.
- **Estados.** Sin sesion, con sesion sin boveda, con boveda y categoria apagada, de otra cuenta de Git Rudder
  (con y sin la accion pendiente), toast al guardar tras "Ahora no".

| Clave         | ES                                                                                      | EN                                                                                         |
| ------------- | --------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------ |
| localChip     | Solo en este equipo                                                                     | This device only                                                                           |
| localTip      | El token está solo en el llavero de este equipo. No se sube a la nube.                  | The token is only in this device's keychain. It isn't uploaded to the cloud.               |
| syncThisAcc   | Sincronizar esta cuenta                                                                 | Sync this account                                                                          |
| syncThisKey   | Sincronizar esta llave                                                                  | Sync this key                                                                              |
| signInToSync  | Inicia sesión para sincronizarla                                                        | Sign in to sync it                                                                         |
| savedLocal    | Cuenta guardada solo en este equipo                                                     | Account saved on this device only                                                          |
| otherUserChip | Añadida con otra cuenta de Git Rudder                                                   | Added with another Git Rudder account                                                      |
| otherUserAsk  | Esta cuenta se añadió con otra cuenta de Git Rudder en este equipo. ¿Subirla a la tuya? | This account was added with another Git Rudder account on this device. Upload it to yours? |
| upload        | Subir a mi cuenta                                                                       | Upload to my account                                                                       |

### 6. K-06 Cambiar la frase de sincronizacion

- **Entrada.** Tarjeta Boveda, y el ofrecimiento tras desbloquear con el codigo (K-03). Si la boveda esta
  bloqueada, primero K-03. No pide la frase actual: el equipo ya tiene la llave, y asi un equipo
  desbloqueado sirve de salida a quien olvido la frase (K-08).
- **Muestra.** Campos "Frase nueva" y "Repite la frase nueva" con el medidor de K-01; texto de que los
  demas equipos siguen funcionando; pregunta "Crees que alguien conoce tu frase anterior?" con dos
  opciones (radio): "No, solo quiero cambiarla" (por defecto) y "Si, o no estoy seguro". Con la segunda,
  una caja `--info` explica que tambien se rota la llave y que hara falta un codigo nuevo.
- **Acciones.** "Cancelar" y "Cambiar la frase" (o "Cambiar y rotar la llave" con la segunda opcion, que
  encadena K-02 y la rotacion de K-04).
- **Estados.** Validacion de K-01, guardando, exito (toast `ok`), 412 (cambio en otro equipo), 403
  `vault_key_check_failed` (este equipo tiene una llave antigua: pide desbloquear de nuevo), sin
  conexion.

| Clave         | ES                                                                                                                                                                             | EN                                                                                                                                                                                   |
| ------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| k6Title       | Cambiar la frase de sincronización                                                                                                                                             | Change sync passphrase                                                                                                                                                               |
| newPass       | Frase nueva                                                                                                                                                                    | New passphrase                                                                                                                                                                       |
| newPassRepeat | Repite la frase nueva                                                                                                                                                          | Repeat the new passphrase                                                                                                                                                            |
| k6Others      | Tus otros equipos siguen funcionando sin pedirte nada.                                                                                                                         | Your other devices keep working without asking you for anything.                                                                                                                     |
| k6Ask         | ¿Crees que alguien conoce tu frase anterior?                                                                                                                                   | Do you think someone knows your old passphrase?                                                                                                                                      |
| askNo         | No, solo quiero cambiarla                                                                                                                                                      | No, I just want to change it                                                                                                                                                         |
| askYes        | Sí, o no estoy seguro                                                                                                                                                          | Yes, or I'm not sure                                                                                                                                                                 |
| k6RotateInfo  | Cambiar la frase no basta si alguien conoce la anterior. También rotaremos la llave: te daremos un código de recuperación nuevo y tus otros equipos te pedirán la frase nueva. | Changing the passphrase isn't enough if someone knows the old one. We'll also rotate the key: you'll get a new recovery code and your other devices will ask for the new passphrase. |
| changeAndRot  | Cambiar y rotar la llave                                                                                                                                                       | Change and rotate key                                                                                                                                                                |
| passChanged   | Frase cambiada                                                                                                                                                                 | Passphrase changed                                                                                                                                                                   |
| oldKey        | Este equipo tiene una llave antigua. Desbloquea la bóveda otra vez.                                                                                                            | This device has an old key. Unlock the vault again.                                                                                                                                  |

### 7. K-07 Regenerar el codigo de recuperacion

- **Entrada.** Tarjeta Boveda > "Nuevo codigo de recuperacion". Requiere boveda desbloqueada.
- **Muestra.** Modal con caja `--warn` "El codigo anterior dejara de servir" y la misma pregunta de K-06
  sobre si alguien conoce el codigo anterior. Con "Si", pide la frase actual (la rotacion la necesita).
- **Acciones.** "Cancelar" y "Generar codigo nuevo" (principal, acento: no borra datos), que lleva a K-02
  en su variante "Tu nuevo codigo de recuperacion".
- **Estados.** Pregunta en No, pregunta en Si con campo de frase, frase incorrecta, generando, exito, 412.

| Clave      | ES                                            | EN                                            |
| ---------- | --------------------------------------------- | --------------------------------------------- |
| k7Title    | Generar un código de recuperación nuevo       | Generate a new recovery code                  |
| k7Warn     | El código anterior dejará de servir.          | The previous code will stop working.          |
| k7Ask      | ¿Crees que alguien conoce el código anterior? | Do you think someone knows the previous code? |
| k7NeedPass | Escribe tu frase para rotar la llave.         | Enter your passphrase to rotate the key.      |
| k7Btn      | Generar código nuevo                          | Generate new code                             |

### 8. K-08 Olvide la frase y el codigo

- **Entrada.** "Olvide la frase" en K-03 y en la tarjeta Boveda. Desktop: modal; VS Code: el panel de
  boveda del webview de ajustes.
- **Muestra.** Tres salidas en este orden, cada una como bloque con titulo y accion:
  1. Otro equipo desbloqueado: desde el se cambia la frase (K-06) sin perder nada. Indica la ruta en
     Desktop y en VS Code.
  2. El codigo de recuperacion: vuelve a K-03 en modo codigo.
  3. Ninguna de las dos: restablecer desde el panel web. Dice que se borran las copias cifradas de la
     nube, que nadie puede recuperarlas y que cada equipo conserva lo suyo. Boton "Abrir el panel web"
     (abre `https://git-rudder.com` en la seccion de la boveda del panel) con icono de enlace externo.
- **Estados.** Uno solo, en Desktop y en VS Code; sin conexion el boton web sigue activo (abre el
  navegador).

| Clave    | ES                                                                                                                                                                                                                                                                                         | EN                                                                                                                                                                                                                                                                              |
| -------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| k8Title  | ¿Olvidaste la frase?                                                                                                                                                                                                                                                                       | Forgot your passphrase?                                                                                                                                                                                                                                                         |
| k8Other  | ¿Tienes otro equipo con la bóveda desbloqueada?                                                                                                                                                                                                                                            | Do you have another device with the vault unlocked?                                                                                                                                                                                                                             |
| k8OtherD | Ábrelo y ve a Configuración › Cuenta y equipo › Bóveda › Cambiar la frase. En VS Code: Configuración de Git Rudder › Cuenta de Git Rudder › Bóveda. No perderás nada.                                                                                                                      | Open it and go to Settings › Account & team › Vault › Change passphrase. In VS Code: Git Rudder Settings › Git Rudder Account › Vault. You won't lose anything.                                                                                                                 |
| k8Code   | ¿Tienes el código de recuperación?                                                                                                                                                                                                                                                         | Do you have the recovery code?                                                                                                                                                                                                                                                  |
| k8Reset  | ¿No tienes ninguna de las dos cosas?                                                                                                                                                                                                                                                       | Have neither?                                                                                                                                                                                                                                                                   |
| k8ResetD | Puedes restablecer la bóveda desde el panel web. Se borran de la nube las copias cifradas de tus cuentas y llaves, y nadie puede recuperarlas, tampoco el soporte de Git Rudder. Lo que tienes en cada equipo se conserva, y el primer equipo que cree una bóveda nueva lo vuelve a subir. | You can reset the vault from the web dashboard. The encrypted copies of your accounts and keys are deleted from the cloud, and nobody can recover them, not even Git Rudder support. What each device has is kept, and the first device to create a new vault uploads it again. |
| openWeb  | Abrir el panel web                                                                                                                                                                                                                                                                         | Open the web dashboard                                                                                                                                                                                                                                                          |

### 9. K-09 Efecto de cerrar sesion sobre los secretos de este equipo

- **Regla (ADR 0012 y decision #57).** Cerrar sesion borra de este equipo la llave de la boveda y las
  copias de cuentas y llaves que llegaron de otros equipos. Lo creado en este equipo se queda, porque
  sin cuenta Git Rudder sigue siendo un cliente Git. Una casilla, apagada por defecto, borra tambien lo creado
  aqui. Lo mismo pasa sin pulsar nada cuando un refresco recibe un 401 con `code` `session_ended`
  (C-07); cualquier otro 401 no borra nada.
- **Desktop.** "Cerrar sesion" de la tarjeta de sesion abre un modal de peligro (460): titulo, texto,
  resumen con recuentos (lo que se borra y lo que se queda, sale de la marca de origen de cada secreto),
  la casilla y, si se marca, una linea `--warn` con la consecuencia. Variante sin boveda (no hay llave
  que borrar). Botones "Cancelar" (foco inicial) y "Cerrar sesion" (peligro).
- **VS Code.** Como lo especifica el brief #68: aviso modal nativo con el mismo texto y dos botones en
  lugar de la casilla ("Cerrar sesión" y "Cerrar sesión y borrar también las añadidas aquí"), tanto desde
  el boton del webview de ajustes como desde el menu de Cuentas de VS Code. Las demas ventanas sueltan la
  llave y ensenan un mensaje de barra de estado. Los textos de esos botones y del mensaje son los de #68
  (`signOutKeep`, `signOutWipe`, `otherWindowOut`).
- **Estados.** Con boveda, sin boveda, casilla marcada, cerrando, sin conexion (la sesion se cierra en el
  servidor al reconectar), exito (toast `info` "Sesion cerrada", sin "(simulado)": B-46).

| Clave          | ES                                                                                                                                                                                   | EN                                                                                                                                            |
| -------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ | --------------------------------------------------------------------------------------------------------------------------------------------- |
| k9Title        | ¿Cerrar sesión en este equipo?                                                                                                                                                       | Sign out on this device?                                                                                                                      |
| k9Body         | Al cerrar sesión se borran de este equipo la llave de tu bóveda y las cuentas y llaves que llegaron de otros equipos. Las que añadiste aquí se quedan para que Git siga funcionando. | Signing out removes your vault key and the accounts and keys that came from other devices. The ones you added here stay so Git keeps working. |
| k9BodyNoVault  | Las cuentas y llaves que añadiste aquí se quedan para que Git siga funcionando. Tus repositorios y ajustes locales no se tocan.                                                      | The accounts and keys you added here stay so Git keeps working. Your repositories and local settings aren't touched.                          |
| k9Removed      | Se borran: {accounts} cuentas y {keys} llaves de otros equipos                                                                                                                       | Removed: {accounts} accounts and {keys} keys from other devices                                                                               |
| k9Kept         | Se quedan: {accounts} cuentas y {keys} llaves añadidas aquí                                                                                                                          | Kept: {accounts} accounts and {keys} keys added here                                                                                          |
| k9Check        | Borrar también las añadidas en este equipo                                                                                                                                           | Also delete the ones added on this device                                                                                                     |
| k9CheckWarn    | Tendrás que volver a conectar esas cuentas para hacer fetch, push o pull requests en este equipo.                                                                                    | You'll need to reconnect those accounts to fetch, push or open pull requests on this device.                                                  |
| signOut        | Cerrar sesión                                                                                                                                                                        | Sign out                                                                                                                                      |
| signingOut     | Cerrando sesión…                                                                                                                                                                     | Signing out…                                                                                                                                  |
| signOutOffline | Sin conexión: la sesión se cerrará en el servidor cuando vuelvas a conectarte.                                                                                                       | Offline: the session will be ended on the server when you reconnect.                                                                          |
| signedOut      | Sesión cerrada                                                                                                                                                                       | Signed out                                                                                                                                    |

Nota: los recuentos `k9Removed` y `k9Kept` son dos parametros con plural cada uno; en implementacion
se parten en claves `_one` y `_other` por parametro. En el diseno basta con dibujar 3 y 1.

### 10. K-10 Web: estado de la boveda y reinicio

- **Donde.** Panel web, "Configuracion en la nube", tarjeta nueva "Boveda" despues de "Que se
  sincroniza". La web solo conoce `vaultEnabled` y `vaultCreatedAt`.
- **Estados de la tarjeta.**
  - Sin configurar: texto y enlace a descargas dentro del panel (B-58: sin salir del panel).
  - Configurada: "Configurada el {date}", explicacion de que esta web no puede leer los secretos, y
    boton de peligro "Restablecer la boveda...".
- **Flujo de reinicio.**
  1. Modal de peligro: que se borra (copias cifradas de cuentas y llaves; los datos de las cuentas se
     quedan), que nadie puede recuperarlas, que cada equipo conserva lo suyo, y antes otra salida: si un
     equipo sigue desbloqueado, cambiar la frase desde el. Campo de confirmacion literal y boton
     "Restablecer la boveda" (peligro), deshabilitado hasta que el texto coincida.
  2. Si la sesion no es reciente, reautenticacion fuerte (W-35, brief #69: contrasena, Google o enlace al
     correo; el OAuth de GitHub o GitLab no basta) y vuelta aqui.
  3. Exito: la tarjeta pasa a "Sin configurar" y toast `ok`.
- **Estados.** Sin configurar, configurada, modal con texto vacio, texto correcto, restableciendo,
  reautenticacion requerida, 429 (10 por hora), error generico (brief #62), exito.
- **Confirmacion literal.** ES "BORRAR SECRETOS", EN "DELETE SECRETS". La API recibe siempre
  `BORRAR SECRETOS`; la interfaz traduce.

| Clave         | ES                                                                                                                                                      | EN                                                                                                                                       |
| ------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------- |
| webVault      | Bóveda                                                                                                                                                  | Vault                                                                                                                                    |
| webVaultNone  | Sin configurar. Se crea en Git Rudder Desktop o VS Code la primera vez que sincronizas una cuenta o una llave.                                          | Not set up. It's created in Git Rudder Desktop or VS Code the first time you sync an account or a key.                                   |
| webVaultOn    | Configurada el {date}                                                                                                                                   | Set up on {date}                                                                                                                         |
| webVaultOnD   | Tus cuentas y llaves viajan cifradas con tu frase de sincronización. Esta web no puede leerlas.                                                         | Your accounts and keys travel encrypted with your sync passphrase. This website can't read them.                                         |
| resetBtnMenu  | Restablecer la bóveda…                                                                                                                                  | Reset vault…                                                                                                                             |
| resetTitle    | ¿Restablecer la bóveda?                                                                                                                                 | Reset the vault?                                                                                                                         |
| resetWhat     | Se borran de la nube las copias cifradas de tus cuentas vinculadas y llaves de IA. Los datos de las cuentas (plataforma, usuario y servidor) se quedan. | The encrypted copies of your linked accounts and AI keys are deleted from the cloud. Account details (platform, username and host) stay. |
| resetNoOne    | Nadie puede recuperar esas copias, tampoco el soporte de Git Rudder.                                                                                    | Nobody can recover those copies, not even Git Rudder support.                                                                            |
| resetKept     | Cada equipo conserva sus propias credenciales; el primero que cree una bóveda nueva vuelve a subir las suyas.                                           | Each device keeps its own credentials; the first one to create a new vault uploads its own again.                                        |
| resetTryFirst | Antes, prueba otra salida: si un equipo sigue desbloqueado, cambia la frase desde él y no perderás nada.                                                | First, try another way: if a device is still unlocked, change the passphrase there and you won't lose anything.                          |
| resetType     | Escribe {phrase} para confirmar                                                                                                                         | Type {phrase} to confirm                                                                                                                 |
| resetBtn      | Restablecer la bóveda                                                                                                                                   | Reset vault                                                                                                                              |
| resetting     | Restableciendo…                                                                                                                                         | Resetting…                                                                                                                               |
| resetDone     | Bóveda restablecida. Te enviamos un correo de aviso.                                                                                                    | Vault reset. We sent you a notification email.                                                                                           |
| resetLimit    | Demasiados intentos. Vuelve a intentarlo en una hora.                                                                                                   | Too many attempts. Try again in an hour.                                                                                                 |

### 11. X-03 Textos sobre cifrado que tienen que seguir siendo verdad

Donde se dibujen estas superficies (algunas son de otros briefs), el texto es este:

| Donde                                                       | ES                                                                                                                                                                                                                               | EN                                                                                                                                                                                                                         |
| ----------------------------------------------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Fila de llave de la organizacion (W-16; Desktop IA, #67)    | Cifradas en los servidores de Git Rudder; Git Rudder las usa para generar en nombre de tu organización.                                                                                                                          | Encrypted on Git Rudder's servers; Git Rudder uses them to generate on behalf of your organization.                                                                                                                        |
| Primera generacion con llave de la organizacion             | Con una llave de la organización, los cambios preparados pasan por los servidores de Git Rudder para generar el texto. Con una llave personal, no.                                                                               | With an organization key, your staged changes go through Git Rudder's servers to generate the text. With a personal key, they don't.                                                                                       |
| Aviso en la fila de llave de la organizacion                | Usa los servidores de Git Rudder                                                                                                                                                                                                 | Uses Git Rudder's servers                                                                                                                                                                                                  |
| Web, "Cuentas vinculadas" (`W.sAccD`, sin cambios)          | Tokens cifrados de extremo a extremo                                                                                                                                                                                             | End-to-end encrypted tokens                                                                                                                                                                                                |
| Web, "Llaves de IA" activado                                | Cifradas de extremo a extremo con tu frase de sincronización                                                                                                                                                                     | End-to-end encrypted with your sync passphrase                                                                                                                                                                             |
| Web, "Llaves de IA" desactivado (`W.sAiKeysD`)              | Desactivado: cada dispositivo guarda sus propias llaves                                                                                                                                                                          | Off: each device keeps its own keys                                                                                                                                                                                        |
| VS Code, ajuste de nube (sustituye a `syncD`, decision #58) | Usar la configuración de mi cuenta de Git Rudder. Este VS Code usa tus cuentas, tus preferencias de Git e IA y, si lo activaste en tu cuenta, tus llaves de IA. El tema, el idioma y el resto del editor los sincroniza VS Code. | Use my Git Rudder account settings. This VS Code uses your accounts, your Git and AI preferences and, if you turned it on in your account, your AI keys. Theme, language and the rest of the editor are synced by VS Code. |
| Cerrar sesion en otro dispositivo (W-22)                    | Ese dispositivo pierde el acceso a tu cuenta y borrará lo que llegó de la nube la próxima vez que se conecte. Si nunca vuelve a conectarse, conserva lo que ya tenía: revoca también sus tokens en cada plataforma.              | That device loses access to your account and will delete what came from the cloud the next time it connects. If it never reconnects, it keeps what it had: revoke its tokens on each platform too.                         |
| Ayuda general de la boveda                                  | Nadie puede recuperar tus secretos si pierdes la frase y el código de recuperación, tampoco el soporte de Git Rudder.                                                                                                            | Nobody can recover your secrets if you lose the passphrase and the recovery code, not even Git Rudder support.                                                                                                             |

### 12. X-04 Interruptores de sincronizacion de secretos con el estado de la boveda

- **Donde.** Web "Configuracion en la nube" > "Que se sincroniza" (filas "Cuentas vinculadas" y "Llaves
  de IA"). Desktop tendra los mismos interruptores en Ajustes > Cuenta (D-51, brief #67), con estos
  mismos estados; los textos de la variante Desktop son los de #67.
- **Estados de cada fila** (la descripcion bajo la etiqueta cambia; el toggle guarda al instante, B-57):
  - Apagado (llaves de IA por defecto): descripcion de apagado.
  - Encendido con boveda: descripcion E2E mas "Boveda configurada el {date}".
  - Encendido sin boveda: linea `--warn` "Requiere tu frase...". En la web no hay boton (no puede crear la
    boveda); en Desktop, boton "Configurar" que abre K-01.
  - Encendido con la boveda bloqueada en este equipo (solo Desktop): "Boveda bloqueada en este equipo" y
    "Desbloquear".
  - Al apagar: modal de peligro. Desactivar borra de la nube los metadatos y las copias cifradas; los
    equipos conservan su copia local y la cuenta elegida por repositorio.

| Clave         | ES                                                                                                                                           | EN                                                                                                                                    |
| ------------- | -------------------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------- |
| accOffD       | Desactivado: cada dispositivo guarda sus propias cuentas                                                                                     | Off: each device keeps its own accounts                                                                                               |
| needsVault    | Requiere tu frase de sincronización; se configura en Git Rudder Desktop o VS Code. Mientras tanto se sincronizan las cuentas sin sus tokens. | Requires your sync passphrase, set up in Git Rudder Desktop or VS Code. Until then, accounts sync without their tokens.               |
| needsVaultApp | Requiere tu frase de sincronización.                                                                                                         | Requires your sync passphrase.                                                                                                        |
| configure     | Configurar                                                                                                                                   | Set up                                                                                                                                |
| vaultSince    | Bóveda configurada el {date}                                                                                                                 | Vault set up on {date}                                                                                                                |
| offAccTitle   | ¿Dejar de sincronizar las cuentas vinculadas?                                                                                                | Stop syncing linked accounts?                                                                                                         |
| offKeysTitle  | ¿Dejar de sincronizar las llaves de IA?                                                                                                      | Stop syncing AI keys?                                                                                                                 |
| offBody       | Se borran de la nube sus datos y sus copias cifradas. Cada equipo conserva su copia local y la cuenta que usa cada repositorio.              | Their details and encrypted copies are deleted from the cloud. Each device keeps its local copy and the account each repository uses. |
| offBtn        | Dejar de sincronizar                                                                                                                         | Stop syncing                                                                                                                          |

## Reglas

- **Nadie puede recuperar los secretos**, tampoco el soporte de Git Rudder. Lo dicen K-01, K-02, K-08, K-10 y
  la ayuda (X-03). Ningun texto promete recuperacion, borrado remoto ni "Git Rudder puede ayudarte".
- La boveda se crea cuando hace falta, nunca al iniciar sesion. Crear y desbloquear solo en Desktop y
  VS Code; restablecer solo en la web.
- El codigo se ensena y se confirma antes de subir nada, y nunca se regenera en silencio (reintentar usa
  el mismo).
- No hay cuentas de plataforma compartidas con el equipo (X-01): ningun chip "Equipo" ni opcion
  "Compartir con el equipo" en estas pantallas. Compartir workspaces si sigue.
- La web no anade, no prueba ni edita tokens (X-02).
- Campos de secreto ocultos, con mostrar u ocultar solo mientras se escribe; un valor guardado nunca se
  vuelve a ensenar (B-37). Mascaras calculadas fuera de la UI (B-34).
- Acciones destructivas (cerrar sesion, desactivar la sincronizacion de secretos, restablecer) con modal
  de peligro, boton `--danger` y foco inicial en "Cancelar". Generar codigo o cambiar la frase no borra
  datos: boton de acento.
- Toasts informativos en `--info`, nunca en `--accent` (B-61). Nada de "(simulado)" (B-46).
- Sin ubicacion en ninguna lista de sesiones o dispositivos (decision #53).
- Los enlaces a la web usan `https://git-rudder.com`; los correos de ejemplo, `example.com`.
- Plurales con claves `_one` y `_other` sobre `{count}`; marcadores con nombre (ADR 0017).

## Entregable

- Un zip `git-rudder-handoff-63-boveda.zip` con la estructura del handoff original:
  - `README.md` solo con lo nuevo: flujos, componentes nuevos (campo de secreto, medidor de fortaleza,
    caja del codigo, tarjeta Boveda, chip "Solo en este equipo") y sus medidas.
  - `Git Rudder Boveda.dc.html`: un Design Component con props `product` (`desktop`, `vscode`, `web`),
    `theme`, `lang` y `screen` (`K-01` a `K-10`, `X-04`). Agrupacion: el alta (K-01 y K-02) como un solo
    flujo navegable; K-03 y K-08 juntos; K-04 y K-05 dentro de las listas reales de Plataformas Git e IA
    con datos de muestra mezclados (sincronizada bloqueada, solo en este equipo, sin token, credenciales
    que no coinciden); K-06, K-07 y la rotacion desde la tarjeta Boveda; K-09 desde la tarjeta de sesion;
    K-10 y X-04 en la pantalla "Configuracion en la nube" de la web.
  - `git-rudder-data.js` con un objeto nuevo `VAULT` de pares `[es, en]` con las claves de este brief.
  - `support.js` sin cambios.

## Hecho cuando

- [ ] K-01 a K-09 en escritorio y VS Code (K-08 enlaza al reinicio web) y K-10 en la web
- [ ] Los textos dicen que nadie puede recuperar los secretos
- [ ] Los interruptores de secretos muestran el estado de la bóveda
