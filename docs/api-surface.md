# La API de Ramal: el contrato

Este documento describe la superficie HTTP de la API en la nube de Ramal (`apps/api`),
endpoint por endpoint, y por que cada cosa es como es. Sale del analisis del handoff de
diseno (`docs/design/handoff`: el README y los tres prototipos `*.dc.html`, que mandan sobre
el README cuando se contradicen) y de las decisiones que ya estan tomadas sobre el motor
Git, el cifrado de secretos y la IA.

Es un documento tecnico: aqui se entra en detalle a proposito. El modelo de datos que lo
soporta esta en [data-model.md](data-model.md).

La referencia ejecutable es `GET /openapi.json` del servicio en marcha, y su copia versionada,
`apps/api/openapi.json`. Si este documento y ese JSON no coinciden, el JSON tiene razon y este
documento se corrige.

## Estado

Es una **propuesta de contrato** salvo la fase F0: la salud, el documento OpenAPI y los
errores en problem+json ya estan construidos (seccion [Salud y contrato](#salud-y-contrato)),
y lo que dice este documento de ellos esta comprobado contra el codigo a fecha de 2026-10-01.
Todo lo demas sigue sin construir. Cubre las tres superficies que hablan con la API (Ramal
Desktop, Ramal para VS Code y Ramal Web) y cada accion que los inventarios del diseno
clasificaron como `RAMAL_API`. El
[apendice A](#apendice-a-pantalla-o-accion-y-endpoint) cruza cada pantalla y cada accion
con su endpoint; si una pantalla pinta un dato, algun endpoint lo sirve.

Cada grupo de endpoints lleva la fase del backlog en la que entra:

| Fase | Que trae                                                                                                                                                      |
| ---- | ------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| F0   | Salud y contrato. El esqueleto.                                                                                                                               |
| F1   | Identidad: cuentas, sesiones, inicio de sesion nativo, dispositivos, publicaciones.                                                                           |
| F2   | Sincronizacion personal: preferencias, workspaces, repositorios, cuentas de plataforma, boveda, llaves de IA personales, cambios incrementales, broker OAuth. |
| F3   | Equipos: organizaciones, miembros, invitaciones, asientos, workspaces compartidos.                                                                            |
| F4   | IA de organizacion: llaves del servidor, politica, proxy y consumo.                                                                                           |
| F5   | Facturacion. Fase posterior, con el proveedor de pago sin decidir.                                                                                            |

Las publicaciones (releases y manifiesto del actualizador) estan en F1 aunque no sean
identidad: la URL del actualizador queda grabada en cada instalacion y **no se puede
cambiar despues de la primera build publica**, asi que tiene que existir antes de ella.

## Las decisiones que condicionan todo

### 1. Ramal funciona sin cuenta de Ramal

Todo lo que es Git local (abrir, clonar, hacer commit, ramas, merge, rebase, conflictos,
push con las credenciales del sistema) funciona sin iniciar sesion. La API solo entra en
juego para lo que es de la nube: sincronizar, equipos, llaves de IA de organizacion,
dispositivos y el panel web.

Consecuencia directa en el contrato: los unicos endpoints que un cliente sin cuenta puede
necesitar son publicos y no piden identidad. Son dos grupos:

- Publicaciones y actualizador (`/v1/releases`, `/v1/desktop/updates`).
- El broker OAuth de plataformas que exigen secreto de cliente (`/v1/platform-oauth`),
  para que conectar Bitbucket no obligue a tener cuenta.

### 2. Los secretos de las plataformas Git son de extremo a extremo de verdad

El servidor **no puede descifrar** los tokens de GitHub, GitLab, Bitbucket, Gitea o Azure
DevOps, ni las llaves de IA personales cuando el usuario activa su sincronizacion. Guarda
metadatos en claro (usuario, host, etiqueta, metodo, scopes) y un blob cifrado que solo
abre la llave de la boveda del usuario, que nunca sale de sus dispositivos.

Esto tiene precio, y el usuario lo acepto a sabiendas:

- **No hay cuentas de plataforma compartidas con el equipo.** Compartir un token exigiria
  envolver la llave para cada miembro y deja de ser un secreto de una persona. El toggle
  "Compartir con el equipo" y la etiqueta "Equipo" del diseno web desaparecen.
- **El panel web no puede "Probar conexion".** Probar un token es usarlo, y el servidor no
  lo tiene. El panel muestra los metadatos y permite desconectar; anadir o actualizar un
  token se hace en Desktop o en VS Code, que tienen el motor de Rust.
- **Olvidar la frase de sincronizacion y el codigo de recuperacion pierde las copias de la
  nube.** Los tokens se pueden volver a crear en cada plataforma; nadie, ni el soporte de
  Ramal, puede recuperarlos.

El esquema criptografico esta en [La boveda de extremo a extremo](#la-boveda-de-extremo-a-extremo).

### 3. Las llaves de IA de organizacion no salen nunca del servidor

Una llave de organizacion la da de alta un admin y queda cifrada en reposo en el servidor
(cifrado de sobre: una llave de datos por secreto, AES-256-GCM, y una llave maestra con
identificador para poder rotarla). Ningun endpoint la devuelve, ni siquiera a la
propietaria: los miembros ven proveedor, etiqueta, modelo y el valor enmascarado. Para
usarla, el cliente llama a `POST /v1/ai/generate` y la API hace de proxy.

Las llaves personales van por el otro camino: el motor de Rust del dispositivo llama al
proveedor directamente. Es la unica forma de que Ollama funcione (vive en `localhost` del
usuario) y la unica forma de que el codigo no pase por Ramal cuando la llave es del usuario.

### 4. En Desktop y VS Code, a la API solo le habla el motor de Rust

Ningun webview llama a la API. En Desktop la peticion sale del motor (`ramal-engine`) dentro
del host de Tauri (`ramal-desktop`); en VS Code, del sidecar `ramal-vscode-engine`. Los tokens
viven en el llavero del sistema operativo (Desktop) o en `SecretStorage` (VS Code) y nunca
llegan a un webview. La web llega por el mismo origen a traves del gateway.

Consecuencia: **la API no tiene CORS**. No hay ningun origen de navegador distinto del
propio que tenga que hablarle. Que no exista la cabecera es una defensa, no un olvido.

Las rutas son las mismas dentro y fuera del gateway
([ADR 0007](adr/0007-docker-desarrollo-y-produccion.md)): `/v1/...` en este documento es
`https://ramal.dev/v1/...` en produccion. No hay un prefijo `/api` que se quite por el camino.

### 5. Tres ambitos: personal, organizacion y dispositivo

Cada dato pertenece a uno de estos ambitos, y el ambito decide quien lo ve:

| Ambito       | Que vive aqui                                                                                                                                                                |
| ------------ | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Personal     | Perfil, preferencias, workspaces personales, repositorios, cuentas de plataforma, boveda, llaves de IA personales.                                                           |
| Organizacion | Miembros, invitaciones, asientos, workspaces de la organizacion, llaves de IA de organizacion y su politica.                                                                 |
| Dispositivo  | Rutas locales, repositorios sin remoto, pestanas, borradores. **Nunca llega a la API**, salvo la identidad de un repositorio sin remoto (ver [Repositorios](#repositorios)). |

El "contexto personal" del selector de organizaciones no es una organizacion: es la
ausencia de ella. El contexto activo es estado del cliente (en la web, parte de la URL del
panel) y no se guarda en la API. Dos ventanas en dos contextos no tienen por que pelearse
por un campo compartido.

### 6. Lo que el servidor sabe, y lo que no

Conviene tenerlo escrito, porque es lo que se le promete al usuario.

**Lo que la API ve en claro:** correo, nombre, usuario, zona horaria y foto; organizaciones,
miembros y roles; nombres y colores de workspaces; las URL remotas de los repositorios
(host, propietario y nombre), sus alias y que cuenta usa cada uno; los metadatos de las
cuentas de plataforma (usuario, host, etiqueta, scopes, caducidad declarada); proveedor,
etiqueta, modelo y mascara de las llaves de IA; las preferencias de Git (incluidos
`user.name` y `user.email`), de apariencia y de IA; nombre, sistema y version de cada
dispositivo.

**Lo que la API no ve nunca:** tokens de plataforma, llaves de IA personales, la frase de
sincronizacion, el codigo de recuperacion, la llave de la boveda, rutas locales, contenido
de repositorios. La unica excepcion al contenido es el proxy de IA de organizacion, que
recibe el diff filtrado de ese momento, lo reenvia al proveedor y no lo guarda ni lo
registra (ver [El proxy de IA](#el-proxy-de-ia)).

## Convenciones del contrato

Prefijo `/v1`. Todo JSON. Errores con `application/problem+json` (RFC 9457). Son las mismas
convenciones que Gastonomo, con cuatro anadidos que Ramal necesita: ids generados por el
cliente, `code` en los errores de campo, concurrencia optimista y streaming en el proxy de IA.

- **JSON en camelCase.** Las dos excepciones son externas y estan marcadas donde aparecen:
  el manifiesto del actualizador de Tauri (su formato lo dicta Tauri) y los parametros de la
  pagina de autorizacion nativa (los dicta OAuth 2.0).
- **Instantes en RFC 3339 con `Z`** (`2026-09-30T10:00:00Z`). No hay fechas sin hora en
  este dominio.
- **Binario en base64url sin relleno**: ciphertexts, nonces, sales, llaves envueltas.
- **Enumerados en minusculas y snake_case**: `commit_message`, `ff_only`, `api_token`.
- **Identificadores UUID.** El servidor genera UUIDv7. Las entidades que un dispositivo
  puede crear sin conexion llevan id generado por el cliente (ver
  [Idempotencia](#idempotencia)).
- **Toda lista va envuelta en `items`.** Las paginadas anaden `nextCursor` al lado.
- **Cada respuesta lleva `x-request-id`**, propagando el de la peticion si venia.
- **Tamano maximo del cuerpo: 64 KiB** por defecto. Las excepciones estan en cada endpoint
  (la foto de perfil, la rotacion de la boveda y el proxy de IA).

### Cuerpo de error

```json
{
  "type": "https://ramal.dev/problems/validation",
  "title": "Datos invalidos",
  "status": 422,
  "detail": "Revisa los campos marcados.",
  "code": "validation_failed",
  "errors": [
    { "field": "name", "code": "too_long", "message": "El nombre no puede pasar de 60 caracteres." }
  ]
}
```

- **Todo problema lleva `code`**, un texto estable en snake_case, y cada entrada de `errors`
  tambien. `errors` solo aparece en los errores de validacion. Es lo que el cliente usa para
  decidir; el `type` es la familia y el `code` el motivo concreto.
- **Los mensajes vienen en castellano**, igual que en Gastonomo. Pero la interfaz de Ramal
  es bilingue: por eso cada error de campo lleva un `code` estable y los clientes pintan el
  texto desde `@ramal/i18n` en el idioma de la interfaz. El `message` es el texto de
  respaldo y el que se lee en los logs de un cliente.
- El `detail` de **todos** los 401 es el mismo texto fijo, a proposito: distinguir un token
  que no existe de uno caducado o ya usado diria si se acerto con uno que existe. El motivo
  real queda en la traza del servidor.
- Los 404 y 405 del transporte tambien salen en `problem+json`, con el mismo catcher que
  Gastonomo. Sin el, Salvo negociaria el `Accept` y responderia una pagina HTML. El catcher
  respeta el estado: un 410 sigue siendo 410 aunque su `type` sea generico.
- Los motivos internos (el `reason` de un 401, el recurso que falto en un 404, el detalle de un
  fallo de una dependencia) se quedan en el log; al cliente solo le llega texto pensado para el.

Los `code` que ya existen en el primer corte:

| Origen                           | `type`                   | Estado   | `code`                     |
| -------------------------------- | ------------------------ | -------- | -------------------------- |
| Ruta que no existe               | `not-found`              | 404      | `route_not_found`          |
| Metodo no admitido               | `method-not-allowed`     | 405      | `method_not_allowed`       |
| Cuerpo demasiado grande          | `payload-too-large`      | 413      | `payload_too_large`        |
| Tipo de contenido                | `unsupported-media-type` | 415      | `unsupported_media_type`   |
| Demasiadas peticiones            | `too-many-requests`      | 429      | `too_many_requests`        |
| Servicio no disponible           | `service-unavailable`    | 503      | `service_unavailable`      |
| Otro 4xx del transporte          | `bad-request`            | el suyo  | `bad_request`              |
| Otro 5xx                         | `internal-error`         | 500      | `internal_error`           |
| Validacion de un caso de uso     | `validation`             | 422      | `validation_failed`        |
| Sin sesion valida                | `unauthorized`           | 401      | `unauthorized`             |
| Recurso que no existe o es ajeno | `not-found`              | 404      | `not_found`                |
| Sin permiso, conflicto           | `forbidden`, `conflict`  | 403, 409 | el que fije el caso de uso |

Los de transporte salen de `ramal-api-http/src/problem/from_status.rs` y los de los casos de uso
de `from_error.rs`; un endpoint nuevo anade los suyos.

Los `type` del contrato completo (los del primer corte estan en la tabla de arriba): `validation`, `bad-request`, `invalid-credentials`, `unauthorized`,
`forbidden`, `not-found`, `gone`, `method-not-allowed`, `conflict`, `precondition-failed`,
`precondition-required`, `payload-too-large`, `unsupported-media-type`,
`too-many-requests`, `internal-error`, `bad-gateway`, `gateway-timeout` y
`service-unavailable`.

El dominio `ramal.dev` es el que usa el diseno; si el producto acaba en otro, cambia el
prefijo de `type` y nada mas.

### 404 y no 403, y cuando si hay 403

La regla de Gastonomo se mantiene: **un recurso de otra cuenta responde 404, nunca 403**.
Un 403 confirmaria que existe. Lo mismo cuando el identificador del camino no es un UUID, y
lo mismo cuando la organizacion existe pero quien pregunta no es miembro.

Ramal anade un caso que Gastonomo no tenia: **403 con `code` cuando el recurso es tuyo o de
tu organizacion, lo puedes ver, y lo que falta es permiso**. Un miembro sabe que su organizacion
existe; devolverle 404 al intentar invitar a alguien seria mentirle sobre algo que acaba de ver
en pantalla. Lo que no puedes ver sigue siendo 404 en todos los verbos: el workspace privado de
otro miembro responde 404 a un `GET`, a un `PATCH` y a un `DELETE`, nunca 403, porque el orden de
las comprobaciones tambien dice si algo existe. Los `code` de 403 son:

| code                        | Cuando                                                                                                                                                                             |
| --------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `insufficient_role`         | Tu rol en la organizacion no permite la accion (ver la matriz de roles).                                                                                                           |
| `reauthentication_required` | La accion es sensible y tu ultimo inicio de sesion real en la web tiene mas de 10 minutos, o el metodo no basta para esa accion. La extension `acceptedMethods` dice cuales valen. |
| `web_session_required`      | La accion solo se acepta desde la web (las que piden reautenticacion). Desktop y VS Code abren la pagina de la web.                                                                |
| `native_client_required`    | La accion solo se acepta desde Desktop o VS Code (escribir secretos cifrados o metadatos que informa el dispositivo).                                                              |
| `email_not_verified`        | La accion manda correo a terceros, crea una organizacion, vincula una identidad o cambia el correo, y tu correo no esta verificado.                                                |

### Como se autentica cada peticion

Hay dos formas de presentar la identidad, y nunca se mezclan:

- **Bearer**: `Authorization: Bearer {accessToken}`. La usan los motores de Desktop y VS
  Code. No lleva proteccion CSRF porque no hay credencial ambiental que un sitio ajeno pueda
  hacer viajar.
- **Cookie**: la usa solo la web. Tres cookies, todas `Secure`:

| Cookie                | Contenido                                 | Atributos                                           |
| --------------------- | ----------------------------------------- | --------------------------------------------------- |
| `__Host-ramal_at`     | Access token (JWT)                        | `HttpOnly`, `SameSite=Lax`, `Path=/`, 15 minutos    |
| `__Secure-ramal_rt`   | Refresh token opaco                       | `HttpOnly`, `SameSite=Strict`, `Path=/v1/auth`      |
| `__Host-ramal_csrf`   | Token CSRF                                | Legible desde JS, `SameSite=Strict`, `Path=/`       |
| `__Host-ramal_oauth`  | Valor que ata un flujo OAuth al navegador | `HttpOnly`, `SameSite=Lax`, `Path=/`, `Max-Age=600` |
| `__Host-ramal_signup` | Ticket de un alta por OAuth a medias      | `HttpOnly`, `SameSite=Lax`, `Path=/`, `Max-Age=900` |

Las dos ultimas son `Lax` porque se leen en la vuelta desde el proveedor, que es una navegacion
de otro sitio: con `Strict` el navegador no las mandaria.

Con cookie, **toda peticion con metodo que no sea `GET`, `HEAD` u `OPTIONS` exige la
cabecera `X-CSRF-Token`** con el valor de la cookie CSRF (doble envio), y la API comprueba
ademas que `Origin` sea el propio. Si llegan las dos formas a la vez, gana Bearer y las
cookies se ignoran.

El refresh token web va con `Path=/v1/auth` para que el navegador no lo mande en cada
peticion: solo viaja a los endpoints que lo usan.

**El access token es un JWT de 15 minutos firmado con Ed25519** (`kid` y `typ: at+jwt` en la
cabecera; solo se acepta `alg: EdDSA` con un `kid` conocido). Lleva `iss`
(`https://ramal.dev`), `aud` (`ramal-api`), `sub` (usuario), `sid` (sesion), `cli` (`web`,
`desktop` o `vscode`), `auth_time` (ultimo inicio de sesion real), `iat` y `exp`. No lleva
roles: los roles son por organizacion y cambian, y se leen de la base en cada peticion que
los necesita.

Validarlo no toca PostgreSQL, igual que en Gastonomo. Pero hay endpoints que un dispositivo
revocado **no debe poder usar ni un minuto mas**: los que entregan secretos cifrados, el
feed de cambios, el proxy de IA y las escrituras de cuentas de plataforma y llaves de IA (que
deciden a donde va un secreto). Esos llevan la marca **viva** en la tabla de endpoints y
comprueban ademas que la sesion `sid` no este revocada, con una lectura por clave primaria.
Es el compromiso: la revocacion es instantanea donde importa y no cuesta una consulta en el
resto.

**Reautenticacion reciente.** Las acciones marcadas **reauth** exigen una sesion web cuyo
`auth_time` tenga menos de 10 minutos; si no, 403 `reauthentication_required`. La web manda al
usuario a confirmar su contrasena (`POST /v1/auth/reauthenticate`), a repetir el OAuth o a usar
un enlace al correo, y vuelve. Las marcadas **reauth fuerte** (eliminar la cuenta y reiniciar la
boveda) no aceptan el OAuth de GitHub ni de GitLab, que puede completarse sin pedir nada si el
navegador ya tiene sesion alli: valen la contrasena, Google con `max_age=0` o el enlace al correo.

**Una sesion nativa nunca cuenta como reautenticacion reciente.** Al crearse hereda el
`auth_time` de la sesion web que la aprobo, pero los endpoints con **reauth** solo se aceptan con
cookie (403 `web_session_required` con Bearer). Desktop y VS Code abren con `openExternal` la
pagina de la web que corresponde (cambiar el correo o la contrasena, eliminar la cuenta,
transferir o eliminar una organizacion) y no llaman a esos endpoints. Si una sesion nativa
pudiera cambiar el correo en sus primeros 10 minutos, un codigo de dispositivo conseguido con
phishing bastaria para quedarse la cuenta.

### Paginacion

Por cursor. Parametros `cursor` y `limit`; respuesta `{ "items": [...], "nextCursor": "..." }`,
con `nextCursor: null` en la ultima pagina. `limit` por defecto 50 y como maximo 200; un
valor mayor se acota en silencio.

El cursor **es opaco de verdad**: base64url de una estructura con version, firmada con HMAC
para que no se pueda fabricar. Un cursor que no sea de esta API es **422** con
`code: invalid_cursor`, no "desde el principio". Gastonomo aprendio que un cursor legible
invita a construirlo a mano.

Se paginan las listas que pueden crecer sin techo razonable: repositorios, miembros,
invitaciones, publicaciones y consumo de IA. El resto (workspaces, cuentas de plataforma,
llaves, dispositivos, sesiones) tiene un tope duro por cuenta y se devuelve entero, tambien
dentro de `items`:

| Recurso                           | Tope | Al pasarlo                       |
| --------------------------------- | ---- | -------------------------------- |
| Workspaces por contexto           | 100  | 409 `limit_reached`              |
| Repositorios por workspace        | 200  | 409 `limit_reached`              |
| Cuentas de plataforma por usuario | 50   | 409 `limit_reached`              |
| Llaves de IA personales           | 20   | 409 `limit_reached`              |
| Llaves de IA por organizacion     | 20   | 409 `limit_reached`              |
| Dispositivos vivos por usuario    | 20   | el mas antiguo sin uso se revoca |

### Idempotencia

El riesgo es el mismo que en Gastonomo, con una diferencia: aqui los dispositivos trabajan
sin conexion y encolan escrituras. Un reintento no puede duplicar un workspace ni una
cuenta. Hay tres mecanismos, y cada endpoint dice cual usa:

1. **Id generado por el cliente.** Workspaces, cuentas de plataforma y llaves de IA
   personales se crean con un `id` (UUIDv4 o v7) que manda el cliente. Repetir el `POST`
   con el mismo id y el mismo cuerpo devuelve **200 con el recurso guardado**; con otro
   cuerpo es **409** `id_conflict`. Gastonomo descarto este camino porque un segundo `POST`
   con el mismo id seria un 409 sin saber si el primero se guardo; aqui no pasa, porque el
   reintento identico responde 200.

   Hay dos razones para usarlo aqui y no la cabecera:
   - Un dispositivo sin conexion necesita el id **antes** de hablar con la API: crea el
     workspace, le anade repositorios y encola las tres llamadas en orden.
   - En las cuentas de plataforma y las llaves de IA el id es **obligatorio**: el cifrado
     de extremo a extremo liga el ciphertext a su registro (el id entra en los datos
     asociados del AEAD), y el cliente cifra antes de que la API sepa nada.

2. **Upsert por identidad natural.** Registrar un repositorio es un upsert por su remoto
   normalizado (o por su identidad local de dispositivo). Repetirlo no crea nada nuevo.

3. **Cabecera `Idempotency-Key`**, con el protocolo exacto de Gastonomo: opcional, hasta
   200 caracteres, 24 horas de vida, huella SHA-256 del cuerpo, `Idempotent-Replay: true`
   en la respuesta repetida, 409 `idempotency_key_reused` con otro cuerpo, 409
   `idempotency_key_in_progress` si hay dos a la vez, y un 422 no quema la clave. La
   aceptan los `POST` que crean algo sin id de cliente y se pueden repetir por un doble
   clic o una red caida: crear organizacion, crear llave de IA de organizacion, y en F5
   comprar asientos y abrir un checkout.

El id de una cuenta de plataforma o de una llave de IA personal que se borro **no se puede
volver a usar**: un `POST` con ese id responde 409 `id_conflict`. El id entra en los datos
asociados del secreto, y reutilizarlo permitiria colocar un blob antiguo en una cuenta nueva con
otros metadatos (ver [La boveda](#la-boveda-de-extremo-a-extremo)).

`PUT` y `DELETE` son idempotentes por definicion y no necesitan nada: borrar dos veces
responde 204 las dos.

### Concurrencia optimista

Los documentos de ajustes que se sincronizan entre dispositivos (preferencias de Git, de
apariencia y de IA, los ajustes de sincronizacion y la cabecera de la boveda) llevan un
numero de `version` entero que la respuesta devuelve tambien como `ETag: "7"`.

- Toda escritura exige `If-Match: "7"`. Sin la cabecera, **428** `precondition-required`.
- Si la version no coincide, **412** `precondition-failed` con `code: version_conflict` y el
  documento vigente en la extension `current`, para que el cliente no tenga que hacer un
  `GET` antes de reintentar.
- Las preferencias se escriben con **JSON Merge Patch** (RFC 7396): el cliente solo manda
  los campos que el usuario cambio. Reintentar tras un 412 es volver a aplicar el mismo
  parche sobre `current`.

El merge patch no es un capricho. Un cliente viejo que hiciera `PUT` del documento entero
borraria los campos que anadio una version mas nueva y que el no conoce. Con un parche,
cada cliente solo toca lo suyo.

Por que no marcas de tiempo por campo: la persona que edita es una sola, los conflictos de
verdad son raros (dos maquinas cambiando el mismo ajuste en el mismo minuto), y un numero
de version se razona en una linea.

### Limites de peticiones

Como en Gastonomo: ventana fija en memoria del proceso, el cliente se identifica por la
**ultima** entrada de `X-Forwarded-For` (la que anade el proxy de confianza) o por la IP del
par, y 429 con `Retry-After`. Las cifras son configurables con `RAMAL__RATE_LIMIT__*`.

| Grupo                                                                                                | Limite                                                                                                             | Clave                 |
| ---------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------ | --------------------- |
| `register`, `login`, `password/forgot`, `password/reset`, `reauthenticate`, completar registro OAuth | 10 por minuto                                                                                                      | IP                    |
| `login` fallido                                                                                      | 10 cada 15 minutos                                                                                                 | correo (exista o no)  |
| `password/forgot` y reenvio de la verificacion                                                       | 5 por hora y 10 al dia                                                                                             | correo de destino     |
| `oauth/{provider}/start`                                                                             | 20 por minuto                                                                                                      | IP                    |
| `auth/refresh`, `auth/app/token`                                                                     | 60 por minuto                                                                                                      | IP                    |
| `auth/device/code`                                                                                   | 10 por minuto, y un tope global de flujos pendientes                                                               | IP                    |
| Sondeo del device flow                                                                               | el `interval` que se devuelve                                                                                      | device code           |
| `auth/device/lookup`                                                                                 | 10 por minuto; 50 codigos fallidos por hora                                                                        | usuario y, aparte, IP |
| Endpoints que mandan correo (verificacion, invitaciones, cambio de correo)                           | 5 por hora; invitaciones 50 al dia por organizacion y 20 al dia por quien invita, sumando todas sus organizaciones | usuario               |
| `GET /v1/me/export`                                                                                  | 5 por hora                                                                                                         | usuario               |
| `GET /v1/vault`                                                                                      | 20 al dia                                                                                                          | usuario               |
| Escrituras de la boveda (crear, cambiar frase, recuperacion, rotar, reiniciar)                       | 10 por hora                                                                                                        | usuario               |
| `POST /v1/ai/generate`                                                                               | 20 por minuto, 2 a la vez por usuario y organizacion, mas la cuota reservada de la organizacion                    | usuario               |
| Broker OAuth de plataformas                                                                          | 10 por minuto al iniciar; 120 por hora al canjear o refrescar                                                      | IP                    |
| Publicaciones y actualizador                                                                         | 120 por minuto                                                                                                     | IP                    |
| Resto de endpoints autenticados                                                                      | 600 por minuto                                                                                                     | usuario               |

El limite por correo en `login` existe para frenar a quien prueba contrasenas contra una
cuenta concreta desde muchas IP. Se aplica exista o no la cuenta, para no convertirse en un
oraculo. El limite por correo de destino en `password/forgot` y en el reenvio de la verificacion
impide que alguien llene de correos el buzon de otro desde muchas IP. Ademas hay una alarma sobre
la tasa global de respuestas `email_taken` del registro: es el oraculo que se acepta (ver
[Registro](#registro)) y hay que ver si alguien lo esta recorriendo.

Lo que estos limites **no** son: defensa contra adivinar tokens. Los tokens opacos son 256
bits de CSPRNG. Lo que acotan es el ruido, la carga, el correo y el gasto de IA.

## Todos los endpoints

Marcas de acceso: **publico** (sin identidad), **web** (solo sesion de cookie), **nativa**
(solo Bearer de Desktop o VS Code), **sesion** (cualquiera de las dos), **viva** (comprueba
que la sesion no este revocada), **reauth** (sesion web con inicio de sesion real de hace menos
de 10 minutos), **reauth fuerte** (lo mismo, con contrasena, Google o enlace al correo),
**admin** y **owner** (rol minimo en la organizacion), **publicacion** (identidad OIDC del
workflow de publicacion).

```
Salud y contrato                                                        F0
GET    /health/live                              200  publico
GET    /health/ready                             200 | 503  publico
GET    /openapi.json                             200  publico
GET    /swagger-ui/                              200  solo con RAMAL__SERVER__SWAGGER_UI=true (desarrollo)

Autenticacion web                                                       F1
POST   /v1/auth/register                         201  publico, limitado
POST   /v1/auth/login                            200  publico, limitado
POST   /v1/auth/refresh                          200  cookie o refreshToken
POST   /v1/auth/logout                           204  sesion o refreshToken
POST   /v1/auth/reauthenticate                   200  web, limitado
POST   /v1/auth/reauthenticate/email-link        202  web, limitado
POST   /v1/auth/reauthenticate/email-link/confirm  204  publico (token)
POST   /v1/auth/password/forgot                  202  publico, limitado
POST   /v1/auth/password/reset                   204  publico, limitado
POST   /v1/auth/email/verify                     204  publico (token)
POST   /v1/auth/email/verification               202  sesion (reenviar)
POST   /v1/auth/email-change/confirm             204  publico (token)
POST   /v1/auth/email-change/revert              204  publico (token)
POST   /v1/auth/oauth/{provider}/start           200  publico (login); web, CSRF (link, reauth)
GET    /v1/auth/oauth/{provider}/callback        302  publico
GET    /v1/auth/oauth/signup                     200  cookie de alta pendiente
POST   /v1/auth/oauth/signup                     201  cookie de alta pendiente

Inicio de sesion de las apps nativas                                    F1
GET    /v1/auth/app/authorize                    200  web
POST   /v1/auth/app/authorize                    302  web (formulario), inicio de sesion de menos de 5 minutos
POST   /v1/auth/app/token                        200  publico
POST   /v1/auth/device/code                      200  publico, sin Origin ni cookies
POST   /v1/auth/device/lookup                    200  web, limitado
POST   /v1/auth/device/decision                  204  web, inicio de sesion de menos de 5 minutos

Sesiones                                                                F1
GET    /v1/auth/sessions                         200  sesion
DELETE /v1/auth/sessions/{id}                    204  sesion
POST   /v1/auth/sessions/revoke-others           204  sesion

Perfil y cuenta                                                         F1 (foto en F2)
GET    /v1/me                                    200  sesion
PATCH  /v1/me                                    200  sesion
POST   /v1/me/password                           204  web (reauth si no habia contrasena)
POST   /v1/me/email-change                       202  web, reauth
PUT    /v1/me/avatar                             200  sesion
DELETE /v1/me/avatar                             204  sesion
GET    /v1/avatars/{id}                          200  publico (URL de capacidad)
GET    /v1/me/identities                         200  sesion
DELETE /v1/me/identities/{provider}              204  web, reauth
GET    /v1/me/export                             200  sesion, viva
DELETE /v1/me                                    204  web, reauth fuerte
GET    /v1/overview                              200  sesion  ?organizationId=   (F2)

Dispositivos                                                            F1
GET    /v1/devices                               200  sesion
PATCH  /v1/devices/current                       200  nativa
PATCH  /v1/devices/{id}                          200  sesion
DELETE /v1/devices/{id}                          204  sesion

Organizaciones                                                          F3
GET    /v1/organizations                         200  sesion
POST   /v1/organizations                         201  sesion, Idempotency-Key
GET    /v1/organizations/{orgId}                 200  miembro
PATCH  /v1/organizations/{orgId}                 200  admin
DELETE /v1/organizations/{orgId}                 204  web, owner, reauth
POST   /v1/organizations/{orgId}/leave           204  miembro
POST   /v1/organizations/{orgId}/transfer-ownership  200  web, owner, reauth
GET    /v1/organizations/{orgId}/seats           200  miembro
GET    /v1/organizations/{orgId}/members         200  miembro  ?cursor=&limit=
PATCH  /v1/organizations/{orgId}/members/{userId}    200  admin
DELETE /v1/organizations/{orgId}/members/{userId}    204  admin
GET    /v1/organizations/{orgId}/invitations     200  admin  ?cursor=&limit=
POST   /v1/organizations/{orgId}/invitations     201  admin
POST   /v1/organizations/{orgId}/invitations/{id}/resend  202  admin
DELETE /v1/organizations/{orgId}/invitations/{id}         204  admin
POST   /v1/invitations/preview                   200  publico (token)
POST   /v1/invitations/accept                    200  sesion (token)
GET    /v1/me/invitations                        200  sesion
POST   /v1/me/invitations/{id}/accept            200  sesion
DELETE /v1/me/invitations/{id}                   204  sesion

Facturacion                                                             F5
GET    /v1/organizations/{orgId}/subscription    200  miembro
POST   /v1/organizations/{orgId}/subscription/checkout   200  owner, Idempotency-Key
POST   /v1/organizations/{orgId}/subscription/seats      200  owner, Idempotency-Key
POST   /v1/organizations/{orgId}/subscription/portal     200  owner
POST   /v1/billing/webhooks/{provider}           204  publico (firma del proveedor)

Workspaces                                                              F2 (compartidos en F3)
GET    /v1/workspaces                            200  sesion  ?organizationId=
POST   /v1/workspaces                            201 | 200  sesion, id del cliente
GET    /v1/workspaces/{id}                       200  sesion
PATCH  /v1/workspaces/{id}                       200  sesion
DELETE /v1/workspaces/{id}                       204  sesion
POST   /v1/workspaces/{id}/repositories          201 | 200  sesion
DELETE /v1/workspaces/{id}/repositories/{entryId}    204  sesion

Repositorios                                                            F2
GET    /v1/repositories                          200  sesion  ?q=&workspaceId=&linkedAccountId=&kind=&cursor=&limit=
POST   /v1/repositories                          201 | 200  sesion (upsert por identidad)
POST   /v1/repositories/batch                    200  nativa
GET    /v1/repositories/{id}                     200  sesion
PATCH  /v1/repositories/{id}                     200  sesion
DELETE /v1/repositories/{id}                     204  sesion
DELETE /v1/repositories/{id}/workspaces          204  sesion (quitar de todos)

Cuentas de plataformas Git                                              F2
GET    /v1/linked-accounts                       200  sesion
POST   /v1/linked-accounts                       201 | 200  nativa, viva, id del cliente
GET    /v1/linked-accounts/{id}                  200  sesion
PATCH  /v1/linked-accounts/{id}                  200  sesion, viva (la web solo label e isDefault)
DELETE /v1/linked-accounts/{id}                  204  sesion, viva
PUT    /v1/linked-accounts/{id}/secret           200  nativa, viva
DELETE /v1/linked-accounts/{id}/secret           204  nativa, viva

Boveda                                                                  F2
GET    /v1/vault                                 200  nativa, viva
POST   /v1/vault                                 201  nativa, viva
PUT    /v1/vault/passphrase                      200  nativa, viva, If-Match
PUT    /v1/vault/recovery                        200  nativa, viva, If-Match
POST   /v1/vault/rotate                          200  nativa, viva, If-Match
GET    /v1/vault/secrets                         200  nativa, viva
DELETE /v1/vault                                 204  web, reauth fuerte

Llaves de IA personales                                                 F2
GET    /v1/ai/keys                               200  sesion
POST   /v1/ai/keys                               201 | 200  nativa, viva, id del cliente
PATCH  /v1/ai/keys/{id}                          200  nativa, viva
DELETE /v1/ai/keys/{id}                          204  sesion, viva
PUT    /v1/ai/keys/{id}/secret                   200  nativa, viva
DELETE /v1/ai/keys/{id}/secret                   204  nativa, viva

Preferencias sincronizadas                                              F2
GET    /v1/me/preferences                        200  sesion
GET    /v1/me/preferences/{namespace}            200  sesion
PATCH  /v1/me/preferences/{namespace}            200  sesion, If-Match, merge patch
GET    /v1/me/sync-settings                      200  sesion
PATCH  /v1/me/sync-settings                      200  sesion, If-Match

IA de organizacion                                                      F4
GET    /v1/organizations/{orgId}/ai-keys         200  miembro
POST   /v1/organizations/{orgId}/ai-keys         201  admin, Idempotency-Key
PATCH  /v1/organizations/{orgId}/ai-keys/{id}    200  admin
DELETE /v1/organizations/{orgId}/ai-keys/{id}    204  admin
POST   /v1/organizations/{orgId}/ai-keys/{id}/test   200  admin
GET    /v1/organizations/{orgId}/ai-settings     200  miembro
PATCH  /v1/organizations/{orgId}/ai-settings     200  admin
GET    /v1/organizations/{orgId}/ai-usage        200  admin  ?from=&to=&cursor=&limit=
POST   /v1/ai/generate                           200  nativa, viva (JSON o SSE)

Cambios incrementales                                                   F2
GET    /v1/sync/changes                          200 | 410  nativa, viva  ?since=&limit=

Publicaciones y actualizador                                            F1
GET    /v1/releases/latest                       200  publico  ?product=&channel=
GET    /v1/releases                              200  publico  ?product=&channel=&cursor=&limit=
GET    /v1/desktop/updates/{target}/{arch}/{currentVersion}  200 | 204  publico  ?channel=
POST   /v1/admin/releases                        201  publicacion
PATCH  /v1/admin/releases/{id}                   200  publicacion

Broker OAuth de plataformas                                             F2
GET    /v1/platform-oauth/providers              200  publico
GET    /v1/platform-oauth/{provider}/authorize   302  publico
GET    /v1/platform-oauth/{provider}/callback    302  publico
POST   /v1/platform-oauth/{provider}/token       200  publico, no-store
POST   /v1/platform-oauth/{provider}/refresh     200  publico, no-store
```

## Salud y contrato

**F0. Construido.** Como Gastonomo, con `checkedAt` y sin el detalle de los fallos.

```
GET /health/live    200 siempre, sin tocar dependencias; checks siempre vacio
GET /health/ready   200 si status=ok, 503 si status=degraded (mismo JSON, no problem+json)

{"status":"ok"|"degraded",
 "checks":[{"name":"database","status":"ok"|"error","latencyMs":3}],
 "version":"0.1.0",
 "checkedAt":"2026-09-30T10:00:00Z"}
```

- `checkedAt` es el instante de la comprobacion, en RFC 3339 con `Z`. La hora sale del puerto
  `Clock` de `ramal-api-application`: el nucleo no lee el reloj.
- **El motivo de un fallo no viaja.** Un componente en `error` solo dice eso; el detalle
  (timeout, conexion rechazada, esquema atrasado) queda en el log del servidor.
- `database` no es un `SELECT 1`: lee la ultima migracion aplicada y la compara con la mas nueva
  que trae el binario. Mientras las migraciones del arranque no terminan (se reintentan cada 10
  segundos), `ready` responde 503 y `live` sigue en 200.

`secrets` (todavia no existe) comprobara que la llave maestra activa (la que cifra las llaves de IA de
organizacion) esta cargada y descifra un valor de prueba. Sin ella la API arranca, pero
`ready` responde 503: un despliegue que no puede abrir las llaves de organizacion no debe
recibir trafico del proxy de IA. Antes de F4 el check no existe.

`GET /openapi.json` lo genera `salvo-oapi` desde el propio router (OpenAPI 3.1), y de ese JSON
sale `@ramal/api-client` (`openapi-typescript` + `openapi-fetch`). Es publico. El documento se
versiona en `apps/api/openapi.json`: un test de `ramal-api-http` falla si se queda atras y
`just api-openapi` lo reescribe. El esquema `Problem` va en el documento aunque ningun endpoint
lo declare todavia, porque cualquier ruta puede responderlo.

`GET /swagger-ui/` es el visor, **solo en desarrollo**: la API solo lo monta con
`RAMAL__SERVER__SWAGGER_UI=true`, que encienden `compose.yaml` y `just api-run`. Por defecto va
apagado y la ruta responde 404 como cualquier otra, asi que una imagen de produccion no lo sirve
aunque el gateway reenviara la ruta (el de produccion tampoco la enrutara;
[ADR 0007](adr/0007-docker-desarrollo-y-produccion.md)). Swagger UI ha tenido fallos de XSS, y
servido en `ramal.dev` correria con las cookies de sesion y la cookie CSRF a su alcance. Sus
assets van embebidos en el binario: funciona sin salida a internet.
Los tipos del protocolo del motor (Desktop y VS Code) no salen de aqui: salen de
`ramal-protocol` con `ts-rs`, porque el motor no es un cliente HTTP de la web.

## Autenticacion web

**F1.** Registro, login y OAuth de identidad son **solo de la web** y siempre responden con
cookies. Las apps nativas nunca envian una contrasena a la API: inician sesion a traves de
la pagina web (ver [Inicio de sesion de las apps nativas](#inicio-de-sesion-de-las-apps-nativas)).
Esto quita de golpe el formulario de contrasena de Desktop y VS Code, que era la forma mas
facil de que un proceso local malicioso o una extension ajena la capturara.

### Registro

```
POST /v1/auth/register
{
  "name": "Marta Ruiz",
  "email": "marta@ramal.dev",
  "password": "********",
  "organizationName": "ACME Studio",
  "acceptedTerms": true,
  "termsVersion": "2026-09",
  "locale": "es"
}
```

Responde **201** con `{ "user": Me, "csrfToken": "..." }` y las tres cookies. `Me` se describe
en [Perfil](#perfil-y-cuenta).

- `name` de 1 a 80 caracteres tras recortar. `email` con forma de correo y 254 como maximo.
  `password` de 8 a 128 caracteres; el medidor de fuerza es visual y vive en el cliente.
- `acceptedTerms` tiene que ser `true`, y se guarda `termsVersion` con la fecha. Sin la
  version no hay forma de saber que acepto cada usuario cuando cambien los terminos.
- `organizationName` es opcional. **El prototipo lo pedia y lo tiraba**; aqui, si viene, se
  crea la organizacion con el usuario como owner en la misma transaccion. En F1, antes de
  que existan organizaciones (F3), el campo se acepta y se ignora con un aviso en la traza.
- El `username` no se pide: se deriva de la parte local del correo (`marta`) y se anade un
  sufijo numerico si esta cogido. Se cambia despues desde el perfil.
- Se manda el correo de verificacion (ver mas abajo). **No hace falta verificar para
  entrar**: la cuenta funciona, con dos limites hasta verificar (crear organizaciones e
  invitar), porque son las dos acciones que mandan correo a terceros o crean algo que otros
  ven.

| Codigo | Cuando                                                                                                                |
| ------ | --------------------------------------------------------------------------------------------------------------------- |
| 201    | Cuenta creada, sesion iniciada.                                                                                       |
| 409    | `email_taken`: el correo ya tiene cuenta.                                                                             |
| 422    | Campos invalidos, todos en la misma respuesta. `acceptedTerms` falso es un error de campo con `code: terms_required`. |
| 429    | Limite de frecuencia.                                                                                                 |

**Por que un 409 en el registro si eso dice quien tiene cuenta.** Es el mismo dato que da la
pantalla de login de cualquier producto con "esta cuenta ya existe", y la alternativa
(responder siempre 201 y mandar un correo "ya tienes cuenta") deja al usuario sin saber que
paso. Es una enumeracion que se acepta a sabiendas
([ADR 0008](adr/0008-autenticacion-con-tokens.md)): la acotan el limite de frecuencia y una
alarma sobre la tasa global de `email_taken`. `password/forgot` y el cambio de correo, en cambio,
responden siempre lo mismo, porque ahi no hay ninguna pantalla que necesite la diferencia.

### Login

```
POST /v1/auth/login
{ "email": "marta@ramal.dev", "password": "********" }
```

**200** con `{ "user": Me, "csrfToken": "..." }` y las cookies. **401**
`invalid-credentials` si no coincide, con el mismo texto exista o no el correo, y con el
calculo de Argon2id hecho tambien cuando la cuenta no existe (contra un hash fijo). Ese calculo
no esconde que el correo tiene cuenta (el registro ya lo dice); iguala el coste de cada intento
y hace que una cuenta creada con OAuth y sin contrasena responda exactamente igual que las
demas.

En login solo se exige que la contrasena no este vacia. El minimo de 8 es una regla de
registro; el prototipo lo aplicaba tambien al entrar, y eso impide entrar a quien tenga una
contrasena anterior a la regla.

### Refresco con rotacion y deteccion de reutilizacion

```
POST /v1/auth/refresh
{ "refreshToken": "rt_9f3c..." }        (nativa; la web no manda cuerpo y usa la cookie)
```

Nativa: **200** con un par nuevo:

```json
{
  "accessToken": "eyJ...",
  "tokenType": "Bearer",
  "expiresIn": 900,
  "refreshToken": "rt_4b1a...",
  "sessionId": "0192f3c4-..."
}
```

Web: **200** con `{ "csrfToken": "..." }` y las cookies renovadas.

- El refresh token es opaco, 256 bits en base64url con prefijo `rt_`. En la base solo esta su
  SHA-256.
- **Rota en cada uso.** El token presentado se marca rotado y apunta al que lo sustituye.
- **Presentar un token ya rotado revoca la sesion entera** (su familia): un refresh token se
  usa una vez, y verlo dos veces significa que alguien tiene una copia. Se echa a los dos.
- **Excepcion por carrera conocida: 30 segundos.** Si el token presentado se roto hace menos
  de 30 segundos y su sucesor todavia no se ha usado, la respuesta es **409**
  `refresh_superseded`, sin tokens y sin revocar nada. Existe por VS Code: cada ventana lanza
  su propio motor, todas comparten el mismo `SecretStorage`, y dos ventanas que despiertan a
  la vez presentan el mismo token. Sin esta ventana, abrir dos ventanas cerraria la sesion.
  El cliente que recibe el 409 vuelve a leer el token guardado (ya lo guardo la otra ventana)
  y reintenta. Un ladron que cae en esa ventana tampoco se lleva nada: el 409 no trae tokens.
- La caducidad es de la sesion, no del token: **inactividad** de 14 dias en la web y 90 en
  las apps nativas (cada refresco la alarga), y un **maximo absoluto** de 365 dias. Pasado
  cualquiera de los dos, 401 y a iniciar sesion.
- **El refresco con el token en el cuerpo es solo para los motores**: rechaza cualquier peticion
  que traiga cabecera `Origin` o cookies de Ramal. Un script en una pagina no puede usarlo.
- **Un 401 al refrescar es definitivo para el motor**: hace la misma limpieza que al cerrar
  sesion (ver [Logout](#logout)), aunque nadie pulse nada.

| Codigo | Cuando                                                                                        |
| ------ | --------------------------------------------------------------------------------------------- |
| 200    | Par nuevo.                                                                                    |
| 401    | Token desconocido, sesion revocada o caducada, o reutilizacion detectada (que ademas revoca). |
| 409    | `refresh_superseded`: otra ventana acaba de rotarlo.                                          |

### Logout

`POST /v1/auth/logout` revoca la sesion de quien llama y responde **204** siempre, tambien si
ya estaba revocada. Identifica la sesion por el `sid` del access token o, si este ya caduco,
por el refresh token del cuerpo o de la cookie. En la web borra las cookies. Si la sesion era
de un dispositivo, el dispositivo sale de la lista de sincronizados.

Lo que pasa con los secretos del dispositivo al cerrar sesion lo decide el cliente, y el
contrato lo fija asi: el motor borra la llave de la boveda cacheada y las copias
sincronizadas de tokens y llaves que no se crearon en ese equipo. Lo que el usuario creo en
ese equipo se queda en su llavero: sin cuenta, Ramal sigue siendo un cliente Git que necesita
esas credenciales para hacer push. La misma limpieza corre cuando un refresco recibe un 401
definitivo: un dispositivo revocado desde otro equipo no conserva las copias sincronizadas.

### Reautenticacion

```
POST /v1/auth/reauthenticate
{ "password": "********" }
```

**200** con cookies nuevas cuyo access token lleva `auth_time` de ahora. **401**
`invalid-credentials` si no coincide. Hay otras dos formas, para cuentas sin contrasena:

- **OAuth**: `POST /v1/auth/oauth/{provider}/start` con `intent: "reauth"`. La identidad que
  vuelve tiene que estar ya vinculada a ese usuario. Con Google se pide `prompt=login` y
  `max_age=0`, y se comprueba `auth_time` en el ID token. GitHub y GitLab no tienen forma de
  obligar a volver a entrar, asi que su reautenticacion vale para **reauth** pero no para
  **reauth fuerte**.
- **Enlace al correo**: `POST /v1/auth/reauthenticate/email-link` (sin cuerpo) manda al correo
  verificado un enlace de 15 minutos y un solo uso, `https://ramal.dev/reauthenticate#token=...`.
  `POST /v1/auth/reauthenticate/email-link/confirm { "token": "..." }` responde **204** y marca
  como reautenticada la sesion que lo pidio, aunque el enlace se abra en otro dispositivo: el
  token prueba el buzon y queda atado a esa sesion. El siguiente refresco de la pagina trae el
  `auth_time` nuevo. Vale para **reauth fuerte**.

El metodo de la ultima reautenticacion se guarda en la sesion, y un 403
`reauthentication_required` lleva en `acceptedMethods` los que valen para esa accion.

### Recuperar la contrasena

`POST /v1/auth/password/forgot { "email": "..." }` responde **202 con el cuerpo vacio
siempre**, exista o no el correo. Todo el comportamiento es el de Gastonomo, que ya pago por
aprenderlo:

- Enlace de 30 minutos, un solo uso, token guardado hasheado.
- Emitir uno invalida los anteriores; dos peticiones con menos de un minuto de diferencia no
  emiten ni mandan correo, para que nadie deje a otro sin recuperar la cuenta pidiendo el
  enlace en bucle. Ademas, como mucho 5 correos por hora y 10 al dia a la misma direccion.
- El correo sale de la peticion a una tarea de fondo, para que el tiempo de respuesta no
  delate la cuenta.

El enlace es `https://ramal.dev/reset-password#token=...`. **El token va en el fragmento**,
que el navegador no envia al servidor ni queda en los logs del gateway; la pagina lo lee y lo
manda en el cuerpo de:

```
POST /v1/auth/password/reset
{ "token": "...", "newPassword": "********" }
```

**204**. Cambia la contrasena, **marca el correo como verificado** (recibir el enlace prueba
que el buzon es suyo) y revoca todas las sesiones de la cuenta, tambien las de los
dispositivos. **400** `invalid_token` si no existe, caduco o ya se uso, con el mismo texto
para los tres casos. **422** si la contrasena no cumple.

Ademas limpia lo que un atacante pudo dejar preparado (preapropiacion, ver
[OAuth de identidad](#oauth-de-identidad-github-gitlab-y-google)):

- Borra siempre el cambio de correo pendiente.
- Si el correo **no estaba verificado** antes del restablecimiento, borra tambien las identidades
  vinculadas, los dispositivos y la boveda, y manda al correo un resumen de lo que quito. Quien
  registro el correo de otra persona pudo vincular su GitHub o crear una boveda; la duena real,
  que acaba de probar el buzon, empieza con una cuenta limpia.

Lo mismo pasa cuando aceptar una invitacion verifica el correo.

Las dos paginas (pedir el enlace y poner la nueva contrasena) no existen en el diseno, que
solo ensenaba un toast. Hay que disenarlas.

### Cambiar o establecer la contrasena

```
POST /v1/me/password
{ "currentPassword": "********", "newPassword": "********" }
```

**204**. Solo desde la web: Desktop y VS Code abren esta pagina. Revoca todas las sesiones menos
la de quien llama y manda un aviso al correo. Si la cuenta no tenia contrasena (solo OAuth),
`currentPassword` se omite y se exige **reauth** en su lugar: establecer la primera contrasena
de una cuenta es darle una puerta nueva. **401** si `currentPassword` no coincide, **422** si la
nueva no cumple.

### Verificar el correo

- Al registrarse, y al pedir reenvio, se manda un enlace de 48 horas:
  `https://ramal.dev/verify-email#token=...`.
- `POST /v1/auth/email/verify { "token": "..." }` responde **204** y marca el correo. Es
  publico: el token es la prueba, y verificar desde el movil sin haber iniciado sesion ahi es
  el caso normal. **400** `invalid_token` si no vale.
- `POST /v1/auth/email/verification` (con sesion, sin cuerpo) reenvia. **202** siempre;
  **409** `email_already_verified` si ya lo estaba. Como mucho 5 correos por hora y 10 al dia a
  la misma direccion.

### Cambiar el correo

```
POST /v1/me/email-change
{ "newEmail": "marta@acme.studio" }
```

**202** siempre, solo desde la web, con **reauth** y con el correo actual verificado (**403**
`email_not_verified` si no). Desktop y VS Code abren esta pagina con `openExternal`.

- Si la direccion nueva no tiene cuenta, le manda un enlace de 24 horas. Si ya la tiene, le manda
  un correo sin enlace que dice que alguien intento usar esa direccion: la respuesta no cambia,
  para que el formulario no diga que correos tienen cuenta.
- A la direccion vieja le manda un aviso con un enlace que, durante 7 dias, **deshace el cambio**:
  `POST /v1/auth/email-change/revert { "token": "..." }` responde **204**, cancela el cambio si
  estaba pendiente o devuelve el correo anterior si ya se aplico, y cierra todas las sesiones. Es
  la salida de quien pierde la cuenta porque otro cambio su correo.

El cambio no se aplica hasta confirmar: `POST /v1/auth/email-change/confirm { "token": "..." }`
responde **204**, cambia el correo y lo deja verificado. Hasta entonces `GET /v1/me` devuelve
`pendingEmail`. **409** `email_taken` si alguien registro la direccion entre medias.

El prototipo web dejaba editar el correo en un campo de texto y aplicarlo al instante. Sin
confirmacion, un error tipografico deja la cuenta atada a un buzon que no es de nadie, y la
recuperacion de contrasena deja de funcionar.

### OAuth de identidad: GitHub, GitLab y Google

Sirve para **entrar en Ramal**, no para conectar una cuenta de plataforma para Git. Son dos
cosas distintas y el contrato no las mezcla:

- El OAuth de identidad pide solo el perfil y el correo (`read:user user:email` en GitHub,
  `openid email profile` en GitLab y en Google). El token del proveedor se usa para leer el
  perfil y **se descarta en la misma peticion**: no se guarda.
- Conectar una cuenta de plataforma para fetch, push y pull requests pide scopes de
  repositorio, se hace desde el motor del dispositivo y el token acaba cifrado de extremo a
  extremo (ver [Cuentas de plataformas Git](#cuentas-de-plataformas-git)).

Solo GitLab.com, no instancias propias: cada instancia necesitaria su propia aplicacion
OAuth registrada, y la identidad de Ramal no puede depender de un servidor de terceros.

**Inicio.** `POST /v1/auth/oauth/{provider}/start { "intent": "login", "returnTo": "/dashboard" }`

- `provider`: `github`, `gitlab` o `google`. Otro valor es 404.
- `intent`: `login` (por defecto), `link` (vincular a la cuenta con sesion) o `reauth`. `link` y
  `reauth` exigen sesion web y la cabecera CSRF; `link` exige ademas **reauth** y el correo de la
  cuenta verificado (**403** `email_not_verified`), porque vincular otro metodo de entrada es dar
  una puerta nueva.
- `returnTo` se resuelve contra `https://ramal.dev` con un analizador WHATWG (el crate `url`) y
  tiene que dar el mismo origen; se guarda solo la ruta y la query. Comprobar "empieza por `/` y no
  por `//`" no basta: `/\evil.example` o `/%09/evil.example` pasan esa regla y el navegador los
  convierte en `//evil.example`. Cualquier otra cosa se sustituye por `/dashboard`.

Responde **200** con `{ "url": "https://github.com/login/oauth/authorize?..." }` y la pagina
navega ahi. Empieza con un `POST` y no con un `GET` para que ninguna navegacion de otro sitio
pueda lanzar una vinculacion con la cookie de sesion `Lax`. Pone una cookie `__Host-ramal_oauth`
con un valor aleatorio cuyo hash queda en el flujo. **La cookie ata el flujo al navegador que lo
empezo**: sin ella, un atacante podria empezar un flujo con su cuenta y hacer que la victima lo
terminara (login CSRF), y la victima acabaria trabajando dentro de la cuenta del atacante.

**Vuelta.** `GET /v1/auth/oauth/{provider}/callback?code=...&state=...`

Es una navegacion del navegador, asi que no responde `problem+json`: siempre redirige.

| Situacion                                                                                                               | Resultado                                                                |
| ----------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------ |
| La identidad ya esta vinculada a una cuenta                                                                             | Sesion iniciada, 302 a `returnTo`.                                       |
| No hay identidad, el correo del proveedor esta verificado y coincide con una cuenta cuyo correo tambien esta verificado | Se vincula y se inicia sesion. Los dos lados prueban control del buzon.  |
| Coincide con una cuenta cuyo correo **no** esta verificado                                                              | No se vincula. 302 a `/login?error=account_exists`.                      |
| El proveedor no da un correo verificado                                                                                 | 302 a `/login?error=email_unverified`.                                   |
| No hay ninguna cuenta con ese correo                                                                                    | Alta pendiente: cookie `__Host-ramal_signup` y 302 a `/signup/complete`. |
| `intent=link` y la identidad ya es de otra cuenta                                                                       | No se vincula. 302 a `returnTo` con `error=identity_in_use`.             |
| `intent=link` o `reauth` y la sesion actual no es la que empezo el flujo                                                | No pasa nada. 302 a `/login?error=oauth_failed`.                         |
| `intent=reauth` y la identidad no esta vinculada a ese usuario                                                          | No reautentica. 302 a `returnTo` con `error=reauth_failed`.              |
| El usuario rechazo el permiso o el proveedor fallo                                                                      | 302 a `/login?error=oauth_denied` u `oauth_failed`.                      |

**Por que no se vincula con una cuenta sin verificar.** Es el ataque de preapropiacion:
alguien registra con contrasena el correo de la victima y espera. Si el primer "Continuar con
Google" de la victima se vinculara a esa cuenta, el atacante seguiria entrando con su
contrasena. Asi, la victima recupera la cuenta con "La olvidaste", que verifica el correo,
cambia la contrasena, echa todas las sesiones del atacante y, como el correo no estaba
verificado, borra lo que el atacante pudo dejar vinculado (ver
[Recuperar la contrasena](#recuperar-la-contrasena)).

**Que es un correo verificado del proveedor.** Solo el que el proveedor marca como tal; sin
indicador, no verificado:

- GitHub: el principal y verificado de `/user/emails`, no el del perfil publico, que puede no
  traerlo o traer uno sin verificar.
- GitLab: `email_verified=true` en el ID token de `openid email profile`. `/api/v4/user` no trae
  ningun indicador de verificacion y por eso no se usa.
- Google: `email_verified` en el ID token.

Vincular una identidad y cambiar el correo exigen el correo de la cuenta verificado. Sin esa
regla, quien registrara primero el correo de otra persona podria vincular su propio GitHub o
pedir un cambio de correo a su buzon, y seguir entrando despues de que la duena real recupere la
cuenta con "La olvidaste".

**Completar el alta.** El prototipo creaba la cuenta al volver del proveedor y se saltaba los
terminos y el nombre de la organizacion. Aqui hay un paso mas:

- `GET /v1/auth/oauth/signup` (con la cookie de alta pendiente) devuelve
  `{ "provider": "github", "email": "...", "suggestedName": "Marta Ruiz" }`.
- `POST /v1/auth/oauth/signup` con
  `{ "name": "Marta Ruiz", "acceptedTerms": true, "termsVersion": "2026-09", "organizationName": null, "locale": "es" }`
  crea la cuenta con el correo ya verificado, vincula la identidad, inicia sesion y responde
  **201** con `{ "user": Me, "csrfToken": "..." }`. El alta pendiente caduca en 15 minutos;
  despues, **410** `signup_expired`.

El ticket viaja en una cookie y no en la URL por lo mismo que los enlaces de correo: nada que
de acceso a una cuenta debe quedar en un log o en el historial.

**Identidades vinculadas.** `GET /v1/me/identities` las lista
(`{ "items": [{ "provider": "github", "email": "...", "createdAt": "...", "lastUsedAt": "..." }] }`).
Vincular otra es el mismo inicio con `intent=link` (**reauth**, correo verificado).
`DELETE /v1/me/identities/{provider}` la desvincula (**reauth**); **409** `last_sign_in_method`
si es la unica forma de entrar y la cuenta no tiene contrasena. Vincular y desvincular mandan
un aviso al correo de la cuenta.

## Inicio de sesion de las apps nativas

**F1.** Desktop y VS Code inician sesion a traves de la web, con Authorization Code + PKCE
(RFC 7636, con las recomendaciones de RFC 8252 para apps nativas) y con el device flow
(RFC 8628) como alternativa. Es el paso que el diseno no dibuja en ningun sitio y sin el que
no hay sincronizacion: hacen falta la pagina "Autorizar Ramal Desktop en este equipo" y la
pagina para introducir el codigo del device flow.

Los clientes estan registrados en configuracion, no en la base:

| client_id       | Redirecciones aceptadas                                                                                                                                             |
| --------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `ramal-desktop` | Exactamente `http://127.0.0.1:{puerto}/callback` o `http://[::1]:{puerto}/callback`, con puerto entre 1024 y 65535 (RFC 8252 permite cualquier puerto en loopback). |
| `ramal-vscode`  | Exactamente `{esquema}://ramal.ramal/auth-callback` con esquema `vscode`, `vscode-insiders`, `vscodium`, `cursor` o `windsurf`.                                     |

**La redireccion se valida analizandola, nunca con un prefijo.** Con un prefijo,
`http://127.0.0.1:1@evil.example/callback` pasaria la comprobacion y su host real seria
`evil.example`: un atacante mandaria a una victima con sesion el enlace de autorizar con su
propio reto PKCE y se quedaria con el codigo. Un solo validador, el mismo para este login y para
el [broker de plataformas](#el-flujo-con-broker), analiza la URL con el crate `url` y exige:

- esquema `http` y host exactamente `127.0.0.1` o `[::1]`;
- usuario y contrasena vacios, y puerto explicito entre 1024 y 65535;
- ruta exactamente `/callback`, sin query ni fragmento;
- que la URL vuelta a serializar sea identica a la recibida.

Para VS Code: esquema de la lista, host `ramal.ramal`, ruta `/auth-callback` y nada mas. La
redireccion final se construye desde la URL ya analizada, anadiendo `code` y `state` con la API
de `url`, nunca concatenando texto.

**Por que loopback en Desktop y no `ramal://`.** Los deep links de Tauri necesitan el plugin
de instancia unica en Windows y Linux, solo funcionan con la app instalada y en macOS solo
desde `/Applications`. Como canal de login son fragiles; `ramal://` queda para abrir un
repositorio desde la web o desde VS Code.

**Por que una lista de esquemas en VS Code y no la URL que devuelva `asExternalUri`.** En la
version de navegador de VS Code (Codespaces en el navegador) `asExternalUri` devuelve una URL
`https` de un dominio ajeno que no se puede validar; aceptarla seria una redireccion abierta
con un codigo de autorizacion dentro. En esos entornos el cliente usa el device flow. En VS
Code de escritorio, tambien con Remote SSH, WSL o Dev Containers, el esquema `vscode://`
funciona y el editor lo reenvia a la extension remota.

### Authorization Code + PKCE

1. El motor genera `code_verifier`, `code_challenge` (S256) y `state`, abre el navegador del
   sistema en la pagina web `https://ramal.dev/authorize/app` con los parametros estandar de
   OAuth (`client_id`, `redirect_uri`, `code_challenge`, `code_challenge_method=S256`,
   `state`) mas `device_name` para ensenarlo. Estos parametros van en snake_case porque los
   produce cualquier libreria OAuth (el crate `oauth2` en el motor).
2. La pagina exige sesion web (si no la hay, pasa por login o registro y vuelve) con un inicio
   de sesion real de **menos de 5 minutos**; si es mas antiguo, el `GET` responde 403
   `reauthentication_required` y la pagina pide confirmar la contrasena o repetir el OAuth antes
   de ensenar nada. Una sesion web robada o un script inyectado no aprueban
   una sesion nativa sin esa confirmacion. Luego valida la peticion:

   ```
   GET /v1/auth/app/authorize?client_id=ramal-desktop&redirect_uri=...&code_challenge=...&code_challenge_method=S256&state=...
   ```

   **200** con lo que la pagina ensena:

   ```json
   {
     "client": { "id": "ramal-desktop", "name": "Ramal Desktop" },
     "deviceName": "MacBook Pro de Marta",
     "user": { "name": "Marta Ruiz", "email": "marta@ramal.dev" }
   }
   ```

   `deviceName` es el nombre que declara el equipo, y la pagina lo presenta asi, sin caracteres
   de control ni de direccion de texto. **400** `invalid_client`, `invalid_redirect_uri` o
   `invalid_challenge`. Con un `redirect_uri` invalido la pagina **nunca redirige**: ensena el
   error. Redirigir un error a una URL no validada es justo la redireccion abierta que se quiere
   evitar.

3. El usuario pulsa "Autorizar" o "Cancelar". Es un **formulario** de nivel superior, no una
   llamada de JavaScript:

   ```
   POST /v1/auth/app/authorize
   Content-Type: application/x-www-form-urlencoded

   client_id=ramal-desktop&redirect_uri=http%3A%2F%2F127.0.0.1%3A53123%2Fcallback&code_challenge=...&state=...&approve=true&csrf_token=...
   ```

   **302** a `http://127.0.0.1:53123/callback?code=...&state=...`. Si `approve` es `false`, la
   redireccion lleva `error=access_denied`. El token CSRF va como campo porque un formulario no
   puede poner cabeceras, y la API comprueba tambien `Origin`. El codigo **nunca aparece en un
   cuerpo de respuesta** que un script de la pagina pueda leer. Vale **60 segundos** y un solo
   uso.

4. El motor canjea el codigo:

   ```
   POST /v1/auth/app/token
   {
     "grantType": "authorization_code",
     "clientId": "ramal-desktop",
     "code": "...",
     "codeVerifier": "...",
     "redirectUri": "http://127.0.0.1:53123/callback",
     "device": {
       "installationId": "0192f3c4-7a1e-7c3b-9e0d-5f4a2b1c8d77",
       "kind": "desktop",
       "name": "MacBook Pro de Marta",
       "os": "macos",
       "osVersion": "15.1",
       "arch": "arm64",
       "appVersion": "2.4.0",
       "deviceSecret": "base64url(32 bytes)"
     }
   }
   ```

   El canje rechaza cualquier peticion con cabecera `Origin` o con cookies de Ramal: lo hace un
   motor, nunca un navegador.

   **200**:

   ```json
   {
     "accessToken": "eyJ...",
     "tokenType": "Bearer",
     "expiresIn": 900,
     "refreshToken": "rt_9f3c...",
     "sessionId": "0192f3c5-...",
     "device": { "...": "Device, ver Dispositivos" },
     "user": { "...": "Me, ver Perfil" }
   }
   ```

**El dispositivo se registra aqui**, al canjear: es el "registrar al iniciar sesion" que pide
el panel web. `installationId` y `deviceSecret` los genera el cliente una vez por instalacion;
el secreto (256 bits) vive en el llavero y la API guarda solo su hash. Con los dos, volver a
iniciar sesion en la misma maquina reutiliza la fila del dispositivo (conserva el nombre que el
usuario le puso) en vez de dejar un fantasma por cada login. El `installationId` solo no basta:
no es secreto, y quien lo conociera se quedaria con la fila de otro. Un dispositivo **revocado**
no se reactiva nunca: un inicio de sesion nuevo en esa instalacion crea otra fila. Si ese
dispositivo tenia otra sesion viva, se revoca (`replaced`): un dispositivo tiene como mucho
una sesion.

La sesion nueva **hereda el `auth_time` de la sesion web que la aprobo** y nunca cuenta como
reautenticacion reciente (ver [Como se autentica cada peticion](#como-se-autentica-cada-peticion)).
Cada dispositivo nuevo manda al correo de la cuenta el aviso "Nuevo dispositivo autorizado", con
el nombre que declara el equipo y un enlace a la pagina de dispositivos para revocarlo.

Errores del canje, todos **400** con `code`: `invalid_grant` (codigo desconocido, caducado,
de otro cliente, con otra `redirect_uri` o con un verificador que no casa). **Si un codigo ya
usado se presenta otra vez, ademas se revoca la sesion que se emitio con el**: segun RFC 6749
es la senal de que alguien intercepto el codigo.

### Device flow

Para VS Code en el navegador, entornos sin navegador, o cuando un firewall bloquea el puerto
de loopback.

1. `POST /v1/auth/device/code { "clientId": "ramal-vscode", "device": { ...como arriba } }`,
   sin `Origin` ni cookies, como el canje. 10 por minuto e IP, y un tope global de flujos
   pendientes.

   ```json
   {
     "deviceCode": "dc_7e21...",
     "userCode": "WDJB-MJHT",
     "verificationUri": "https://ramal.dev/activate",
     "expiresIn": 600,
     "interval": 5
   }
   ```

   **No hay `verificationUriComplete`.** Un enlace con el codigo ya puesto es lo que usa el
   phishing del device flow: el atacante empieza un flujo, pone el nombre del equipo de la
   victima y le manda el enlace para que solo tenga que pulsar "Aprobar". El usuario escribe en
   la pagina el codigo que ve en su propio equipo.

   El codigo de usuario son 8 caracteres de un alfabeto sin vocales ni caracteres que se
   confundan (`BCDFGHJKLMNPQRSTVWXZ`), como recomienda RFC 8628. Sin vocales no se forman
   palabras, y sin `0/O` ni `1/I` no hay erratas al teclearlo.

2. En `https://ramal.dev/activate` el usuario inicia sesion y escribe el codigo. La pagina
   pregunta que es:

   `POST /v1/auth/device/lookup { "userCode": "WDJB-MJHT" }` responde **200** con
   `{ "client": {...}, "device": { "declaredName": "VS Code en ubuntu-dev", "os": "linux" }, "requestedAgoSeconds": 42, "requestedFrom": { "city": "Madrid", "country": "ES" }, "expiresAt": "..." }`
   o **404** si no existe o caduco. La pagina ensena "Pedido hace 42 segundos", el nombre como
   "nombre que declara el equipo" (sin caracteres de control ni de direccion de texto) y, si
   GeoIP esta activo, la ciudad y el pais, con un aviso si no coinciden con los del navegador que
   aprueba; sin GeoIP, `requestedFrom` es `null`. Limites: 10 por minuto por usuario y, aparte,
   por IP, y 50 codigos fallidos por hora y usuario. Un limite por sesion no bastaria: una sesion
   nueva lo esquiva.

3. `POST /v1/auth/device/decision { "userCode": "WDJB-MJHT", "approve": true }` responde
   **204**. Como en la pagina de autorizar, exige un inicio de sesion real de menos de 5 minutos.

   Aprobar no es inocuo para quien aprueba: un codigo pendiente ajeno que alguien acierte y
   apruebe mete el equipo de otra persona en la cuenta de quien aprueba, que empezaria a subir
   alli sus metadatos. Por eso el codigo hay que escribirlo, y el limite de fallos por usuario.

4. Mientras, el motor sondea cada `interval` segundos:

   `POST /v1/auth/app/token { "grantType": "device_code", "clientId": "ramal-vscode", "deviceCode": "dc_7e21..." }`

   Responde **200** con el mismo cuerpo que el canje de codigo cuando se aprobo, o **400** con
   los `code` de RFC 8628: `authorization_pending`, `slow_down` (sondeo mas rapido que el
   intervalo; el intervalo sube 5 segundos), `access_denied` o `expired_token`. Consumir la
   aprobacion es un `UPDATE ... SET status = 'consumed' WHERE status = 'approved' RETURNING`: dos
   sondeos a la vez no se llevan dos sesiones, y el segundo recibe `invalid_grant`.

Aqui los 400 llevan `code` aunque no sean conflictos: son los estados que define el
estandar, y cualquier cliente OAuth los espera con ese nombre.

## Sesiones activas

**F1.** Una sesion es un inicio de sesion en un cliente: una familia de refresh tokens. Es lo
que lista "Sesiones activas" en los ajustes de Desktop. Los dispositivos son otra vista del
mismo hecho y tienen su propia seccion.

```
GET /v1/auth/sessions
{
  "items": [
    {
      "id": "0192f3c5-...",
      "client": "desktop",
      "current": true,
      "device": { "id": "...", "name": "MacBook Pro de Marta", "kind": "desktop", "os": "macos", "osVersion": "15.1", "appVersion": "2.4.0" },
      "browser": null,
      "location": { "city": "Madrid", "country": "ES" },
      "createdAt": "2026-09-30T08:12:00Z",
      "lastUsedAt": "2026-09-30T10:02:00Z"
    },
    {
      "id": "0192f3c6-...",
      "client": "web",
      "current": false,
      "device": null,
      "browser": "Firefox en Linux",
      "location": null,
      "createdAt": "...",
      "lastUsedAt": "..."
    }
  ]
}
```

- Solo sesiones vivas: sin revocar y sin caducar.
- `current` marca la de quien pregunta, por el `sid` del token. Es la pastilla "Este
  dispositivo" del diseno.
- `browser` es un resumen del `User-Agent` (familia y sistema), no la cadena entera.
- **`location` es opcional y viene apagado por defecto.** El diseno ensena una ciudad; eso
  exige geolocalizar la IP con una base GeoIP, y es un dato personal que hay que justificar.
  Si se activa (`RAMAL__SESSIONS__GEOIP_DATABASE`), se guarda solo ciudad y pais al crear la
  sesion, nunca la IP. Si no, `null` y la interfaz ensena solo el dispositivo y la hora.

`DELETE /v1/auth/sessions/{id}` revoca una sesion (el boton "Cerrar"). **204**; **404** si no
es tuya. Revocar la propia equivale a cerrar sesion. Si la sesion era de un dispositivo, el
dispositivo tambien queda revocado.

`POST /v1/auth/sessions/revoke-others` revoca todas menos la actual (el boton "Cerrar todas
las demas"). **204**, y un aviso al correo de la cuenta.

Si una sesion revocada era de un dispositivo que llego a descargar la boveda, la API marca la
boveda con "rotacion recomendada" y todos los dispositivos ensenan "Rotar la llave de la boveda"
hasta que alguno rote (ver [La boveda](#la-boveda-de-extremo-a-extremo)).

**Lo que la revocacion no puede hacer.** Corta el acceso futuro: el siguiente refresco falla
y, en los endpoints **viva**, el siguiente acceso tambien. Pero un dispositivo robado ya
tiene en su llavero los tokens que descifro. La revocacion no borra nada remoto. Por eso,
cuando se revoca un dispositivo, la interfaz debe recomendar rotar los tokens de las
plataformas que usaba. Es honesto decirlo; prometer un borrado remoto seria mentir.

## Avisos de seguridad

**F1.** No hay segundo factor todavia ([Lo que no existe todavia](#lo-que-no-existe-todavia)), y
una sesion se puede conseguir sin que el usuario lo note: con un codigo de dispositivo
conseguido con phishing o con una sesion web robada. La senal que le queda es el correo. Cada uno
de estos hechos manda un aviso al correo de la cuenta y deja un evento de auditoria personal
(`audit_events` con `organization_id` nulo):

| Hecho                                                                  | Aviso                                                       |
| ---------------------------------------------------------------------- | ----------------------------------------------------------- |
| Un dispositivo nuevo inicia sesion                                     | "Nuevo dispositivo autorizado", con enlace a Dispositivos   |
| Se vincula o se quita una identidad                                    | Proveedor y fecha                                           |
| Se establece o se cambia la contrasena                                 | Fecha y que sesiones se cerraron                            |
| Se pide un cambio de correo                                            | A la direccion anterior, con el enlace de deshacer (7 dias) |
| "Cerrar todas las demas sesiones"                                      | Cuantas se cerraron                                         |
| Boveda: crear, cambiar la frase, regenerar el codigo, rotar, reiniciar | La operacion y desde que dispositivo                        |
| Primera descarga de la boveda en un dispositivo                        | "Un equipo nuevo descargo tu boveda"                        |

Ningun aviso lleva secretos. El unico enlace que actua sin sesion es el de deshacer el cambio de
correo, y es de un solo uso.

## Perfil y cuenta

**F1**, la foto en **F2**.

### Leer el perfil

```
GET /v1/me
{
  "id": "0192f3c0-...",
  "name": "Marta Ruiz",
  "username": "marta",
  "email": "marta@ramal.dev",
  "emailVerified": true,
  "pendingEmail": null,
  "timezone": "Europe/Madrid",
  "locale": "es",
  "avatarUrl": "https://ramal.dev/v1/avatars/5b0c2e8a-...",
  "hasPassword": true,
  "vaultEnabled": true,
  "vaultCreatedAt": "2026-09-05T10:00:00Z",
  "identities": [{ "provider": "github", "email": "marta@ramal.dev" }],
  "organizations": [{ "id": "0192f3d0-...", "name": "ACME Studio", "role": "owner" }],
  "createdAt": "2026-09-01T09:00:00Z"
}
```

Es lo que pide la guarda de rutas de la web, el saludo "Hola, {nombre}", las iniciales del
avatar, la tarjeta de cuenta de VS Code (nombre, correo, organizacion) y el perfil de
Desktop. `organizations` va resumido aqui porque las tres superficies lo pintan en la
cabecera y seria una segunda peticion en cada arranque; el detalle (asientos, plan) esta en
`/v1/organizations`.

`vaultEnabled` y `vaultCreatedAt` (nulo sin boveda) son lo unico que la web sabe de la boveda:
le bastan para ensenar si esta configurada y desde cuando, "Reiniciar la boveda" y el aviso de
que las cuentas se sincronizan sin secretos. La cabecera completa de la boveda (envoltorios y
parametros) solo la leen los clientes nativos.

Las iniciales y el color del avatar los deriva el cliente, como en el prototipo.

### Editar el perfil

```
PATCH /v1/me
{ "name": "Marta Ruiz", "username": "mruiz", "timezone": "America/Bogota", "locale": "es" }
```

**200** con `Me`. Cualquier subconjunto de los cuatro campos.

- `username`: de 3 a 39 caracteres, minusculas, cifras y guiones, sin guion al principio ni
  al final. Se guarda en minusculas. **409** `username_taken`. Hay una lista de nombres
  reservados (`admin`, `api`, `ramal`, `support`...) que responde igual que uno cogido.
- `timezone`: un nombre IANA que exista en la base de zonas del servidor. La lista del
  prototipo (seis zonas) es un ejemplo, no un catalogo.
- `locale`: `es` o `en`. Es el idioma de los correos y el de arranque de la web. El idioma de
  la interfaz de las apps es una preferencia de apariencia y se sincroniza aparte.
- El correo no se cambia aqui: tiene su propio flujo con confirmacion.

Editar el perfil no es "guardar un borrador": el boton "Guardar cambios" de Desktop manda
este `PATCH`, y la web hace lo mismo con los campos de nombre.

### Foto de perfil

- `PUT /v1/me/avatar` con el cuerpo en binario (`image/png`, `image/jpeg` o `image/webp`,
  hasta 2 MiB). **200** con `{ "avatarUrl": "..." }`.
- El servidor **vuelve a codificar** la imagen a WebP de 256x256 y descarta todos los
  metadatos. No es cosmetico: las fotos de movil llevan EXIF con coordenadas GPS.
- **2 MiB comprimidos pueden ser gigas al decodificar.** Antes de decodificar se leen las
  dimensiones y se rechaza (**415**) una imagen de mas de 4096 pixeles por lado o de mas de 16
  millones de pixeles, o animada; el decodificador corre con `image::Limits` (`max_alloc` de 64
  MiB), en `spawn_blocking` y con 5 segundos de tope.
- La imagen se sirve con `X-Content-Type-Options: nosniff` y
  `Content-Security-Policy: default-src 'none'`.
- `DELETE /v1/me/avatar` la quita. **204**.
- `GET /v1/avatars/{id}` sirve la imagen **sin autenticacion**, con cache larga. El `id` es
  aleatorio (122 bits) y cambia con cada foto nueva: es una URL de capacidad.

**Por que publica.** La foto la tienen que pintar la web (`<img>`), los webviews de Desktop y
VS Code (que no pueden poner una cabecera `Authorization` en un `<img>`) y los companeros de
organizacion. Con una URL de capacidad todos la cargan igual, y cambiarla invalida la
anterior. La alternativa, que el motor la descargara y la pasara como `data:` URL, complica
tres clientes para proteger una foto que el usuario ya decidio ensenar a su equipo.

**415** si el tipo no es de imagen o no se puede decodificar, **413** si pasa de 2 MiB.

### Exportar los datos

`GET /v1/me/export` responde **200** con `Content-Disposition: attachment; filename="ramal-export-2026-09-30.json"`:

```json
{
  "exportedAt": "2026-09-30T10:00:00Z",
  "formatVersion": 1,
  "user": { "...": "Me sin avatarUrl" },
  "identities": [],
  "sessions": [],
  "devices": [],
  "organizations": [{ "id": "...", "name": "ACME Studio", "role": "owner", "joinedAt": "..." }],
  "workspaces": [],
  "repositories": [],
  "linkedAccounts": [],
  "aiKeys": [],
  "syncSettings": {},
  "preferences": { "git": {}, "appearance": {}, "ai": {} },
  "vault": { "enabled": true, "createdAt": "..." }
}
```

- Es sincrono: con los volumenes de una cuenta cabe en una respuesta. Si algun dia no cabe,
  pasa a una tarea con enlace de descarga.
- **Los secretos no se exportan**, ni siquiera cifrados. Sin la llave de la boveda el
  ciphertext no sirve para nada, y un fichero de exportacion acaba en carpetas de descargas y
  copias de seguridad. De cada cuenta y cada llave va el metadato y `hasSecret`.
- Es **viva**: un fichero con todos los datos de la cuenta no sale para una sesion revocada.

Cubre el derecho de portabilidad y el boton "Exportar configuracion" del panel.

### Eliminar la cuenta

```
DELETE /v1/me
{ "confirmation": "marta@ramal.dev" }
```

**204** y cookies borradas. Exige **reauth fuerte** (contrasena, Google o enlace al correo) y
que `confirmation` sea el correo de la cuenta: el prototipo lo hacia con un solo clic de
confirmacion, y es la unica accion de la API que no tiene vuelta atras. Desde Desktop se abre
esta pagina de la web.

- **409** `sole_owner_of_organization` con `organizations: [{ id, name }]` si es la unica
  owner de una organizacion **con otros miembros**. Tiene que transferirla o eliminarla antes.
  Las organizaciones donde es el unico miembro se eliminan con la cuenta.
- Se borra todo en una transaccion: sesiones, dispositivos, boveda y secretos, cuentas de
  plataforma, llaves, repositorios, workspaces personales y los privados que tuviera en
  organizaciones. Los workspaces que compartio en una organizacion **se quedan en la
  organizacion**, sin dueno, y los editan los admins: son trabajo del equipo.
- **Borrado real, no logico.** No hay periodo de gracia: el diseno dice "Esta accion es
  irreversible" y el borrado inmediato es lo que se espera de una peticion de supresion. El
  unico rastro que queda es el de auditoria de las organizaciones, con el identificador y sin
  correo ni nombre.
- Los repositorios en disco no se tocan, como dice el diseno.

### Resumen del panel

`GET /v1/overview?organizationId=0192f3d0-...` (o `organizationId=personal`, o sin parametro,
que es personal):

```json
{
  "workspaces": 3,
  "repositories": 11,
  "linkedAccounts": 7,
  "members": 4,
  "pendingInvitations": 1,
  "devices": 3
}
```

Alimenta las cuatro tarjetas del resumen y los contadores del menu lateral sin pedir cuatro
listas enteras. En el contexto personal `members` es 1 y `pendingInvitations` 0.
`repositories` y `linkedAccounts` son siempre los personales: no dependen del contexto.

El diseno contaba las invitaciones pendientes como miembros. Aqui van separadas y la
interfaz decide si las suma; sumarlas en el dato impediria ensenarlas distinto.

## Dispositivos

**F1.** Un dispositivo es una instalacion de Ramal Desktop o de la extension de VS Code con
sesion iniciada. Es la tarjeta "Dispositivos sincronizados" del resumen web.

```
GET /v1/devices
{
  "items": [
    {
      "id": "0192f3c1-...",
      "kind": "desktop",
      "name": "MacBook Pro de Marta",
      "os": "macos",
      "osVersion": "15.1",
      "arch": "arm64",
      "appVersion": "2.3.1",
      "latestVersion": "2.4.0",
      "updateAvailable": true,
      "current": false,
      "lastSyncAt": "2026-09-30T09:58:00Z",
      "lastSeenAt": "2026-09-30T09:58:00Z",
      "createdAt": "2026-09-01T09:05:00Z"
    }
  ]
}
```

- Solo los que tienen una sesion viva. Cerrar sesion en un equipo lo saca de la lista.
- `lastSyncAt` lo actualiza cada lectura del feed de cambios, como mucho una vez por minuto
  por dispositivo. No hay un endpoint de latido aparte: si un dispositivo sincroniza, esta
  vivo, y si no, "Sincronizado hace 3 d" es exactamente lo que hay que ensenar.
- `latestVersion` y `updateAvailable` salen de las publicaciones del canal estable. Es la
  pista "version desactualizada" que el analisis echaba en falta (el PC de la oficina con la
  2.3.1 mientras la actual es la 2.4.0), y la API tiene los dos datos.
- La abreviatura (`MAC`, `WIN`, `VS`), el color del punto (menos de 60 minutos) y el texto
  relativo los deriva el cliente.

`PATCH /v1/devices/{id} { "name": "Portatil de casa" }` lo renombra (1 a 80 caracteres).
**200**.

`PATCH /v1/devices/current { "appVersion": "2.4.1", "osVersion": "15.2" }`, solo desde una
sesion nativa, lo manda el motor despues de actualizarse. **200**.

`DELETE /v1/devices/{id}` es "Cerrar sesion en este dispositivo": revoca su sesion, marca el
dispositivo como revocado (para siempre: si vuelve a iniciar sesion sera otra fila) y lo saca
de la lista. **204**; **404** si no es tuyo. El diseno lo hacia sin confirmacion; la API no
la exige, pero la interfaz deberia pedirla, porque el dispositivo pierde la sincronizacion. Si
el dispositivo habia descargado la boveda, la boveda queda con "rotacion recomendada".

El nombre por defecto lo propone el cliente a partir del nombre del equipo. El nombre del
equipo puede ser personal ("PC de Marta Ruiz Gonzalez"); por eso es editable y por eso el
cliente lo ensena antes de iniciar sesion en la pagina de autorizacion.

## Organizaciones y contexto personal

**F3.** El selector del panel ensena "ACME Studio", "Personal" y "Crear organizacion". Un
usuario puede pertenecer a varias organizaciones y siempre tiene su contexto personal.

```
GET /v1/organizations
{
  "items": [
    {
      "id": "0192f3d0-...",
      "name": "ACME Studio",
      "role": "owner",
      "seats": { "total": 5, "used": 5, "activeMembers": 4, "pendingInvitations": 1 },
      "plan": null,
      "createdAt": "2026-09-02T10:00:00Z"
    }
  ]
}
```

- `role` es el de quien pregunta. Las iniciales del cuadrado del selector (`AS`) las deriva
  el cliente del nombre.
- `plan` es `null` hasta F5. Con facturacion sera
  `{ "id": "pro", "name": "Ramal Pro", "status": "active", "renewsAt": "2026-10-12T00:00:00Z" }`,
  que es la tarjeta de plan de Desktop.
- El contexto personal no aparece: el cliente anade la entrada "Personal" siempre.

`POST /v1/organizations { "name": "ACME Studio" }` (acepta `Idempotency-Key`) crea la
organizacion con quien llama como owner. **201** con la organizacion. **403**
`email_not_verified` si el correo no esta verificado. **409** `limit_reached` si ya creo 5.
Nombre de 1 a 60 caracteres, sin caracteres de control ni de direccion de texto: el nombre
aparece en los correos de invitacion, que van como texto plano y nunca convierten el nombre en
un enlace, para que una organizacion no sirva para mandar spam con la firma de Ramal.

`GET /v1/organizations/{orgId}` devuelve una. **404** si no eres miembro.

`PATCH /v1/organizations/{orgId} { "name": "ACME" }` (admin) la renombra.

`DELETE /v1/organizations/{orgId}` (owner, **reauth**, solo web, cuerpo `{ "confirmation": "ACME Studio" }`)
la elimina con todo lo suyo: miembros, invitaciones, workspaces de la organizacion, llaves de
IA de organizacion y su consumo. En F5 exige ademas que no haya una suscripcion activa
(**409** `active_subscription`).

`POST /v1/organizations/{orgId}/leave` sale de la organizacion. **204**. **409**
`owner_cannot_leave` si eres owner: primero se transfiere.

`POST /v1/organizations/{orgId}/transfer-ownership { "userId": "..." }` (owner, **reauth**,
solo web) convierte en owner a otro miembro y a quien llama en admin, en una transaccion. **200**
con la organizacion. **422** si el usuario no es miembro. Desktop abre esta pagina de la web para
transferir o eliminar, porque una sesion nativa no cuenta como reautenticacion.

**Una sola owner por organizacion.** El diseno la ensena bloqueada y unica. Varias owners
reparten la responsabilidad de facturar y de eliminar la organizacion, y no hay ninguna
pantalla que lo pida. Lo fuerza un indice unico parcial en la base.

### Que cambia con el contexto

| Dato                          | Contexto personal                | Contexto de organizacion                                               |
| ----------------------------- | -------------------------------- | ---------------------------------------------------------------------- |
| Workspaces                    | Los personales                   | Los compartidos de la organizacion y los privados tuyos dentro de ella |
| Toggle "Compartido con {org}" | No aparece                       | Aparece                                                                |
| Repositorios                  | Los tuyos                        | Los tuyos (los repositorios son siempre personales)                    |
| Cuentas de plataforma         | Las tuyas                        | Las tuyas (no hay cuentas de equipo)                                   |
| Equipo y asientos             | Llamada a crear una organizacion | Miembros, invitaciones y asientos                                      |
| Llaves de IA                  | Las personales                   | Las personales y las de la organizacion                                |

El analisis del diseno pedia definir que se ve en "Personal" en Equipo y Cuentas; la
respuesta es esta tabla. La pantalla de Equipo en contexto personal no existe en el diseno y
hay que dibujarla.

## Miembros y roles

**F3.**

```
GET /v1/organizations/{orgId}/members?cursor=&limit=
{
  "items": [
    {
      "userId": "0192f3c0-...",
      "name": "Marta Ruiz",
      "username": "marta",
      "email": "marta@ramal.dev",
      "avatarUrl": null,
      "role": "owner",
      "joinedAt": "2026-09-02T10:00:00Z",
      "isYou": true
    }
  ],
  "nextCursor": null
}
```

Ordenados por rol (owner, admins, miembros) y despues por nombre. Todos los miembros pueden
ver la lista: el diseno no dibuja una vista de solo lectura, pero un miembro tiene que saber
con quien comparte workspaces.

`PATCH /v1/organizations/{orgId}/members/{userId} { "role": "admin" }` (admin) cambia el rol
entre `admin` y `member`. **409** `owner_role_locked` si el objetivo es la owner (se cambia
con la transferencia). **422** si el rol es `owner`.

`DELETE /v1/organizations/{orgId}/members/{userId}` (admin) lo quita. **204**. **409**
`owner_role_locked` si es la owner. Al quitar a alguien:

- Sus workspaces privados dentro de la organizacion se eliminan: eran suyos, pero estaban
  en un contexto al que ya no pertenece.
- Los workspaces que compartio **se quedan**, sin dueno, editables por los admins. Esto es
  lo que obliga a guardar los repositorios de un workspace compartido por su remoto y no por
  el registro personal de quien los anadio (ver [Workspaces](#workspaces)).
- Sus dispositivos dejan de recibir datos de la organizacion en la siguiente sincronizacion:
  el feed les manda la baja y el cliente purga lo de ese contexto.
- Deja de poder usar las llaves de IA de la organizacion en la siguiente peticion (el proxy
  comprueba la pertenencia en cada una).

### La matriz de roles

| Accion                                                                | owner | admin | member |
| --------------------------------------------------------------------- | :---: | :---: | :----: |
| Ver la organizacion, miembros, asientos y llaves de IA (enmascaradas) |  si   |  si   |   si   |
| Crear workspaces en la organizacion (privados o compartidos)          |  si   |  si   |   si   |
| Editar o borrar un workspace compartido **propio**                    |  si   |  si   |   si   |
| Editar o borrar cualquier workspace compartido                        |  si   |  si   |   no   |
| Usar las llaves de IA de la organizacion (proxy)                      |  si   |  si   |   si   |
| Invitar, reenviar y revocar invitaciones                              |  si   |  si   |   no   |
| Cambiar roles entre admin y member, quitar miembros                   |  si   |  si   |   no   |
| Gestionar llaves de IA de la organizacion y su politica, ver consumo  |  si   |  si   |   no   |
| Renombrar la organizacion                                             |  si   |  si   |   no   |
| Facturacion, transferir, eliminar la organizacion                     |  si   |  no   |   no   |

Un admin puede cambiar el rol de otro admin y quitarlo. No puede tocar a la owner. Sin la
jerarquia completa de permisos que no pide ninguna pantalla, esta tabla es la regla entera.

## Invitaciones

**F3.**

```
POST /v1/organizations/{orgId}/invitations
{ "email": "lucia@acme.studio", "role": "member" }
```

**201** con la invitacion:

```json
{
  "id": "0192f3e0-...",
  "email": "lucia@acme.studio",
  "role": "member",
  "invitedBy": { "userId": "0192f3c0-...", "name": "Marta Ruiz" },
  "createdAt": "2026-09-30T10:00:00Z",
  "expiresAt": "2026-10-07T10:00:00Z",
  "lastSentAt": "2026-09-30T10:00:00Z",
  "expired": false
}
```

Manda un correo con `https://ramal.dev/invitations#token=...`. El token son 256 bits
aleatorios y en la base solo esta su hash. Caduca a los 7 dias. El correo lleva el nombre de la
organizacion y el de quien invita como texto plano, sin enlaces ni caracteres de control o de
direccion de texto. Ademas del limite por organizacion, quien invita puede mandar como mucho 20
invitaciones al dia sumando todas sus organizaciones.

**El asiento se comprueba con la organizacion bloqueada.** Invitar, reenviar una invitacion
caducada, aceptar y cambiar el numero de asientos hacen primero
`SELECT ... FROM organizations WHERE id = $1 FOR UPDATE` y despues cuentan. Sin el cerrojo, dos
invitaciones a la vez veran el mismo asiento libre y la organizacion acabaria con uno de mas.

| Codigo | Cuando                                                                                                                                                |
| ------ | ----------------------------------------------------------------------------------------------------------------------------------------------------- |
| 201    | Invitacion creada y correo en cola.                                                                                                                   |
| 403    | `insufficient_role` (no eres admin) o `email_not_verified`.                                                                                           |
| 409    | `already_member`, `invitation_pending` (ya hay una viva para ese correo; se reenvia, no se duplica) o `no_seats_available`.                           |
| 422    | Correo invalido, o rol distinto de `admin` y `member`. El correo se valida con la misma regla que el registro, no con "contiene @" como el prototipo. |

**`no_seats_available` y no una compra silenciosa.** El prototipo subia el numero de asientos
al invitar por encima del limite. En produccion eso es cobrar sin confirmacion. La API
rechaza y la interfaz ofrece "Anadir asientos", que en F5 es una compra con precio a la
vista.

**Las invitaciones pendientes ocupan asiento**, como en el diseno. Si no lo ocuparan,
aceptar una invitacion podria fallar por falta de sitio despues de que la persona se hubiera
registrado para aceptarla, que es el peor momento para enterarse.

`GET /v1/organizations/{orgId}/invitations` (admin) lista las pendientes, con `expired: true`
en las caducadas que todavia no se purgaron. El panel las mezcla con los miembros en una sola
tabla, como el diseno.

`POST /v1/organizations/{orgId}/invitations/{id}/resend` (admin) **202**. Genera un token
nuevo (el enlace anterior deja de valer), alarga la caducidad otros 7 dias y manda el correo.
**429** si se reenvio hace menos de 5 minutos o ya van 10 envios.

`DELETE /v1/organizations/{orgId}/invitations/{id}` (admin) la revoca. **204**. Es el
"Quitar" de una fila pendiente. Revocar borra la fila: no hace falta guardar el correo de
alguien que nunca llego a entrar.

### Aceptar una invitacion

La pagina de la invitacion no esta en el diseno; hace falta.

`POST /v1/invitations/preview { "token": "..." }` es publico y responde **200** con lo que la
pagina ensena antes de pedir login o registro:

```json
{
  "organizationName": "ACME Studio",
  "inviterName": "Marta Ruiz",
  "email": "lucia@acme.studio",
  "role": "member",
  "expiresAt": "..."
}
```

**404** si el token no existe o se revoco, **410** `invitation_expired` si caduco. Ensenar el
correo completo a quien tiene el token no filtra nada: el token se mando a ese correo.

`POST /v1/invitations/accept { "token": "..." }` con sesion. **200** con la organizacion.

- El correo de la sesion tiene que coincidir con el de la invitacion (sin distinguir
  mayusculas): **409** `invitation_email_mismatch` si no. La pagina ofrece cerrar sesion y
  entrar con la cuenta correcta.
- Aceptar **verifica el correo** si no lo estaba: el token llego a ese buzon. Y, como al
  restablecer la contrasena, si el correo no estaba verificado se borran las identidades
  vinculadas, los dispositivos, las sesiones y la boveda que pudo dejar quien registro el correo
  antes que su duena, y se manda un resumen.
- La invitacion se borra y queda la membresia, con quien invito y cuando.
- Es idempotente para quien ya es miembro por esa invitacion: repetir responde 200.

Las invitaciones para mi, sin enlace (la lista de Desktop "invitaciones pendientes para mi"
que el analisis echaba en falta):

- `GET /v1/me/invitations` lista las invitaciones vivas dirigidas al correo de la cuenta,
  **solo si el correo esta verificado**. Si no lo esta, la lista sale vacia: con un correo
  sin verificar cualquiera podria registrarse con la direccion de otro y ver sus
  invitaciones.
- `POST /v1/me/invitations/{id}/accept` la acepta (**403** `email_not_verified` sin verificar).
- `DELETE /v1/me/invitations/{id}` la rechaza. **204**. Solo funciona con invitaciones
  dirigidas al correo verificado de quien llama; cualquier otro id es **404**, igual que si no
  existiera.

## Asientos

**F3.**

```
GET /v1/organizations/{orgId}/seats
{ "total": 5, "used": 5, "activeMembers": 4, "pendingInvitations": 1, "purchasable": false }
```

Es la tarjeta "5 de 5 asientos usados" con su barra. `used = activeMembers + pendingInvitations`.
El porcentaje de la barra lo calcula el cliente. Todo lo que ocupa o libera un asiento toma antes
la fila de la organizacion con `FOR UPDATE` (ver [Invitaciones](#invitaciones)).

Hasta que exista facturacion (F5), el total de una organizacion nueva sale de configuracion
(`RAMAL__ORGANIZATIONS__DEFAULT_SEATS`, por defecto 5, que es lo que ensena el diseno) y
`purchasable` es `false`: la interfaz esconde "Anadir asientos". Con F5, `purchasable` pasa a
`true` para owners y el boton lleva a la compra.

## Suscripcion y facturacion

**F5, fase posterior.** Esta seccion fija la forma para que el resto del contrato no tenga
que cambiar cuando llegue, pero **hay una decision abierta que la bloquea: el proveedor de
pago**. Las opciones que hay que comparar son un procesador (Stripe Billing) o un comerciante
de registro que se encarga del IVA y los impuestos de cada pais (Paddle, Lemon Squeezy). Con
clientes en Europa y America, el IVA por pais pesa mas que la comision. Tampoco estan
decididos: si hay plan personal de pago o solo de organizacion, el precio por asiento, y que
funciones dependen del plan. La tarjeta "Ramal Pro" de Desktop es lo unico que el diseno
ensena.

Lo que se propone, sea cual sea el proveedor:

- `GET /v1/organizations/{orgId}/subscription` (miembro):

  ```json
  {
    "plan": { "id": "pro", "name": "Ramal Pro" },
    "status": "active",
    "seatsTotal": 5,
    "renewsAt": "2026-10-12T00:00:00Z",
    "cancelAtPeriodEnd": false
  }
  ```

  `status`: `trialing`, `active`, `past_due`, `canceled`. **404** si la organizacion no tiene
  suscripcion (plan gratuito).

- `POST /v1/organizations/{orgId}/subscription/checkout { "planId": "pro", "seats": 5 }`
  (owner, `Idempotency-Key`) devuelve `{ "url": "..." }`: la pagina de pago del proveedor. La
  API nunca ve una tarjeta.
- `POST /v1/organizations/{orgId}/subscription/seats { "quantity": 7 }` (owner,
  `Idempotency-Key` obligatoria) cambia el numero de asientos con prorrateo. **200** con la
  suscripcion, o **202** con `{ "url": "..." }` si el proveedor exige confirmar el pago. **409**
  `seats_below_usage` si se piden menos asientos de los que se usan.
- `POST /v1/organizations/{orgId}/subscription/portal` (owner) devuelve `{ "url": "..." }`: el
  portal del proveedor para metodo de pago, facturas y cancelacion. Es el boton "Gestionar
  suscripcion".
- `POST /v1/billing/webhooks/{provider}`: publico, verifica la firma del proveedor, deduplica
  por identificador de evento y actualiza el total de asientos de la organizacion. La fuente
  de verdad del numero de asientos es el proveedor; la API guarda la copia que le llega por
  aqui.

## Workspaces

**F2** los personales, **F3** los de organizacion. Un workspace agrupa repositorios; un
repositorio puede estar en varios.

```
GET /v1/workspaces?organizationId=0192f3d0-...
{
  "items": [
    {
      "id": "0192f3f0-...",
      "organizationId": "0192f3d0-...",
      "name": "Backend",
      "color": "lane-1",
      "visibility": "organization",
      "ownerUserId": "0192f3c0-...",
      "canEdit": true,
      "repositories": [
        {
          "entryId": "0192f3f1-...",
          "kind": "remote",
          "remoteKey": "github.com/acme/api-gateway",
          "name": "api-gateway",
          "displayName": "Gateway",
          "platform": "github",
          "host": "github.com",
          "cloneUrl": "https://github.com/acme/api-gateway.git",
          "repositoryId": "0192f400-..."
        }
      ],
      "createdAt": "2026-09-03T10:00:00Z",
      "updatedAt": "2026-09-20T10:00:00Z"
    }
  ]
}
```

- `organizationId`: sin parametro devuelve **todos** los visibles (los personales y los de
  cada organizacion), que es lo que necesita la barra lateral de Desktop. Con un id, los de
  esa organizacion; con `personal`, los personales. La web filtra por su contexto activo.
- `visibility`: `private` o `organization`. En el contexto personal siempre es `private`. En
  una organizacion, `private` es "Solo tu" y `organization` es "Compartido con {org}".
- `color` es un **token de la paleta**, `lane-0` a `lane-5`, no un color. El color real lo
  decide el tema: el de Ramal oscuro, el claro o el de VS Code no pintan igual el mismo
  carril. Guardar `oklch(...)` en la base congelaria un tema.
- `canEdit` lo calcula la API con la matriz de roles, para que la interfaz no repita la regla.
- `repositories` va embebido: las tarjetas pintan las chips de golpe y un workspace tiene
  como mucho 200 entradas.
- `displayName` es **el alias de quien pregunta** si tiene ese repositorio en su cuenta, o el
  nombre si no. Los alias son personales: el "Cliente Git" de Marta no es el de Diego.
- `repositoryId` es el registro de quien pregunta que casa con esa entrada, o `null` si no lo
  tiene en su cuenta. Con `null`, Desktop ensena "no clonado en este equipo" y ofrece clonar
  con `cloneUrl`.
- **`cloneUrl` lo escribe otro miembro**, asi que el motor lo trata como entrada no confiable:
  la API solo acepta `https://`, `ssh://` o `usuario@host:ruta` (ver
  [Repositorios](#repositorios)), y el motor clona sin protocolos peligrosos y elige la cuenta por
  el host exacto, nunca por `platform`, que solo sirve para pintar el icono.
- Las entradas de repositorios **sin remoto** (`kind: "local"`, con `originDevice`) solo las
  ve el dueno de ese repositorio, tambien dentro de un workspace compartido. Nadie mas puede
  clonar una carpeta que solo existe en el disco de otra persona.

`POST /v1/workspaces`:

```json
{
  "id": "0192f3f0-...",
  "name": "Frontend",
  "organizationId": null,
  "visibility": "private",
  "color": null,
  "repositoryIds": ["0192f400-..."]
}
```

**201** con el workspace; **200** si ya existia con ese id y el mismo contenido (reintento).

- `id` lo genera el cliente (ver [Idempotencia](#idempotencia)).
- `color` opcional: si no viene, se asigna `lane-{n % 6}` con `n` el numero de workspaces del
  contexto, que es la regla del prototipo. El diseno no tiene selector de color; el campo
  existe para cuando lo tenga.
- `visibility` por defecto `private` en el contexto personal y `organization` en una
  organizacion (el diseno web marca "Compartido" por defecto al crear).
- `repositoryIds` opcional: crea el workspace con esos repositorios dentro, que es el "+ Nuevo
  workspace" del menu de un repositorio.
- **404** si `organizationId` es de una organizacion donde no eres miembro. **409**
  `limit_reached` al pasar de 100 por contexto. **409** `id_conflict` si el id existe con otro
  contenido.

`PATCH /v1/workspaces/{id} { "name": "Plataforma", "visibility": "private", "color": "lane-3" }`
cualquier subconjunto. **200**. **403** `insufficient_role` si es un workspace compartido que
ves pero no puedes editar. El workspace privado de otro miembro es **404** en todos los verbos:
contestar 403 diria que existe. Cambiar `visibility` a `organization` fuera de una organizacion
es **422**. Un workspace no cambia de contexto: moverlo de personal a una organizacion es crear
otro.

`DELETE /v1/workspaces/{id}` **204**. Los repositorios no se tocan; solo desaparece la
agrupacion (el texto correcto del diseno es el de Desktop: "Los repositorios no se
eliminan"). El prototipo web ensenaba "Sin repositorios" en esa confirmacion por un error de
clave.

### Repositorios dentro de un workspace

`POST /v1/workspaces/{id}/repositories { "repositoryId": "0192f400-..." }` anade un
repositorio de tu cuenta. **201** con la entrada, **200** si ya estaba. **409**
`limit_reached` al pasar de 200.

`DELETE /v1/workspaces/{id}/repositories/{entryId}` lo saca. **204**.

`DELETE /v1/repositories/{id}/workspaces` saca un repositorio de todos los workspaces que
puedes editar. Es "Quitar de todos" del submenu. **204**.

Los menus de Desktop (casillas que se quedan abiertas) y de la web (casilla por clic) usan
los mismos dos endpoints, una llamada por casilla.

**Por que las entradas guardan el remoto y no apuntan al registro personal.** Lo facil era
una tabla `workspace_id, repository_id` con el repositorio de quien lo anadio. Falla en el
caso que importa: Marta anade `api-gateway` al workspace compartido "Backend"; despues se va
de la organizacion o quita el repositorio de su cuenta, y el repositorio desaparece del
workspace de todo el equipo. Por eso una entrada guarda la identidad del repositorio, que es
su remoto normalizado, mas una copia de nombre, plataforma, host y URL de clonado. Cada
miembro la casa con su propio registro por `remoteKey`, y el workspace sobrevive a quien lo
monto. Solo las entradas de repositorios sin remoto apuntan a un registro, porque no tienen
otra identidad.

## Repositorios

**F2.** Es la lista "Repositorios vinculados a tu cuenta" de la web y la copia en la nube del
registro de repositorios de Desktop. Cada registro es personal: el alias y la cuenta que usa
son de quien lo tiene.

### Identidad: el remoto normalizado, nunca la ruta

**La API no guarda rutas locales.** Ni las del propio dispositivo. Una ruta
(`/home/marta/clientes/proyecto-secreto`) dice cosas del usuario que no hacen falta y en otra
maquina no sirve para nada. El registro local con las rutas vive en cada dispositivo.

La identidad de un repositorio con remoto es su **`remoteKey`**: la URL del remoto principal
normalizada. El remoto principal es `origin` si existe; si no, el unico que haya; si hay
varios sin `origin`, el primero por nombre.

| URL del remoto                                      | `remoteKey`                          |
| --------------------------------------------------- | ------------------------------------ |
| `https://github.com/ACME/Web-App.git`               | `github.com/acme/web-app`            |
| `git@github.com:acme/web-app.git`                   | `github.com/acme/web-app`            |
| `ssh://git@gitlab.empresa.com:2222/equipo/api`      | `gitlab.empresa.com:2222/equipo/api` |
| `https://marta@dev.azure.com/banca/core/_git/pagos` | `dev.azure.com/banca/core/pagos`     |
| `git@ssh.dev.azure.com:v3/banca/core/pagos`         | `dev.azure.com/banca/core/pagos`     |

Las reglas: host en minusculas, sin esquema ni usuario, sin puerto si es el de por defecto
del esquema, sin `.git` ni barra final, y la forma `usuario@host:ruta` pasada a `host/ruta`.
La ruta se pasa a minusculas **solo en los hosts que no distinguen mayusculas** (github.com,
gitlab.com, bitbucket.org, dev.azure.com); en un Gitea propio `Equipo/API` y `equipo/api`
pueden ser dos repositorios.

La normalizacion vive en un solo sitio, `ramal-core-domain`, y la API la recalcula siempre a
partir de `remoteUrl`: **nunca acepta un `remoteKey` del cliente**. Dos versiones del cliente
que normalizaran distinto partirian un repositorio en dos. Cambiar una regla de normalizacion
en el futuro es una migracion de datos, no un cambio de codigo.

Un repositorio **sin remoto** no tiene identidad global. Su identidad es la pareja
(dispositivo de origen, id local) que genera el dispositivo, y en la web aparece como "Local
en {dispositivo}".

### Endpoints

```
GET /v1/repositories?q=api&workspaceId=&linkedAccountId=&kind=&cursor=&limit=
{
  "items": [
    {
      "id": "0192f400-...",
      "kind": "remote",
      "name": "gitflow-desktop",
      "alias": "Cliente Git",
      "remoteKey": "github.com/marta-dev/gitflow-desktop",
      "cloneUrl": "https://github.com/marta-dev/gitflow-desktop.git",
      "host": "github.com",
      "platform": "github",
      "linkedAccountId": "0192f410-...",
      "linkedAccount": { "id": "0192f410-...", "platform": "github", "username": "marta-dev", "label": "Personal" },
      "originDevice": null,
      "workspaces": [{ "workspaceId": "0192f3f0-...", "entryId": "0192f3f1-..." }],
      "lastActivityAt": "2026-09-30T09:55:00Z",
      "createdAt": "2026-09-01T09:10:00Z",
      "updatedAt": "2026-09-20T12:00:00Z"
    }
  ],
  "nextCursor": null
}
```

- Orden por nombre visible (alias o nombre), sin distinguir mayusculas.
- `q` busca en nombre, alias, `remoteKey`, **usuario de la cuenta vinculada** y nombre de los
  workspaces. El texto del buscador del diseno promete "cuenta" y el prototipo no la buscaba.
- `linkedAccount` va embebido porque la columna "Origen" pinta `@usuario` en cada fila.
- `originDevice` (`{ "id", "name" }`) solo en los locales: es el "Local en MacBook Pro" que el
  prototipo tenia escrito a mano.
- `workspaces` incluye las entradas de workspaces que quien pregunta puede ver: las de sus
  workspaces y las de los compartidos de sus organizaciones que casan por `remoteKey`.
- `lastActivityAt` es la columna "Actualizado". El diseno no decia que significaba; aqui es
  **la ultima actividad que un dispositivo informo** (ultimo commit de HEAD, ultimo fetch o
  ultima apertura, lo mas reciente). Es la misma definicion que usa el inicio de Desktop para sus
  tarjetas. El motor lo informa con `PATCH` como mucho una vez por hora por repositorio y la API
  solo lo mueve hacia delante: un valor anterior al guardado se ignora sin error.
- La rama, ahead, behind y cambios de las tarjetas de Desktop **no estan aqui**: son estado
  vivo de Git que calcula el motor, no datos de la nube.

`POST /v1/repositories` registra o actualiza por identidad:

```json
{
  "remoteUrl": "git@github.com:marta-dev/gitflow-desktop.git",
  "name": "gitflow-desktop",
  "platform": null,
  "alias": null,
  "linkedAccountId": null
}
```

o, sin remoto:

```json
{ "local": { "deviceId": "0192f3c1-...", "localId": "0192f420-..." }, "name": "notas" }
```

**201** si es nuevo, **200** con el registro existente si esa identidad ya estaba (y aplica
los campos que vengan). Repetirlo no duplica nada.

- `remoteUrl` **con contrasena o token dentro** (`https://usuario:ghp_xxx@github.com/...`) es
  **422** `remote_url_has_credentials`, y la API no la guarda ni la registra. El cliente ya
  deberia limpiarla; esto es la segunda barrera para no acabar con un token en claro en la
  base por la puerta de atras.
- **Solo tres formas de URL:** `https://`, `ssh://` o `usuario@host:ruta`. Cualquier otra
  (`ext::`, `fd::`, `file:`, `http://`), o una que empiece por `-` o lleve espacios o caracteres de
  control, es **422** `invalid_remote_url`. La URL de clonado acaba en el workspace compartido que
  otros miembros clonan, y `ext::` ejecuta un comando en el equipo de quien clona.
- `cloneUrl` es el remoto tal cual pero sin credenciales: se conserva `git@` en SSH y se
  quita cualquier usuario en HTTPS.
- `platform` se deduce del host conocido (github.com, gitlab.com, bitbucket.org,
  dev.azure.com y ssh.dev.azure.com, gitea.com, codeberg.org). Para un host propio se toma la
  plataforma de la cuenta vinculada, o la que diga el cliente, o `null`. **Es solo para pintar**:
  ningun cliente elige credenciales por `platform`, sino por el host exacto de la cuenta (un
  `github.com.evil.example` declarado como `github` no recibe el token de GitHub).
- `local.deviceId` tiene que ser un dispositivo de quien llama.

`POST /v1/repositories/batch { "items": [ ...hasta 200 como el anterior ] }` (nativa) es lo
mismo en lote, para la primera sincronizacion de un equipo con cincuenta repositorios.
**200** con `{ "items": [{ "index": 0, "status": 201, "repository": {...} }, { "index": 1, "status": 422, "problem": {...} }] }`.
Un elemento malo no tumba a los demas.

`PATCH /v1/repositories/{id}`:

```json
{
  "alias": "Cliente Git",
  "linkedAccountId": "0192f410-...",
  "platform": "gitlab",
  "lastActivityAt": "..."
}
```

**200**. Es "Asignar alias", elegir la cuenta del repositorio y la actividad. `alias` de
hasta 60 caracteres o `null` para quitarlo. **422** `unknown_reference` si la cuenta no es
tuya (la base lo impide igualmente con una clave foranea compuesta). `lastActivityAt` solo
avanza.

**Mientras "Cuentas vinculadas" esta apagada**, borrar las cuentas en la nube deja
`linkedAccountId` en `null` en cada repositorio, y ese cambio llega por el feed. Los dispositivos
conservan en local la cuenta que eligieron para cada repositorio y no aplican ese `null`: la
categoria esta en pausa para ellos. Al volver a encenderla, suben sus cuentas y vuelven a
asignarlas.

`DELETE /v1/repositories/{id}` es "Quitar de la cuenta" en la web y "Quitar de la lista" en
Desktop. **204**.

- Borra el registro y sus entradas en tus workspaces **privados**. Las entradas en workspaces
  compartidos son del equipo y se quedan: el repositorio sigue en "Backend" para los demas.
- Los dispositivos lo sacan de su lista al recibir la baja en el feed. **Nunca tocan la
  carpeta.** La casilla "Ademas, mover la carpeta a la papelera" de Desktop es una accion
  local del dispositivo donde se pulsa, no de la API.
- Si despues se vuelve a abrir la carpeta en un equipo, se registra de nuevo.

## Cuentas de plataformas Git

**F2.** Varias cuentas por plataforma, una predeterminada por plataforma, y cada repositorio
usa una. El token es de extremo a extremo: la API guarda metadatos y un blob que no puede
abrir.

```
GET /v1/linked-accounts
{
  "items": [
    {
      "id": "0192f410-...",
      "platform": "gitlab",
      "host": "gitlab.empresa.com",
      "username": "mruiz",
      "label": "Empresa",
      "method": "pat",
      "scopes": ["api", "read_repository"],
      "isDefault": true,
      "status": "valid",
      "lastVerifiedAt": "2026-09-30T08:00:00Z",
      "tokenExpiresAt": "2027-01-01T00:00:00Z",
      "hasSecret": true,
      "secretKeyVersion": 1,
      "usedInRepositories": 2,
      "createdAt": "2026-09-01T09:20:00Z",
      "updatedAt": "2026-09-30T08:00:00Z"
    }
  ]
}
```

- `platform`: `github`, `gitlab`, `bitbucket`, `gitea`, `azure`. El orden de los grupos y las
  etiquetas, abreviaturas y colores de cada plataforma son configuracion estatica del
  cliente, no datos de la API.
- `method`: `oauth`, `pat`, `api_token` (Bitbucket) o `ssh_token` (SSH para el transporte mas
  un token para la API). **"App password" no existe**: Atlassian dejo de admitir app passwords
  de Bitbucket el 9 de junio de 2026. Los datos del prototipo lo usaban.
- `status` y `lastVerifiedAt` **los informa el dispositivo**, que es quien puede probar el
  token. Es el punto verde de la tarjeta. El servidor no los calcula: no tiene el token.
- `tokenExpiresAt` tambien lo declara el cliente (los tokens OAuth de GitHub pueden
  caducar a las 8 horas y los PAT pueden llevar fecha), para avisar de "caduca pronto" en todas las
  superficies.
- `hasSecret` dice si hay blob cifrado. Una cuenta puede estar sincronizada sin secreto
  (boveda sin configurar): en otro equipo aparece con "Actualizar credenciales".
- `usedInRepositories` es el "Usada en N repos".
- **Nunca devuelve el blob.** El blob se lee por la boveda o por el feed, y solo desde un
  cliente nativo.

`POST /v1/linked-accounts` (nativa, **viva**):

```json
{
  "id": "0192f410-...",
  "platform": "gitlab",
  "host": "gitlab.empresa.com",
  "username": "mruiz",
  "label": "Empresa",
  "method": "pat",
  "scopes": ["api", "read_repository"],
  "isDefault": null,
  "status": "valid",
  "lastVerifiedAt": "2026-09-30T08:00:00Z",
  "tokenExpiresAt": null
}
```

**201**, o **200** si es un reintento identico.

- `id` lo genera el cliente y es obligatorio: el motor cifra el token con ese id dentro de los
  datos asociados antes de llamar. **409** `id_conflict` si es el id de una cuenta que ya se
  borro: la API guarda los ids retirados y no deja reutilizarlos.
- `host` en minusculas y sin esquema. Para Azure DevOps incluye la organizacion
  (`dev.azure.com/banca`), porque ahi la organizacion es parte de a donde se conecta.
- `isDefault` `null` aplica la regla del prototipo: es predeterminada si es la primera de su
  plataforma.
- **409** `account_exists` si ya hay una cuenta con la misma plataforma, host y usuario.
  **409** `sync_category_disabled` si el usuario apago la sincronizacion de cuentas. **409**
  `limit_reached` al pasar de 50.

El flujo completo de "Anadir cuenta" en el dispositivo: el motor obtiene el token (OAuth o
PAT), lo prueba contra la plataforma y lee usuario y scopes, lo guarda en el llavero local,
crea la cuenta aqui y sube el blob con `PUT .../secret`. Son dos llamadas y no una a
proposito: todo el ciphertext entra por un solo camino (ver la boveda), y una cuenta sin blob
durante unos milisegundos es un estado valido.

`PATCH /v1/linked-accounts/{id}` (**viva**):

```json
{
  "label": "Empresa",
  "isDefault": true,
  "scopes": ["api"],
  "status": "expired",
  "lastVerifiedAt": "...",
  "tokenExpiresAt": "..."
}
```

**200**. `isDefault: true` quita la marca a la anterior de la misma plataforma en la misma
transaccion ("Hacer predeterminada"). `isDefault: false` sobre la predeterminada es **422**:
siempre hay una por plataforma mientras quede alguna.

- `label` e `isDefault` se pueden cambiar desde cualquier sesion. La web puede usarlo para eso
  aunque el diseno web no lo pida.
- `scopes`, `status`, `lastVerifiedAt` y `tokenExpiresAt` **los informa el dispositivo**, que es
  el unico que puede probar el token. Desde la web son **403** `native_client_required`: una
  sesion web robada no puede pintar como valida una cuenta que no lo es, ni al reves.
- `platform`, `host`, `username` y `method` no se cambian: deciden a donde va el token. Otra
  cuenta en otro host es otra cuenta.

`DELETE /v1/linked-accounts/{id}` es "Desconectar" (**viva**). **204**.

- Borra la cuenta y su blob. Los repositorios que la usaban se quedan sin cuenta.
- Si era la predeterminada, **pasa a serlo la primera que quede de esa plataforma** (por
  fecha de alta). El prototipo no promovia ninguna y dejaba la plataforma sin predeterminada.
- Revocar el permiso en la plataforma lo hace el dispositivo antes de llamar aqui, porque es
  quien tiene el token. Desde la web no se puede: se avisa de que el token sigue valido en la
  plataforma hasta que se revoque alli.

Secreto de la cuenta, solo nativa y **viva**:

```
PUT /v1/linked-accounts/{id}/secret
{ "keyVersion": 1, "alg": "xchacha20poly1305", "nonce": "base64url(24 bytes)", "ciphertext": "base64url" }
```

**200** con la cuenta (`hasSecret: true`). **409** `vault_not_set_up` si no hay boveda, **409**
`vault_key_version_mismatch` si `keyVersion` no es la vigente (el dispositivo tiene una llave
vieja y tiene que desbloquear otra vez). **413** si el ciphertext pasa de 16 KiB. **403**
`native_client_required` desde la web. La escritura lee la version de la llave con
`SELECT key_version FROM vaults WHERE user_id = $1 FOR SHARE` y la compara dentro de la misma
transaccion: un `PUT` que leyo la version antes de una rotacion no puede escribir despues de ella
un secreto que ya nadie podria abrir.

`DELETE /v1/linked-accounts/{id}/secret` quita solo el blob. **204**.

Lo que va dentro del blob lo define el cliente y el servidor no lo lee. El contrato entre
clientes es un JSON versionado que lleva tambien a donde va el token:
`{ "v": 1, "platform": "gitlab", "host": "gitlab.empresa.com", "username": "mruiz", "method": "pat", "accessToken": "...", "refreshToken": "...", "expiresAt": "...", "tokenType": "bearer" }`.
El motor que lo abre compara `platform`, `host`, `username` y `method` con los de la fila y, si
no coinciden, no usa el token y ensena "Las credenciales no coinciden con la cuenta". Los
metadatos en claro los controla el servidor; la copia sellada, no.

**Lo que no existe, a proposito:** `POST .../test` (el servidor no puede usar el token),
compartir con el equipo, y cualquier endpoint que acepte un token en claro.

## La boveda de extremo a extremo

**F2.** Es lo que hace posible guardar tokens de plataforma y llaves de IA personales en la
nube sin que la nube pueda leerlos. Toda la criptografia ocurre en el motor de Rust
(`crates/vault`, planificado), que es el mismo en Desktop y en el sidecar de VS Code. La web
no cifra ni descifra nada, y por eso no tiene endpoints de escritura aqui salvo el reinicio.

### El esquema

1. **Llave de la boveda (VK):** 256 bits aleatorios, generados en el primer dispositivo que
   activa la boveda. Cifra todos los secretos del usuario.
2. **Frase de sincronizacion.** De ella sale, con **Argon2id**, una llave que envuelve la VK.
   Parametros por defecto: 64 MiB de memoria, 3 pasadas, 4 carriles y sal de 16 bytes. El
   servidor rechaza parametros por debajo de 64 MiB y 3 pasadas.
3. **Codigo de recuperacion:** 160 bits aleatorios, que el usuario ve una sola vez como 32
   caracteres en base32 de Crockford en grupos de cuatro. Envuelve la VK por segunda vez, con
   una llave derivada con HKDF-SHA256. No necesita Argon2: 160 bits no se adivinan.
4. **Envoltorio y secretos con XChaCha20-Poly1305** (nonce aleatorio de 24 bytes). Con un
   nonce de 192 bits no hay que llevar la cuenta de cuantos mensajes cifra una misma llave, y
   la web no necesita soportarlo porque no cifra.
5. **Subllaves y datos asociados (AAD) de cada secreto.** La VK no cifra directamente: de ella
   salen con HKDF-SHA256 `K_seal` (`"ramal:vault:v1:seal"`) y `K_check`
   (`"ramal:vault:v1:check"`). Los secretos se sellan con `K_seal` y AAD
   `ramal:vault:v1:{userId}:{kind}:{recordId}:{keyVersion}`, con `kind` `platform_credential`
   o `ai_key`. Un ciphertext copiado a otra fila, a otro usuario o a otra version de la llave
   no descifra. El servidor no puede comprobarlo (no tiene la llave); es un contrato entre
   clientes y esta aqui escrito para que cualquier implementacion lo cumpla igual.
6. **El contenido sellado lleva su destino.** Ademas del secreto, el JSON sellado repite los
   metadatos que deciden a donde se envia: plataforma, host, usuario y metodo de una cuenta;
   proveedor y URL base de una llave. El motor los compara con la fila al abrir y no usa un
   secreto que no coincide. Sin esto, quien controle la fila podria apuntar una llave a su
   propio servidor, o guardar un blob antiguo de GitHub en una cuenta nueva con su host, y cada
   dispositivo le enviaria el secreto descifrado.
7. **Comprobante de la llave:**
   `keyCheck = HMAC-SHA256(K_check, "ramal:vault:key-check:v1:{userId}:{keyVersion}")`. El
   dispositivo lo manda al crear la boveda y en cada cambio de envoltorio o rotacion; el
   servidor **guarda solo su SHA-256** y compara hashes en tiempo constante: **solo quien tiene
   la VK puede cambiar como se envuelve**. Sin esto, una sesion robada podria sustituir los
   envoltorios por basura y dejar al usuario sin acceso a sus secretos. Guardar el hash evita
   que quien lea la tabla tenga una prueba reutilizable. El comprobante no ayuda a sacar la VK.
8. **Cada dispositivo guarda la VK desbloqueada** en el llavero del sistema (Desktop) o en
   `SecretStorage` (VS Code). La frase se pide una vez por dispositivo.

**Por que una frase aparte y no la contrasena de login.** Por dos motivos, y los dos bastan
solos: el servidor ve la contrasena en claro cada vez que alguien inicia sesion, asi que una
llave derivada de ella no seria de extremo a extremo; y las cuentas creadas con GitHub,
GitLab o Google no tienen contrasena.

**Lo que se asume.** Quien robe la base de datos tiene los envoltorios y puede probar frases
sin conexion. **Y no hace falta robar la base**: cualquier sesion nativa recibe los envoltorios
con `GET /v1/vault`, tambien una conseguida con phishing. Por eso la primera descarga de la
boveda en cada dispositivo deja un evento de auditoria y manda el correo "Un equipo nuevo
descargo tu boveda", y `GET /v1/vault` tiene un tope de 20 al dia por usuario. La defensa de
fondo es el coste de Argon2id y la longitud de la frase. El cliente exige
un minimo (12 caracteres y una puntuacion de fortaleza) porque el servidor no ve la frase y
no puede exigir nada. Una mejora posible, que no entra en F2, es mezclar en la derivacion un
valor del servidor cifrado con su llave maestra: protege frente a un robo de solo la base, no
frente a un servidor comprometido.

### Endpoints

`GET /v1/vault` (nativa, **viva**):

```json
{
  "enabled": true,
  "keyVersion": 1,
  "version": 3,
  "kdf": {
    "alg": "argon2id",
    "salt": "base64url",
    "memoryKib": 65536,
    "iterations": 3,
    "parallelism": 4
  },
  "passphraseWrap": { "alg": "xchacha20poly1305", "nonce": "base64url", "wrappedKey": "base64url" },
  "recoveryWrap": {
    "kdf": "hkdf-sha256",
    "salt": "base64url",
    "alg": "xchacha20poly1305",
    "nonce": "base64url",
    "wrappedKey": "base64url"
  },
  "createdAt": "2026-09-05T10:00:00Z",
  "passphraseUpdatedAt": "2026-09-05T10:00:00Z",
  "recoveryUpdatedAt": "2026-09-05T10:00:00Z",
  "rotationRecommended": false
}
```

Sin boveda: **200** con `{ "enabled": false }`. No es un 404 porque no configurada es un
estado normal que la interfaz ensena, no un error. `ETag` con `version`. `rotationRecommended`
pasa a `true` cuando se revoca un dispositivo que habia descargado la boveda, y vuelve a `false`
con la siguiente rotacion. 20 lecturas al dia por usuario; la primera de cada dispositivo manda
un aviso al correo.

`POST /v1/vault` crea la boveda. Cuerpo: `kdf`, `passphraseWrap`, `recoveryWrap` y
`keyCheck` (la API guarda su SHA-256). **201**. **409** `vault_exists`. **422** con parametros de Argon2id por debajo del
minimo o tamanos de sal, nonce o llave que no cuadran. El dispositivo ensena el codigo de
recuperacion **antes** de llamar y exige que el usuario confirme que lo guardo.

`PUT /v1/vault/passphrase` cambia la frase: `{ "keyCheck": "...", "kdf": {...}, "passphraseWrap": {...} }`
con `If-Match`. **200**. **403** `vault_key_check_failed` si el comprobante no coincide. La VK
no cambia: los demas dispositivos siguen funcionando sin volver a pedir nada.

`PUT /v1/vault/recovery` regenera el codigo de recuperacion:
`{ "keyCheck": "...", "recoveryWrap": {...} }` con `If-Match`. **200**.

En los dos casos la envoltura anterior **deja de valer ante el servidor; quien guardo la
envoltura anterior la sigue abriendo** con la frase o el codigo viejos, porque la VK es la misma.
La interfaz pregunta si el usuario cree que alguien los conoce y, si dice que si, encadena
`POST /v1/vault/rotate`.

`POST /v1/vault/rotate` cambia la propia VK. Se usa cuando se sospecha que un dispositivo con
la llave cacheada esta comprometido.

```json
{
  "fromKeyVersion": 1,
  "keyCheck": "comprobante de la VK vieja",
  "newKeyCheck": "comprobante de la VK nueva",
  "kdf": { "...": "..." },
  "passphraseWrap": { "...": "..." },
  "recoveryWrap": { "...": "..." },
  "secrets": [
    {
      "kind": "platform_credential",
      "recordId": "0192f410-...",
      "nonce": "...",
      "ciphertext": "..."
    },
    { "kind": "ai_key", "recordId": "0192f430-...", "nonce": "...", "ciphertext": "..." }
  ]
}
```

Con `If-Match`, hasta 2 MiB. Antes de llamar, el dispositivo pide la frase (la envoltura nueva
la necesita) y genera, ademas de la VK nueva, **un codigo de recuperacion nuevo**, que ensena con
la misma confirmacion que al configurar: ningun dispositivo guarda el codigo, asi que no hay otra
forma de envolver la VK nueva para la recuperacion, y generarlo en silencio dejaria al usuario
sin salida. Despues descarga todos los secretos (`GET /v1/vault/secrets`), los descifra con la VK
vieja, los sella con la nueva y lo sube todo en una peticion. La API lo aplica **en una sola
transaccion**, con la fila de `vaults` tomada con `FOR UPDATE`: la nueva version de la llave, los
envoltorios, el hash del comprobante nuevo y todos los blobs, o nada. Rotar apaga
`rotationRecommended`. **409** `vault_secrets_mismatch` si la lista no es exactamente el conjunto de
secretos que hay ahora (falta uno, sobra uno o alguien anadio otro entre medias): rotar
dejando un secreto con la llave vieja lo haria ilegible para siempre. **200** con la cabecera
nueva.

Despues de rotar, **todos los demas dispositivos tienen que volver a introducir la frase**:
su VK cacheada es la vieja. Lo saben porque el feed les trae la cabecera con otro
`keyVersion`. Es el comportamiento correcto para una rotacion; para cambiar solo la frase
esta `PUT /v1/vault/passphrase`, que no molesta a nadie.

`GET /v1/vault/secrets` (nativa, **viva**) devuelve todos los blobs:
`{ "items": [{ "kind", "recordId", "keyVersion", "alg", "nonce", "ciphertext", "updatedAt" }] }`.
Lo usan la rotacion y el primer desbloqueo en un dispositivo nuevo.

### Reinicio: cuando se olvidaron la frase y el codigo

`DELETE /v1/vault` (web, **reauth fuerte**, cuerpo `{ "confirmation": "BORRAR SECRETOS" }`)
**borra la boveda y todos los blobs**: los de las cuentas de plataforma y los de las llaves de
IA. Los metadatos se quedan (`hasSecret: false`). **204**. Se manda un correo de aviso.

- Es de la web y con reautenticacion fuerte porque es el camino de quien no tiene nada mas: no
  puede exigir el comprobante de una llave que se perdio. Pedirla es lo que impide que una sesion
  robada lo use para destruir, y por eso no vale el OAuth de GitHub o GitLab, que puede
  completarse sin pedir nada. Es tambien el unico sitio donde existe el reinicio: en Desktop y
  VS Code, "Olvide la frase" explica la situacion y enlaza aqui.
- **No se pierde tanto como parece.** Cada dispositivo guarda su propia copia de los tokens
  en el llavero local. El primer dispositivo que cree una boveda nueva vuelve a subir los que
  tenga. Lo que se pierde son las copias que solo estaban en la nube.
- Los dispositivos ven en el feed `vault.enabled: false`, borran la VK cacheada y proponen
  configurar la boveda otra vez.

### Que se sincroniza y que no

| Dato                                 | Con la boveda configurada                              | Sin boveda                       |
| ------------------------------------ | ------------------------------------------------------ | -------------------------------- |
| Metadatos de cuentas de plataforma   | Si (si la categoria esta activa)                       | Si (si la categoria esta activa) |
| Tokens de plataforma                 | Si, cifrados                                           | No: cada dispositivo los suyos   |
| Metadatos de llaves de IA personales | Solo si la categoria esta activa (apagada por defecto) | Igual                            |
| Llaves de IA personales              | Si, cifradas, si la categoria esta activa              | No                               |
| Llaves SSH y GPG                     | Nunca                                                  | Nunca                            |

## Llaves de IA personales

**F2.** Solo existen en la API si el usuario activa "Llaves de IA" en la sincronizacion, que
**viene apagada**: "cada dispositivo guarda sus propias llaves". Con la categoria apagada,
todos los endpoints de escritura responden **409** `sync_category_disabled` y la lista sale
vacia.

```
GET /v1/ai/keys
{
  "items": [
    {
      "id": "0192f430-...",
      "provider": "anthropic",
      "label": "Personal",
      "model": "claude-opus-5",
      "baseUrl": null,
      "masked": "sk-ant-••••••••4f2a",
      "isDefault": true,
      "hasSecret": true,
      "secretKeyVersion": 1,
      "createdAt": "2026-09-06T10:00:00Z",
      "updatedAt": "2026-09-06T10:00:00Z"
    }
  ]
}
```

- `provider`: `anthropic`, `openai`, `groq`, `google`, `mistral`, `openrouter`, `ollama`,
  `custom`. `baseUrl` obligatorio para `openrouter`, `ollama` y `custom` (el formulario del
  prototipo lo pedia y lo perdia al guardar).
- `masked` lo calcula el cliente con la regla corregida de `@ramal/domain`: con menos de 12
  caracteres, solo puntos; si no, prefijo conocido mas cuatro ultimos. Guardar la mascara en
  claro deja ver como mucho el prefijo y cuatro caracteres, que es lo que ensena cualquier
  panel de proveedor, y permite pintar la lista en un dispositivo nuevo antes de desbloquear
  la boveda.
- `model` es texto libre. **La lista de modelos no la da la API** ni va escrita en la app:
  la pide el motor al endpoint de modelos de cada proveedor. Los modelos del prototipo ya
  estaban caducados el dia del analisis.
- Una llave de Ollama no tiene secreto.

`POST /v1/ai/keys` (nativa, **viva**) con `id` del cliente, `provider`, `label`, `model`,
`baseUrl`, `masked` e `isDefault`. **201** o **200** si es reintento. Si es la primera, queda
predeterminada. **409** `limit_reached` al pasar de 20. **409** `id_conflict` si el id es de una
llave borrada. `baseUrl` tiene que ser `https://`, salvo en `ollama`.

`PATCH /v1/ai/keys/{id}` (nativa, **viva**) cambia `label`, `model`, `masked` o `isDefault`. Solo
hay una predeterminada entre todas las llaves personales. **No cambia `baseUrl` ni `provider`**:
deciden a donde se envia la llave. `baseUrl` solo cambia en `PUT /v1/ai/keys/{id}/secret`, en la
misma peticion que sube el secreto sellado con la URL nueva dentro. La excepcion es `ollama`, que
no tiene secreto: su `baseUrl` se cambia con este `PATCH`, y el motor la trata como se explica
abajo.

`DELETE /v1/ai/keys/{id}` (**viva**) **204**. Si era la predeterminada, pasa a serlo la primera
que quede, como en el prototipo.

`PUT /v1/ai/keys/{id}/secret` y `DELETE /v1/ai/keys/{id}/secret` funcionan igual que los de
las cuentas de plataforma, con `kind` `ai_key` en los datos asociados y un tope de 4 KiB de
ciphertext (una llave de API no pasa de unos cientos de bytes). El `PUT` acepta ademas
`baseUrl`. El contenido sellado es `{ "v": 1, "provider", "baseUrl", "apiKey" }`, y el motor no
usa una llave cuyo `provider` o `baseUrl` no coincidan con la fila. Una llave de `ollama` no
admite secreto (**422**): no hay nada sellado que proteja su URL, asi que el motor solo acepta de
la nube una URL de Ollama en loopback (`localhost`, `127.0.0.1` o `[::1]`) y cualquier otra la
tiene que confirmar el usuario en ese equipo antes de mandarle codigo.

**Las llaves personales no se usan nunca en el servidor.** No hay ningun endpoint que genere
texto con una llave personal: eso lo hace el motor llamando al proveedor.

## Preferencias sincronizadas

**F2.** Tres documentos por usuario, uno por espacio de nombres, mas los ajustes de que se
sincroniza. Todos con version y `If-Match` (ver [Concurrencia optimista](#concurrencia-optimista)).

```
GET /v1/me/preferences
{
  "items": [
    { "namespace": "git", "version": 4, "updatedAt": "...", "updatedByDevice": { "id": "0192f3c1-...", "name": "MacBook Pro de Marta" }, "document": { "...": "..." } },
    { "namespace": "appearance", "version": 2, "updatedAt": "...", "updatedByDevice": null, "document": { "...": "..." } },
    { "namespace": "ai", "version": 7, "updatedAt": "...", "updatedByDevice": null, "document": { "...": "..." } }
  ]
}
```

`updatedByDevice` es el dispositivo de la ultima escritura, o `null` si se escribio desde la web
o el dispositivo ya no existe. Es lo que permite ensenar "cambiado desde tu MacBook" cuando un
`If-Match` llega tarde. El mismo campo viaja en el feed.

`GET /v1/me/preferences/{namespace}` devuelve uno, con `ETag`. Un espacio que nunca se
escribio devuelve el documento por defecto con `version: 0`, y `If-Match: "0"` lo crea.

`PATCH /v1/me/preferences/{namespace}` con `Content-Type: application/merge-patch+json` e
`If-Match`. **200** con el documento entero y la version nueva. **412** con `current` si otro
dispositivo escribio antes. **422** si el resultado no cumple el esquema del espacio.
Apagar la categoria no bloquea este endpoint: la pausa la cumplen los dispositivos (ver
[Que se sincroniza](#que-se-sincroniza)).

**El servidor valida cada documento contra su esquema**, estricto y sin campos desconocidos.
Tres clientes en versiones distintas leen el mismo documento; si uno guardara basura, los
otros dos se romperian. Cuando una version nueva del cliente necesite un campo nuevo, primero
se anade al esquema del servidor, que siempre va por delante.

### `git`

```json
{
  "userName": "Marta Ruiz",
  "userEmail": "marta@ramal.dev",
  "defaultBranch": "main",
  "signCommits": false,
  "autoFetchMinutes": 15,
  "pullMode": "rebase",
  "pruneOnFetch": true
}
```

- `autoFetchMinutes`: `0` (apagado), `5`, `15` o `30`.
- `pullMode`: `merge`, `rebase` o `ff_only`.
- `signCommits` es `false` por defecto aunque el prototipo lo traia encendido: firmar sin una
  llave configurada hace fallar cada commit.
- Las llaves SSH **no se sincronizan nunca**. Solo viven en el disco o en el agente.
- El motor aplica esto a `git config --global` en cada dispositivo. Es la seccion Git de los
  ajustes de Desktop y la de VS Code.

### `appearance`

```json
{ "theme": "dark", "language": "es", "graphNodes": "avatar" }
```

- `theme`: `dark`, `light` o `system`. El diseno solo tiene claro y oscuro; `system` es la
  opcion que el analisis pedia y que se anade al selector.
- `graphNodes`: `avatar` (iniciales) o `dot`.
- **VS Code no lo aplica nunca**: usa el tema y el idioma del editor, como dice el diseno
  ("En VS Code se usa siempre el tema del editor").

### `ai`

```json
{
  "features": {
    "commitMessage": { "enabled": true, "key": null },
    "pullRequest": { "enabled": true, "key": { "scope": "personal", "keyId": "0192f430-..." } },
    "commitSummary": {
      "enabled": false,
      "key": { "scope": "organization", "organizationId": "0192f3d0-...", "keyId": "0192f440-..." }
    }
  },
  "language": "app",
  "conventionalCommits": true,
  "excludePatterns": ["*.sqlite", "secrets/**"],
  "extraInstructions": "Usa el imperativo."
}
```

- Una llave por funcion, como Desktop. VS Code tenia una sola; el modelo por funcion es el que
  contiene al otro, y el ajuste unico de VS Code se traduce a las tres funciones con la misma
  llave.
- `key: null` es "Llave predeterminada".
- **Resolucion**, en el motor: la llave referenciada si existe en este dispositivo; si no, la
  predeterminada; si no, la primera; si no, ninguna y la funcion avisa. Una referencia a una
  llave personal que solo existe en otro equipo (con la sincronizacion de llaves apagada, cada
  equipo tiene las suyas con ids distintos) cae en la predeterminada de este. Es lo esperable
  y no un error.
- `language`: `app` (el de la interfaz; en VS Code, el del editor), `es` o `en`.
- `excludePatterns` son globs **anadidos** a la lista fija de ficheros sensibles (`.env*`,
  `*.pem`, `*.key`, `id_rsa*`, `*.p12`...). La lista fija se aplica siempre, en el motor y en la
  API, y no tiene interruptor: este documento se puede escribir desde la web y se sincroniza, y
  un interruptor dejaria que una sesion web robada hiciera que todos los equipos mandaran sus
  `.env` al proveedor de IA. `extraInstructions` hasta 500 caracteres. El prototipo recogia los
  dos y no los aplicaba.

### Que se sincroniza

```
GET /v1/me/sync-settings
{
  "version": 2,
  "workspacesAndRepositories": true,
  "linkedAccounts": true,
  "aiPreferences": true,
  "aiKeys": false,
  "gitPreferences": true,
  "appearance": true,
  "updatedAt": "2026-09-30T10:00:00Z"
}
```

`PATCH /v1/me/sync-settings { "linkedAccounts": false }` con `If-Match`. **200** con el
documento y lo que se borro: `{ ..., "deleted": { "linkedAccounts": 7 } }`.

Apagar una categoria no significa lo mismo en todas, y es a proposito:

- **Las dos que llevan secretos (`linkedAccounts` y `aiKeys`) borran sus datos de la nube** en
  la misma peticion: metadatos y blobs, siempre los dos. No hay opcion de borrar solo los blobs:
  quien apaga "Cuentas vinculadas" espera que dejen de estar en la nube, no que se queden quietas
  ahi. Los dispositivos conservan sus copias locales y reciben los borrados como lapidas en el
  feed. La interfaz tiene que confirmarlo con un aviso de peligro. Mientras sigan apagadas, la
  API rechaza cualquier escritura de esa categoria con **409** `sync_category_disabled`: un
  cliente viejo que no sepa de la pausa no puede volver a subir lo que el usuario quito.
- Borrar las cuentas deja sin cuenta a los repositorios que las usaban (`linkedAccountId: null`
  en el feed). **Los dispositivos conservan su eleccion local de cuenta por repositorio** mientras
  la categoria este apagada y no aplican ese `null`; al encenderla, suben sus cuentas y vuelven a
  asignarlas.
- **Encender una categoria de secretos sin boveda** no sube ningun secreto. El siguiente
  dispositivo que sincroniza ve la categoria activa y, si no hay boveda, propone crearla con la
  frase de sincronizacion. La web no puede crearla.
- **Las otras cuatro se pausan**: los datos se quedan, los dispositivos dejan de subir y de
  aplicar cambios, y al encenderla otra vez se retoma. Apagar la apariencia porque se quiere
  un tema distinto en cada equipo no tiene por que borrar nada. Pausar "Workspaces y
  repositorios" no toca los workspaces compartidos de las organizaciones: son del equipo, no
  sincronizacion personal. **La pausa la cumplen los dispositivos, no la API**: el panel web
  puede seguir editando esos datos, porque el panel es la copia de la nube y pausar solo decide
  si los equipos la siguen.
- **El feed no filtra por categoria.** Entrega todo lo que cambio, y cada dispositivo ignora lo
  de las categorias en pausa y lo guarda sin aplicarlo; al reanudar, aplica la copia de la nube
  que ya tiene. Las categorias de secretos no necesitan nada especial: al apagarse se borran, y
  sus borrados llegan como lapidas.

**Estos ajustes son de la cuenta, no del dispositivo.** El interruptor "Sync settings from the
cloud" de VS Code es otra cosa: decide si **esa instalacion** aplica lo que viene de la nube,
se guarda en la configuracion local de VS Code y no llama a la API. El texto del prototipo
decia que tambien sincronizaba las llaves de IA; manda lo que diga la cuenta.

El "Guardar" de la cabecera de la pagina de la web mezclaba cambios instantaneos con un
guardado global. Con este contrato cada interruptor guarda al cambiar (con su `If-Match`) y
el boton sobra; el perfil, que si es un formulario, guarda con su propio boton.

## Llaves de IA de organizacion

**F4.** Las da de alta un admin, viven cifradas en el servidor y los miembros las usan a
traves del proxy sin verlas nunca. Es la llave "ACME (equipo)" con alcance "Organizacion" de
los ajustes de IA de Desktop.

```
GET /v1/organizations/{orgId}/ai-keys
{
  "items": [
    {
      "id": "0192f440-...",
      "provider": "openai",
      "label": "ACME (equipo)",
      "model": "gpt-5-mini",
      "baseUrl": null,
      "masked": "sk-proj-••••••••9b1c",
      "isDefault": true,
      "createdBy": { "userId": "0192f3c0-...", "name": "Marta Ruiz" },
      "createdAt": "2026-09-10T10:00:00Z",
      "updatedAt": "2026-09-10T10:00:00Z",
      "lastUsedAt": "2026-09-30T09:40:00Z"
    }
  ]
}
```

Todos los miembros ven la lista enmascarada, porque la necesitan para elegir llave en "Donde
usar la IA". Nadie ve mas.

`POST /v1/organizations/{orgId}/ai-keys` (admin, `Idempotency-Key`):

```json
{
  "provider": "openai",
  "label": "ACME (equipo)",
  "apiKey": "sk-proj-...",
  "model": "gpt-5-mini",
  "baseUrl": null,
  "isDefault": true
}
```

**201** con la llave enmascarada. El alta y la rotacion (el `PATCH` con `apiKey`) son **los
unicos endpoints que reciben en claro un secreto de un tercero para guardarlo**, y es por
diseno: la llave es de la organizacion y el servidor tiene que poder usarla. Llega por TLS, se
cifra en el acto y el valor en claro no se escribe en ningun log ni traza (tipos `secrecy` con
`Debug` redactado).

- Cifrado de sobre ([ADR 0013](adr/0013-proxy-de-ia-de-organizacion.md)): una llave de datos
  aleatoria por secreto, AES-256-GCM con nonce de 96 bits y datos asociados
  `ramal:org-ai-key:v1:{orgId}:{keyId}`; la llave de datos se envuelve con la llave maestra
  activa (`RAMAL__SECRETS__KEKS` y `RAMAL__SECRETS__ACTIVE_KEK`) con datos asociados
  `ramal:org-ai-dek:v1:{orgId}:{keyId}:{kekId}`. Rotar la llave maestra solo reenvuelve las
  llaves de datos, que son 32 bytes cada una; por eso el `kekId` va en la envoltura y no en los
  datos asociados del secreto, que no se tocan.
- **`ollama` no se admite** (**422**): el servidor no puede llegar al `localhost` de nadie.
- **Host fijo para los proveedores conocidos.** `openai`, `anthropic`, `groq`, `google`,
  `mistral` y `openrouter` van a `api.openai.com`, `api.anthropic.com`, `api.groq.com`,
  `generativelanguage.googleapis.com`, `api.mistral.ai` y `openrouter.ai`; para ellos `baseUrl`
  tiene que ser `null` (**422** si no). Solo `custom` lleva `baseUrl`, y solo si
  `RAMAL__AI__ALLOW_CUSTOM_BASE_URLS=true`. Sin esto, un admin podria apuntar la llave de otra
  persona a su propio servidor y recibirla en la cabecera `Authorization`.
- **`baseUrl` es una puerta de SSRF** y se trata como tal: solo `https`, y la guarda actua **al
  conectar, en cada llamada** (alta, prueba y generacion), con un resolvedor propio que descarta
  las direcciones privadas, de loopback, de enlace local, de metadatos de la nube y sus formas
  IPv6 (las redes exactas estan en ADR 0013), sin seguir redirecciones y sin el proxy del
  entorno. Comprobar el DNS solo al guardar no impide que el nombre apunte despues a otra
  direccion.
- La mascara la calcula el servidor con la misma regla que el cliente.

`PATCH /v1/organizations/{orgId}/ai-keys/{id}` (admin) cambia `label`, `model`, `isDefault`,
`apiKey` (rotar la llave: se cifra con una llave de datos nueva) o `baseUrl` de una llave
`custom`. **200**. **Cambiar `baseUrl` exige mandar `apiKey` en la misma peticion** (**422**
`api_key_required` si no), deja un evento de auditoria y avisa a la owner de la organizacion: un
admin no puede redirigir una llave que no tiene.

`DELETE /v1/organizations/{orgId}/ai-keys/{id}` (admin) **204**. Las referencias a ella en
las preferencias de IA de los miembros caen en la predeterminada por la regla de resolucion.

`POST /v1/organizations/{orgId}/ai-keys/{id}/test` (admin) hace una llamada minima al
proveedor (la lista de modelos) con la llave. **200** con
`{ "ok": true, "provider": "openai", "model": "gpt-5-mini", "latencyMs": 412 }` o
`{ "ok": false, "error": "unauthorized" }` (`unauthorized`, `quota`, `network`, `model_not_found`).
Aqui si hay prueba de conexion, a diferencia de las cuentas de plataforma: el servidor tiene
la llave.

### Politica de IA de la organizacion

```
GET /v1/organizations/{orgId}/ai-settings
{
  "proxyEnabled": true,
  "allowedFeatures": ["commit_message", "pull_request", "commit_summary"],
  "dailyRequestsPerMember": 200,
  "monthlyTokenBudget": 5000000,
  "myUsage": { "requestsToday": 37, "dailyLimit": 200, "monthTokens": 1834000 },
  "updatedAt": "..."
}
```

`PATCH` (admin) cambia `proxyEnabled`, `allowedFeatures`, `dailyRequestsPerMember` y
`monthlyTokenBudget`. `monthlyTokenBudget` `null` es sin tope. Los miembros la leen para saber que
funciones tienen y cuanto les queda: `myUsage` trae sus peticiones de hoy (contando las
reservadas), su limite diario y los tokens que lleva la organizacion este mes. La politica no
elige llave por funcion (eso es la preferencia de cada miembro) y no viaja en el feed: las apps
la leen al abrir los ajustes de IA y cuando una generacion devuelve un 409 de politica.

`GET /v1/organizations/{orgId}/ai-usage?from=2026-09-01T00:00:00Z&to=...` (admin) devuelve el
consumo por miembro, funcion y llave (peticiones, tokens de entrada y salida, errores). Es la
vista de gasto que un admin necesita para que las llaves de organizacion no sean un cheque en
blanco. No hay pantalla en el diseno; el endpoint entra en F4 y la pantalla es la fila W-36 de
gaps.md (P2).

## El proxy de IA

**F4.** `POST /v1/ai/generate`, solo desde una sesion nativa y **viva**. Solo para llaves de
organizacion. Las personales nunca pasan por aqui. La web no tiene interfaz de IA, asi que una
sesion web no puede gastar llaves.

```json
{
  "organizationId": "0192f3d0-...",
  "keyId": null,
  "feature": "commit_message",
  "language": "es",
  "options": { "conventionalCommits": true, "extraInstructions": "Usa el imperativo." },
  "input": {
    "branch": "feature/graph-ui",
    "files": [
      {
        "path": "src/graph/zoom.ts",
        "status": "added",
        "binary": false,
        "patch": "@@ -0,0 +1,88 @@\n+export ..."
      },
      { "path": "src/graph/lanes.ts", "status": "modified", "binary": false, "patch": "..." }
    ]
  }
}
```

- `keyId` `null` es la predeterminada de la organizacion.
- `feature` decide la forma de `input`:
  - `commit_message`: `branch` y `files` (el diff preparado, ya filtrado).
  - `pull_request`: `baseBranch`, `headBranch`, `commits` (`sha`, `summary`, `body`) y
    `files` (estadistica por fichero y, si cabe, el parche), mas `template` opcional.
  - `commit_summary`: `commit` (`sha`, `message`) y `files`.
- **Es una entrada estructurada, no un prompt libre.** El servidor monta el prompt con las
  plantillas de `crates/ai`, las mismas que usa el motor con una llave personal, asi que el
  resultado es igual por los dos caminos.
- **Eso no impide usarla como chat**: `extraInstructions`, `template`, los cuerpos de commit y
  los parches son texto libre y la salida vuelve al cliente. Lo que acota el abuso son los
  limites: `extraInstructions` hasta 500 caracteres; un tope de tokens de salida por funcion
  (`max_tokens` de 400 para `commit_message`, 1500 para `pull_request` y 600 para
  `commit_summary`); como mucho 2 generaciones a la vez por usuario y organizacion; y la cuota
  reservada antes de llamar.

**La cuota se reserva antes de llamar.** Comprobar la tabla de consumo y escribir la fila al
terminar deja pasar a la vez todas las peticiones paralelas, multiplicadas por las replicas. Por
eso cada generacion toma la fila de contadores de la organizacion con `FOR UPDATE`, comprueba la
cuota diaria del miembro, el presupuesto del mes y las generaciones en curso de ese usuario, y
deja una fila `pending` con los tokens de entrada mas el tope de salida antes de llamar al
proveedor. Al terminar se liquida con el uso real. Si el cliente cancela, se liquida con la
entrada mas el tope de salida, porque el proveedor cobra la llamada entera.

Respuesta con `Accept: application/json`: **200** con el resultado entero:

```json
{
  "feature": "commit_message",
  "result": {
    "summary": "feat(graph): anade zoom al grafo",
    "description": "- Nuevo modulo de zoom\n- Ajusta el calculo de carriles"
  },
  "excludedPaths": [".env.local"],
  "usage": { "inputTokens": 1834, "outputTokens": 96, "provider": "openai", "model": "gpt-5-mini" }
}
```

Con `Accept: text/event-stream`, el mismo contenido en streaming (SSE), que es lo que pinta
el "Generando..." del diseno a medida que llega el texto:

```
event: delta
data: {"text":"feat(graph): "}

event: delta
data: {"text":"anade zoom al grafo"}

event: result
data: {"summary":"feat(graph): anade zoom al grafo","description":"- ..."}

event: usage
data: {"inputTokens":1834,"outputTokens":96,"provider":"openai","model":"gpt-5-mini"}

event: done
data: {}
```

`result` segun la funcion: `{ summary, description }`, `{ title, body }` o `{ summary }`. Un
fallo antes de empezar el stream es un codigo HTTP normal; un fallo a mitad es
`event: error` con `{ "code": "..." }` y cierre. El cliente solo recibe la categoria del error,
nunca el texto del proveedor, que puede traer fragmentos de la peticion. Si el cliente cierra la
conexion (el usuario cancela), la API corta la llamada al proveedor. La respuesta lleva
`X-Accel-Buffering: no` para que el gateway no acumule el stream.

| Codigo | Cuando                                                                                                                                                                        |
| ------ | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| 200    | Resultado, en JSON o en stream.                                                                                                                                               |
| 404    | No eres miembro de la organizacion, o la llave no existe en ella.                                                                                                             |
| 409    | `ai_feature_disabled` (la politica no permite esa funcion), `ai_proxy_disabled` o `no_default_key`.                                                                           |
| 413    | La entrada pasa de 256 KiB.                                                                                                                                                   |
| 422    | La forma de `input` no corresponde a `feature`.                                                                                                                               |
| 429    | Limite por minuto, `ai_concurrency_limit` (ya hay 2 generaciones en curso) o `ai_quota_exceeded` (cuota diaria del miembro o presupuesto mensual), con `Retry-After`.         |
| 502    | `ai_provider_error`: el proveedor rechazo la llave, no tiene saldo o fallo. Lleva solo la categoria (`unauthorized`, `quota`, `unavailable`), nunca el mensaje del proveedor. |
| 504    | El proveedor no respondio a tiempo (60 segundos).                                                                                                                             |

El 502 y el 504 no son un 500 a proposito: el cliente tiene que distinguir "Ramal fallo" de
"la llave de tu organizacion no tiene saldo", porque lo segundo lo arregla un admin.

### Que datos salen, a donde, y que se guarda

Esto tiene que estar escrito igual en la interfaz, porque es codigo del usuario saliendo de
su maquina.

- **Que se envia:** el diff de los ficheros preparados (o los commits de la rama), rutas,
  estados, nombres de rama, mensajes de commit, las instrucciones extra y el idioma. Antes de
  enviar, el motor quita los ficheros sensibles, los binarios y los que casan con
  `excludePatterns`, y recorta cada parche y el total para caber en 256 KiB.
- **A donde:** a la API de Ramal por TLS, y de ahi al proveedor de la llave con la llave de la
  organizacion.
- **La API vuelve a pasar el filtro de ficheros sensibles** y quita lo que se hubiera colado.
  Lo que quita lo devuelve en `excludedPaths`. Es defensa en profundidad: un cliente viejo o
  con un fallo no puede mandar un `.env` al proveedor.
- **Que se registra:** organizacion, usuario, llave, funcion, proveedor, modelo, tokens
  reservados y reales, latencia y resultado. Sirve para las cuotas y la vista de consumo, y se
  conserva 90 dias.
- **Que no se registra ni se guarda, nunca:** la entrada ni la salida. Ni en la base, ni en
  los logs, ni en una traza de error. El cuerpo de la peticion no pasa por el middleware de
  trazas.
- **Lo que haga el proveedor con los datos** depende de su politica para su API, y la
  interfaz lo dice con el nombre del proveedor delante.

## Cambios incrementales

**F2.** `GET /v1/sync/changes?since=<cursor>&limit=`, nativa y **viva**. Es lo que permite a
un dispositivo ponerse al dia sin bajarse todo en cada arranque. La mecanica viene de
Gastonomo (ADR 0023), con dos cosas que Ramal necesita y Gastonomo no: **borrados de verdad**
y **datos compartidos entre usuarios**.

```json
{
  "cursor": "djE6eyJ1Ijo0Mn0...",
  "hasMore": false,
  "profile": { "...": "Me" },
  "syncSettings": null,
  "vault": { "enabled": true, "keyVersion": 1, "version": 3, "rotationRecommended": false },
  "preferences": [
    { "namespace": "git", "version": 5, "updatedAt": "...", "document": { "...": "..." } }
  ],
  "organizations": [{ "id": "0192f3d0-...", "name": "ACME Studio", "role": "owner" }],
  "workspaces": [],
  "repositories": [],
  "linkedAccounts": [
    {
      "...": "LinkedAccount",
      "secret": { "keyVersion": 1, "alg": "xchacha20poly1305", "nonce": "...", "ciphertext": "..." }
    }
  ],
  "aiKeys": [],
  "organizationAiKeys": [],
  "deleted": [
    { "type": "workspace", "id": "0192f3f0-..." },
    { "type": "organization", "id": "..." }
  ]
}
```

- Cada coleccion reutiliza el DTO de su endpoint. Las cuentas y las llaves personales llevan
  aqui su `secret` embebido: es la forma en la que un dispositivo recibe los blobs sin una
  peticion por cuenta.
- `profile`, `syncSettings` y `vault` solo aparecen si cambiaron. Las preferencias, solo los
  espacios que cambiaron.
- `deleted` lista lo que hay que borrar en local, por tipo e id. Borrar algo que no se tiene
  no es un error: el cliente lo ignora.
- Un `deleted` de tipo `organization` significa que el usuario ya no es miembro: el cliente
  purga todo lo de esa organizacion.
- No van en el feed los miembros, las invitaciones, las sesiones, los dispositivos ni la
  politica de IA de la organizacion: los dispositivos no los necesitan al momento, y la web no
  usa el feed. Que un miembro deja la organizacion llega como `deleted` de tipo `organization`.
- **El feed no filtra por las categorias de sincronizacion.** Entrega todo; cada dispositivo
  ignora lo de las categorias en pausa (ver [Que se sincroniza](#que-se-sincroniza)).

### El cursor

**Un contador por ambito, no una marca de tiempo.** Hay un contador para el ambito personal
del usuario y uno por cada organizacion. Cada cambio toma el siguiente numero de su ambito
con el mismo `UPDATE ... RETURNING` con cerrojo de fila de Gastonomo, y de ahi sale la misma
garantia: **cuando el numero N de un ambito es visible, ninguno menor de ese ambito esta
todavia por confirmar**. Una secuencia global no la daria (dos transacciones pueden confirmar
en orden inverso), y una marca de tiempo se salta filas del mismo milisegundo o con el reloj
hacia atras.

Un contador por ambito, y no uno por usuario, porque un workspace compartido cambia para
todos los miembros a la vez. Con un contador por usuario habria que escribir el cambio una vez
por miembro. Con uno por organizacion se escribe una vez y lo leen todos.

El cursor es **opaco**: base64url de `{ "v": 1, "u": 42, "o": { "0192f3d0-...": 17 } }`
firmado con HMAC. Un cursor que no sea de esta API es **422** `invalid_cursor`.

- **Organizacion nueva** (aparece en las membresias y no en el cursor): se lee desde cero,
  que es su estado completo.
- **Organizacion que ya no esta**: se quita del cursor y se manda su `deleted`.
- **Cada ambito se corta en un tramo cerrado**: se piden `limit + 1` cambios de cada uno y el
  cursor avanza en cada ambito hasta donde el lote esta completo. `hasMore` es `true` si
  algun ambito se quedo a medias. `limit` por ambito: 200 por defecto, 1000 como tope.
- **El cliente avanza su cursor despues de guardar el lote**, en la misma transaccion local
  que los datos. Nunca al recibirlo.
- **Sin `since` es el arranque en frio**: se lee todo desde cero. Como el registro guarda una
  fila por entidad (la del ultimo cambio), leer desde cero es leer el estado actual, no la
  historia.

### Borrados y visibilidad

- Cada borrado deja una **lapida** con su numero de cambio. Las lapidas se purgan a los **90
  dias**. Un cliente cuyo cursor en algun ambito es anterior a la ultima lapida purgada
  recibe **410** `cursor_expired`: se ha podido perder un borrado, y la unica respuesta
  correcta es una resincronizacion completa (pedir sin `since` y, al terminar, borrar en
  local lo que no haya llegado). Tratarlo como "desde cero" en silencio dejaria en el
  dispositivo cosas que ya no existen.
- **El feed ensena el estado tal como lo ve quien pregunta.** Un workspace privado de otro
  miembro vive en el ambito de la organizacion, pero a los demas les llega como `deleted`, no
  como dato. Si pasa de compartido a privado, los demas reciben el borrado; si pasa de privado
  a compartido, lo reciben como nuevo. Algun `deleted` de algo que el cliente nunca tuvo es el
  precio, y es inofensivo.

### Lo que el feed no es

Es de solo lectura. Los dispositivos escriben por los endpoints normales, cada uno con su
idempotencia, y encolan esas llamadas cuando no hay conexion: el motor guarda una cola
persistente que respeta el orden, reintenta con el mismo id de cliente y trata como respuestas
normales el 409 `id_conflict`, el 412 y el 409 `sync_category_disabled` (descarta la escritura y
avisa, en vez de reintentarla para siempre). No hay un endpoint de "subir
lote de cambios": seria una segunda forma de escribir cada entidad, con su propia validacion
que mantener en paralelo.

Tampoco empuja: el dispositivo pregunta al arrancar, al volver el foco, despues de sus propias
escrituras y cada 5 minutos. Un canal de avisos en tiempo real (SSE con "hay cambios") esta en
[Lo que no existe todavia](#lo-que-no-existe-todavia).

## Publicaciones y descargas

**F1**, antes de la primera build publica.

```
GET /v1/releases/latest?product=desktop&channel=stable
{
  "product": "desktop",
  "version": "2.4.0",
  "channel": "stable",
  "releasedAt": "2026-09-30T08:00:00Z",
  "notesUrl": "https://ramal.dev/releases/2.4.0",
  "minimumSupportedVersion": "2.0.0",
  "platforms": [
    {
      "os": "macos",
      "requirements": { "es": "macOS 12 o posterior", "en": "macOS 12 or later" },
      "installMethod": "brew_cask",
      "artifacts": [
        { "id": "0192f500-...", "kind": "dmg", "arch": "arm64", "primary": true, "url": "https://github.com/<propietario>/ramal-releases/releases/download/v2.4.0/Ramal-2.4.0-arm64.dmg", "fileName": "Ramal-2.4.0-arm64.dmg", "sizeBytes": 112459776, "sha256": "base64url", "external": false },
        { "id": "0192f501-...", "kind": "dmg", "arch": "x64", "primary": false, "url": "...", "fileName": "Ramal-2.4.0-x64.dmg", "sizeBytes": 123731968, "sha256": "...", "external": false }
      ]
    }
  ]
}
```

- `product`: `desktop` o `vscode`. La extension tiene su propia version (la 1.3.0 del diseno
  frente a la 2.4.0 de Desktop), asi que son dos publicaciones distintas y la pagina hace dos
  peticiones.
- `channel`: `stable` o `beta`. `beta` devuelve la mas reciente de las dos: quien esta en beta
  tambien recibe las estables.
- `kind`: `dmg`, `nsis`, `msi`, `deb`, `rpm`, `appimage`, `flathub`, `vsix`, `marketplace` y
  `openvsx`. `arch` puede ser `universal` en el `.dmg` que sirve para los dos procesadores de
  macOS. **Las etiquetas visibles** ("Apple Silicon (.dmg)", "Fedora / RHEL
  (.rpm)") **las pone la web** desde `@ramal/i18n` con `kind` y `arch`; guardarlas en la base
  seria guardar texto de interfaz en dos idiomas en un sitio que no es el suyo.
- `external: true` son enlaces a una tienda (Flathub, Marketplace, Open VSX): sin tamano ni
  sha256, y con URL fijas del codigo, no las que diga la CI. Coincide con el diseno, que no
  ensena tamano para Flathub.
- `installMethod` es `brew_cask`, `winget`, `flatpak` o `null`, y la web escribe el comando que
  se copia desde una tabla fija de `@ramal/domain`. No es texto libre porque la landing invita a
  pegarlo en una terminal.
- `sha256` y la firma del actualizador estan para quien quiera verificar la descarga; la
  pagina de "gracias por descargar" que el analisis proponia puede ensenarlos.
- `requirements` va por plataforma y publicacion porque depende de la build. El texto del
  diseno para Linux ("Ubuntu 20.04+, Debian 11+") **no vale con Tauri 2**, que necesita
  `webkit2gtk-4.1`: la base real es Ubuntu 22.04 y Debian 12.
- La tarjeta "Windows Portable (.zip)" del diseno se quita
  ([ADR 0016](adr/0016-actualizaciones-y-descargas.md)): el empaquetador de Tauri no la produce y
  no se actualizaria sola.
- El instalador MSI se ofrece para despliegues gestionados y no se actualiza solo; la tarjeta lo
  dice. El de actualizacion automatica en Windows es el NSIS.

La landing se prerenderiza con esta respuesta en tiempo de build y la refresca en el cliente;
el sistema operativo se detecta en el navegador. Respuesta con
`Cache-Control: public, max-age=300, stale-while-revalidate=3600` y `ETag`.

`GET /v1/releases?product=desktop&channel=stable&cursor=&limit=` pagina las publicaciones con
sus notas. Es "Notas de la version" del pie de pagina.

### Manifiesto del actualizador de Tauri

```
GET /v1/desktop/updates/{target}/{arch}/{currentVersion}?channel=stable
```

Es la URL que va dentro de `tauri.conf.json` con las variables de Tauri: `{{target}}`
(`darwin`, `windows`, `linux`), `{{arch}}` (`x86_64`, `aarch64`) y `{{current_version}}`.

- **204** si no hay nada mas nuevo para ese destino.
- **200** con el formato que dicta Tauri, **en snake_case** porque no es nuestro:

```json
{
  "version": "2.4.1",
  "notes": "Corrige el calculo de carriles con ramas muy largas.",
  "pub_date": "2026-10-05T08:00:00Z",
  "url": "https://github.com/<propietario>/ramal-releases/releases/download/v2.4.1/Ramal_2.4.1_aarch64.app.tar.gz",
  "signature": "dW50cnVzdGVkIGNvbW1lbnQ6..."
}
```

- Solo se ofrecen los paquetes que Tauri sabe actualizar: `.app` en macOS, NSIS en Windows y
  AppImage en Linux. El manifiesto da un solo paquete por sistema y arquitectura, asi que en
  Windows el formato de actualizacion es el NSIS; quien instalo el MSI, un `.deb` o un `.rpm` no
  recibe actualizacion por aqui.
- **Despliegue gradual:** cada publicacion tiene un porcentaje. La app manda la cabecera
  `X-Ramal-Rollout-Bucket` con un numero de 0 a 99 que calcula una vez de una semilla aleatoria
  propia, y la API ofrece la version si ese numero cae por debajo del porcentaje. Sin cabecera,
  solo se ofrece al 100%. No se manda el `installationId`: seria un identificador estable de la
  instalacion en cada consulta, tambien sin cuenta.
- **Solo hacia delante:** la API compara con `version_sort` y solo ofrece una version
  estrictamente mayor que `{currentVersion}`; nunca una igual o anterior.
- **Retirada:** una publicacion retirada deja de ofrecerse, y nada mas. Nadie baja de version: a
  quien ya la instalo le llega la siguiente correccion. Es el interruptor de emergencia que GitHub
  Releases por si solo no da.
- La firma es la del par de llaves del actualizador de Tauri, que es independiente de la
  firma de codigo de macOS y Windows. **Perder la llave privada deja todas las instalaciones
  sin poder actualizarse nunca mas**: copia fuera de linea obligatoria.
- `tauri.conf.json` lleva como segundo endpoint el `latest.json` estatico del repositorio de
  publicaciones, por si la API no responde.

### Registro de publicaciones

`POST /v1/admin/releases` y `PATCH /v1/admin/releases/{id}` los llama la CI, que se autentica
con el token OIDC de GitHub Actions del workflow de publicacion: audiencia `ramal-api`,
repositorio y workflow fijados en la configuracion (`RAMAL__RELEASES__OIDC_*`). Es un reino de
autenticacion aparte: no es un usuario y no puede hacer nada mas. Hasta que exista, sirve un token
de publicacion (`RAMAL__RELEASES__PUBLISH_TOKEN_HASH`) del que la API guarda solo el hash y que
compara en tiempo constante.

```json
{
  "product": "desktop",
  "version": "2.4.1",
  "channel": "beta",
  "releasedAt": "2026-10-05T08:00:00Z",
  "notesUrl": "https://ramal.dev/releases/2.4.1",
  "notes": { "es": "...", "en": "..." },
  "rolloutPercentage": 10,
  "minimumSupportedVersion": "2.0.0",
  "platforms": [
    {
      "os": "macos",
      "requirements": { "es": "macOS 12 o posterior", "en": "macOS 12 or later" },
      "installMethod": "brew_cask",
      "artifacts": [
        {
          "kind": "dmg",
          "arch": "arm64",
          "primary": true,
          "url": "...",
          "fileName": "...",
          "sizeBytes": 112459776,
          "sha256": "..."
        },
        {
          "kind": "app_tar_gz",
          "arch": "arm64",
          "primary": false,
          "url": "...",
          "updaterTarget": "darwin-aarch64",
          "updaterSignature": "..."
        }
      ]
    }
  ]
}
```

**201**. **409** `release_exists` si la version ya esta. `PATCH` cambia `channel` (promover de
beta a estable), `rolloutPercentage`, `minimumSupportedVersion` y `yanked` (retirar). Los
artefactos de actualizador (`app_tar_gz` y los que llevan `updaterTarget`) no aparecen en la
pagina de descargas.

**La API no se cree a ciegas lo que registra la CI**, porque quien tenga esa credencial decide lo
que los usuarios descargan y ejecutan:

- Cada `url` tiene que empezar por
  `https://github.com/<propietario>/ramal-releases/releases/download/v{version}/`, con el
  propietario de la configuracion y la version de la propia publicacion (**422** si no).
- `installMethod` es un valor de la lista, nunca un comando.
- `updaterSignature` se verifica al registrar con la llave publica de minisign del actualizador
  que la API tiene en su configuracion. Una firma que no verifica es **422**.
- `notes` se guarda como texto plano o Markdown, y la web y el dialogo de actualizacion de Desktop
  lo pintan sin HTML.

**`minimumSupportedVersion`** es la version por debajo de la cual una instalacion deja de poder
usar la nube. El motor la lee de `GET /v1/releases/latest` al buscar actualizaciones; si esta por
debajo, deja de sincronizar y la interfaz ensena "Actualiza para seguir sincronizando", con el
boton de actualizar. Todo lo local sigue funcionando (decision 1).

## Broker OAuth de plataformas

**F2.** Solo para los proveedores que **exigen un secreto de cliente** para canjear el codigo.
Una app de escritorio no puede llevar un secreto dentro del binario (se extrae en cinco
minutos), asi que la API pone el secreto en el canje y **no guarda el token**.

Para el resto no hace falta y no se usa:

| Plataforma                    | Como se conecta desde el motor, sin la API                         | Broker |
| ----------------------------- | ------------------------------------------------------------------ | ------ |
| GitHub                        | Device flow de la OAuth App (solo necesita el `client_id`)         | No     |
| GitLab.com                    | Authorization Code + PKCE con aplicacion no confidencial, loopback | No     |
| GitLab propio, Gitea, Forgejo | PAT (o PKCE si el admin registra una aplicacion)                   | No     |
| Azure DevOps                  | PAT de organizacion (el OAuth de Azure DevOps se retira en 2026)   | No     |
| Bitbucket Cloud               | OAuth 2.0 con consumer, que exige secreto; o un API token          | **Si** |

`GET /v1/platform-oauth/providers` devuelve que proveedores tiene el broker y los `client_id`
publicos de los flujos que no lo necesitan, para poder rotarlos sin publicar una version:

```json
{
  "items": [
    { "platform": "github", "flow": "device", "clientId": "Ov23li...", "broker": false },
    { "platform": "gitlab", "flow": "pkce", "clientId": "c0ffee...", "broker": false },
    { "platform": "bitbucket", "flow": "broker", "clientId": null, "broker": true }
  ]
}
```

El motor lleva estos valores compilados y usa la respuesta solo para sobreescribirlos: tiene
que poder conectar una cuenta aunque la API no responda.

### El flujo con broker

1. El motor genera un verificador propio y su reto S256, abre un servidor en loopback y lanza
   el navegador a:

   `GET /v1/platform-oauth/bitbucket/authorize?redirect_uri=http://127.0.0.1:53124/callback&code_challenge=...&state=...`

   La API guarda el flujo (reto, `redirect_uri` de loopback, `state` del cliente) y redirige a
   Bitbucket con **su propio** `state`. Solo acepta `redirect_uri` de loopback, con el mismo
   validador estricto que el [inicio de sesion de las apps](#inicio-de-sesion-de-las-apps-nativas):
   un prefijo dejaria pasar `http://127.0.0.1:1@evil.example/callback`, y el codigo de Bitbucket
   de la victima iria a otro servidor.

2. Bitbucket vuelve a `GET /v1/platform-oauth/bitbucket/callback?code=...&state=...`. La API
   **no canjea todavia**: redirige al loopback del motor con
   `?code=...&flow=<id del flujo>&state=<state del cliente>`, construido con la API de `url`. El
   navegador aterriza en el receptor del motor, que sirve la pagina "Cuenta conectada, vuelve a
   Ramal" (o el error del proveedor). Si el flujo no existe o caduco, la API no tiene a donde
   redirigir y manda al navegador a `https://ramal.dev/connect/error?reason=expired`, una pagina
   de la web que explica que hay que empezar otra vez desde la app.
3. El motor canjea:

   ```
   POST /v1/platform-oauth/bitbucket/token
   { "flowId": "...", "code": "...", "codeVerifier": "..." }
   ```

   La API comprueba que el verificador casa con el reto del flujo, canjea el codigo con
   Bitbucket poniendo el secreto, **devuelve los tokens en la respuesta y borra el flujo**.

   ```json
   {
     "accessToken": "...",
     "refreshToken": "...",
     "expiresIn": 7200,
     "scopes": ["repository", "pullrequest"],
     "tokenType": "bearer"
   }
   ```

4. Los tokens de Bitbucket caducan a las 2 horas. Para refrescarlos:
   `POST /v1/platform-oauth/bitbucket/refresh { "refreshToken": "..." }` con la misma
   respuesta.

**Por que el reto propio.** Bitbucket no admite PKCE. Sin el reto, cualquiera que viera un
codigo de autorizacion podria canjearlo a traves del broker, que pone el secreto por el. Con
el reto, el codigo solo lo canjea quien empezo el flujo. Es PKCE entre el motor y la API donde
el proveedor no lo da.

**Lo que se promete sobre el token:** pasa por la memoria del proceso durante el canje y
nada mas. No se escribe en la base, no se registra (el middleware de trazas registra la ruta
sin la query y nunca el cuerpo), y las dos respuestas llevan `Cache-Control: no-store`. Es el
unico sitio de toda la API por donde pasa un token de plataforma, y por eso esta aislado en su
propio modulo.

**Sin cuenta de Ramal.** El broker es publico porque Ramal funciona sin cuenta y conectar
Bitbucket es parte de trabajar con Git. Lo acota el limite por IP. El riesgo que se asume: el
broker permite usar un refresh token robado de nuestro consumer; quien lo tiene ya tiene
acceso a esa cuenta de Bitbucket hasta que caduque.

## Lo que el cliente deriva, y no pide

Hay datos del diseno que parecen de la API y no lo son. Pedirlos al servidor seria repetir en
Rust lo que `@ramal/domain` ya calcula igual para las tres superficies.

- **Iniciales y colores de avatar** (personas, organizaciones y cuentas): del nombre, con la
  paleta de carriles.
- **Tiempo relativo** ("hace 5 min", "Sincronizado hace 3 d"): de los instantes, con
  `Intl.RelativeTimeFormat`.
- **Abreviatura y punto de estado de un dispositivo**: de `kind`, `os` y `lastSyncAt`.
- **Porcentaje de asientos**: de `used` y `total`.
- **Nombre visible de un repositorio**: alias o nombre (la API ya lo da resuelto en las
  entradas de workspace, porque ahi depende de quien pregunta).
- **Agrupar y filtrar repositorios** por plataforma, cuenta o workspace.
- **Mascara de una llave personal**: la calcula el motor antes de subirla.
- **Pull requests, checks y revisiones**: el motor los pide a cada plataforma con el token de
  la cuenta del repositorio. **La API no ve ningun pull request.** El panel web no ensena pull
  requests, asi que no hace falta ningun proxy.
- **Todo el estado de Git**: rama, ahead, behind, cambios, grafo, conflictos, stashes.

## Reglas que no se pueden romper

Si un cambio en la API choca con alguna de estas reglas, el cambio esta mal, no la regla.

1. **La API nunca recibe en claro un token de plataforma, una llave de IA personal, la frase
   de sincronizacion, el codigo de recuperacion ni la llave de la boveda.** Ningun endpoint los
   acepta. La unica excepcion es el canje del broker OAuth, donde un token de Bitbucket pasa
   por memoria, no se guarda, no se registra y sale con `no-store`.
2. **Una llave de IA de organizacion no sale nunca del servidor.** Ningun endpoint la
   devuelve, a nadie, en ningun formato. Solo la mascara.
3. **La API no usa tokens de plataforma.** No llama a GitHub, GitLab, Bitbucket, Gitea ni
   Azure DevOps en nombre de un usuario. La unica llamada a una plataforma es el canje y el
   refresco del broker, y el OAuth de identidad, cuyo token se descarta en la misma peticion.
4. **Un recurso de otra cuenta, o de una organizacion donde no eres miembro, responde 404,
   nunca 403.** El 403 solo existe para "es tuyo, pero te falta permiso".
5. **Ningun token se guarda en claro.** Refresh, recuperacion, verificacion, cambio de correo,
   invitacion, codigo de autorizacion, device code y codigo de usuario se guardan hasheados.
   Ninguno viaja en la ruta ni en la query de una peticion a la API: van en el cuerpo, y los
   enlaces de correo los llevan en el fragmento.
6. **Reutilizar un refresh token ya rotado revoca la sesion entera**, salvo la ventana de 30
   segundos sin sucesor usado, que no entrega tokens.
7. **La API no guarda rutas locales.** Ni del dispositivo que pregunta ni de otros.
8. **La entrada y la salida del proxy de IA no se guardan ni se registran.** Ni en la base, ni
   en logs, ni en trazas de error.
9. **El cursor del feed nunca es una marca de tiempo**, y el cliente lo avanza despues de
   guardar el lote, nunca al recibirlo. Un cursor caducado es 410, nunca "desde cero" en
   silencio.
10. **Las acciones irreversibles de la cuenta exigen reautenticacion reciente**: eliminar la
    cuenta, reiniciar la boveda, eliminar o transferir una organizacion, cambiar el correo,
    quitar un metodo de inicio de sesion.
11. **No hay CORS.** La web entra por el mismo origen y las apps por su motor. Anadir un origen
    permitido es abrir una puerta que ningun cliente necesita.
12. **Cada ciphertext esta ligado a su registro por los datos asociados** (usuario, tipo,
    registro, version de llave; en las llaves de organizacion, organizacion y llave, con la llave
    maestra en la envoltura). Un blob copiado a otra fila no descifra. **Y cada secreto de la
    boveda lleva sellado su destino** (host de la cuenta, URL base de la llave): un dispositivo
    no usa un secreto cuyo destino sellado no coincide con la fila.
13. **Toda escritura que un dispositivo puede repetir es idempotente**: por id del cliente, por
    identidad natural, por `Idempotency-Key` o por semantica de `PUT` y `DELETE`.
14. **La API no llama a una URL que venga de un usuario sin la guarda de SSRF**: solo `https`,
    sin redirecciones, sin redes privadas, comprobado al conectar en cada llamada; y los
    proveedores de IA conocidos tienen el host fijo.
15. **Toda lista va en `items`, todo error en `problem+json`, y los 401 tienen un solo texto.**
16. **Una sesion nativa nunca cuenta como reautenticacion reciente**, y las acciones con
    reautenticacion solo se aceptan desde la web.
17. **Ninguna redireccion se valida con un prefijo.** Las de loopback y `returnTo` se analizan con
    un analizador WHATWG y se construyen desde la URL analizada.

## Lo que no existe todavia

- **Segundo factor (TOTP, llaves de acceso).** El diseno no lo tiene y el analisis lo senalo
  como habitual en un producto que guarda credenciales de Git. Con la boveda de extremo a
  extremo el riesgo de un login robado es menor que en un diseno con cifrado de servidor (el
  atacante no puede leer los tokens sin la frase), pero sigue pudiendo borrar y ver metadatos.
  Primer candidato despues de F3.
- **SSO de organizacion (SAML, OIDC de empresa).** Cuando haya clientes que lo pidan.
- **Avisos en tiempo real del feed** (SSE con "hay cambios"). Hoy los dispositivos preguntan.
- **Historial de seguridad visible para el usuario** (inicios de sesion, cambios de
  contrasena, reinicios de la boveda). La tabla de auditoria existe desde F1 y los avisos por
  correo tambien; la pantalla no.
- **Cuentas de plataforma compartidas con el equipo y "Probar conexion" desde la web.**
  Descartadas por la decision de extremo a extremo, no pendientes.
- **Editar preferencias de Git, apariencia o IA desde la web.** El diseno solo pone los
  interruptores de sincronizacion; los endpoints ya lo permitirian si algun dia se dibuja.
- **Foto de organizacion, reordenar workspaces, selector de color.** Los datos lo soportan
  (el color ya es un campo); falta el diseno.
- **Pagina de precios y facturacion.** F5, con el proveedor sin decidir.
- **Recuperar secretos sin frase ni codigo.** No es un hueco: es imposible por diseno.

## Lo que hay que vigilar

- **El access token no se invalida al revocar**, salvo en los endpoints **viva**. Sigue valido
  hasta 15 minutos para leer el perfil o los workspaces. Es el compromiso de un JWT sin estado,
  acotado a lo que no entrega secretos.
- **El limite de peticiones es una ventana en memoria de cada proceso.** Con varias replicas el
  limite real se multiplica. Un limite compartido necesita Redis o equivalente. Pesa mas en el
  broker publico y en las cuotas de IA; las cuotas de IA se reservan en la base, no en memoria,
  precisamente por eso.
- **La boveda se puede atacar sin conexion con una copia de la base, o con las envolturas que
  recibe cualquier sesion nativa.** El coste de Argon2id y la longitud minima de la frase son toda
  la defensa; el correo de la primera descarga por dispositivo es la alarma. Subir los parametros
  por defecto es barato: cada boveda guarda los suyos.
- **La API ve metadatos que dicen mucho**: nombres de repositorios privados, hosts internos,
  nombres de clientes en los alias. Es lo que el usuario acepta al sincronizar, y el panel de
  sincronizacion tiene que decirlo con esas palabras.
- **El proxy de IA hace pasar codigo por Ramal.** Esta acotado (solo llaves de organizacion,
  entrada estructurada, filtro doble, nada guardado), pero cada organizacion tiene que saberlo
  antes de dar de alta una llave.
- **Perder la llave maestra del servidor pierde todas las llaves de IA de organizacion.** Copia
  fuera de linea, igual que la llave del actualizador.
- **La URL del actualizador es para siempre.** Va dentro de cada instalacion.
- **La normalizacion de remotos es un contrato de datos.** Cambiarla parte repositorios en dos y
  exige migracion.
- **La ciudad de las sesiones exige GeoIP**, que es un dato personal. Viene apagada.
- **El refresco con varias ventanas de VS Code** depende de la ventana de 30 segundos. Si
  aparecen cierres de sesion inesperados, es lo primero que hay que mirar.
- **El limite por correo en `login` deja que un tercero bloquee 15 minutos** el inicio de sesion
  con contrasena de una cuenta concreta. Se acepta: la ventana es corta, el OAuth sigue
  funcionando, y sin limite por cuenta se pueden probar contrasenas desde muchas IP.
- **El broker publico** permite gastar el cupo de nuestro consumer de Bitbucket a cualquiera.
  Si se abusa, pasa a exigir sesion de Ramal y el camino sin cuenta queda en el API token.

## Apendice A: pantalla o accion y endpoint

"Motor" significa que lo resuelve el motor de Rust del dispositivo sin la API. "Sin endpoint"
significa que la accion desaparece o es local, y dice por que.

### Ramal Web

| Pantalla                                    | Accion o dato                                                                                       | Endpoint                                                                                                                                     | Fase |
| ------------------------------------------- | --------------------------------------------------------------------------------------------------- | -------------------------------------------------------------------------------------------------------------------------------------------- | ---- |
| Landing                                     | "Descargar para {SO}"                                                                               | `GET /v1/releases/latest?product=desktop`                                                                                                    | F1   |
| Landing                                     | Tarjetas de descarga: principal, alternativas, tamano, requisitos, comando                          | `GET /v1/releases/latest?product=desktop` y `?product=vscode`                                                                                | F1   |
| Landing                                     | Version y fecha de la seccion de descargas                                                          | `version` y `releasedAt` de la misma respuesta                                                                                               | F1   |
| Landing                                     | VS Code Marketplace, Open VSX, `.vsix`                                                              | artefactos `external` de `product=vscode`                                                                                                    | F1   |
| Landing                                     | Copiar comando de instalacion                                                                       | Sin endpoint: portapapeles; la web escribe el comando desde `installMethod`                                                                  | F1   |
| Pie                                         | Notas de la version                                                                                 | `GET /v1/releases`                                                                                                                           | F1   |
| Nav publica                                 | "Abrir panel" y guarda de rutas                                                                     | `GET /v1/me`                                                                                                                                 | F1   |
| Login                                       | Correo y contrasena                                                                                 | `POST /v1/auth/login`                                                                                                                        | F1   |
| Login                                       | "La olvidaste"                                                                                      | `POST /v1/auth/password/forgot` (pagina nueva)                                                                                               | F1   |
| Nueva: restablecer                          | Nueva contrasena                                                                                    | `POST /v1/auth/password/reset`                                                                                                               | F1   |
| Registro                                    | Nombre, correo, contrasena, organizacion, terminos                                                  | `POST /v1/auth/register`                                                                                                                     | F1   |
| Login y registro                            | Continuar con GitHub, GitLab o Google                                                               | `POST /v1/auth/oauth/{provider}/start` y `GET .../callback`                                                                                  | F1   |
| Nueva: completar alta                       | Terminos y organizacion tras OAuth                                                                  | `GET` y `POST /v1/auth/oauth/signup`                                                                                                         | F1   |
| Nueva: verificar correo                     | Enlace, reenvio                                                                                     | `POST /v1/auth/email/verify`, `POST /v1/auth/email/verification`                                                                             | F1   |
| Nueva: autorizar app                        | "Autorizar Ramal Desktop / VS Code"                                                                 | `GET` y `POST /v1/auth/app/authorize`                                                                                                        | F1   |
| Nueva: activar                              | Codigo del device flow                                                                              | `POST /v1/auth/device/lookup`, `POST /v1/auth/device/decision`                                                                               | F1   |
| Nueva: reautenticar                         | Confirmar contrasena, repetir el OAuth o pedir un enlace al correo antes de una accion sensible     | `POST /v1/auth/reauthenticate`, `POST /v1/auth/oauth/{provider}/start` (`reauth`), `POST /v1/auth/reauthenticate/email-link` y `.../confirm` | F1   |
| Nueva: deshacer cambio de correo            | Enlace de la direccion anterior                                                                     | `POST /v1/auth/email-change/revert`                                                                                                          | F1   |
| Nueva: error de conexion de plataforma      | Flujo del broker desconocido o caducado                                                             | Sin endpoint: la API redirige a `/connect/error`                                                                                             | F2   |
| Nueva: invitacion                           | Ver y aceptar                                                                                       | `POST /v1/invitations/preview`, `POST /v1/invitations/accept`                                                                                | F3   |
| Nueva: invitaciones para mi                 | Aceptar o rechazar las invitaciones al correo de la cuenta                                          | `GET /v1/me/invitations`, `POST /v1/me/invitations/{id}/accept`, `DELETE /v1/me/invitations/{id}`                                            | F3   |
| Cabecera del panel                          | Avatar, iniciales, correo del menu                                                                  | `GET /v1/me`                                                                                                                                 | F1   |
| Cabecera del panel                          | Selector de organizacion y "Personal"                                                               | `GET /v1/organizations`                                                                                                                      | F3   |
| Cabecera del panel                          | "Crear organizacion" (formulario nuevo)                                                             | `POST /v1/organizations`                                                                                                                     | F3   |
| Cabecera del panel                          | Cerrar sesion                                                                                       | `POST /v1/auth/logout`                                                                                                                       | F1   |
| Menu lateral                                | Contadores de cada seccion                                                                          | `GET /v1/overview`                                                                                                                           | F2   |
| Resumen                                     | "Hola, {nombre}"                                                                                    | `GET /v1/me`                                                                                                                                 | F1   |
| Resumen                                     | Cuatro tarjetas                                                                                     | `GET /v1/overview`                                                                                                                           | F2   |
| Resumen                                     | Dispositivos sincronizados: nombre, app, version, SO, ultima sincronizacion, version desactualizada | `GET /v1/devices`                                                                                                                            | F1   |
| Resumen                                     | "Cerrar sesion en este dispositivo"                                                                 | `DELETE /v1/devices/{id}`                                                                                                                    | F1   |
| Resumen (nuevo)                             | Renombrar dispositivo                                                                               | `PATCH /v1/devices/{id}`                                                                                                                     | F1   |
| Dispositivos y sesiones (nuevo)             | Sesiones web con "este navegador", cerrar una, "Cerrar todas las demas"                             | `GET /v1/auth/sessions`, `DELETE /v1/auth/sessions/{id}`, `POST /v1/auth/sessions/revoke-others`                                             | F1   |
| Workspaces                                  | Tarjetas con chips de repositorios                                                                  | `GET /v1/workspaces?organizationId=`                                                                                                         | F2   |
| Workspaces                                  | Nuevo workspace (nombre, compartido)                                                                | `POST /v1/workspaces`                                                                                                                        | F2   |
| Workspaces                                  | "Compartido con {org}" / "Solo tu"                                                                  | `PATCH /v1/workspaces/{id}` (`visibility`)                                                                                                   | F3   |
| Workspaces                                  | Renombrar, eliminar                                                                                 | `PATCH` y `DELETE /v1/workspaces/{id}`                                                                                                       | F2   |
| Repositorios                                | Tabla: nombre y alias, origen con @cuenta, "Local en {dispositivo}", workspaces, actualizado        | `GET /v1/repositories`                                                                                                                       | F2   |
| Repositorios                                | Buscar repositorio, cuenta o workspace                                                              | `GET /v1/repositories?q=`                                                                                                                    | F2   |
| Repositorios                                | Casilla de workspace en el menu                                                                     | `POST /v1/workspaces/{id}/repositories`, `DELETE /v1/workspaces/{id}/repositories/{entryId}`                                                 | F2   |
| Repositorios                                | "Quitar de la cuenta"                                                                               | `DELETE /v1/repositories/{id}`                                                                                                               | F2   |
| Cuentas vinculadas                          | Grupos por plataforma, tarjeta, "N repos"                                                           | `GET /v1/linked-accounts`                                                                                                                    | F2   |
| Cuentas vinculadas                          | "+ Conectar"                                                                                        | Sin endpoint: la web explica que se conecta desde Desktop o VS Code (decision 2)                                                             | F2   |
| Cuentas vinculadas                          | "Probar conexion"                                                                                   | Sin endpoint: desaparece (decision 2)                                                                                                        | F2   |
| Cuentas vinculadas                          | "Compartir con el equipo", pastilla Tuya/Equipo                                                     | Sin endpoint: desaparece (decision 2)                                                                                                        | F2   |
| Cuentas vinculadas                          | "Desconectar"                                                                                       | `DELETE /v1/linked-accounts/{id}`                                                                                                            | F2   |
| Equipo                                      | "N de M asientos usados" y barra                                                                    | `GET /v1/organizations/{orgId}/seats`                                                                                                        | F3   |
| Equipo                                      | "Anadir asientos"                                                                                   | `POST /v1/organizations/{orgId}/subscription/seats` (oculto antes de F5)                                                                     | F5   |
| Equipo                                      | Miembros con rol                                                                                    | `GET /v1/organizations/{orgId}/members`                                                                                                      | F3   |
| Equipo                                      | Invitaciones pendientes                                                                             | `GET /v1/organizations/{orgId}/invitations`                                                                                                  | F3   |
| Equipo                                      | Selector de rol                                                                                     | `PATCH /v1/organizations/{orgId}/members/{userId}`                                                                                           | F3   |
| Equipo                                      | Invitar (correo, rol)                                                                               | `POST /v1/organizations/{orgId}/invitations`                                                                                                 | F3   |
| Equipo                                      | Reenviar invitacion                                                                                 | `POST /v1/organizations/{orgId}/invitations/{id}/resend`                                                                                     | F3   |
| Equipo                                      | Quitar miembro                                                                                      | `DELETE /v1/organizations/{orgId}/members/{userId}`                                                                                          | F3   |
| Equipo                                      | Quitar una invitacion pendiente                                                                     | `DELETE /v1/organizations/{orgId}/invitations/{id}`                                                                                          | F3   |
| Equipo                                      | Copiar correo                                                                                       | Sin endpoint: portapapeles                                                                                                                   | F3   |
| Ajustes de la organizacion (nuevo)          | Renombrar                                                                                           | `PATCH /v1/organizations/{orgId}`                                                                                                            | F3   |
| Ajustes de la organizacion (nuevo)          | Eliminar (reauth, confirmacion escrita)                                                             | `DELETE /v1/organizations/{orgId}`                                                                                                           | F3   |
| Ajustes de la organizacion (nuevo)          | Salir                                                                                               | `POST /v1/organizations/{orgId}/leave`                                                                                                       | F3   |
| Ajustes de la organizacion (nuevo)          | Transferir la propiedad (reauth)                                                                    | `POST /v1/organizations/{orgId}/transfer-ownership`                                                                                          | F3   |
| IA de la organizacion (nuevo, admins)       | Lista, alta, rotar, quitar llaves                                                                   | `GET`, `POST`, `PATCH` y `DELETE /v1/organizations/{orgId}/ai-keys`                                                                          | F4   |
| IA de la organizacion (nuevo, admins)       | Probar conexion                                                                                     | `POST /v1/organizations/{orgId}/ai-keys/{id}/test`                                                                                           | F4   |
| IA de la organizacion (nuevo, admins)       | Politica: funciones, cuota diaria, presupuesto                                                      | `GET` y `PATCH /v1/organizations/{orgId}/ai-settings`                                                                                        | F4   |
| IA de la organizacion (nuevo, admins, W-36) | Consumo por miembro, funcion y llave                                                                | `GET /v1/organizations/{orgId}/ai-usage`                                                                                                     | F4   |
| Configuracion en la nube                    | Perfil: nombre                                                                                      | `PATCH /v1/me`                                                                                                                               | F1   |
| Configuracion en la nube                    | Perfil: correo                                                                                      | `POST /v1/me/email-change`, `POST /v1/auth/email-change/confirm`                                                                             | F1   |
| Configuracion en la nube (nuevo)            | Contrasena: cambiar o establecer la primera                                                         | `POST /v1/me/password`                                                                                                                       | F1   |
| Configuracion en la nube                    | Que se sincroniza (seis interruptores)                                                              | `GET` y `PATCH /v1/me/sync-settings`                                                                                                         | F2   |
| Configuracion en la nube                    | Exportar configuracion                                                                              | `GET /v1/me/export`                                                                                                                          | F1   |
| Configuracion en la nube                    | Eliminar cuenta                                                                                     | `DELETE /v1/me`                                                                                                                              | F1   |
| Configuracion en la nube (nuevo)            | Estado de la boveda (configurada, desde cuando) y reiniciarla                                       | `vaultEnabled` y `vaultCreatedAt` de `GET /v1/me`; `DELETE /v1/vault`                                                                        | F2   |
| Configuracion en la nube (nuevo)            | Identidades vinculadas: listar, vincular, quitar                                                    | `GET /v1/me/identities`, `POST /v1/auth/oauth/{provider}/start` (`link`), `DELETE /v1/me/identities/{provider}`                              | F1   |

### Ramal Desktop

| Pantalla                          | Accion o dato                                                                                                 | Endpoint                                                                                                                                               | Fase            |
| --------------------------------- | ------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------ | --------------- |
| Primer arranque (nuevo)           | Iniciar sesion o seguir sin cuenta                                                                            | Navegador + `POST /v1/auth/app/token` (codigo o device flow)                                                                                           | F1              |
| Barra de titulo                   | Avatar                                                                                                        | `GET /v1/me` (y `profile` del feed)                                                                                                                    | F1              |
| Barra de titulo                   | Idioma y tema                                                                                                 | `PATCH /v1/me/preferences/appearance`                                                                                                                  | F2              |
| Inicio                            | Lista de repositorios sincronizada, filtros por cuenta y workspace                                            | `GET /v1/sync/changes`                                                                                                                                 | F2              |
| Inicio                            | Abrir, iniciar o clonar un repositorio                                                                        | Motor (Git) + `POST /v1/repositories`                                                                                                                  | F2              |
| Inicio                            | Primera sincronizacion de muchos repositorios                                                                 | `POST /v1/repositories/batch`                                                                                                                          | F2              |
| Clonar                            | Workspace de destino                                                                                          | `POST /v1/workspaces/{id}/repositories`                                                                                                                | F2              |
| Menu de repositorio               | Asignar alias                                                                                                 | `PATCH /v1/repositories/{id}`                                                                                                                          | F2              |
| Menu de repositorio               | Casillas de workspaces                                                                                        | `POST` y `DELETE /v1/workspaces/{id}/repositories...`                                                                                                  | F2              |
| Menu de repositorio               | Quitar de todos                                                                                               | `DELETE /v1/repositories/{id}/workspaces`                                                                                                              | F2              |
| Menu de repositorio               | + Nuevo workspace                                                                                             | `POST /v1/workspaces` con `repositoryIds`                                                                                                              | F2              |
| Menu de repositorio               | Quitar de la lista                                                                                            | `DELETE /v1/repositories/{id}` (la papelera es local)                                                                                                  | F2              |
| Menu de repositorio               | Renombrar carpeta, mostrar en carpeta, terminal, copiar ruta                                                  | Sin endpoint: sistema operativo                                                                                                                        | -               |
| Barra lateral                     | + Nuevo workspace                                                                                             | `POST /v1/workspaces`                                                                                                                                  | F2              |
| Menu de workspace                 | Renombrar, eliminar                                                                                           | `PATCH` y `DELETE /v1/workspaces/{id}`                                                                                                                 | F2              |
| Workspace                         | "Abrir todos" con repositorios no clonados aqui                                                               | Entradas con `repositoryId: null` y `cloneUrl`, del feed                                                                                               | F2              |
| Workspace                         | Pull requests agregados                                                                                       | Motor contra cada plataforma                                                                                                                           | -               |
| Ajustes > Perfil                  | Nombre, usuario, zona horaria, "Guardar cambios"                                                              | `GET` y `PATCH /v1/me`                                                                                                                                 | F1              |
| Ajustes > Perfil                  | Cambiar el correo o la contrasena                                                                             | Sin endpoint en Desktop: abre la pagina de la web con `openExternal` (la sesion nativa no cuenta como reautenticacion)                                 | F1              |
| Ajustes > Perfil                  | Cambiar foto                                                                                                  | `PUT /v1/me/avatar`                                                                                                                                    | F2              |
| Ajustes > Cuenta y equipo         | Tarjeta de plan: nombre, renovacion, asientos                                                                 | `GET /v1/organizations` (`plan`, `seats`)                                                                                                              | F3 (plan en F5) |
| Ajustes > Cuenta y equipo         | Gestionar suscripcion                                                                                         | `POST /v1/organizations/{orgId}/subscription/portal`                                                                                                   | F5              |
| Ajustes > Cuenta y equipo         | "Sesion iniciada como", Cerrar sesion                                                                         | `GET /v1/me`, `POST /v1/auth/logout`                                                                                                                   | F1              |
| Ajustes > Cuenta y equipo         | Organizacion y miembros, invitar, reenviar, rol, quitar                                                       | Los mismos que la pantalla Equipo de la web                                                                                                            | F3              |
| Ajustes > Cuenta y equipo         | Transferir o eliminar la organizacion, eliminar la cuenta                                                     | Sin endpoint en Desktop: abre la pagina de la web                                                                                                      | F3              |
| Ajustes > Cuenta y equipo (nuevo) | Que se sincroniza: los seis interruptores                                                                     | `GET` y `PATCH /v1/me/sync-settings` (los de secretos proponen crear la boveda si no hay)                                                              | F2              |
| Ajustes > Cuenta y equipo (nuevo) | Invitaciones pendientes para mi                                                                               | `GET /v1/me/invitations`, `POST .../accept`, `DELETE`                                                                                                  | F3              |
| Ajustes > Cuenta y equipo         | Sesiones activas, "Este dispositivo", "Cerrar", "Cerrar todas las demas"                                      | `GET /v1/auth/sessions`, `DELETE /v1/auth/sessions/{id}`, `POST /v1/auth/sessions/revoke-others`                                                       | F1              |
| Ajustes > Plataformas Git         | Lista por plataforma, predeterminada, usada en N repos                                                        | Feed o `GET /v1/linked-accounts`                                                                                                                       | F2              |
| Ajustes > Plataformas Git         | Anadir cuenta por OAuth                                                                                       | Motor contra la plataforma (device flow o PKCE); Bitbucket con `/v1/platform-oauth/bitbucket/*`; despues `POST /v1/linked-accounts` y `PUT .../secret` | F2              |
| Ajustes > Plataformas Git         | Anadir cuenta con token                                                                                       | Motor prueba el token; `POST /v1/linked-accounts` y `PUT .../secret`                                                                                   | F2              |
| Ajustes > Plataformas Git         | Actualizar credenciales                                                                                       | `PUT /v1/linked-accounts/{id}/secret`, `PATCH` de estado                                                                                               | F2              |
| Ajustes > Plataformas Git         | Hacer predeterminada                                                                                          | `PATCH /v1/linked-accounts/{id}` (`isDefault`)                                                                                                         | F2              |
| Ajustes > Plataformas Git         | Desconectar                                                                                                   | Motor revoca el permiso en la plataforma si se puede; despues `DELETE /v1/linked-accounts/{id}`                                                        | F2              |
| Ajustes > Plataformas Git         | Probar conexion y punto de estado                                                                             | Motor contra la plataforma; `PATCH` de `status` y `lastVerifiedAt`                                                                                     | F2              |
| Ajustes > IA                      | Llaves personales                                                                                             | Llavero local; con sincronizacion, `POST`, `PATCH`, `DELETE /v1/ai/keys` y `PUT .../secret`                                                            | F2              |
| Ajustes > IA                      | Llaves "De la organizacion", de solo lectura                                                                  | Solo `GET /v1/organizations/{orgId}/ai-keys`; el alta es W-16 en la web                                                                                | F4              |
| Ajustes > IA                      | Probar conexion                                                                                               | Personal: motor. Organizacion: `POST .../ai-keys/{id}/test`, solo admins                                                                               | F4              |
| Ajustes > IA                      | Cuota que le queda al miembro                                                                                 | `myUsage` de `GET /v1/organizations/{orgId}/ai-settings`                                                                                               | F4              |
| Ajustes > IA                      | Donde usar la IA, idioma, Conventional Commits, excluir, instrucciones                                        | `PATCH /v1/me/preferences/ai`                                                                                                                          | F2              |
| Commit, PR, detalle               | Generar con llave personal                                                                                    | Motor contra el proveedor                                                                                                                              | -               |
| Commit, PR, detalle               | Generar con llave de organizacion                                                                             | `POST /v1/ai/generate`                                                                                                                                 | F4              |
| Ajustes > Git                     | Identidad y comportamiento                                                                                    | `PATCH /v1/me/preferences/git`                                                                                                                         | F2              |
| Ajustes > Git                     | Llaves SSH                                                                                                    | Sin endpoint: nunca se sincronizan                                                                                                                     | -               |
| Ajustes > Apariencia              | Tema, idioma, nodos del grafo                                                                                 | `PATCH /v1/me/preferences/appearance`                                                                                                                  | F2              |
| Nueva: boveda                     | Configurar frase y codigo, desbloquear, cambiar frase, regenerar codigo, rotar (tambien cuando se recomienda) | `/v1/vault`, `/v1/vault/passphrase`, `/v1/vault/recovery`, `/v1/vault/rotate`, `/v1/vault/secrets`                                                     | F2              |
| Nueva: boveda                     | Olvide la frase y el codigo                                                                                   | Sin endpoint en Desktop: explica y abre el reinicio de la web                                                                                          | F2              |
| Fondo                             | Ponerse al dia al arrancar, al volver el foco, cada 5 minutos                                                 | `GET /v1/sync/changes`                                                                                                                                 | F2              |
| Fondo                             | Version tras actualizarse                                                                                     | `PATCH /v1/devices/current`                                                                                                                            | F1              |
| Fondo                             | Buscar actualizaciones                                                                                        | `GET /v1/desktop/updates/{target}/{arch}/{currentVersion}`                                                                                             | F1              |
| Fondo                             | Version minima: "Actualiza para seguir sincronizando"                                                         | `minimumSupportedVersion` de `GET /v1/releases/latest`                                                                                                 | F1              |
| Fondo                             | Cola de escrituras sin conexion                                                                               | Los endpoints de cada entidad, en orden y con el mismo id de cliente                                                                                   | F2              |

### Ramal para VS Code

| Pantalla                  | Accion o dato                                               | Endpoint                                                                                   | Fase |
| ------------------------- | ----------------------------------------------------------- | ------------------------------------------------------------------------------------------ | ---- |
| Menu Accounts (nuevo)     | Iniciar sesion en Ramal                                     | Navegador + `UriHandler` + `POST /v1/auth/app/token`; device flow en entornos de navegador | F1   |
| Ajustes > Cuenta de Ramal | Tarjeta: nombre, correo, organizacion, plan                 | `GET /v1/me`; plan de `GET /v1/organizations`                                              | F1   |
| Ajustes > Cuenta de Ramal | Abrir panel web                                             | Sin endpoint: `env.openExternal`                                                           | -    |
| Ajustes > Cuenta de Ramal | Cambiar correo o contrasena, que se sincroniza              | Sin endpoint: abre la pagina de la web                                                     | -    |
| Ajustes > Cuenta de Ramal | Cerrar sesion                                               | `POST /v1/auth/logout`                                                                     | F1   |
| Ajustes > Cuenta de Ramal | "Sync settings from the cloud"                              | Sin endpoint: ajuste local de la instalacion                                               | -    |
| Ajustes > Plataformas Git | Anadir, predeterminada, desconectar                         | Los mismos que Desktop                                                                     | F2   |
| Ajustes > IA              | Llaves, funciones, llave, idioma, Conventional Commits      | Los mismos que Desktop                                                                     | F2   |
| Ajustes > Git             | Fetch automatico, pull, firmar, podar                       | `PATCH /v1/me/preferences/git`                                                             | F2   |
| Cambios, grafo, detalle   | Generar con llave de organizacion                           | `POST /v1/ai/generate`                                                                     | F4   |
| Barra de estado           | "Ramal: @cuenta" y proveedor de IA                          | `profile` y `preferences` del feed                                                         | F2   |
| Fondo                     | Registrar la instalacion como dispositivo                   | `POST /v1/auth/app/token` (bloque `device`)                                                | F1   |
| Fondo                     | Cuenta de plataforma para el remoto del repositorio abierto | Repositorios del feed: `remoteKey` a `linkedAccountId`                                     | F2   |
| Fondo                     | Ponerse al dia                                              | `GET /v1/sync/changes`                                                                     | F2   |
| Pull Requests             | Lista y creacion                                            | Motor contra la plataforma                                                                 | -    |
| Paleta                    | "Open in Ramal Desktop"                                     | Sin endpoint: deep link `ramal://`                                                         | -    |
