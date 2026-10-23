# Git Rudder API

API de Git Rudder en Rust: Salvo 1.0, PostgreSQL 18 con SQLx y arquitectura hexagonal.
El primer corte trae la salud del servicio, el documento OpenAPI, errores en
problem+json, request id, configuracion `GIT_RUDDER__` y migraciones al arrancar, en segundo plano y
con reintento cada 10 segundos.

El porque de cada pieza esta en `docs/architecture.md` (seccion 4.5) y en los ADR
0003, 0004 y 0007. El contrato HTTP, en `docs/api-surface.md`.

## Crates

| Crate                                                     | Que hace                                                      |
| --------------------------------------------------------- | ------------------------------------------------------------- |
| `crates/domain` (`git-rudder-api-domain`)                 | Reglas puras. Sin IO, sin reloj, sin serde                    |
| `crates/application` (`git-rudder-api-application`)       | Puertos, casos de uso, `AppError` y fakes (feature `testing`) |
| `crates/infrastructure` (`git-rudder-api-infrastructure`) | Configuracion, telemetria, PostgreSQL y reloj del sistema     |
| `crates/http` (`git-rudder-api-http`)                     | Salvo: rutas, handlers, DTOs, problem+json, OpenAPI           |
| `crates/api` (`git-rudder-api`)                           | Binario y composition root                                    |

`crates/api/tests/architecture.rs` lee `Cargo.lock` y falla si una capa importa lo
que no le toca. `git-rudder-api` no depende de Salvo: el servidor arranca dentro de
`git-rudder-api-http`.

Los tests de endpoint sin base de datos van en `crates/http/tests` con fakes. Los que necesitan
PostgreSQL van en `crates/infrastructure/tests` y, de punta a punta con el binario, en
`crates/api/tests`: `git-rudder-api-http` no puede alcanzar SQLx ni en sus tests.

## Uso

```sh
just api-run          # arranca contra el Postgres de compose (just db-up)
just api-test         # tests sin base de datos
just api-test-db      # tests contra Postgres; no se pueden saltar
just api-cov          # cobertura: 80% en dominio y aplicacion, 60% global
just api-openapi      # reescribe apps/api/openapi.json
just api-docker       # imagen de produccion
```

Despues de cambiar una ruta o un DTO: `just api-openapi` y `just api-client`, y se
commitean los dos. Un test falla si `openapi.json` se queda atras.

La imagen se prueba arrancada con `bash apps/api/scripts/smoke-image.sh`.

## Rutas

| Ruta            | Respuesta                                                         |
| --------------- | ----------------------------------------------------------------- |
| `/health/live`  | 200 siempre, sin tocar dependencias                               |
| `/health/ready` | 200 `ok` o 503 `degraded`, con cada componente y latencia         |
| `/openapi.json` | El documento OpenAPI                                              |
| `/swagger-ui/`  | El visor, sin CDN. Solo con `GIT_RUDDER__SERVER__SWAGGER_UI=true` |

`/health/ready` pide que PostgreSQL responda y que su esquema este al dia con las
migraciones del binario. Las dos sondas llevan `checkedAt` y nunca el detalle de un fallo, que
queda en el log. Todo error lleva un `code` estable (`route_not_found`,
`method_not_allowed`...); la lista esta en `docs/api-surface.md`.

## Configuracion

Variables `GIT_RUDDER__` con `__` como separador (`GIT_RUDDER__DATABASE__URL`). Todas tienen
valor por defecto; la lista esta en `.env.example`. Una clave desconocida hace
fallar el arranque.

## Anadir un endpoint

Sigue la receta 9.2 de `docs/architecture.md`: dominio, puerto y fake, caso de
uso, adaptador con su migracion, handler con `#[endpoint]` en
`router/<contexto>.rs` y cableado en `crates/api/src/bootstrap/<contexto>.rs`.
