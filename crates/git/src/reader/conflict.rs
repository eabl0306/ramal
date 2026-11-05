//! `ConflictReader`: los lados de un fichero en conflicto y sus bloques.
//!
//! Los lados salen del indice (`ls-files -u`: etapa 1 la base, 2 la nuestra, 3
//! la suya), no del fichero del arbol de trabajo, que el usuario puede haber
//! tocado. Los bloques se vuelven a calcular con `git merge-file` pidiendo
//! marcadores zdiff3 (desde git 2.35; antes, diff3): asi las lineas comunes salen
//! del bloque y siempre esta el marcador base (B-19).

use async_trait::async_trait;
use git_rudder_core_application::{ConflictReader, GitReadError, GitReader};
use git_rudder_core_domain::conflict::{ConflictFile, parse_conflict_markers};
use git_rudder_core_domain::status::ConflictCode;
use git_rudder_core_domain::{RepoPath, RepoRoot, TrustLevel};

use super::GitCliReader;
use super::support::{failed, path_arg};
use crate::cli::GitCommand;

/// git mira los primeros 8000 bytes para decidir si un fichero es binario.
const BINARY_SNIFF: usize = 8000;

/// Un lado mas grande que esto se elige entero: no se resuelve linea a linea.
const MAX_SIDE_BYTES: usize = 8 * 1024 * 1024;

#[async_trait]
impl ConflictReader for GitCliReader {
    async fn read_conflict(
        &self,
        repo: &RepoRoot,
        path: &RepoPath,
    ) -> Result<ConflictFile, GitReadError> {
        let stages = self.stages(repo, path).await?;
        let code = code_of(&stages).ok_or_else(|| GitReadError::Missing {
            what: format!("conflict in {path}"),
        })?;
        let [base, ours, theirs] = self.blobs(repo, &stages).await?;
        let binary = [&base, &ours, &theirs]
            .into_iter()
            .flatten()
            .any(|blob| blob.len() > MAX_SIDE_BYTES || is_binary(blob));
        let content = match (&ours, &theirs) {
            (Some(ours), Some(theirs)) if !binary => {
                let merged = self
                    .merge(repo, ours, base.as_deref().unwrap_or_default(), theirs)
                    .await?;
                parse_conflict_markers(&merged).ok()
            }
            _ => None,
        };
        Ok(ConflictFile {
            path: path.to_string(),
            code,
            binary,
            content,
        })
    }
}

/// Sha del blob de cada etapa (base, nuestra, suya), si esta.
type Stages = [Option<String>; 3];

impl GitCliReader {
    async fn stages(&self, repo: &RepoRoot, path: &RepoPath) -> Result<Stages, GitReadError> {
        let command = GitCommand::new("ls-files")
            .arg("-u")
            .arg("-z")
            .paths([path_arg(path)]);
        let stdout = self.stdout(repo, TrustLevel::Untrusted, &command).await?;
        let text = String::from_utf8_lossy(&stdout);
        let mut stages: Stages = [None, None, None];
        for entry in text.split('\0').filter(|entry| !entry.is_empty()) {
            // `<modo> <sha> <etapa>\t<ruta>`
            let Some((meta, _)) = entry.split_once('\t') else {
                continue;
            };
            let mut parts = meta.split(' ').skip(1);
            let (Some(sha), Some(stage)) = (parts.next(), parts.next()) else {
                continue;
            };
            let slot = stage
                .parse::<usize>()
                .ok()
                .and_then(|stage| stage.checked_sub(1))
                .and_then(|index| stages.get_mut(index));
            if let Some(slot) = slot {
                *slot = Some(sha.to_owned());
            }
        }
        Ok(stages)
    }

    async fn blobs(
        &self,
        repo: &RepoRoot,
        stages: &Stages,
    ) -> Result<[Option<Vec<u8>>; 3], GitReadError> {
        let [base, ours, theirs] = stages;
        let (base, ours, theirs) = tokio::try_join!(
            self.blob(repo, base.as_deref()),
            self.blob(repo, ours.as_deref()),
            self.blob(repo, theirs.as_deref()),
        )?;
        Ok((base, ours, theirs).into())
    }

    async fn blob(
        &self,
        repo: &RepoRoot,
        sha: Option<&str>,
    ) -> Result<Option<Vec<u8>>, GitReadError> {
        let Some(sha) = sha else {
            return Ok(None);
        };
        let command = GitCommand::new("cat-file").arg("blob").revisions([sha]);
        self.stdout(repo, TrustLevel::Untrusted, &command)
            .await
            .map(Some)
    }

    /// `git merge-file -p` sobre los tres lados escritos en ficheros temporales.
    async fn merge(
        &self,
        repo: &RepoRoot,
        ours: &[u8],
        base: &[u8],
        theirs: &[u8],
    ) -> Result<String, GitReadError> {
        let io = |error: std::io::Error| GitReadError::Failed {
            detail: format!("could not prepare the conflict sides: {error}"),
        };
        let dir = tempfile::tempdir().map_err(io)?;
        let sides = [("ours", ours), ("base", base), ("theirs", theirs)];
        for (name, bytes) in sides {
            tokio::fs::write(dir.path().join(name), bytes)
                .await
                .map_err(io)?;
        }
        let zdiff3 = self.version().await?.capabilities.zdiff3;
        let style = if zdiff3 { "--zdiff3" } else { "--diff3" };
        let command = GitCommand::new("merge-file")
            .arg("-p")
            .arg(style)
            .arg("-L")
            .arg("ours")
            .arg("-L")
            .arg("base")
            .arg("-L")
            .arg("theirs")
            .paths([
                dir.path().join("ours"),
                dir.path().join("base"),
                dir.path().join("theirs"),
            ]);
        let output = self.run(repo.as_path(), &command).await?;
        // Sale con el numero de conflictos; un codigo negativo (mas de 127) es un
        // fallo de verdad.
        match output.code {
            Some(0..=127) => Ok(String::from_utf8_lossy(&output.stdout).into_owned()),
            _ => Err(failed("merge-file", &output)),
        }
    }
}

/// La forma del conflicto por las etapas que hay.
const fn code_of(stages: &Stages) -> Option<ConflictCode> {
    let [base, ours, theirs] = stages;
    Some(match (base.is_some(), ours.is_some(), theirs.is_some()) {
        (true, true, true) => ConflictCode::BothModified,
        (false, true, true) => ConflictCode::BothAdded,
        (true, true, false) => ConflictCode::DeletedByThem,
        (true, false, true) => ConflictCode::DeletedByUs,
        (false, true, false) => ConflictCode::AddedByUs,
        (false, false, true) => ConflictCode::AddedByThem,
        (true, false, false) => ConflictCode::BothDeleted,
        (false, false, false) => return None,
    })
}

/// Como git: hay un NUL en el principio.
fn is_binary(blob: &[u8]) -> bool {
    blob.iter().take(BINARY_SNIFF).any(|byte| *byte == 0)
}
