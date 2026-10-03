//! Complejidad de cada funcion de Rust del repo, con los mismos limites que el
//! TypeScript (ADR 0005): cognitiva hasta 15, contada como Sonar, y ciclomatica
//! hasta 10, contada como la regla `complexity` de ESLint.
//!
//! Existe porque clippy no mide ninguna de las dos. `clippy::cognitive_complexity`
//! es un conteo plano, sin penalizar el anidamiento y sin contar `&&` ni `||`: una
//! funcion con bucles anidados que Sonar puntua con 17 pasaba limpia. Y clippy no
//! tiene lint ciclomatico: una cadena de 14 `if` seguidos (ciclomatica 15) pasaba
//! tambien. rust-code-analysis se descarto: su ultima version es de 2023, no
//! entiende los closures `async` y su cognitiva no cuenta los `loop`.
//!
//! Lo que no ve: el codigo dentro de las macros (un `tokio::select!`, por
//! ejemplo), igual que clippy, y la recursion, que Sonar suma y aqui no.

mod cognitive;
mod cyclomatic;
mod files;
mod functions;

use std::path::{Path, PathBuf};

// proc-macro2 solo se declara para encender `span-locations`: sin esa feature,
// syn no sabe en que linea esta cada funcion fuera de una macro procedural.
use proc_macro2 as _;

pub use files::rust_files;
pub use functions::FunctionScore;

/// Limite de complejidad cognitiva: el mismo que `cognitive-complexity-threshold`
/// de clippy.toml y que `sonarjs/cognitive-complexity` en @git-rudder/eslint-config.
pub const COGNITIVE_LIMIT: u32 = 15;

/// Limite de complejidad ciclomatica: el mismo que `complexity` en
/// @git-rudder/eslint-config.
pub const CYCLOMATIC_LIMIT: u32 = 10;

/// Puntuaciones de todas las funciones (y closures) de un fuente de Rust.
///
/// # Errors
///
/// El error de syn si el fuente no es Rust valido. Quien llama lo trata como un
/// fallo: un fichero que no se puede medir no puede pasar en verde.
pub fn analyze_source(source: &str) -> Result<Vec<FunctionScore>, syn::Error> {
    syn::parse_file(source).map(|file| functions::collect(&file))
}

/// Una funcion que pasa de un limite.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Violation {
    /// Fichero donde vive.
    pub file: PathBuf,
    /// La funcion y su puntuacion.
    pub score: FunctionScore,
    /// Que limite rompe y con que valor.
    pub problem: String,
}

/// Lo que encontro una pasada sobre el repo.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Report {
    /// Ficheros leidos.
    pub files: usize,
    /// Funciones y closures medidas.
    pub functions: usize,
    /// Las que pasan de algun limite.
    pub violations: Vec<Violation>,
}

/// Por que no se pudo medir.
#[derive(Debug, thiserror::Error)]
pub enum CheckError {
    /// Una raiz que no existe: medir menos de lo pedido seria un verde falso.
    #[error("{0} does not exist")]
    MissingRoot(PathBuf),
    /// Ninguna raiz, o raices sin un solo fichero de Rust.
    #[error("no Rust files found under the given roots")]
    NoFiles,
    /// Un fichero que no se pudo leer.
    #[error("cannot read {path}: {source}")]
    Read {
        /// El fichero.
        path: PathBuf,
        /// El error de E/S.
        source: std::io::Error,
    },
    /// Un fichero que syn no entiende.
    #[error("cannot parse {path}:{line}: {message}")]
    Parse {
        /// El fichero.
        path: PathBuf,
        /// La linea del error.
        line: usize,
        /// El mensaje de syn.
        message: String,
    },
}

/// Mide todos los `.rs` bajo `roots` y devuelve las funciones que pasan de algun
/// limite.
///
/// # Errors
///
/// [`CheckError`] si falta una raiz, no hay ningun fichero o alguno no se puede
/// leer o analizar.
pub fn check(roots: &[PathBuf]) -> Result<Report, CheckError> {
    let mut report = Report::default();
    for file in rust_files(roots)? {
        let scores = analyze_file(&file)?;
        report.files += 1;
        report.functions += scores.len();
        report.violations.extend(
            scores
                .into_iter()
                .filter_map(|score| violation(&file, score)),
        );
    }
    if report.files == 0 {
        return Err(CheckError::NoFiles);
    }
    Ok(report)
}

fn analyze_file(path: &Path) -> Result<Vec<FunctionScore>, CheckError> {
    let source = std::fs::read_to_string(path).map_err(|source| CheckError::Read {
        path: path.to_path_buf(),
        source,
    })?;
    analyze_source(&source).map_err(|error| CheckError::Parse {
        path: path.to_path_buf(),
        line: error.span().start().line,
        message: error.to_string(),
    })
}

fn violation(file: &Path, score: FunctionScore) -> Option<Violation> {
    let mut problems = Vec::new();
    if score.cognitive > COGNITIVE_LIMIT {
        problems.push(format!(
            "cognitive complexity {} (limit {COGNITIVE_LIMIT})",
            score.cognitive
        ));
    }
    if score.cyclomatic > CYCLOMATIC_LIMIT {
        problems.push(format!(
            "cyclomatic complexity {} (limit {CYCLOMATIC_LIMIT})",
            score.cyclomatic
        ));
    }
    (!problems.is_empty()).then(|| Violation {
        file: file.to_path_buf(),
        score,
        problem: problems.join(", "),
    })
}

#[cfg(test)]
mod tests;
