# ADR 0023: Despliegue en la instancia compartida apps-01 y dominio

- Estado: aceptado
- Fecha: 2026-10-01
- La eleccion de dominio la sustituye el [ADR 0027](0027-renombrado-a-git-rudder.md):
  es propio, `git-rudder.com`, y no un subdominio del dominio personal. El resto sigue vigente.

## Contexto

[ADR 0007](0007-docker-desarrollo-y-produccion.md) fija las imagenes y la forma de la pila del
servidor (gateway, API, web y PostgreSQL) y deja para otro ADR donde se despliega. Hacen falta
tres respuestas: en que maquina corre, con que dominio, y como se vuelve atras.

**La maquina que ya existe.** `apps-01` es una instancia de Lightsail en `us-east-1` con 2 vCPU,
4 GB de RAM y 80 GB de disco. Ya sirve Gastonomo y Grimdex, cada uno con su PostgreSQL, detras de
un Caddy compartido. El patron esta hecho y probado en Gastonomo: imagenes construidas en la CI y
publicadas en GHCR por el SHA del commit, despliegue por SSH con vuelta atras automatica, un
script `rollback.sh`, copias nocturnas de la base a un bucket de S3 con un usuario que no puede
borrar, y una comprobacion de salud desde fuera.

**Lo que usa hoy, medido por Ezequiel en la maquina el 2026-10-01:**

| Recurso  | Uso                                                  |
| -------- | ---------------------------------------------------- |
| Memoria  | 815 MB usados de 3832 MB (15 %); 3016 MB disponibles |
| Swap     | 0 de 2047 MB                                         |
| Carga    | 0,2                                                  |
| Disco    | 10,3 % de 76,45 GB                                   |
| Procesos | 156                                                  |

Un analisis anterior sumo los limites de memoria que declaran los ficheros de compose de Caddy y
de Gastonomo y concluyo que la maquina estaba al limite. Era un error: un limite de compose es un
techo, no un consumo. El uso real es el de la tabla. Git Rudder anade, estimado, de 300 a 500 MB de uso
real (la API, su PostgreSQL, el nginx de la web y el gateway). Cabe con margen.

**El dominio.** Los dominios que usaba el diseno eran de terceros y estaban en uso. El 2026-10-01,
cuando se escribio este ADR, no habia ninguno propio para el producto, y el patron de los otros
productos de Ezequiel es un subdominio de `bahoque.com` (`gastonomo.bahoque.com` y
`api.gastonomo.bahoque.com`). Con eso se decidio. El 2026-10-02, al renombrar el producto, Ezequiel
decidio registrar `git-rudder.com`: la eleccion esta en el
[ADR 0027](0027-renombrado-a-git-rudder.md) y las tablas de abajo ya llevan el dominio propio.

El dominio importa mas de lo normal por [ADR 0016](0016-actualizaciones-y-descargas.md): **la URL
del actualizador queda grabada en cada instalacion** y no se puede cambiar despues de la primera
build publica. Tambien van en el dominio el `iss` de los tokens
([ADR 0008](0008-autenticacion-con-tokens.md)), los enlaces de los correos, las URL de vuelta de
OAuth y el `type` de los errores.

## Decision

### Donde corre

La nube de Git Rudder se despliega en **`apps-01`, la instancia compartida que ya existe**, con el
mismo patron que Gastonomo:

- Un directorio propio (`/srv/apps/git-rudder`), un proyecto de compose propio, un `.env` propio y
  contenedores con el prefijo `git-rudder-`. Su PostgreSQL es suyo, con su volumen; no comparte datos
  con las otras aplicaciones.
- Solo Caddy publica puertos en la maquina. El gateway de Git Rudder se une a la red `edge` de Caddy;
  la API, la web y la base solo estan en la red interna de Git Rudder.
- Cada servicio declara su limite de memoria en `compose.prod.yaml` (orientativo: API 512 MB,
  PostgreSQL 512 MB, web 128 MB, gateway 64 MB). Son techos para que una fuga en una aplicacion
  no tumbe a las otras, no lo que se espera usar.
- **Las imagenes se construyen en la CI, nunca en la maquina.** Compilar Rust en una maquina
  compartida es la forma mas rapida de tumbar lo que ya funciona.

**Si el uso crece**, el plan es pasar al plan de Lightsail de 8 GB. Se revisa con `free -m` y
`docker stats --no-stream`; la senal es que la memoria disponible baje de forma sostenida del
30 %. Subir de plan es crear una instancia nueva desde una instantanea y mover la IP estatica, con
un corte corto para las tres aplicaciones.

### Dominio y enrutado

