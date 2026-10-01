# ADR 0008: Access token JWT corto, refresh token opaco rotatorio y PKCE para las apps nativas

- Estado: aceptado
- Fecha: 2026-09-30
- Revisado: 2026-10-01. El `iss` usa el dominio real
  ([ADR 0023](0023-despliegue-en-apps-01-y-dominio.md)); la pagina del device flow no ensena
  ubicacion en la primera version; y un refresco fallido lleva un `code` concreto cuando la sesion
  termino, que es lo unico que hace borrar las copias sincronizadas
  ([ADR 0012](0012-boveda-de-extremo-a-extremo.md))

## Contexto

Tres productos comparten una cuenta de Ramal, y cada uno inicia sesion en un entorno distinto:

- **Ramal Web** es un navegador. Registro, login con contrasena y OAuth de identidad con GitHub,
  GitLab y Google, recuperacion de contrasena y el panel.
- **Ramal Desktop** es una app de Tauri. Su webview no debe ver ningun token: las llamadas a la
  API las hace el motor de Rust ([ADR 0010](0010-un-motor-para-escritorio-y-vscode.md)).
- **Ramal for VS Code** es una extension. El motor corre en un sidecar, los secretos viven en
  `SecretStorage` y cada ventana de VS Code lanza su propio motor, todos compartiendo el mismo
  almacen.

Lo que el diseno y el contrato imponen:

- La sesion sobrevive a cerrar la app. En las apps nativas, meses.
- "Sesiones activas" con dispositivo, "Este dispositivo" y un boton para cerrar cada una; en la
  web, "Cerrar sesion en este dispositivo" para cada equipo sincronizado.
- Revocar un dispositivo tiene que cortar ya, no dentro de un rato, el acceso a lo que entrega
  secretos cifrados.
- Ramal funciona sin cuenta ([ADR 0014](0014-usar-ramal-sin-cuenta.md)): iniciar sesion es algo
  que se hace para sincronizar, nunca una puerta de entrada al producto.
- La API no tiene mas estado compartido que PostgreSQL. No hay Redis.
- La arquitectura hexagonal exige que el nucleo no sepa que existen los JWT ni las cookies.

Las opciones eran un proveedor de identidad externo (Auth0, Clerk, Keycloak, Zitadel) o que la
propia API fuera el servidor de autorizacion; y dentro de lo segundo, sesiones de servidor con
cookie, un solo token de larga vida, o el par access mas refresh. Gastonomo eligio el par; Ramal
lo hereda y lo amplia en tres sitios: firma asimetrica, cookies en la web e inicio de sesion
nativo a traves del navegador.

## Decision

La API de Ramal es el servidor de autorizacion, con endpoints que siguen la forma de OAuth 2.0
para que un proveedor externo pudiera sustituirla mas adelante sin rehacer los clientes. Todos
los endpoints estan en [api-surface.md](../api-surface.md); aqui va el porque.

### El access token es un JWT Ed25519 de 15 minutos

- Lleva `iss` (`https://api.ramal.bahoque.com`), `aud` (`ramal-api`), `sub` (usuario), `sid`
  (sesion), `cli` (`web`, `desktop` o `vscode`), `auth_time` (ultimo inicio de sesion real), `iat`
  y `exp`. No lleva roles: los roles son por organizacion, cambian, y se leen de la base en cada
  peticion que los necesita. `iss` y `aud` se comprueban desde el primer dia: cuando haya un
  segundo servicio que valide tokens, un token emitido para otro destino no le servira.
- Se firma con Ed25519 y lleva `kid` y `typ: at+jwt` en la cabecera. El verificador solo acepta
  `alg: EdDSA` con un `kid` que conozca y `typ: at+jwt`; cualquier otra combinacion es 401, sin
  intentar otro algoritmo. Las llaves llegan por configuracion
  (`RAMAL__AUTH__SIGNING_KEYS`, un mapa de `kid` a llave privada, y
  `RAMAL__AUTH__ACTIVE_SIGNING_KEY`). Firma la activa y verifican todas: rotar es anadir una
  llave nueva, activarla y retirar la vieja pasados 15 minutos, sin cerrar ninguna sesion.
