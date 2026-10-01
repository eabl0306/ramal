//! Adaptador driven del git CLI: implementa `GitReader` lanzando el ejecutable de
//! git (ADR 0009). Nada de libgit2.
//!
//! Reglas que evitan los errores clasicos de envolver una CLI:
//!
//! - Formatos de maquina siempre (`-z`, `--format` con `%x00` y `%x1f`), mas
//!   `--no-pager`, sin color y `-c core.quotepath=off`.
//! - Entorno fijo: `LC_ALL=C`, `GIT_TERMINAL_PROMPT=0` y, en las lecturas,
//!   `GIT_OPTIONAL_LOCKS=0`. Se borran las variables que redirigirian git a otro
//!   repositorio (`GIT_DIR` y familia): un hook que lanzara el motor las heredaria.
//! - Ningun texto externo llega a argv como opcion: las opciones son `&'static
//!   str` por tipo, las revisiones van tras `--end-of-options` y la ruta del
//!   repositorio nunca va en argv (es el directorio de trabajo del proceso).
//! - Las refs se leen con `for-each-ref`, nunca con la decoracion `%D` del log.

pub mod cli;
pub mod parse;
pub mod reader;

pub use cli::{GitBinary, GitCommand, GitRunner, RunError, RunLimits, RunOutput};
pub use reader::{GitCliConfig, GitCliReader};
