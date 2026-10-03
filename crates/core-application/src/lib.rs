//! Casos de uso y puertos del motor de Git. Solo conoce el dominio: el acceso real
//! al repositorio llega por los puertos que implementa `git-rudder-git`, y quien ejecuta
//! los futuros es el runtime del host (aqui no hay tokio).

pub mod error;
pub mod port;
pub mod state;
pub mod use_case;

#[cfg(any(test, feature = "testing"))]
pub mod testing;

pub use error::EngineError;
pub use port::git_reader::{
    GitCapabilities, GitReadError, GitReader, GitVersion, GitVersionInfo, LogQuery,
    MINIMUM_GIT_VERSION,
};
pub use state::open_repositories::OpenRepositories;
pub use use_case::engine::{DescribeEngine, EngineDescription};
pub use use_case::graph::LoadGraphWindow;
pub use use_case::repo::{OpenRepository, OpenedRepository};
