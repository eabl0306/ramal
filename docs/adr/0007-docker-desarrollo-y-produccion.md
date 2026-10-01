# ADR 0007: Docker para desarrollo y para produccion, con gateway del mismo origen

- Estado: aceptado
- Fecha: 2026-09-30

## Contexto

Hacen falta dos cosas distintas y conviene no confundirlas. En desarrollo se quiere arrancar la
pila del servidor con un comando, con recarga en caliente y con los puertos a mano. En
produccion se quiere una imagen pequena, sin toolchain dentro, sin codigo montado, que no corra
como root y que sepa decir si esta sana.

Solo la API, la web y su base de datos se dockerizan. Ramal Desktop y la extension de VS Code
se compilan con sus toolchains y se distribuyen como instaladores y VSIX
([ADR 0016](0016-actualizaciones-y-descargas.md)).

Hay una restriccion que viene del contrato: **la API no envia cabeceras CORS**, a proposito. La
web tiene que llegar a la API desde su mismo origen, y Desktop y VS Code no llaman a la API
desde ningun webview: lo hace el motor de Rust, que no es un navegador y no necesita CORS.

Y hay una leccion que Gastonomo pago cara: con un gateway que enruta por prefijos, **un prefijo
olvidado mando 25 rutas de la API a la SPA con todos los tests en verde**. Los GET devolvian el
`index.html` con 200 y los POST un 405 en HTML. Los jobs de la API y de la web pasaban cada uno
por su lado; nadie probaba la pila compuesta entrando por la puerta de verdad.

## Decision

### Desarrollo: `compose.yaml`

| Servicio  | Imagen                    | Puerto                   | Notas                                                                                                                                                                                                                                                                                                   |
| --------- | ------------------------- | ------------------------ | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `db`      | `postgres:18-alpine`      | 5432                     | Healthcheck con `pg_isready` y volumen con nombre montado en `/var/lib/postgresql`, que es lo que espera la imagen 18 (con la ruta vieja no arranca)                                                                                                                                                    |
| `api`     | `apps/api/Dockerfile.dev` | 8080                     | `cargo watch` con el codigo montado. `CARGO_TARGET_DIR` en un volumen fuera del bind mount, o docker crea en el repositorio del host un directorio de root que no se puede borrar sin sudo. Los directorios de compilacion quedan fuera del watch, o cada escritura de cargo dispara otra recompilacion |
| `web`     | `apps/web/Dockerfile.dev` | 5173                     | Servidor de Vite con HMR. Solo se monta el codigo fuente; `node_modules` es el del contenedor                                                                                                                                                                                                           |
| `mailpit` | `axllent/mailpit`         | 8025 (web) y 1025 (SMTP) | Buzon de desarrollo para los correos de verificacion y recuperacion. No sale nada a internet                                                                                                                                                                                                            |
| `gateway` | `nginx:alpine`            | 8090                     | La puerta de entrada: sirve la web y reenvia a la API lo que es suyo                                                                                                                                                                                                                                    |

- La base de datos va sin perfil y el resto en el perfil `full`. Asi `just db-up` levanta solo
  PostgreSQL, que es lo que necesitan los tests de la API, y `just up` levanta la pila entera.
- El contexto de build de la API y de la web es la raiz del repositorio: alli viven el
  workspace de Cargo con su `Cargo.lock` y el lockfile de pnpm
  ([ADR 0001](0001-monorepo-poliglota.md)).
- Todos los puertos y credenciales salen de `.env`, con valores por defecto.

### El gateway enruta por prefijo, y el orden importa

`docker/gateway.dev.conf` (y su gemelo de produccion) tiene un `location` por cada prefijo de la
API, siempre antes del `location /` que manda todo lo demas a la web:

```
location /health/        -> api
location = /openapi.json -> api
location /swagger-ui     -> api   (solo desarrollo)
location /v1/            -> api
location /               -> web
```

- Las rutas de la API son las mismas dentro y fuera del gateway: `/v1/auth/refresh` es
  `/v1/auth/refresh` para la web, para el motor de Rust y para quien lee un log de la API. No
  hay un prefijo `/api` que el gateway quite por el camino.
- Todo `location` de la API manda `X-Forwarded-For $proxy_add_x_forwarded_for`. El limite de
  peticiones identifica al cliente por la **ultima** entrada de esa cabecera, que es la que anade
  el gateway ([ADR 0008](0008-autenticacion-con-tokens.md)); sin ella, todas las peticiones
  parecen venir del gateway.
- Todo va bajo `/v1`, asi que un prefijo nuevo es raro. Cuando aparezca, necesita su `location`
  en los dos ficheros del gateway y una sonda en `tools/scripts/smoke-stack.sh`, en el mismo PR.

### La pila se prueba entrando por la puerta

`tools/scripts/smoke-stack.sh` recibe la URL del gateway y comprueba lo que un test de una sola
pieza no puede ver. La senal de fallo que busca es `text/html` en una ruta de la API:

