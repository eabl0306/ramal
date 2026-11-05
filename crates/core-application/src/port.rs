//! Puertos del motor: lo que los casos de uso necesitan del exterior.
//!
//! Cada puerto nace con su primer caso de uso; no se declaran puertos que nadie
//! usa. Los planificados (GitWriter, SecretStore, CloudClient, PlatformClient,
//! AiClient, Clock...) ya tienen sitio en docs/architecture.md.
//!
//! - `git_reader`: version, descubrimiento, refs y log del grafo.
//! - `repo_reader`: el resto de lecturas de un repositorio, por area.
//! - `host`: lo que depende de donde corre el motor (eventos, vigilancia,
//!   confianza, busqueda en carpetas y cambio de git).

pub mod git_reader;
pub mod host;
pub mod repo_reader;