- **Por que Ed25519 y no HS256 como Gastonomo.** El ADR 0008 de Gastonomo lo dejaba escrito: con
  un secreto simetrico, cualquier proceso que valide tokens puede firmarlos, y el dia que haya
  mas de un servicio toca cambiar. En Ramal ese dia se ve venir (el gateway o un servicio de
  publicaciones podrian validar), y cambiar el algoritmo con clientes que viven meses en el
  campo es mas caro que empezar bien. Ed25519 da firmas cortas, verificacion rapida y ninguna
  dependencia de RSA.
- **Implementacion.** `jsonwebtoken` 11 sin ninguno de sus backends criptograficos: el de
  `rust_crypto` arrastra `rsa`, que tiene RUSTSEC-2023-0071 sin parche y tumbaria `cargo deny`,
  y el de `aws_lc_rs` necesita cmake y un compilador de C. El proveedor Ed25519 se escribe con
  `ed25519-dalek`, igual que Gastonomo escribio su HS256 con `hmac` y `sha2`, y se anade a la
  lista de dependencias permitidas de `ramal-api-infrastructure`.
- **Validarlo no toca PostgreSQL.** Es lo que permite que una peticion protegida no cueste una
  consulta extra.
- **Salvo en los endpoints marcados como vivos.** Los que entregan secretos cifrados, el feed de
  cambios y el proxy de IA comprueban ademas que la sesion `sid` no este revocada, con una
  lectura por clave primaria. Es el compromiso: revocacion instantanea donde importa y ninguna
  consulta en el resto.
- Lo valida un middleware de `ramal-api-http` que deja en el `Depot` el usuario, la sesion y el
  cliente. El puerto es `TokenIssuer`, en `ramal-api-application`; que sea un JWT Ed25519 lo
  decide el adaptador de `ramal-api-infrastructure`, y los tests de los casos de uso usan un
  emisor falso que no firma nada.

### El refresh token es opaco y rota en cada uso

- 256 bits aleatorios en base64url con prefijo `rt_`. En la base solo esta su SHA-256: quien lea
  la base no se lleva sesiones utilizables.
- **Rota en cada uso** y el usado apunta a su sucesor. **Presentar un token ya rotado revoca la
  sesion entera**: un refresh token se usa una vez, y verlo dos veces significa que alguien tiene
  una copia. Se echa a los dos, al ladron y al dueno, y el dueno vuelve a entrar.
- **Excepcion por una carrera conocida, de 30 segundos.** Si el token presentado se roto hace
  menos de 30 segundos y su sucesor todavia no se ha usado, la respuesta es 409
  `refresh_superseded`, sin tokens y sin revocar nada. Existe por VS Code: dos ventanas que
  despiertan a la vez presentan el mismo token desde el mismo `SecretStorage`, y sin esta ventana
  abrir dos ventanas cerraria la sesion. La ventana que recibe el 409 vuelve a leer el almacen,
  donde la otra ya guardo el par nuevo, y reintenta. Un ladron que caiga en la ventana tampoco se
  lleva nada: el 409 no trae tokens.
- La caducidad es de la sesion: 14 dias de inactividad en la web y 90 en las apps nativas (cada
  refresco la alarga), y un maximo absoluto de 365 dias.
- **Cuando la sesion termino, el 401 lo dice con un `code`.** Si el refresco encuentra la sesion
  revocada, caducada o detecta la reutilizacion, responde 401 con `code` `session_ended`; un token
  que no reconoce recibe el `unauthorized` de siempre. El `detail` es el mismo texto fijo en los
  dos. El motor solo borra las copias sincronizadas con `session_ended`: un 401 que venga de un
  proxy o de un fallo de configuracion no debe vaciar los equipos de nadie. Distinguir los dos casos
  dice si un refresh token existio, y se acepta: son 256 bits aleatorios que nadie puede adivinar, y
  quien presenta uno revocado ya lo tenia.
- **Un dispositivo tiene como mucho una sesion.** Volver a iniciar sesion desde la misma
  instalacion reutiliza la fila del dispositivo y revoca la sesion anterior. La instalacion se
  identifica por un `installationId` y se prueba con un **secreto de dispositivo**: 256 bits
  aleatorios que el cliente genera una vez, guarda en el llavero y presenta en cada canje; la API
  guarda su hash. El `installationId` solo no basta, porque no es secreto: quien lo conociera se
  quedaria con la fila de otro. Un dispositivo revocado no se reactiva nunca: un inicio de sesion
  nuevo en esa instalacion crea otra fila, para que un portatil robado y revocado no vuelva como el
  mismo equipo de confianza.

