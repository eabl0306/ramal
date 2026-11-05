//! Casos de uso sobre repositorios abiertos: abrir, confiar y abrir un submodulo.

mod open_repository;
mod open_submodule;
mod trust_repository;

pub use open_repository::{OpenDependencies, OpenRepository, OpenedRepository};
pub use open_submodule::OpenSubmodule;
pub use trust_repository::TrustRepository;
