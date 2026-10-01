//! Lanza git con el entorno fijo, limites de tiempo y de salida, y lo mata si la
//! llamada se abandona.
//!
//! `kill_on_drop` mata a git si el futuro se suelta (por el limite de tiempo o
//! porque el host deja de esperar). Matar el grupo de procesos entero, que hace
//! falta cuando git lanza hijos (ssh, helpers, hooks), llega con la cancelacion de
//! operaciones largas (planificada): las lecturas del primer corte no lanzan hijos.

use std::io;
use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::time::{Duration, Instant};

use thiserror::Error;
use tokio::io::{AsyncRead, AsyncReadExt};
use tokio::process::Command;

use super::GitCommand;

/// Opciones globales de cada llamada: sin paginador, rutas sin escapar y sin color.
const GLOBAL_ARGS: &[&str] = &[
    "--no-pager",
    "-c",
    "core.quotepath=off",
    "-c",
    "color.ui=false",
];

/// Entorno fijo de cada llamada. `LC_ALL=C`: los mensajes de git no cambian con el
/// idioma del usuario. `GIT_TERMINAL_PROMPT=0`: git nunca espera una contrasena en
/// una terminal que no existe. `GIT_OPTIONAL_LOCKS=0`: una lectura en segundo plano
/// nunca toma `index.lock` mientras el usuario trabaja en su terminal (las
/// escrituras, planificadas, no lo llevaran).
const FIXED_ENV: &[(&str, &str)] = &[
    ("LC_ALL", "C"),
    ("GIT_TERMINAL_PROMPT", "0"),
    ("GIT_OPTIONAL_LOCKS", "0"),
];

/// Variables que redirigen git a otro repositorio o le cuelan configuracion. Si el
/// motor arranca desde un hook de git (o desde una terminal dentro de uno) las
/// heredaria, y leeria el repositorio equivocado.
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

/// Lo que se guarda de stderr: basta para el log y acota la memoria.
const STDERR_LIMIT: usize = 64 * 1024;

/// Limites de una llamada.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RunLimits {
    /// Tiempo maximo hasta que git termina.
    pub timeout: Duration,
    /// Bytes maximos de stdout.
    pub max_output_bytes: usize,
}

impl Default for RunLimits {
    /// Dos minutos y 256 MiB: un `log --topo-order` sobre un historial de un millon
    /// de commits tarda segundos y ocupa unos 200 MB.
    fn default() -> Self {
        Self {
            timeout: Duration::from_secs(120),
            max_output_bytes: 256 * 1024 * 1024,
        }
    }
}

/// Fallo al lanzar git o al leer su salida.
#[derive(Debug, Error)]
pub enum RunError {
    /// No se pudo lanzar el ejecutable.
    #[error("git could not be launched: {0}")]
    NotFound(io::Error),
    /// El directorio de trabajo no existe.
    #[error("working directory {} does not exist", .0.display())]
    MissingDirectory(PathBuf),
    /// git no termino a tiempo.
    #[error("git did not finish within {0:?}")]
    TimedOut(Duration),
    /// La salida supero el limite.
    #[error("git output exceeded {0} bytes")]
    OutputTooLarge(usize),
    /// Fallo de E/S hablando con git.
    #[error("could not talk to git: {0}")]
    Io(#[from] io::Error),
}

/// Resultado de una llamada que llego a terminar.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RunOutput {
    /// Codigo de salida, o `None` si git murio por una senal.
    pub code: Option<i32>,
    /// Salida estandar completa.
    pub stdout: Vec<u8>,
    /// Comienzo de stderr, para el log.
    pub stderr: String,
}

impl RunOutput {
    /// git salio con 0.
    #[must_use]
    pub const fn success(&self) -> bool {
        matches!(self.code, Some(0))
    }
}

/// Lanzador de git con un ejecutable y unos limites.
#[derive(Debug, Clone)]
pub struct GitRunner {
    binary: PathBuf,
    limits: RunLimits,
}

impl GitRunner {
    /// Lanzador de `binary`.
    #[must_use]
    pub const fn new(binary: PathBuf, limits: RunLimits) -> Self {
        Self { binary, limits }
    }