| Sonda                                    | Esperado                                                                               |
| ---------------------------------------- | -------------------------------------------------------------------------------------- |
| `GET /health/live` y `GET /health/ready` | 200 con JSON                                                                           |
| Un `GET` bajo `/v1/` sin token           | 401 con `application/problem+json`. Si llega `text/html`, el gateway lo mando a la SPA |
| Un `POST` bajo `/v1/`                    | Una respuesta de la API, no un 405 en HTML de nginx                                    |
| `GET /openapi.json`                      | 200 con JSON                                                                           |
| `GET /` y `GET /en/`                     | 200 con HTML prerenderizado, con contenido real y no un cascaron vacio                 |
| Una ruta profunda de la SPA              | 200 con HTML (el fallback de la SPA)                                                   |

`just stack-smoke` levanta la pila, corre el script y la tira; la CI lo ejecuta en cada PR
([ADR 0006](0006-estrategia-de-testing.md)). En el primer corte usa la pila de desarrollo con el
perfil `full`. En cuanto existan los Dockerfile de produccion de la API y de la web, pasa a un
`infra/compose.smoke.yaml` con esas imagenes y el gateway de produccion, que es lo que se
despliega, con credenciales efimeras generadas en cada ejecucion. Es lo que hizo Gastonomo al
final (su ADR 0015), y el motivo es el mismo: la configuracion que importa probar es la de
produccion.

### Produccion

- **API:** multi-stage con `cargo-chef` sobre `debian:trixie-slim`, usuario no root (uid 10001) y
  `HEALTHCHECK` contra `/health/live` ([ADR 0003](0003-rust-con-salvo.md)). La imagen no lleva
  cargo ni codigo fuente.
- **Web:** build estatico con pnpm y nginx sin root sirviendo el resultado: las paginas
  prerenderizadas de la landing, el fallback de la SPA para el resto de rutas
  ([ADR 0015](0015-framework-web-con-prerender.md)), `no-store` para el HTML e `immutable` para
  los assets con hash. No hay servidor de Node.
- **Gateway:** el unico servicio con puerto publicado. La web, la API y la base de datos solo se
  alcanzan por la red interna. Pone las cabeceras de seguridad y la CSP de la web
  ([ADR 0011](0011-ui-compartida-y-puente-tipado.md)), con `frame-ancestors 'none'` y
  `X-Frame-Options: DENY` en toda respuesta de la web y de la API para que ninguna pagina (la de
  autorizar una app, la de activar un codigo, el reinicio de la boveda o el borrado de la cuenta)
  se pueda cargar dentro de un marco ajeno; no expone `/swagger-ui`, y desactiva el
  buffer en la ruta del proxy de IA para que el streaming SSE llegue a medida que se genera (la
  API manda ademas `X-Accel-Buffering: no`). Su limite de cuerpo queda por encima del mayor de la
  API (2 MiB, la foto de perfil y la rotacion de la boveda); los limites exactos por endpoint los
  aplica la API.
- `restart: unless-stopped`, healthchecks y limites de CPU y memoria por servicio. La base de
  datos no publica puertos.

Donde se despliega esa pila (instancia propia, compartida o un proveedor gestionado) queda para
su propio ADR cuando se decida. Este fija las imagenes y la forma de la pila, no el destino.

## Consecuencias

A favor:

- `just up` levanta base de datos, API, web, buzon de correo y gateway, y `just down` lo borra.
- La web y la API comparten origen en desarrollo y en produccion, asi que la API puede seguir
  sin CORS y las cookies de sesion de la web funcionan sin configuracion de dominio cruzado.
- El fallo que tuvo Gastonomo con un prefijo olvidado lo caza una sonda en cada PR.
- La imagen de produccion no lleva ni cargo ni node: menos superficie y menos peso.
- Los healthchecks hacen que `depends_on: service_healthy` funcione de verdad.

En contra:

- La primera compilacion de la imagen de desarrollo de la API tarda varios minutos: instala
  `cargo-watch` y compila `ramal-api` con todas sus dependencias. Despues queda en la cache del
  volumen.
- El bind mount del codigo es rapido en Linux y lento en macOS y Windows. Quien desarrolle ahi
  seguramente prefiera `cargo run` en local contra `just db-up`.
- Mientras `just stack-smoke` use la pila de desarrollo, el gateway que se prueba no es
  exactamente el de produccion. Queda cerrado cuando exista `infra/compose.smoke.yaml`.
- Cada prefijo nuevo de la API exige tocar tres ficheros. Es una friccion deseada: obliga a
  pensar si de verdad hace falta un prefijo fuera de `/v1`.

## Alternativas descartadas

**CORS en la API.** Abriria una puerta que ningun cliente necesita: la web entra por el mismo
origen y las apps por su motor. Que la cabecera no exista es una defensa, no un olvido.

**Un prefijo `/api` que el gateway quita.** Resolveria con un solo `location` el problema del
prefijo olvidado, a cambio de que las rutas de la API sean distintas dentro y fuera del gateway,
y de que la documentacion, los logs y el motor de Rust tengan que saber en que lado estan. Con
todo bajo `/v1`, la lista de prefijos es corta y la sonda del smoke test cubre el olvido.

**Proxy de Vite como unica solucion de desarrollo.** Sirve para `pnpm dev` sin Docker, y se
mantiene para eso, pero no prueba la configuracion de nginx que se despliega.

**Un servidor de Node para la web.** No hay SSR ([ADR 0015](0015-framework-web-con-prerender.md)):
seria un proceso mas que vigilar para servir ficheros estaticos.
