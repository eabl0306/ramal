# ADR 0003: API en Rust con Salvo 1.0 y OpenAPI generado con salvo-oapi

- Estado: aceptado
- Fecha: 2026-09-30

## Contexto

La API de Ramal es servidor de autorizacion para los tres productos, guarda documentos
sincronizados y blobs de la boveda, publica el manifiesto del actualizador y hace de proxy de
IA para las llaves de organizacion, con respuestas en streaming (SSE). Tiene que ser rapida de
arrancar, barata de desplegar y dificil de romper en tiempo de ejecucion.

Rust da tipos fuertes, ausencia de nulls y un binario unico sin runtime que instalar en la
imagen. Ademas permite algo concreto de Ramal: el crate `ramal-ai` (planificado) lo usan el
motor del dispositivo y la API, asi que el prompt que sale con una llave personal y el que sale
con una llave de organizacion son el mismo codigo.

Salvo es el framework de Gastonomo y el equipo ya lo conoce. Salvo publico la 1.0.0 el 24 de
septiembre de 2026. Segun su changelog, de la 0.96 que usa Gastonomo a la 1.0 cambian poco:
TLS recibe el `CryptoProvider` de rustls de forma explicita y cambia la integracion con
OpenTelemetry.

Lo que si hay que decidir es como se genera el documento OpenAPI. Gastonomo lo construye a mano
con los builders de utoipa porque la macro `#[utoipa::path]` espera funciones libres y sus
handlers son structs con dependencias inyectadas. Su ADR 0003 deja escrito el precio: si
alguien anade una ruta y olvida documentarla, nada lo detecta. En Ramal ese documento es la
fuente de `@ramal/api-client` y de los tests de contrato de `ramal-cloud-client`, asi que un
documento desincronizado rompe clientes.

## Decision

Salvo 1.0.0 sobre Tokio, con `default-features = false` y las features `server`,
`server-handle`, `http1`, `http2`, `test` y `oapi`. Las features por defecto traen `aws-lc-rs`,
que necesita cmake y un compilador de C para algo que aqui no se usa: TLS lo termina el
gateway. Edition 2024 y Rust 1.98.1, por encima del minimo de 1.94 que piden salvo 1.0 y sqlx
0.9.

### Estructura

- `ramal-api-http` es el unico crate que conoce Salvo: router, handlers, DTOs, problem+json,
  catcher, documento OpenAPI y los middlewares de request id, limite de peticiones e
  idempotencia. Es un adaptador driving.
- `ramal-api` es el binario y el composition root (`src/bootstrap.rs` mas un modulo por
  contexto en `src/bootstrap/`). Construye los adaptadores, los casos de uso y el router.
- Configuracion con figment desde el entorno, prefijo `RAMAL__` y `__` como separador de nivel,
  con valores por defecto en el codigo. `.env.example` es la lista de lo que se puede mover.
- Apagado ordenado: SIGTERM y SIGINT llaman a `stop_graceful` con margen, igual que Gastonomo.

### OpenAPI con salvo-oapi, no con utoipa

El documento se genera con salvo-oapi 1.0.0, la feature `oapi` de Salvo, en lugar de con
utoipa 6.0.0.

- salvo-oapi es un crate propio del proyecto Salvo, derivado de utoipa, que no depende de
  utoipa. Su modelo es OpenAPI 3.2.
- Cada handler se declara con `#[endpoint]` y cada DTO deriva `ToSchema`. El documento se
  construye recorriendo el router (`OpenApi::new(...).merge_router(&router)`), asi que una ruta
  sin su documentacion no puede existir: la ruta es la documentacion.
- **Se conserva el patron de Gastonomo de handlers como structs con sus casos de uso
  inyectados.** `#[endpoint]` acepta un bloque `impl` con un metodo `handle` sobre un tipo
  concreto (se comprobo en el codigo de salvo-oapi-macros 1.0.0). Lo que no acepta son bloques
  `impl` genericos, y eso fija una regla: los handlers guardan los casos de uso detras de `Arc`,
  y los casos de uso guardan sus puertos como `Arc<dyn Puerto>`, que es lo que ya pide
  `async-trait`. Un handler generico sobre el tipo del puerto no compila.
- Se sirve en `/openapi.json`. La Swagger UI va en `/swagger-ui` con la feature `swagger-ui`
  de salvo-oapi, que la feature `oapi` de Salvo activa. Sus assets van embebidos en el binario
  (Swagger UI 5.32.11 con `rust-embed`), asi que funciona sin salida a internet, que era un
  hueco de Gastonomo.
- El documento generado se versiona en `apps/api/openapi.json`. Un test de `ramal-api-http` lo
  compara con lo que genera el router y falla si difieren; `just api-openapi` lo reescribe. Ese
  fichero es la fuente de `@ramal/api-client` y de los tests de contrato del motor.

### Streaming

El proxy de IA responde en SSE ([ADR 0013](0013-proxy-de-ia-de-organizacion.md)). La feature
de Salvo que lo da entra con ese endpoint, en F4, no en el primer scaffold: cada feature que se
activa es codigo que se compila y se audita.

### Imagen

Multi-stage con `cargo-chef` sobre `debian:trixie-slim`, usuario no root (uid 10001) y
`HEALTHCHECK` contra `/health/live`, igual que Gastonomo. El contexto de build es la raiz del
repositorio, porque el workspace de Cargo y su `Cargo.lock` viven alli
([ADR 0001](0001-monorepo-poliglota.md)).

## Consecuencias

A favor:

- El documento OpenAPI no se puede quedar atras respecto a las rutas, y el test del fichero
  versionado hace que cualquier cambio del contrato aparezca como diff en el PR.
- El framework esta confinado a un crate. Sustituirlo no toca el nucleo.
- La Swagger UI funciona sin red.
- Un binario unico y pequeno, sin runtime que instalar en la imagen.

En contra:

- Salvo tiene menos ejemplos e integraciones de terceros que axum, y la 1.0 tiene seis dias en
  la fecha de este ADR. Se fija la version exacta y se sube a proposito.
- Las macros de salvo-oapi generan codigo que no se ve y registran los endpoints en un
  inventario estatico. Un error de la macro se lee peor que un error del builder de utoipa.
- La regla de "nada de `impl` genericos en los handlers" empuja a trait objects en los puertos.
  Ya era el estilo, pero ahora es obligatorio.
- La calidad del documento depende de que cada DTO describa bien su esquema (ejemplos,
  formatos, campos opcionales). Que el documento exista no significa que sea bueno; eso lo sigue
  mirando la revision.

## Alternativas descartadas

**utoipa 6.0 con builders a mano, como Gastonomo.** Funciona, pero mantiene el hueco conocido:
una ruta nueva sin documentar pasa todos los tests. Con un cliente generado del documento, ese
hueco se convierte en un cliente que no sabe que la ruta existe.

**utoipa con `#[utoipa::path]`.** Espera funciones libres y hay que registrar cada ruta dos
veces, en el router y en el documento. Mismo problema con otra forma.

**axum.** Mas ecosistema, pero el equipo ya conoce Salvo y la diferencia no compensa cambiar de
framework respecto al repositorio de referencia. Esta vetado en `deny.toml` para que no entre
por una dependencia transitiva.

**Features por defecto de Salvo.** Traen `aws-lc-rs`, con cmake y compilador de C en cada
maquina y en la imagen de build, para un TLS que termina el gateway.
