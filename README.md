# Ramal

Cliente de Git con tres productos que comparten cuenta: Ramal Desktop (Tauri), Ramal for
VS Code y Ramal Web. Por debajo hay una API en Rust y un solo motor de Git, tambien en Rust, que
usan escritorio y VS Code.

El primer corte esta hecho: abrir un repositorio y ver su grafo funciona en escritorio y en
VS Code, con el mismo motor y el mismo componente. La API tiene la salud y el contrato. La web
tiene la landing en dos idiomas, entrar y crear cuenta de maqueta y la pagina de estado. Lo que
falta esta en [docs/backlog.md](docs/backlog.md).

## Arrancar en cinco minutos

Necesitas: Node 24 con corepack, Rust con rustup (`rust-toolchain.toml` fija 1.99.1 y rustup lo
instala solo), Docker y [just](https://just.systems). Para la cobertura y la auditoria,
`cargo install cargo-llvm-cov cargo-deny`. Reserva unos 15 GB de disco: el `target/` de Rust
de un `just check` completo ronda los 12 GB.

pnpm va siempre por corepack y tiene que ser el 11. Si tienes otro pnpm global, quitalo:

```bash
npm rm -g pnpm && corepack enable pnpm
```

```bash
cp .env.example .env
just setup     # instala dependencias, engancha los hooks y revisa los toolchains
just up        # base de datos, API, web, gateway y buzon de correo en Docker
```

Cuando los contenedores esten sanos:

- http://localhost:8090 la web y la API bajo el mismo origen
- http://localhost:8090/health/ready el estado del servicio
- http://localhost:8090/swagger-ui/ el contrato de la API (solo en desarrollo)
- http://localhost:5173 la web sola, con recarga en caliente
- http://localhost:8025 el buzon de correo de desarrollo

Para tumbarlo todo, `just down`.

**Escritorio en Linux** necesita los paquetes de sistema de Tauri:

```bash
sudo apt-get install libwebkit2gtk-4.1-dev build-essential curl wget file libxdo-dev \
  libssl-dev libayatana-appindicator3-dev librsvg2-dev
```

Sin ellos, el lado nativo corre en Docker con todo dentro: `just desktop-docker-rust-check`,
`just desktop-docker-build` y `just desktop-docker-e2e-native`. Antes hace falta `just setup`.

**VS Code**: la extension se arranca con "Run Extension" desde `apps/vscode`. Los pasos estan
en [su README](apps/vscode/README.md).

## Que hay dentro

```
apps/api            API en Rust (Salvo + SQLx). Cinco crates, uno por capa.
apps/web            Web en React Router 8: landing prerenderizada y SPA.
apps/desktop        Ramal Desktop: Tauri 2 con el motor en el mismo proceso.
apps/vscode         Ramal for VS Code: extension host y webviews.
apps/vscode-engine  El motor como proceso hijo de la extension (JSON-RPC por stdio).
crates/             El motor de Git, compartido por escritorio y VS Code.
packages/           TypeScript compartido: dominio de interfaz, textos, UI, vista del
                    repositorio, puente con el motor, cliente de la API y configuracion.
tools/              Reglas de calidad comunes, complejidad de Rust, imagen de Tauri y scripts.
docker/             Gateway de nginx de desarrollo.
docs/               Arquitectura, contrato de la API, modelo de datos, backlog y ADR.
```

## Comandos

Todo pasa por `just`. Escribe `just` a secas para ver la lista completa.

| Comando             | Que hace                                                               |
| ------------------- | ---------------------------------------------------------------------- |
| `just setup`        | Instala dependencias, engancha los hooks y revisa los toolchains       |
| `just doctor`       | Ensena las versiones y falla si pnpm no es el 11.18                    |
| `just check`        | Todo el enforcement, igual que la CI pero sin lo caro                  |
| `just ci`           | `check` mas auditoria, tipos generados, e2e, builds y la pila completa |
| `just fmt`          | Formatea todo (prettier y rustfmt)                                     |
| `just lint`         | Lint de todo                                                           |
| `just test`         | Tests unitarios de todo                                                |
| `just cov`          | Cobertura con sus umbrales                                             |
| `just e2e`          | Playwright de las tres apps, VS Code de verdad y la API con Postgres   |
| `just build`        | Build de produccion                                                    |
| `just guard`        | Reglas comunes: tamano, tests apagados, TODO sin ticket y mas          |
| `just up`           | Levanta la pila de desarrollo en Docker                                |
| `just down`         | La tumba y borra los volumenes                                         |
| `just stack-smoke`  | Levanta una pila aparte, la prueba por el gateway y la tumba           |
| `just db-up`        | Levanta solo PostgreSQL                                                |
| `just protocol-gen` | Regenera los tipos del protocolo del motor                             |
| `just api-client`   | Regenera el cliente de la API desde `apps/api/openapi.json`            |

Hay recetas por pieza con los prefijos `api-`, `engine-`, `packages-`, `web-`, `desktop-` y
`vscode-`. Por ejemplo `just api-test-db` o `just desktop-e2e`.

## Como se fuerza la calidad

Las reglas no estan solo escritas: rompen el build.

- Ningun fichero de codigo pasa de 300 lineas, ni de 400 si es de test.
- Complejidad cognitiva hasta 15 y ciclomatica hasta 10, en TypeScript y en Rust. Funciones de
  hasta 50 lineas, 5 parametros y 4 niveles de anidamiento.
- Nada de codigo muerto, TODO sin ticket, tests apagados sin motivo ni limites apagados en linea.
- Cobertura minima del 80% en el nucleo de cada pieza y del 60% en el total.
- Las capas no se cruzan: lo comprueban tests de arquitectura en Rust y ESLint en TypeScript.

Se aplica en tres sitios: `just check` en local, los hooks de git en cada commit y GitHub
Actions en cada push. El check requerido se llama `all-green`. Antes de abrir un PR, `just ci`.
El detalle esta en el [ADR 0005](docs/adr/0005-enforcement-automatizado.md).

## Documentacion

- [docs/architecture.md](docs/architecture.md): la arquitectura hexagonal y como se aplica en
  cada pieza.
- [docs/api-surface.md](docs/api-surface.md): el contrato de la API, endpoint por endpoint.
- [docs/data-model.md](docs/data-model.md): el modelo de datos de la API.
- [docs/backlog.md](docs/backlog.md): lo que falta, por epicas.
- [docs/adr/](docs/adr/): por que se decidio cada cosa.
- [CLAUDE.md](CLAUDE.md): convenciones del repositorio, para personas y agentes.
- Cada app y cada paquete tiene su README: [apps/api](apps/api/README.md),
  [apps/web](apps/web/README.md), [apps/desktop](apps/desktop/README.md),
  [apps/vscode](apps/vscode/README.md), [crates](crates/README.md).

## Licencia

[FSL-1.1-ALv2](LICENSE.md): el codigo se puede leer, usar y modificar salvo para un producto que
compita con Ramal, y cada version pasa a Apache 2.0 a los dos anos. El porque esta en el
[ADR 0021](docs/adr/0021-licencia-fsl.md).
