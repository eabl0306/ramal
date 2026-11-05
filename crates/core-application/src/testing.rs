//! Dobles de test para los puertos del motor.
//!
//! Solo se compila con la feature `testing` (o en los tests de este crate).
//! Permite probar los casos de uso sin git, sin disco y sin runtime asincrono.

mod block_on;
mod fake_git_reader;
mod fake_host;
mod fake_repo_reader;
mod samples;

pub use block_on::block_on;
pub use fake_git_reader::{FakeCalls, FakeGitReader};
pub use fake_host::{FakeGitSetup, FakeScanner, FakeWatcher, MemoryTrustStore, RecordingSink};
pub use fake_repo_reader::FakeRepoReader;
pub use samples::{commit, sha, version_info};
