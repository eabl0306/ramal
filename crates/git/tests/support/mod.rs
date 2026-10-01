//! Constructor de repositorios de prueba con el propio git en un tempdir.
//!
//! El entorno se fija para que el resultado sea identico en cualquier maquina:
//! `HOME` y la configuracion global apuntan al tempdir, `GIT_CONFIG_NOSYSTEM=1`,
//! autor y fechas fijas (cada commit avanza el reloj un minuto) y `LC_ALL=C`. Sin
//! eso, la configuracion de quien corre el test (firma obligatoria, rama por
//! defecto, hooks) se cuela en el resultado. Tambien se borran `GIT_DIR` y su
//! familia: los tests corren desde hooks de git (pre-push) que las exportan.
//!
//! Lo usan los tests de `ramal-git`, los de `ramal-engine` y los del sidecar
//! (estos dos lo incluyen con `#[path]`, porque no pueden depender de este crate).

#![allow(
    dead_code,
    reason = "each test binary that includes this module uses a different subset"
)]
#![allow(
    clippy::expect_used,
    reason = "a broken fixture must stop the test loudly"
)]

use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

use tempfile::TempDir;

/// Primer instante de los commits de prueba: 2026-09-30T10:00:00+02:00.
const FIRST_COMMIT_SECONDS: u64 = 1_790_755_200;

/// Variables que el entorno del test puede traer y que redirigirian git.
const SCRUBBED_ENV: &[&str] = &[
    "GIT_DIR",
    "GIT_WORK_TREE",
    "GIT_INDEX_FILE",
    "GIT_OBJECT_DIRECTORY",
    "GIT_ALTERNATE_OBJECT_DIRECTORIES",
    "GIT_COMMON_DIR",
    "GIT_NAMESPACE",
    "GIT_PREFIX",
    "GIT_CONFIG_PARAMETERS",
];

/// Un repositorio de prueba en un directorio temporal que se borra al soltarlo.
pub struct FixtureRepo {
    temp: TempDir,
    clock: AtomicU64,
}

impl FixtureRepo {
    /// Repositorio vacio con la rama `main` sin nacer.
    pub fn new() -> Self {
        let temp = tempfile::tempdir().expect("could not create a temp dir");
        std::fs::create_dir(temp.path().join("home")).expect("could not create HOME");
        std::fs::create_dir(temp.path().join("repo")).expect("could not create the repo dir");
        std::fs::write(temp.path().join("home").join(".gitconfig"), "")
            .expect("could not create an empty global config");
        let repo = Self {
            temp,
            clock: AtomicU64::new(FIRST_COMMIT_SECONDS),
        };
        repo.git(&["init", "--quiet", "--initial-branch=main"]);
        repo
    }

    /// Raiz del arbol de trabajo.
    pub fn path(&self) -> PathBuf {
        self.temp.path().join("repo")
    }

    /// Raiz tal como la devuelve git (sin enlaces simbolicos; en macOS el tempdir
    /// vive bajo un enlace).
    pub fn canonical_path(&self) -> PathBuf {
        std::fs::canonicalize(self.path()).expect("the repo dir must exist")
    }

    /// Directorio temporal que no es un repositorio, fuera del de este.
    pub fn outside(&self) -> PathBuf {
        let dir = self.temp.path().join("outside");
        std::fs::create_dir_all(&dir).expect("could not create a plain dir");
        dir
    }

    /// Ejecuta git en el repositorio y devuelve stdout sin el salto final.
    pub fn git(&self, args: &[&str]) -> String {
        let output = self
            .command(args)
            .output()
            .expect("git must be installed to run these tests");
        assert!(
            output.status.success(),
            "git {args:?} failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        String::from_utf8_lossy(&output.stdout)
            .trim_end()
            .to_owned()
    }

    fn command(&self, args: &[&str]) -> Command {
        let home = self.temp.path().join("home");
        let date = format!("{} +0200", self.clock.load(Ordering::Relaxed));
        let mut command = Command::new("git");
        command.args(args).current_dir(self.path());
        for name in SCRUBBED_ENV {
            command.env_remove(name);
        }
        command
            .env("HOME", &home)
            .env("XDG_CONFIG_HOME", home.join(".config"))
            .env("GIT_CONFIG_GLOBAL", home.join(".gitconfig"))
            .env("GIT_CONFIG_NOSYSTEM", "1")
            .env("LC_ALL", "C")
            .env("GIT_AUTHOR_NAME", "Ana Ruiz")
            .env("GIT_AUTHOR_EMAIL", "ana@example.com")
            .env("GIT_COMMITTER_NAME", "Ana Ruiz")
            .env("GIT_COMMITTER_EMAIL", "ana@example.com")
            .env("GIT_AUTHOR_DATE", &date)
            .env("GIT_COMMITTER_DATE", &date);
        command
    }

    fn tick(&self) {
        self.clock.fetch_add(60, Ordering::Relaxed);
    }

    /// Commit vacio con el mensaje dado; devuelve su sha.
    pub fn commit(&self, message: &str) -> String {
        self.tick();
        self.git(&["commit", "--quiet", "--allow-empty", "-m", message]);
        self.rev("HEAD")
    }

    /// Crea una rama en HEAD sin cambiarse a ella.
    pub fn branch(&self, name: &str) {
        self.git(&["branch", name]);
    }

    /// Se cambia a una rama existente.
    pub fn checkout(&self, name: &str) {
        self.git(&["checkout", "--quiet", name]);
    }

    /// Crea una rama en HEAD y se cambia a ella.
    pub fn checkout_new(&self, name: &str) {
        self.git(&["checkout", "--quiet", "-b", name]);
    }

    /// Merge sin avance rapido de una o varias ramas (varias: octopus).
    pub fn merge(&self, branches: &[&str], message: &str) -> String {
        self.tick();
        let mut args = vec!["merge", "--quiet", "--no-ff", "-m", message];
        args.extend_from_slice(branches);
        self.git(&args);
        self.rev("HEAD")
    }

    /// Tag ligero en HEAD.
    pub fn tag(&self, name: &str) {
        self.git(&["tag", name]);
    }

    /// Tag anotado en HEAD.
    pub fn annotated_tag(&self, name: &str, message: &str) {
        self.tick();
        self.git(&["tag", "-a", name, "-m", message]);
    }

    /// Separa HEAD en una revision.
    pub fn detach(&self, revision: &str) {
        self.git(&["checkout", "--quiet", "--detach", revision]);
    }

    /// Escribe un fichero en el arbol de trabajo.
    pub fn write(&self, relative: &str, contents: &str) {
        std::fs::write(self.path().join(relative), contents).expect("could not write a file");
    }

    /// Sha completo de una revision.
    pub fn rev(&self, revision: &str) -> String {
        self.git(&["rev-parse", "--verify", revision])
    }

    /// Repositorio con historia: `main` con dos commits y un tag, una rama
    /// `feature` que sale del primero y vuelve con un merge.
    pub fn with_merged_feature() -> Self {
        let repo = Self::new();
        repo.commit("Initial commit");
        repo.checkout_new("feature");
        repo.commit("Add graph");
        repo.checkout("main");
        repo.commit("Fix login");
        repo.merge(&["feature"], "Merge branch 'feature'");
        repo.tag("v1.0");
        repo
    }
}

/// Ruta como texto, para los parametros JSON de los tests del motor.
pub fn path_text(path: &Path) -> String {
    path.to_string_lossy().into_owned()
}