### La web usa cookies HttpOnly con proteccion CSRF

| Cookie                | Contenido                                           | Atributos                                                       |
| --------------------- | --------------------------------------------------- | --------------------------------------------------------------- |
| `__Host-ramal_at`     | Access token                                        | `HttpOnly`, `Secure`, `SameSite=Lax`, `Path=/`, 15 minutos      |
| `__Secure-ramal_rt`   | Refresh token                                       | `HttpOnly`, `Secure`, `SameSite=Strict`, `Path=/v1/auth`        |
| `__Host-ramal_csrf`   | Token CSRF                                          | Legible desde JavaScript, `Secure`, `SameSite=Strict`, `Path=/` |
| `__Host-ramal_oauth`  | Valor aleatorio que ata un flujo OAuth al navegador | `HttpOnly`, `Secure`, `SameSite=Lax`, `Path=/`, `Max-Age=600`   |
| `__Host-ramal_signup` | Ticket de un alta por OAuth a medias                | `HttpOnly`, `Secure`, `SameSite=Lax`, `Path=/`, `Max-Age=900`   |

- Ningun token de sesion es legible desde JavaScript. Eso no hace inofensivo un XSS: un script
  inyectado no se lleva los tokens, pero actua con ellos mientras la pagina esta abierta, y sin
  defensas propias podria aprobar el inicio de sesion de una app nativa con un reto suyo y llevarse
  una sesion de 90 dias. Las defensas estan en la seccion de las apps nativas.
- Las dos cookies de flujo son `Lax` y no `Strict` porque la vuelta desde el proveedor es una
  navegacion de otro sitio: con `Strict` el navegador no las mandaria.
- El refresh token va con `Path=/v1/auth` para que el navegador solo lo mande a los endpoints que
  lo usan, no en cada peticion. La ruta es la misma dentro y fuera del gateway
  ([ADR 0007](0007-docker-desarrollo-y-produccion.md)).
- Toda peticion con cookie cuyo metodo no sea `GET`, `HEAD` u `OPTIONS` exige la cabecera
  `X-CSRF-Token` con el valor de la cookie CSRF (doble envio), y la API comprueba ademas que
  `Origin` sea el propio. `SameSite` es una segunda red, no la defensa.
- Si llegan a la vez cookie y `Authorization: Bearer`, gana Bearer y las cookies se ignoran. Las
  apps nativas usan Bearer, que no necesita proteccion CSRF porque no hay credencial ambiental
  que un sitio ajeno pueda hacer viajar.
- Las cookies funcionan porque la web y la API comparten origen a traves del gateway. Esa es la
  razon de que la API no tenga CORS.

### Las apps nativas inician sesion a traves de la web, con PKCE

Desktop y VS Code **nunca envian una contrasena a la API** ni tienen formulario de contrasena. El
motor abre el navegador del sistema en la pagina `/authorize/app` de la web con Authorization
Code y PKCE (S256), siguiendo RFC 7636 y las recomendaciones de RFC 8252 para apps nativas. La
pagina reutiliza el login normal (contrasena o cualquier OAuth), ensena "Autorizar Ramal Desktop
en este equipo" y redirige con un codigo de un solo uso que caduca en 60 segundos. El motor lo
canjea en `POST /v1/auth/app/token` con su verificador, y en ese canje se registra el
dispositivo.

| Cliente         | Redireccion                                                                                                                                                                   |
| --------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `ramal-desktop` | Exactamente `http://127.0.0.1:{puerto}/callback` o `http://[::1]:{puerto}/callback`, con un receptor que el motor abre en un puerto aleatorio de loopback (1024 a 65535)      |
| `ramal-vscode`  | Exactamente `{esquema}://ramal.ramal/auth-callback`, recibida por el `UriHandler` de la extension, con esquema `vscode`, `vscode-insiders`, `vscodium`, `cursor` o `windsurf` |

