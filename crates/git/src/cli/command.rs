//! `GitCommand`: argumentos de una llamada a git sin texto externo en posicion de
//! opcion.
//!
//! Un nombre de rama remota o una revision pueden venir de otros, y un valor que
//! empieza por `-` se convertiria en una opcion de git. Por eso el tipo solo deja
//! meter argumentos fijos (`&'static str`, conocidos al compilar), numeros que se
//! formatean aqui y revisiones detras de `--end-of-options` (git >= 2.24). Lo
//! que venga de fuera no tiene forma de llegar a argv como opcion.

use std::ffi::OsString;

/// Argumentos de una llamada a git, sin el ejecutable ni las opciones globales.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GitCommand {
    args: Vec<OsString>,
    revisions_started: bool,
}

impl GitCommand {
    /// Empieza por el subcomando (`log`, `for-each-ref`...).
    #[must_use]
    pub fn new(subcommand: &'static str) -> Self {
        Self {
            args: vec![OsString::from(subcommand)],
            revisions_started: false,
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