El dominio propio y su subdominio para la API:

| Nombre               | Que sirve                                                                                                                            |
| -------------------- | ------------------------------------------------------------------------------------------------------------------------------------ |
| `git-rudder.com`     | La web (landing y panel) y, bajo el mismo origen, los prefijos de la API (`/v1/`, `/health/` y `/openapi.json`)                      |
| `api.git-rudder.com` | Solo la API: el motor de Desktop y VS Code, el actualizador, el registro de publicaciones de la CI, el webhook de Paddle y el broker |

- Los dos son registros A en Route 53 que apuntan a la IP de la instancia. Caddy pide y renueva
  sus certificados con Let's Encrypt. El registro DNS tiene que existir antes de que Caddy arranque
  con ese nombre, o la peticion de certificado falla y gasta cupo.
- **Caddy manda los dos nombres al gateway de Git Rudder**, no a la API directamente: asi las cabeceras
  de seguridad, el limite de cuerpo y el enrutado viven en un solo sitio. Bajo
  `api.git-rudder.com` el gateway solo enruta los prefijos de la API; cualquier otra ruta es un
  404 en problem+json.
- **La web sigue en el mismo origen que su API.** La API no envia CORS (ADR 0007), asi que la web
  llama a `https://git-rudder.com/v1/...` y nunca al subdominio `api.`. La imagen de la web se
  construye con su propio origen; si se hornea el de la API, el navegador sale a otro origen y no
  funciona nada. Es el error que Gastonomo dejo escrito.
- **El nginx del gateway necesita `real_ip`** (`set_real_ip_from` con la red `edge` y
  `real_ip_header X-Forwarded-For`). Detras de Caddy, sin eso, el limite de peticiones de la API
  ve a todos los clientes con la IP de Caddy.

Las URL que quedan fijadas:

- Actualizador:
  `https://api.git-rudder.com/v1/desktop/updates/{{target}}/{{arch}}/{{current_version}}?channel=stable&bundle={{bundle_type}}`
  ([ADR 0016](0016-actualizaciones-y-descargas.md)).
- `iss` de los tokens: `https://api.git-rudder.com`.
- Enlaces de los correos y paginas de la web (restablecer contrasena, verificar el correo,
  invitaciones, autorizar una app, activar un codigo): `https://git-rudder.com/...`.
- `type` de los errores: `https://git-rudder.com/problems/<slug>`.
- Remitente del correo: un subdominio de `git-rudder.com`
  ([ADR 0024](0024-correo-transaccional-con-resend.md)).

### De un commit a produccion

Igual que Gastonomo:

1. `release.yml` construye las imagenes de produccion y las publica en GHCR con dos etiquetas: el
   SHA del commit y `main`. **Se despliega siempre el SHA**; `main` es solo un puntero comodo.
2. `deploy.yml` entra como el usuario `deploy`, se autentica en GHCR con el token efimero del job,
   apunta la etiqueta que corre en `.previous-tag`, escribe la nueva en el `.env` y hace
   `docker compose pull` y `docker compose up -d --wait`.
3. Comprueba **desde fuera de la maquina** que `/health/live` y `/health/ready` responden 200 en
   los dos nombres y que la raiz de la web responde 200. Es la unica comprobacion que cubre DNS,
   Caddy, TLS y la aplicacion a la vez.
4. Si algo falla, vuelve sola a `.previous-tag`. A mano, `./rollback.sh` vuelve a la anterior o a
   un SHA concreto.

Ademas:

- **Copias de la base** cada noche con `backup-postgres@git-rudder` al bucket de S3 que ya existe,
  con el usuario que no puede borrar. Exige que el servicio se llame `db` y que el `.env` tenga
  `POSTGRES_USER` y `POSTGRES_DB`.
- **Comprobacion de salud cada hora desde fuera** (`healthcheck.yml`), como Gastonomo.
- Los secretos de produccion (llaves de firma de tokens, KEK de las llaves de IA de organizacion,
  contrasena SMTP, secreto del webhook de Paddle) van en el `.env` del servidor, con permisos
  0600 y del usuario `deploy`. Los que cambian con el despliegue los escribe `deploy.yml` desde
  los secretos de Actions.

### Volver atras cuando hay migraciones

Las migraciones solo van hacia delante ([ADR 0004](0004-postgresql-con-sqlx.md)), asi que volver
a una imagen anterior deja la base por delante del codigo. Dos reglas:

1. **Migraciones en dos pasos.** Nunca se borra ni se renombra una columna en la misma version que
   deja de usarla: primero sale el codigo que ya no la usa y en una version posterior la migracion
   que la quita. Asi la version anterior siempre funciona con el esquema nuevo.
