# ADR 0004: PostgreSQL con SQLx y migraciones solo hacia delante

- Estado: aceptado
- Fecha: 2026-09-30
- Revisado: 2026-10-01, para alinearlo con el codigo del primer corte

## Contexto

La API necesita persistencia relacional con garantias que no se negocian: el registro de
cambios que usan los dispositivos para sincronizar exige que un numero de cambio visible
implique que ninguno menor de su ambito esta por confirmar; las claves foraneas compuestas
impiden apuntar a la cuenta de otro usuario; los tokens de un solo uso se marcan en la misma
transaccion que los consume. Eso es trabajo de una base de datos transaccional, no de la
aplicacion.

El modelo completo esta en [data-model.md](../data-model.md): 37 tablas en seis fases. Las
opciones en Rust eran un ORM (SeaORM, Diesel) o un cliente SQL (SQLx), y dentro de SQLx, las
macros `query!` verificadas contra una base en tiempo de compilacion o consultas en tiempo de
ejecucion.

## Decision

PostgreSQL 18 con SQLx 0.9.0, sin ORM.

### Por que PostgreSQL 18

- `uuidv7()` de serie, sin extension. Las claves primarias son UUIDv7 y las inserciones caen al
  final del indice B-tree.
- `uuid_extract_version` (desde la 17) permite el `CHECK` que exige UUIDv4 o v7 en los ids que
  genera el cliente.
- `ON DELETE SET NULL (columna)` (desde la 15) en las claves foraneas compuestas.
- `citext` para correos y nombres de usuario.

No hay Redis ni otra base: todo el estado compartido vive en PostgreSQL, y los limites de
peticiones van en memoria de cada proceso (con la limitacion que eso tiene, descrita en el
[ADR 0008](0008-autenticacion-con-tokens.md)).

### SQLx sin macros de consulta

- Dependencia con `default-features = false` y las features `runtime-tokio`,
  `tls-rustls-ring`, `postgres`, `macros`, `migrate`, `uuid` y `chrono`. `macros` hace falta
  para `sqlx::migrate!` y `#[derive(FromRow)]`, no para `query!`.
- Las consultas se escriben con `sqlx::query` y `sqlx::query_as` en tiempo de ejecucion. Nada
  de `query!`: obligaria a tener una base de datos al compilar o a mantener al dia los ficheros
  de `.sqlx` en modo offline, y un `cargo build` recien clonado tiene que funcionar sin
  PostgreSQL.
- El pool se crea de forma perezosa. El adaptador de salud lee con timeout la ultima migracion
  aplicada (`_sqlx_migrations`) y la compara con la mas nueva que trae el binario, y
  `/health/ready` responde 503 si la base no contesta o su esquema va por detras. Un `SELECT 1`
  no basta: la base contestaria con el esquema a medias. Un esquema por delante si vale, porque
  cada migracion es compatible con el binario anterior.
- Los repositorios viven en `git-rudder-api-infrastructure` e implementan los puertos de
  `git-rudder-api-application`. Ninguna consulta sale de ese crate.
- Ante un interbloqueo (`40P01`) entre transacciones que tocan dos ambitos del registro de
  cambios, el repositorio reintenta una vez. El caso esta descrito en el modelo de datos.

### Migraciones solo hacia delante

- Ficheros SQL en `apps/api/migrations/<AAAAMMDDHHMMSS>_<nombre>.sql`, con el formato de
  Gastonomo. Las fechas son de orden, no de calendario: una por fase, y pasos de 100 dentro del
  mismo dia. Cada fichero empieza con un comentario en castellano que explica por que existe.
- **No hay migraciones de bajada.** Deshacer algo es una migracion nueva hacia delante. Una
  bajada que nunca se ejecuta en produccion es codigo sin probar que da una falsa sensacion de
  seguridad, y en una tabla con datos reales rara vez se puede escribir sin perder algo.
- **Una migracion fusionada en `main` no se edita nunca.** SQLx guarda la suma de cada
  migracion aplicada y se niega a arrancar si el fichero cambio. Es la herramienta haciendo
  cumplir la regla.
- **Cada migracion es compatible con el binario anterior.** Durante un despliegue conviven un
  momento la version vieja y la nueva contra el mismo esquema. Un renombrado se hace en pasos:
  anadir lo nuevo, rellenar, cambiar el codigo, y quitar lo viejo en una migracion posterior.
- Se embeben con `sqlx::migrate!` y se aplican al arrancar, en segundo plano. Si fallan, el fallo
  queda en el log, el proceso sigue vivo y `/health/ready` responde 503, como en Gastonomo: asi el
  contenedor no entra en un bucle de reinicios. Dos diferencias con Gastonomo, las dos para que
  una base lenta no cueste un reinicio: el servidor escucha mientras tanto (`/health/live`
  responde desde el primer segundo) y un fallo se reintenta cada 10 segundos hasta que sale bien
  (`crates/api/src/bootstrap/migrations.rs`).