    /// Ruta del ejecutable.
    #[must_use]
    pub fn binary(&self) -> &Path {
        &self.binary
    }

    /// Ejecuta `command` en `cwd` (o en el directorio actual) y espera a que
    /// termine. Un codigo de salida distinto de 0 no es un error aqui: lo decide
    /// quien llama, porque para `symbolic-ref -q` un 1 es una respuesta.
    ///
    /// # Errors
    ///
    /// [`RunError`] si git no se puede lanzar, el directorio no existe, no termina a
    /// tiempo o su salida supera el limite.
    pub async fn run(
        &self,
        cwd: Option<&Path>,
        command: &GitCommand,
    ) -> Result<RunOutput, RunError> {
        if let Some(dir) = cwd
            && !tokio::fs::metadata(dir)
                .await
                .is_ok_and(|meta| meta.is_dir())
        {
            return Err(RunError::MissingDirectory(dir.to_path_buf()));
        }
        let started = Instant::now();
        let mut process = self.process(cwd, command);
        let mut child = process.spawn().map_err(|error| match error.kind() {
            io::ErrorKind::NotFound | io::ErrorKind::PermissionDenied => RunError::NotFound(error),
            _ => RunError::Io(error),
        })?;
        let stdout = child.stdout.take();
        let stderr = child.stderr.take();
        let collect = async {
            let (stdout, stderr) = tokio::try_join!(
                read_capped(stdout, self.limits.max_output_bytes),
                read_truncated(stderr, STDERR_LIMIT),
            )?;
            let status = child.wait().await?;
            Ok::<_, RunError>(RunOutput {
                code: status.code(),
                stdout,
                stderr: String::from_utf8_lossy(&stderr).into_owned(),
            })
        };
        let output = tokio::time::timeout(self.limits.timeout, collect)
            .await
            .map_err(|_| RunError::TimedOut(self.limits.timeout))??;
        tracing::debug!(
            subcommand = %command.subcommand(),
            code = ?output.code,
            elapsed_ms = started.elapsed().as_millis(),
            "git finished"
        );
        Ok(output)
    }

    fn process(&self, cwd: Option<&Path>, command: &GitCommand) -> Command {
        let mut process = Command::new(&self.binary);
        process
            .args(GLOBAL_ARGS)
            .args(command.args())
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .kill_on_drop(true);
        for name in SCRUBBED_ENV {
            process.env_remove(name);
        }
        process.envs(FIXED_ENV.iter().copied());
        if let Some(dir) = cwd {
            process.current_dir(dir);
        }
        process
    }
}

/// Lee todo `reader` y falla si pasa de `limit` bytes.
async fn read_capped<R: AsyncRead + Unpin>(
    reader: Option<R>,
    limit: usize,
) -> Result<Vec<u8>, RunError> {
    let Some(reader) = reader else {
        return Ok(Vec::new());
    };
    let mut buffer = Vec::new();
    let cap = u64::try_from(limit).unwrap_or(u64::MAX).saturating_add(1);
    reader.take(cap).read_to_end(&mut buffer).await?;
    if buffer.len() > limit {
        return Err(RunError::OutputTooLarge(limit));
    }
    Ok(buffer)
}

/// Lee todo `reader` pero solo guarda los primeros `limit` bytes. Se sigue leyendo
/// para que git no se bloquee con la tuberia llena.
async fn read_truncated<R: AsyncRead + Unpin>(
    reader: Option<R>,
    limit: usize,
) -> Result<Vec<u8>, RunError> {
    let Some(mut reader) = reader else {
        return Ok(Vec::new());
    };
    let mut kept = Vec::new();
    let mut chunk = [0_u8; 8 * 1024];
    loop {
        let read = reader.read(&mut chunk).await?;
        if read == 0 {
            return Ok(kept);
        }
        let room = limit.saturating_sub(kept.len()).min(read);
        kept.extend_from_slice(chunk.get(..room).unwrap_or_default());
    }
}

#[cfg(test)]
mod tests;
