# ADR 0012: Boveda de secretos de extremo a extremo

- Estado: aceptado
- Fecha: 2026-09-30
- Ratificado: 2026-10-01, el cierre de sesion, con una casilla para borrar tambien lo creado en el
  equipo y reglas que faltaban (ver [Cambios del 2026-10-01](#cambios-del-2026-10-01))

## Contexto

Un usuario de Git Rudder conecta varias cuentas de GitHub, GitLab, Bitbucket, Gitea o Azure DevOps, y
cada una tiene un token que da acceso a su codigo. Quiere que esas cuentas aparezcan en todos sus
equipos sin volver a crear los tokens en cada uno. Lo mismo, si lo activa, con sus llaves de IA
personales.

El diseno web prometia "Tokens cifrados de extremo a extremo", pero en la misma pantalla dejaba
compartir una cuenta con el equipo y "probar conexion" desde la web, y las dos cosas necesitan que
el servidor pueda descifrar. El analisis tecnico propuso cifrado de sobre en el servidor y cambiar
el texto a "cifrados en reposo". **El usuario decidio lo contrario: extremo a extremo de verdad.
El servidor no puede descifrar nunca estos secretos.** Y acepto a sabiendas lo que eso cuesta:

- Desaparecen las cuentas de plataforma compartidas con el equipo.
- El panel web no puede probar una conexion: solo ve metadatos y puede quitar una cuenta. Anadir
  o editar un token se hace en Desktop o en VS Code.
- Las llaves de IA personales, si el usuario activa su sincronizacion (apagada por defecto), usan
  la misma boveda.
- Si el usuario olvida la frase de sincronizacion y el codigo de recuperacion, pierde las copias
  de la nube. Los tokens se pueden volver a crear en cada plataforma.

Este ADR fija el esquema. El contrato de los endpoints esta en
[api-surface.md](../api-surface.md) y las tablas en [data-model.md](../data-model.md).

## Decision

Toda la criptografia ocurre en el motor de Rust del dispositivo (`git-rudder-vault`, planificado), que
es el mismo en Desktop y en el sidecar de VS Code. La web no cifra ni descifra nada. El servidor
guarda bytes que no entiende.

### Las llaves

**Llave de la boveda (VK).** 256 bits aleatorios por usuario, generados en el primer dispositivo
que activa la boveda. Nunca sale de sus dispositivos en claro.

**Subllaves.** La VK no se usa directamente ni para cifrar ni para el comprobante: de ella salen
dos subllaves con HKDF-SHA256 (sin sal), `K_seal = HKDF(VK, "git-rudder:vault:v1:seal")`, que sella los
secretos, y `K_check = HKDF(VK, "git-rudder:vault:v1:check")`, que calcula el comprobante. Usar la misma
llave para un AEAD y para un HMAC no rompe nada conocido, pero separar los usos cuesta dos lineas
y quita la pregunta. Lo que se envuelve con la frase y con el codigo es la VK.

**Envoltura por frase de sincronizacion.**
`KEK_p = Argon2id(frase normalizada en NFKC, sal de 16 bytes)`, con 64 MiB de memoria, 3 pasadas
y 4 carriles por defecto.

- Los parametros se guardan junto a la envoltura, para poder subirlos mas adelante sin romper las
  bovedas existentes: se piden parametros nuevos la proxima vez que el usuario cambie la frase.
- El servidor rechaza parametros por debajo de 64 MiB y 3 pasadas, y la tabla lo impone con un
  `CHECK`. El cliente tampoco envuelve nunca por debajo del minimo que lleva compilado, diga lo
  que diga el servidor.
- El cliente exige una frase de al menos 12 caracteres y una puntuacion minima de fortaleza. El
  servidor no ve la frase y no puede exigir nada.

**Envoltura por codigo de recuperacion.** Un codigo de 160 bits aleatorios que el usuario ve una
sola vez, como 32 caracteres en base32 de Crockford en grupos de cuatro. Como ya tiene entropia
completa, basta `KEK_r = HKDF-SHA256(codigo, sal, "git-rudder:vault:v1:recovery")`: Argon2id no anade
nada a 160 bits que no se pueden adivinar.

**Envolver la VK.** XChaCha20-Poly1305 con la KEK y un nonce aleatorio de 24 bytes, con datos
asociados `git-rudder:vault:v1:wrap:{userId}:{tipo}:{keyVersion}`, donde el tipo es `passphrase` o
`recovery`. Una envoltura copiada a otro usuario o a otra version de la llave no abre. La llave
envuelta ocupa 48 bytes: 32 de la llave y 16 de la etiqueta.

**Sellar un secreto.** XChaCha20-Poly1305 con `K_seal` y un nonce aleatorio de 24 bytes, con
datos asociados `git-rudder:vault:v1:{userId}:{kind}:{recordId}:{keyVersion}`, donde `kind` es
`platform_credential` o `ai_key`. El `recordId` es el UUID de la cuenta o de la llave, y lo genera
el cliente antes de hablar con la API precisamente para poder calcular estos datos asociados. Un
blob copiado a otra fila, a otro usuario, a otra tabla o a otra version de la llave no descifra.

**El secreto viaja con su destino dentro.** El contenido sellado es un JSON versionado que el
servidor no lee, y lleva, ademas del secreto, los metadatos que deciden a donde se envia:

- Cuenta de plataforma:
  `{ "v": 1, "platform", "host", "username", "method", "accessToken", "refreshToken", "expiresAt", "tokenType" }`.
- Llave de IA personal: `{ "v": 1, "provider", "baseUrl", "apiKey" }`.

Esos metadatos tambien estan en claro en la fila, y el servidor los controla. Si solo estuvieran
ahi, un servidor malicioso o una sesion robada podria cambiar el `baseUrl` de una llave, o
guardar un blob antiguo de GitHub en una cuenta nueva con el mismo id y `host` de un atacante, y
cada dispositivo enviaria el secreto descifrado a donde dijera la fila. Por eso, despues de abrir
un secreto, el motor compara la copia sellada con la fila: **si no coinciden, no usa el secreto**
y la interfaz ensena "Las credenciales no coinciden con la cuenta". La copia sellada manda. Dos
reglas de la API cierran el resto: el `baseUrl` de una llave con secreto solo cambia en la misma
peticion que sube un secreto nuevo, y el id de un secreto borrado no se puede reutilizar (409
`id_conflict`).

Una llave de Ollama no tiene secreto y por tanto nada sellado. Para ella el motor solo acepta de la
nube una URL base en loopback (`localhost`, `127.0.0.1` o `[::1]`); cualquier otra la tiene que
confirmar el usuario en ese equipo antes de enviar nada, porque lo que viaja es su codigo.

**Comprobante de la llave.**
`keyCheck = HMAC-SHA256(K_check, "git-rudder:vault:key-check:v1:{userId}:{keyVersion}")`. El dispositivo
lo manda al crear la boveda y en cada cambio de envoltura o rotacion; el servidor **guarda solo su
SHA-256** y compara hashes en tiempo constante. Solo quien tiene la VK puede cambiar como se
envuelve: sin esto, una sesion robada podria sustituir las envolturas por basura y dejar al
usuario sin acceso a sus secretos. Guardar el hash y no el comprobante evita que quien lea la
tabla obtenga una prueba reutilizable para destruir la boveda con cualquier sesion. El
comprobante no ayuda a sacar la VK: es la salida de un HMAC sobre una subllave.

**Por que XChaCha20-Poly1305 y no AES-256-GCM.** Muchos dispositivos cifran bajo la misma `K_seal`
con nonces aleatorios. Con 96 bits de nonce habria que llevar la cuenta de cuantos mensajes cifra la
llave para no arriesgar una colision; con 192 bits no hace falta. La web no necesita soportarlo
porque no cifra. En el servidor, donde cada secreto tiene su propia llave de datos, AES-256-GCM es
suficiente ([ADR 0013](0013-proxy-de-ia-de-organizacion.md)).

**Por que una frase aparte y no la contrasena de login.** Dos motivos, y cada uno basta solo: el
servidor ve la contrasena en claro cada vez que alguien inicia sesion, asi que una llave derivada
de ella no seria de extremo a extremo; y las cuentas creadas con GitHub, GitLab o Google no tienen
contrasena.

### Donde vive cada cosa

| Dato                           | Dispositivo                                                                                         | Servidor                                                                                                    |
| ------------------------------ | --------------------------------------------------------------------------------------------------- | ----------------------------------------------------------------------------------------------------------- |
| Frase de sincronizacion        | Nunca se guarda: se pide una vez por dispositivo                                                    | Nunca                                                                                                       |
| Codigo de recuperacion         | Nunca se guarda: se ensena una vez                                                                  | Nunca                                                                                                       |
| KEK de frase y de recuperacion | Solo en memoria, limpiada con `zeroize`                                                             | Nunca                                                                                                       |
| VK desbloqueada                | Llavero del sistema (Desktop) o `SecretStorage` (VS Code), entrada `git-rudder/vault-key/<user_id>` | Nunca                                                                                                       |
| Envolturas, sales y parametros | Se descargan al desbloquear                                                                         | Tabla `vaults`                                                                                              |
| Comprobante de la llave        | Se calcula con la VK cuando hace falta                                                              | Solo su SHA-256, en `vaults.key_check_hash`                                                                 |
| Secretos sellados              | Copia local en claro en el llavero, para usarlos                                                    | Columnas `secret_*` de `linked_accounts` y `ai_keys`                                                        |
| Metadatos de cuentas y llaves  | Si, y una copia sellada junto al secreto                                                            | En claro: plataforma, host, usuario, etiqueta, scopes, mascara (calculada en el dispositivo antes de subir) |

### Operaciones

- **Configurar.** El dispositivo genera la VK y el codigo de recuperacion, ensena el codigo y
  exige que el usuario confirme que lo guardo **antes** de subir nada. Luego sube las dos
  envolturas y el comprobante.
- **Desbloquear en un dispositivo nuevo.** Descarga la cabecera de la boveda, deriva la KEK con la
  frase (o con el codigo de recuperacion), desenvuelve la VK, la guarda en el llavero y descarga
  los secretos. La primera descarga de la cabecera en cada dispositivo deja un evento de auditoria
  y manda el correo "Un equipo nuevo descargo tu boveda".
- **Cambiar la frase.** Solo reenvuelve la VK, con el comprobante y `If-Match`. Los demas
  dispositivos siguen funcionando sin pedir nada.
- **Regenerar el codigo de recuperacion.** Igual: una envoltura nueva.
- **Lo que cambiar la frase o el codigo no hace.** La envoltura anterior deja de valer ante el
  servidor, pero la VK es la misma: quien guardo una envoltura vieja (una copia de seguridad, una
  respuesta antigua de `GET /v1/vault`) la sigue abriendo con la frase o el codigo viejos. Por eso
  las dos pantallas preguntan si el usuario cree que alguien los conoce y, si dice que si, encadenan
  una rotacion de la VK.
- **Rotar la VK**, cuando se sospecha de un dispositivo que la tenia en cache o de una frase
  filtrada. El dispositivo pide la frase (la nueva envoltura la necesita), genera una VK nueva y
  **un codigo de recuperacion nuevo**, que ensena con la misma confirmacion que al configurar,
  porque ningun dispositivo guarda el codigo y no hay otra forma de envolver la VK nueva para la
  recuperacion. Despues baja todos los secretos, los descifra con la VK vieja, los sella con la
  nueva y lo sube todo en una peticion. La API lo aplica en una sola transaccion, o nada, y
  responde 409 si la lista no es exactamente el conjunto de secretos actual: rotar dejando uno con
  la llave vieja lo haria ilegible para siempre. Las escrituras de secretos toman la fila de la
  boveda con `FOR SHARE` y la rotacion con `FOR UPDATE`, para que un `PUT` que leyo la version
  vieja no escriba despues de la rotacion. Al terminar, los demas dispositivos tienen que volver a
  introducir la frase.
- **Revocar un dispositivo recomienda rotar.** Cuando se revoca una sesion de un dispositivo que
  llego a descargar la boveda, la API marca la boveda con "rotacion recomendada" y todos los
  dispositivos ensenan "Rotar la llave de la boveda" hasta que alguno rote. El revocado sigue
  teniendo la VK en su llavero; sin rotar, cualquier blob que se suba despues tambien lo podria
  abrir quien tenga ese equipo y una copia de la base.
- **Reiniciar**, cuando se perdieron frase y codigo. Desde la web, con reautenticacion fuerte
  (contrasena, Google con `max_age=0` o un enlace al correo; el OAuth de GitHub o GitLab no basta
  porque puede completarse sin pedir nada) y escribiendo una confirmacion literal: borra la boveda
  y todos los blobs, deja los metadatos y manda un correo de aviso. Es de la web porque es el
  camino de quien ya no tiene la VK y no puede presentar su comprobante; la reautenticacion es lo
  que impide que una sesion robada lo use para destruir.
- **Cerrar sesion** borra del dispositivo la VK en cache y las copias de los secretos (tokens de
  plataforma y llaves de IA personales) que llegaron sincronizados de otros equipos. Lo que el
  usuario creo alli se queda en su llavero: sin cuenta, Git Rudder sigue siendo un cliente Git que
  necesita esas credenciales ([ADR 0014](0014-usar-git-rudder-sin-cuenta.md)). Para un ordenador
  compartido, el dialogo ofrece una casilla **apagada por defecto**, "Borrar también las añadidas
  en este equipo", que borra tambien lo creado alli. El texto propuesto del dialogo es: "Al cerrar
  sesión se borran de este equipo la llave de tu bóveda y las cuentas y llaves que llegaron de
  otros equipos. Las que añadiste aquí se quedan para que Git siga funcionando." Lo sostienen
  estas reglas:
  - **Marca local de origen por secreto.** Para saber que se creo en este equipo, el motor guarda
    junto a cada secreto local si se creo aqui o llego sincronizado, y para que usuario de Git Rudder.
    La marca es del equipo y no viaja a la nube.
  - **Solo un codigo concreto borra sin preguntar.** El motor hace la misma limpieza cuando un
    refresco recibe un 401 con el `code` `session_ended` de la API (sesion revocada, caducada o
    reutilizacion detectada), sin esperar a que el usuario pulse nada: un dispositivo honesto
    revocado desde otro equipo no debe conservar las copias sincronizadas. Un 401 sin ese `code`
    (un proxy, un fallo de configuracion del servidor, un token que la API no reconoce) no borra
    nada: la app pide volver a iniciar sesion y conserva las copias. Si cualquier 401 borrara, un
    fallo del servidor dejaria sin copias a todos los equipos a la vez.
  - **Las demas ventanas de VS Code sueltan la llave.** Cada ventana tiene su motor y todas
    comparten `SecretStorage`. Al cerrar sesion en una, las demas se enteran por
    `SecretStorage.onDidChange` y sueltan la VK de memoria.
  - **Un secreto local queda ligado a quien lo creo** (un usuario de Git Rudder o "sin cuenta"). Si
    despues entra otra cuenta de Git Rudder en el mismo equipo, el motor no lo sube solo a la nube de
    esa otra cuenta. Si se le pregunta o no se le ofrece nunca esta sin decidir.
  - **Un token que se queda puede haber caducado.** Si una cuenta creada aqui refresco su token en
    otro equipo (GitLab y Bitbucket rotan el refresh token), la copia local puede no valer. La
    interfaz pide "Reconectar"; no falla en silencio.

### Los tokens no pasan por el servidor al crearse

El motor obtiene el token de cada plataforma con device flow o con PKCE de cliente publico, asi
que va de la plataforma al dispositivo directamente. Solo donde el proveedor exige un secreto de
cliente (Bitbucket) la API hace el canje del codigo y devuelve el token en la respuesta sin
guardarlo ni registrarlo ([ADR 0018](0018-cuentas-de-plataformas-git.md)). En los dos casos el
token entra al llavero y, si la sincronizacion de cuentas esta activa y hay boveda, se sella y se
sube.

### Llaves de IA personales

Solo existen en la nube si el usuario activa esa categoria de sincronizacion, que viene apagada:
"cada dispositivo guarda sus propias llaves". Si la activa, usan esta misma boveda con `kind`
`ai_key`. Una llave de Ollama no tiene secreto.

## Consecuencias

A favor:

- Quien robe la base de datos, o quien controle el servidor, no puede leer ningun token de
  plataforma ni ninguna llave de IA personal.
- Un login robado tampoco basta para leer: sin la frase no se abre nada. Pero no es inocuo:
  **cualquier sesion nativa recibe las envolturas**, y con ellas puede probar frases sin conexion
  igual que quien robe la base. Por eso cada dispositivo que descarga la boveda por primera vez
  genera un correo y un evento de auditoria, y `GET /v1/vault` tiene un limite de 20 al dia por
  usuario.
- Los usuarios que entran solo con OAuth tienen boveda igual que los demas.
- Cambiar la frase no molesta a los demas dispositivos.
- El esquema esta escrito como contrato entre clientes: cualquier implementacion que siga los
  datos asociados y el formato abre lo que selle otra.

En contra, y es lo que el usuario acepto:

- **No hay cuentas de plataforma compartidas con el equipo.** Compartir exigiria envolver la llave
  para cada miembro, y el token dejaria de ser el secreto de una persona.
- **El panel web no puede probar, anadir ni editar un token.** Solo ve metadatos y puede quitar
  la cuenta, y avisa de que el token sigue valido en la plataforma hasta que se revoque alli.
- **Olvidar frase y codigo pierde las copias de la nube.** Nadie, ni el soporte de Git Rudder, puede
  recuperarlas. Cada dispositivo conserva su copia local, y el primero que cree una boveda nueva
  vuelve a subir las suyas.
- **Un ataque sin conexion con una copia de la base, o con las envolturas que recibe cualquier
  sesion nativa, puede probar frases debiles.** La defensa es el coste de Argon2id y la longitud
  minima de la frase. Subir los parametros por defecto es
  barato, porque cada boveda guarda los suyos.
- **Extremo a extremo protege la confidencialidad, no la disponibilidad ni la frescura.** Un
  servidor malicioso no puede leer ni fabricar un blob valido, pero si puede borrarlo o servir una
  version antigua de ese mismo registro, y el cliente no lo detecta.
- **Revocar un dispositivo no borra lo que ya descifro.** Un dispositivo robado tiene en su llavero
  los tokens abiertos. La interfaz debe recomendar rotar los tokens de las plataformas que usaba;
  prometer un borrado remoto seria mentir.
- **Los metadatos van en claro**: hosts internos, usuarios, nombres de cuentas. Es lo que el
  usuario acepta al sincronizar, y el panel de sincronizacion tiene que decirlo con esas palabras.
- **En Linux sin llavero, `SecretStorage` guarda en texto casi plano**, VK incluida. Se documenta y
  se recomienda configurar un llavero.
- Una frase mas que el usuario tiene que recordar, y un codigo que tiene que guardar. Es friccion
  real en la configuracion.

## Alternativas descartadas

**Cifrado de sobre en el servidor.** Era la recomendacion del analisis: permite compartir con el
equipo y probar desde la web. El usuario la descarto porque el servidor puede descifrar, que es
justo lo que no quiere prometer.

**Derivar la llave de la contrasena de login.** El servidor ve esa contrasena en cada login, y las
cuentas de OAuth no tienen.

**Emparejar dispositivos sin frase** (el nuevo recibe la VK de uno existente por un codigo QR o un
canal local). Mejor experiencia mientras quede un dispositivo, pero perderlos todos pierde lo
mismo, y sigue haciendo falta un camino de recuperacion. Puede anadirse mas adelante como segunda
forma de desbloquear sin cambiar el esquema.

**Un valor secreto del servidor mezclado en la derivacion** (la llamada pimienta, guardada
cifrada con su llave maestra). Protege frente a un robo de solo la base, no frente a un
servidor comprometido. Queda como mejora posible despues de F2.

**PAKE (OPAQUE) o la extension PRF de las passkeys.** Mas elegantes, pero con soporte desigual en
los tres sistemas y en apps que no son navegadores, y con mas piezas criptograficas propias.

**AES-256-GCM en el cliente.** Ver arriba: nonces de 96 bits con muchos dispositivos bajo la misma
llave.

**Compartir cuentas con el equipo envolviendo la VK por miembro.** Descartado por la decision del
usuario, no pendiente.

## Cambios del 2026-10-01

Se ratifica el cierre de sesion de la seccion Operaciones, con estos cambios:

- El texto decia "las copias sincronizadas de los tokens"; son los tokens **y las llaves de IA
  personales**, como ya decian api-surface.md y gaps.md.
- Se anade la casilla "Borrar también las añadidas en este equipo", apagada por defecto.
- Se escriben las reglas que faltaban: la marca local de origen por secreto, el `code`
  `session_ended` como unica senal para borrar al refrescar, las demas ventanas de VS Code que
  sueltan la llave, el secreto ligado a quien lo creo y "Reconectar" para un token que ya no vale.