2. **El arranque tiene que aceptar una base por delante. Hoy no lo hace.** La API aplica las
   migraciones con `MIGRATOR.run` (`apps/api/crates/infrastructure/src/persistence/migrations.rs`)
   sin `set_ignore_missing(true)`. Si se vuelve a una imagen anterior despues de una migracion,
   SQLx encuentra en la base una version que el binario no conoce, responde `VersionMissing`, y
   la API lo reintenta y lo registra cada 10 segundos para siempre. `/health/ready` sigue en 200,
   porque compara la ultima migracion aplicada con la esperada usando `>=`, asi que el fallo no se
   ve desde fuera. **Lo arregla una issue de seguimiento (#517)**: aceptar migraciones desconocidas (o
   tratar ese error como "esquema por delante") y dejarlo en el log una sola vez. Hasta que este
   arreglado, una vuelta atras que cruce una migracion se hace a sabiendas.

### La KEK en una maquina compartida

La llave maestra que cifra las llaves de IA de organizacion
([ADR 0013](0013-proxy-de-ia-de-organizacion.md)) vive en el `.env` de Git Rudder en una maquina que
comparten otras dos aplicaciones. La protegen: el fichero con permisos 0600 del usuario `deploy`,
la llave pasada solo al contenedor de la API, proyectos de compose y redes separados, y ningun
volumen compartido con las otras aplicaciones. Quien tenga root en la maquina puede leerla, igual
que en cualquier servidor sin KMS; por eso hay una copia fuera de linea y el puerto `KeyEnvelope`
deja la puerta abierta a un KMS mas adelante.

## Consecuencias

A favor:

- Coste adicional cero: la maquina, Caddy, los scripts, las copias y el runbook ya existen y estan
  probados.
- Un despliegue es una etiqueta de imagen, y volver atras es otra.
- La web conserva el mismo origen que su API, asi que el contrato sin CORS de ADR 0007 no cambia.
- El dominio es un `.com` propio: no depende de un tercero ni de un TLD dudoso.

En contra:

- **Las tres aplicaciones comparten maquina, kernel y Caddy.** Si la maquina cae, caen las tres; un
  pico de una afecta a las demas hasta su limite de memoria. Subir a 8 GB tiene un corte corto.
- **El dominio hay que registrarlo y renovarlo**, y tiene que estar con sus registros DNS antes de
  la primera build publica: la URL del actualizador queda grabada en cada instalacion. Un olvido de
  renovacion tumba a la vez el actualizador, el `iss` de los tokens y el correo.
- Los certificados de `git-rudder.com` y `api.git-rudder.com` los pide el mismo Caddy que sirve a
  las otras aplicaciones, asi que comparten su cupo semanal de Let's Encrypt. Con pocos nombres no
  se nota.
- Hasta que se arregle el arranque, una vuelta atras que cruce una migracion deja la API
  reintentando en bucle sin que `/health/ready` lo diga.
- El repositorio de infraestructura de `apps-01` tiene que aprender una aplicacion mas (su bloque
  de Caddy, su directorio y su servicio de copias).

## Alternativas descartadas

**Una instancia aparte (`apps-02`) en Lightsail.** Aislaria Git Rudder de las otras dos aplicaciones
con el mismo patron, por unos 12 a 24 USD al mes mas. Se propuso cuando se creia que `apps-01`
estaba al limite; con el uso real medido no hace falta. Queda como salida si el aislamiento
llega a pesar mas que el coste (por ejemplo, con clientes de pago que lo pidan).

**Hetzner.** Lo mas barato por GB, pero otro proveedor, otro cortafuegos y otra factura, con la
serie barata solo en Europa y dos subidas de precio en 2026.

**Una plataforma con PostgreSQL gestionado (Render, Fly.io).** Sin maquina que cuidar, pero de tres
a cuatro veces mas caro, y el gateway del mismo origen hay que montarlo igual. Gastonomo lo
descarto por lo mismo.

**Un subdominio de `bahoque.com`**, el patron de los otros productos de Ezequiel. Fue la decision
del 2026-10-01, cuando no habia dominio propio, y el [ADR 0027](0027-renombrado-a-git-rudder.md) la
cambio: ataba el producto al dominio personal en el actualizador, el `iss`, las pantallas de OAuth y
el remitente del correo, y salir de ahi despues de la primera build publica obliga a mantener el
nombre viejo respondiendo mientras queden instalaciones que lo lleven grabado.

**Los dominios que usaba el diseno.** Son de terceros y estan en uso.