- Los clientes estan registrados en configuracion, no en la base.
- **La redireccion se valida analizandola, no comparando un prefijo.** Un prefijo como
  `http://127.0.0.1:` deja pasar `http://127.0.0.1:1@evil.example/callback`, cuyo host real es
  `evil.example`: el codigo de la victima acabaria en el servidor del atacante. Un solo validador,
  el mismo para el login de las apps y para el broker de plataformas, analiza la URL con el crate
  `url` (WHATWG) y exige esquema `http`, host exactamente `127.0.0.1` o `[::1]`, usuario y
  contrasena vacios, puerto explicito entre 1024 y 65535, ruta exactamente `/callback`, sin query
  ni fragmento, y que la URL vuelta a serializar sea identica a la recibida. Para VS Code, el
  esquema de la lista, host `ramal.ramal`, ruta `/auth-callback` y nada mas. La redireccion se
  construye desde la URL ya analizada, anadiendo `code` y `state` con la API de `url`, nunca
  concatenando texto.
- Con un `redirect_uri` que no valida, la pagina **nunca redirige**: ensena el error. Redirigir un
  error a una URL no validada es la redireccion abierta que se quiere evitar.
- **El consentimiento es un formulario, no una llamada de JavaScript.** "Autorizar" envia un
  `POST` de formulario de nivel superior (con el token CSRF como campo, porque un formulario no
  puede poner cabeceras) y la API responde con un 302 al `redirect_uri`. El codigo nunca aparece
  en un cuerpo de respuesta que un script de la pagina pueda leer.
- **Aprobar exige un inicio de sesion de hace menos de 5 minutos.** `app/authorize` y
  `device/decision` piden confirmar la contrasena o repetir el OAuth si el ultimo inicio de sesion
  real es anterior. Una sesion web robada o un XSS no aprueban nada sin esa confirmacion.
- **Los endpoints que entregan tokens a una app no aceptan nada de un navegador.** `app/token`,
  `device/code` y el refresco con el token en el cuerpo rechazan cualquier peticion que traiga
  cabecera `Origin` o cookies de Ramal. Un motor de Rust no manda ninguna de las dos; un `fetch`
  de un navegador siempre manda `Origin` en un `POST`.
- **Una sesion nativa hereda el `auth_time` de la sesion web que la aprobo**, y nunca cuenta como
  reautenticacion reciente: las acciones marcadas con reautenticacion solo se aceptan desde la web.
  Desktop y VS Code abren la pagina de la web para cambiar el correo o la contrasena, eliminar la
  cuenta o transferir y eliminar una organizacion. Si una sesion nativa nueva pudiera cambiar el
  correo en sus primeros 10 minutos, un codigo de dispositivo conseguido con phishing bastaria para
  quedarse la cuenta.
- Cada dispositivo nuevo que inicia sesion manda al correo de la cuenta el aviso "Nuevo
  dispositivo autorizado", con el nombre que declara el equipo y un enlace a la pagina de
  dispositivos para revocarlo.
- Si un codigo ya usado se presenta otra vez, se revoca ademas la sesion que se emitio con el:
  segun RFC 6749 es la senal de que alguien intercepto el codigo.
- **Por que loopback en Desktop y no un esquema `ramal://`.** Los deep links de Tauri necesitan
  el plugin de instancia unica en Windows y Linux, solo funcionan con la app instalada y en macOS
  solo desde `/Applications`. Como canal de login son fragiles. `ramal://` queda para abrir un
  repositorio desde la web o desde VS Code, y cada deep link se valida como entrada no confiable.
- **Por que una lista de esquemas en VS Code y no la URL que devuelva `asExternalUri`.** En VS
  Code en el navegador (Codespaces web), `asExternalUri` devuelve una URL `https` de un dominio
  ajeno que no se puede validar. Aceptarla seria una redireccion abierta con un codigo de
  autorizacion dentro. En VS Code de escritorio, tambien con Remote SSH, WSL o Dev Containers, el
  esquema `vscode://` funciona y el editor lo reenvia a la extension remota.

### Device flow como alternativa

Para VS Code en el navegador, entornos sin navegador y maquinas donde un cortafuegos bloquea el
puerto de loopback: RFC 8628. El motor pide un codigo, ensena el codigo de usuario y la URL
`/activate`, y sondea `POST /v1/auth/app/token` cada `interval` segundos. El codigo de usuario
son 8 caracteres de un alfabeto sin vocales ni caracteres que se confundan
(`BCDFGHJKLMNPQRSTVWXZ`): sin vocales no se forman palabras y sin `0/O` ni `1/I` no hay erratas.
Un sondeo mas rapido que el intervalo recibe `slow_down` y el intervalo sube 5 segundos.

