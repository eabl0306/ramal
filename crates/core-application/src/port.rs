//! Puertos del motor: lo que los casos de uso necesitan del exterior.
//!
//! Cada puerto nace con su primer caso de uso; no se declaran puertos que nadie
//! usa. Los planificados (GitWriter, SecretStore, CloudClient, PlatformClient,
//! AiClient, Clock...) ya tienen sitio en docs/architecture.md.

pub mod git_reader;
