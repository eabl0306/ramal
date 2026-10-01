# Ramal API

API de Ramal en Rust: Salvo 1.0, PostgreSQL 18 con SQLx y arquitectura hexagonal.
El primer corte trae la salud del servicio, el documento OpenAPI, errores en
problem+json, request id, configuracion `RAMAL__` y migraciones al arrancar.

El porque de cada pieza esta en `docs/architecture.md` (seccion 4.5) y en los ADR
0003, 0004 y 0007. El contrato HTTP, en `docs/api-surface.md`.

## Crates

| Crate                                                | Que hace                                                      |
| ---------------------------------------------------- | ------------------------------------------------------------- |
| `crates/domain` (`ramal-api-domain`)                 | Reglas puras. Sin IO, sin reloj, sin serde                    |
| `crates/application` (`ramal-api-application`)       | Puertos, casos de uso, `AppError` y fakes (feature `testing`) |
| `crates/infrastructure` (`ramal-api-infrastructure`) | Configuracion, telemetria, PostgreSQL y reloj del sistema     |
| `crates/http` (`ramal-api-http`)                     | Salvo: rutas, handlers, DTOs, problem+json, OpenAPI           |
| `crates/api` (`ramal-api`)                           | Binario y composition root                                    |

`crates/api/tests/architecture.rs` lee `Cargo.lock` y falla si una capa importa lo
que no le toca.

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

| Ruta            | Respuesta                                                    |
| --------------- | ------------------------------------------------------------ |
| `/health/live`  | 200 siempre, sin tocar dependencias                          |
| `/health/ready` | 200 `ok` o 503 `degraded`, con cada componente y latencia    |
| `/openapi.json` | El documento OpenAPI                                         |
| `/swagger-ui/`  | El visor, sin CDN. Solo con `RAMAL__SERVER__SWAGGER_UI=true` |

`/health/ready` pide que PostgreSQL responda y que su esquema este al dia con las
migraciones del binario.

## Configuracion

Variables `RAMAL__` con `__` como separador (`RAMAL__DATABASE__URL`). Todas tienen
valor por defecto; la lista esta en `.env.example`. Una clave desconocida hace
fallar el arranque.

## Anadir un endpoint

Sigue la receta 9.2 de `docs/architecture.md`: dominio, puerto y fake, caso de
uso, adaptador con su migracion, handler con `#[endpoint]` en
`router/<contexto>.rs` y cableado en `crates/api/src/bootstrap/<contexto>.rs`.