El device flow es el camino favorito del phishing: el atacante empieza un flujo, pone en el nombre
del dispositivo el del equipo de la victima y le manda el enlace. Y al reves, quien acierte un
codigo pendiente ajeno y lo apruebe mete el equipo de otro en su propia cuenta, que empezara a
subir alli sus metadatos. Por eso:

- No hay `verificationUriComplete`: el usuario escribe en la pagina el codigo que ve en su equipo.
- La pagina dice "Pedido hace N segundos". No dice desde donde: la primera version no geolocaliza
  IPs (gaps Q-05), asi que no hay aviso de pais. Si el phishing por device flow preocupa, el
  siguiente paso seria el pais (no la ciudad) con una base de licencia clara.
- El nombre se ensena como "nombre que declara el equipo", sin caracteres de control ni de
  direccion de texto.
- `device/code` admite 10 peticiones por minuto e IP y hay un tope global de flujos pendientes;
  `device/lookup`, 10 por minuto por usuario y por IP, y 50 codigos fallidos por hora y usuario.
- Consumir una aprobacion es un `UPDATE ... SET status = 'consumed' WHERE status = 'approved'
RETURNING`: un segundo sondeo recibe `invalid_grant`, nunca otra sesion.

El motor intenta primero PKCE y pasa solo al device flow si el receptor de loopback no puede
abrirse o si VS Code corre en el navegador.

### Donde vive cada token

| Cliente | Refresh token                                                                  | Access token        |
| ------- | ------------------------------------------------------------------------------ | ------------------- |
| Web     | Cookie HttpOnly                                                                | Cookie HttpOnly     |
| Desktop | Llavero del sistema con `keyring-core` y el almacen de cada SO                 | Memoria del motor   |
| VS Code | `SecretStorage`, en el extension host; el sidecar lo pide por peticion inversa | Memoria del sidecar |

El webview nunca ve un token. En escritorio lo garantiza la CSP (el webview solo habla por IPC);
en VS Code, `connect-src 'none'` ([ADR 0011](0011-ui-compartida-y-puente-tipado.md)).

### Contrasenas con Argon2id

- Argon2id con los parametros por defecto de `argon2` 0.6, los recomendados por OWASP (19 MiB,
  dos pasadas, un carril), escritos de forma explicita para que se vea que son una decision.
- El calculo sale del hilo del runtime con `spawn_blocking`, como en Gastonomo: por eso el puerto
  de hash de contrasenas es asincrono.
- Si la cuenta no existe, el login calcula igualmente un Argon2id contra un hash fijo. No es para
  esconder que un correo tiene cuenta (el registro lo dice con un 409, ver abajo), sino para que el
  coste de cada intento sea el mismo y el tiempo no distinga una cuenta sin contrasena de una con
  ella.
- **Se acepta que el registro diga si un correo ya tiene cuenta.** Responder siempre lo mismo
  obligaria a crear la cuenta a ciegas y avisar por correo, y el usuario no sabria que paso. La
  enumeracion queda acotada por el limite por IP del registro y por una alarma sobre la tasa global
  de `email_taken`. El cambio de correo, en cambio, responde siempre 202 y falla al confirmar: ahi
  no hay ninguna pantalla que necesite la diferencia.
- De 8 a 128 caracteres al registrarse. Al entrar solo se exige que no este vacia: el minimo es
  una regla de registro, y aplicarla al entrar deja fuera a quien tenga una contrasena anterior a
  la regla.

### OAuth de identidad

- GitHub, GitLab.com y Google, con los scopes minimos de perfil y correo. El token del proveedor
  se usa para leer el perfil y se descarta en la misma peticion. Es una cosa distinta de conectar
  una cuenta de plataforma para Git ([ADR 0018](0018-cuentas-de-plataformas-git.md)).
- Una cookie `__Host-ramal_oauth` ata el flujo al navegador que lo empezo, contra el login CSRF.
- Solo se vincula automaticamente una identidad con una cuenta existente si los dos lados tienen
  el correo verificado. Vincular con una cuenta sin verificar es el ataque de preapropiacion.