- Convenciones del esquema: restricciones con nombre, `COMMENT ON TABLE` en cada tabla, `IF NOT
EXISTS` en cada `CREATE`, `text` con `CHECK` en lugar de tipos `ENUM` (un valor de un `ENUM`
  no se puede quitar), `updated_at` escrito por disparador y borrado real sin `deleted_at`.

### Tests contra un PostgreSQL de verdad

- Los adaptadores de PostgreSQL se prueban contra una base real, nunca contra un doble. Viven
  en `apps/api/crates/infrastructure/tests/` y, los de punta a punta contra el binario real, en
  `apps/api/crates/api/tests/`. No van en `apps/api/crates/http/tests/`: `git-rudder-api-http` no
  puede alcanzar SQLx ni en sus tests (lo prohibe el test de arquitectura), asi que sus tests de
  endpoint usan los fakes de `git-rudder-api-application`.
- Leen `GIT_RUDDER_TEST_DATABASE_URL`. Sin ella se saltan solos, para que `cargo test` pase en una
  maquina sin base de datos. **Con `GIT_RUDDER_TEST_REQUIRE_DATABASE=1`, que no haya URL es un
  fallo**: lo ponen `just api-test-db` y la CI, porque un test que se salta en silencio en la
  CI es un verde falso.
- Se invocan con `--tests`, nunca con una lista de suites escrita a mano. En Gastonomo partir
  un fichero de tests dejo fuera una suite entera sin que nadie lo notara.
- Cada test crea sus propios datos con valores aleatorios (correos, ids) y nunca asume tablas
  vacias. La base de la CI es efimera; la de desarrollo acumula filas y no pasa nada.
- En local la base la levanta `just db-up` con `compose.yaml`; en la CI, el mismo contenedor
  `postgres:18-alpine`. Ojo, igual que en Gastonomo: en la imagen 18 el volumen se monta en
  `/var/lib/postgresql`, no en `/var/lib/postgresql/data`, o el contenedor no arranca.
- No se usa `#[sqlx::test]`: exige `DATABASE_URL` y no sabe saltarse cuando no hay base.
- No se usa testcontainers: `testcontainers` 0.28 es incompatible con
  `testcontainers-modules` 0.15, que depende de la 0.27, y compose ya da lo mismo sin otra
  dependencia.
- Los disparadores del registro de cambios y las cascadas de borrado de cuenta y de
  organizacion tienen tests propios con datos en todas las tablas sincronizadas: es donde un
  error no da ningun sintoma inmediato.

## Consecuencias

A favor:

- SQL a la vista, sin una capa que traducir mentalmente. Las restricciones que importan viven
  en la base y no dependen de que el codigo no tenga fallos.
- Las migraciones viajan con el codigo, se aplican solas al desplegar y su orden cuenta la
  historia de las fases.
- El readiness es real: si la base no responde, `/health/ready` devuelve 503.
- `cargo build` y `cargo test` funcionan en una maquina sin PostgreSQL.

En contra:

- Sin `query!` no hay verificacion de las consultas al compilar. Una columna mal escrita se
  descubre en el test de integracion, no en el compilador. Por eso esos tests son obligatorios
  en la CI y no se pueden saltar.
- Sin ORM, el mapeo de filas a dominio se escribe a mano. Con 37 tablas es bastante codigo, y
  va en `git-rudder-api-infrastructure`, que tiene el mismo limite de 300 lineas por fichero.
- Un fallo real de migracion no mata el proceso: deja la API viva respondiendo 503 y
  reintentando cada 10 segundos. Es deliberado, pero hay que mirar el log o la sonda de
  readiness para enterarse.
- Solo hacia delante significa que un error en una migracion ya aplicada se arregla con otra
  migracion. No hay atajo.

## Alternativas descartadas

**SeaORM o Diesel.** Una capa mas entre el codigo y el SQL, para un esquema cuyo valor esta
precisamente en restricciones, disparadores y claves compuestas que un ORM no modela bien.

**Macros `query!` con modo offline.** Verifican las consultas al compilar, a cambio de una base
de datos al compilar o de ficheros `.sqlx` que hay que regenerar en cada cambio de consulta y
que la CI tiene que comprobar. Los tests de integracion obligatorios cubren lo mismo.

**Migraciones de bajada.** Ver arriba: codigo que no se ejecuta nunca en produccion.

**testcontainers.** Incompatibilidad de versiones hoy, y una dependencia de Docker desde los
tests que compose ya resuelve fuera de ellos.
