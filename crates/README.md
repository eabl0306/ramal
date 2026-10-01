# El motor de Git (crates/)

El motor del cliente, en Rust. Lo usan los dos hosts: Ramal Desktop lo lleva en proceso y la
extension de VS Code como sidecar (`apps/vscode-engine`). Los dos entran por `ramal-engine`, asi
que el protocolo es el mismo en los dos productos.

| Crate                                         | Papel                                                                          |
| --------------------------------------------- | ------------------------------------------------------------------------------ |
| `core-domain` (`ramal-core-domain`)           | Dominio puro: sha, refs y su huella, commits, fecha de git, carriles del grafo |
| `core-application` (`ramal-core-application`) | Puerto `GitReader`, casos de uso, registro de repos abiertos y fakes           |
| `protocol` (`ramal-protocol`)                 | Metodos, DTOs y sobre JSON-RPC; genera los tipos de `@ramal/bridge` con ts-rs  |
| `git` (`ramal-git`)                           | Adaptador del git CLI: version, descubrimiento, refs y log                     |
| `engine` (`ramal-engine`)                     | Dispatch, mapeo dominio a DTO y composition root compartido                    |

Metodos del primer corte: `engine/info`, `repo/open` (solo lo llaman los hosts) y
`graph/window`. El detalle esta en `docs/architecture.md` (4.2 y 6.5).

```sh
just engine-check     # clippy, tests y cobertura (80% en dominio y aplicacion, 60% global)
just protocol-gen     # regenera packages/bridge/src/generated; se commitea
just protocol-check   # falla si lo generado no esta al dia
```

Reglas que conviene saber antes de tocarlo:

- El nucleo (`core-domain` y `core-application`) no depende de serde, tokio ni nada de IO. Lo
  vigilan `engine/tests/architecture.rs` (lee `Cargo.lock`) y `engine/tests/core_purity.rs`
  (el `clippy.toml` de cada nucleo prohibe procesos, disco, red, entorno y reloj).
- Todo git va por el CLI, nunca por libgit2. Git minimo 2.34: lo aplica `repo/open`, y
  `engine/info` lo informa con `supported: false`.
- Los tests de `git`, `engine`, el sidecar y el host de escritorio crean repos reales con el
  constructor de `git/tests/support/mod.rs`, que los demas incluyen con `#[path]`.
- Cambiar un DTO o un metodo: `just protocol-gen` y commitear lo generado en el mismo commit.