- **Un correo es verificado solo si el proveedor lo dice.** GitHub: el principal y verificado de
  `/user/emails`. GitLab: `openid email profile` y `email_verified=true` en el ID token, porque
  `/api/v4/user` no trae ningun indicador. Google: `email_verified`. Sin indicador del proveedor, el
  correo cuenta como no verificado.
- **Vincular otro metodo de entrada es dar una puerta nueva**, asi que pide reautenticacion y el
  correo de la cuenta verificado, empieza con un `POST` con la cabecera CSRF (no con un `GET` que
  una navegacion de otro sitio pudiera lanzar) y manda un aviso al correo. En la vuelta, la
  identidad que llega tiene que ser del usuario de la sesion que empezo el flujo.
- **La reautenticacion por OAuth no siempre prueba algo.** GitHub y GitLab aprueban sin preguntar
  si el navegador ya tiene sesion alli. Google se pide con `prompt=login` y `max_age=0`, y se
  comprueba `auth_time` en el ID token. Por eso eliminar la cuenta y reiniciar la boveda exigen una
  **reautenticacion fuerte**: contrasena, Google, o un enlace de un solo uso al correo; el OAuth de
  GitHub o GitLab basta para el resto de acciones con reautenticacion, no para esas dos.
- Un alta nueva por OAuth pasa por un paso de "completar registro" que pide nombre y aceptar los
  terminos, en lugar de crear la cuenta al volver del proveedor.

### Recuperacion, reautenticacion y limites

- La recuperacion de contrasena es la de Gastonomo tal cual: enlace de 30 minutos y un solo uso,
  202 con cuerpo vacio siempre, un solo enlace vivo por cuenta, nada de reemitir en menos de un
  minuto, y el correo en una tarea de fondo para que el tiempo de respuesta no delate la cuenta.
  El token va en el fragmento de la URL, que el navegador no envia al servidor ni queda en los
  logs del gateway.
- Las acciones irreversibles (eliminar la cuenta, reiniciar la boveda, cambiar el correo,
  establecer la primera contrasena, vincular o quitar un metodo de inicio de sesion, transferir o
  eliminar una organizacion) exigen una sesion web con un `auth_time` de menos de 10 minutos; si no,
  403 `reauthentication_required` y la web pide confirmar la contrasena o repetir el OAuth.
- Cambiar la contrasena revoca todas las sesiones menos la actual; restablecerla por correo las
  revoca todas.
- **Restablecer la contrasena limpia lo que un atacante pudo dejar.** Alguien que registro el
  correo de la victima antes que ella podria haber vinculado su propio GitHub o pedido un cambio de
  correo a su buzon, y volver a entrar despues de que la victima recupere la cuenta. Por eso
  vincular y cambiar el correo exigen el correo verificado, restablecer borra siempre el cambio de
  correo pendiente, y si el correo no estaba verificado antes del restablecimiento borra ademas las
  identidades vinculadas, los dispositivos, las sesiones y la boveda, y manda un resumen de lo que
  quito. Lo mismo al verificar el correo aceptando una invitacion.
- Limites de peticiones en ventana fija en memoria del proceso, con el cliente identificado por
  la **ultima** entrada de `X-Forwarded-For`, como aprendio Gastonomo. Ademas del limite por IP,
  el login fallido tiene un limite por correo (10 cada 15 minutos, exista o no la cuenta) contra
  quien prueba contrasenas de una cuenta concreta desde muchas IP.

### Avisos de seguridad

No hay segundo factor todavia, y una sesion se puede conseguir sin que el usuario lo note. La
senal que le queda es el correo. Cada uno de estos hechos manda un aviso al correo de la cuenta y
deja un evento de auditoria personal (`organization_id` nulo):

- un dispositivo nuevo autorizado;
- vincular o quitar una identidad;
- establecer o cambiar la contrasena;
- un cambio de correo, con un enlace en la direccion anterior que durante 7 dias deshace el cambio
  y cierra todas las sesiones;
- "Cerrar todas las demas sesiones";
- crear la boveda, cambiar la frase, regenerar el codigo, rotar la llave, reiniciarla, y la primera
  descarga de la boveda en cada dispositivo.

## Consecuencias

A favor:

