//! `GitCommand`: argumentos de una llamada a git sin texto externo en posicion de
//! opcion.
//!
//! Un nombre de rama remota o una revision pueden venir de otros, y un valor que
//! empieza por `-` se convertiria en una opcion de git. Por eso el tipo solo deja
//! meter argumentos fijos (`&'static str`, conocidos al compilar), numeros que se
//! formatean aqui y revisiones detras de `--end-of-options` (git >= 2.24). Lo
//! que venga de fuera no tiene forma de llegar a argv como opcion.

use std::ffi::OsString;

use git_rudder_core_domain::Sha;

/// Argumentos de una llamada a git, sin el ejecutable ni las opciones globales.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GitCommand {
    args: Vec<OsString>,
    revisions_started: bool,
    paths_started: bool,
}

impl GitCommand {
    /// Empieza por el subcomando (`log`, `for-each-ref`...).
    #[must_use]
    pub fn new(subcommand: &'static str) -> Self {
        Self {
            args: vec![OsString::from(subcommand)],
            revisions_started: false,
            paths_started: false,
        }
    }

    /// Anade un argumento fijo: una opcion, un formato o un patron constante.
    #[must_use]
    pub fn arg(mut self, arg: &'static str) -> Self {
        self.args.push(OsString::from(arg));
        self
    }

    /// Anade `--max-count=<n>`.
    #[must_use]
    pub fn max_count(mut self, count: usize) -> Self {
        self.args
            .push(OsString::from(format!("--max-count={count}")));
        self
    }

    /// Anade `name` seguido de un numero: `number("--skip=", 5)` da `--skip=5`.
    #[must_use]
    pub fn number(mut self, name: &'static str, value: usize) -> Self {
        self.args.push(OsString::from(format!("{name}{value}")));
        self
    }

    /// Anade una opcion con un valor que viene de fuera en un solo argumento:
    /// `value("--grep=", texto)`. Como empieza por el nombre fijo, el valor nunca
    /// puede convertirse en otra opcion. `name` tiene que acabar en `=`.
    #[must_use]
    pub fn value(mut self, name: &'static str, value: &str) -> Self {
        debug_assert!(name.ends_with('='), "{name} must end with =");
        self.args.push(OsString::from(format!("{name}{value}")));
        self
    }

    /// Anade un sha validado como argumento suelto, para los subcomandos que no
    /// entienden `--end-of-options` (`blame`). Un sha son 40 cifras
    /// hexadecimales: no puede leerse como opcion.
    #[must_use]
    pub fn object(mut self, sha: Sha) -> Self {
        self.args.push(OsString::from(sha.to_hex()));
        self
    }

    /// Anade rutas detras de `--`, que se escribe una sola vez (y despues de las
    /// revisiones). Con `GIT_LITERAL_PATHSPECS=1` son rutas, no patrones.
    #[must_use]
    pub fn paths<I, S>(mut self, paths: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<OsString>,
    {
        if !self.paths_started {
            self.args.push(OsString::from("--"));
            self.paths_started = true;
        }
        self.args.extend(paths.into_iter().map(Into::into));
        self
    }

    /// Anade revisiones o refs detras de `--end-of-options`, que se escribe una
    /// sola vez. Despues de esto git ya no lee nada como opcion.
    #[must_use]
    pub fn revisions<I, S>(mut self, revisions: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<OsString>,
    {
        if !self.revisions_started {
            self.args.push(OsString::from("--end-of-options"));
            self.revisions_started = true;
        }
        self.args.extend(revisions.into_iter().map(Into::into));
        self
    }

    /// Argumentos en orden.
    #[must_use]
    pub fn args(&self) -> &[OsString] {
        &self.args
    }

    /// Subcomando, para el log.
    #[must_use]
    pub fn subcommand(&self) -> String {
        self.args
            .first()
            .map(|arg| arg.to_string_lossy().into_owned())
            .unwrap_or_default()
    }
}