- Una peticion protegida no cuesta ninguna consulta de sesion, salvo en los endpoints vivos,
  donde cuesta una lectura por clave primaria.
- Revocar un dispositivo corta al instante el acceso a secretos, al feed y al proxy de IA.
- Las apps nativas no tienen formulario de contrasena: un proceso local malicioso o una extension
  ajena no tienen donde capturarla.
- Ningun token de sesion de la web es legible desde JavaScript, y un XSS no puede convertir la
  sesion web en una sesion nativa sin una confirmacion reciente del usuario.
- Rotar la llave de firma no cierra ninguna sesion.
- El nucleo no sabe que existen los JWT ni las cookies.

En contra, y hay que saberlo:

- **El access token no se invalida al revocar**, salvo en los endpoints vivos. Sigue valido hasta
  15 minutos para leer el perfil o los workspaces. Es el compromiso habitual de un JWT sin
  estado, acotado a lo que no entrega secretos.
- **El limite de peticiones es una ventana en memoria de cada proceso.** Con varias replicas, el
  limite real se multiplica por el numero de replicas. Un limite compartido necesita Redis o
  equivalente.
- **El limite por correo deja que un tercero bloquee 15 minutos** el login con contrasena de una
  cuenta concreta. Se acepta: la ventana es corta, el OAuth sigue funcionando, y sin limite por
  cuenta se pueden probar contrasenas desde muchas IP.
- **La ventana de 30 segundos es la primera sospechosa** si aparecen cierres de sesion
  inesperados con varias ventanas de VS Code.
- **En Linux sin llavero, `SecretStorage` cae al almacen `basic_text` de Electron**, que en la
  practica es texto plano en disco. Se documenta y se recomienda configurar un llavero.
- **Perder las llaves de firma cierra todas las sesiones**: los tokens firmados con una llave que
  ya no esta configurada no verifican. No se pierde nada mas, pero todos los usuarios tienen que
  volver a entrar.
- El proveedor Ed25519 es codigo propio, pequeno pero criptografico, y necesita sus vectores de
  prueba.
- **Autorizar una app pide confirmar la contrasena** casi siempre, porque el ultimo inicio de
  sesion suele tener mas de 5 minutos. Es friccion a proposito: es el paso que convierte una sesion
  web en una sesion de 90 dias.
- **Desde Desktop y VS Code no se cambia el correo ni la contrasena**: se abre la web. Es un salto
  mas para el usuario, a cambio de que una sesion nativa robada no pueda quedarse la cuenta.
- **El registro dice si un correo tiene cuenta.** Se acepta, acotado y vigilado (ver arriba).

## Alternativas descartadas

**Proveedor de identidad externo (Auth0, Clerk, Keycloak, Zitadel).** Anade un servicio de
terceros en el camino de cada login, con su coste y su modelo de datos, para un producto que ya
tiene que tener su propia API. Los endpoints siguen la forma de OAuth para que el cambio siga
siendo posible.

**Sesiones de servidor con cookie para todos los clientes.** Una cookie en un motor de Rust o en
un sidecar obliga a gestionar un almacen de cookies, y `SameSite` no significa nada fuera de un
navegador. Ademas cada peticion costaria una consulta.

**Un solo token de larga vida.** O se guarda y se consulta en cada peticion, que es una sesion de
servidor con otro nombre, o no se puede revocar.

**Access tokens opacos consultados en cada peticion.** Revocacion instantanea en todas partes, a
cambio de una consulta en cada peticion. Los endpoints vivos dan esa garantia solo donde importa.

**HS256, como Gastonomo.** Ver arriba: el secreto simetrico convierte a cualquier verificador en
emisor.

**Access token en memoria en la web.** Obliga a refrescar en cada carga de pagina y deja el
token al alcance de JavaScript. Con cookies HttpOnly y doble envio, ninguno de los dos problemas.

**Formulario de login dentro de Desktop o VS Code.** Es el sitio mas facil para capturar una
contrasena, y no sirve para los usuarios que entran solo con OAuth.

**Deep link `ramal://` como canal de login en Desktop.** Ver arriba: fragil segun el sistema y el
modo de instalacion.

**Plugin Stronghold de Tauri para guardar tokens.** Tauri lo da por no recomendado y lo retira en
la version 3. El llavero del sistema es lo que el usuario ya conoce y lo que protege el SO.
