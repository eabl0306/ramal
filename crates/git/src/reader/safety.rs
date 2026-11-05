//! Lecturas sin confianza que pasan por los filtros de git.
//!
//! `git status` y un diff del arbol de trabajo pasan cada fichero por su filtro
//! de limpieza (`filter.<driver>.clean` o `.process`), y eso es un programa que
//! elige la configuracion del repositorio. Las tres opciones del ADR 0020
//! (`core.fsmonitor`, diff externo y `textconv`) no lo cubren, asi que en una
//! lectura sin confianza cada filtro configurado se deja sin comando en la linea
//! de comandos (`-c filter.<driver>.clean=`), que gana al fichero. Un filtro sin
//! comando no se ejecuta; `required=false` evita que git falle por eso.

use git_rudder_core_application::GitReadError;
use git_rudder_core_domain::{RepoRoot, TrustLevel};

use super::GitCliReader;
use super::support::failed;
use crate::cli::{GitCommand, Invocation};

impl GitCliReader {
    /// `-c` que dejan sin comando cada filtro configurado, o nada si se confia.
    pub(super) async fn filter_overrides(
        &self,
        repo: &RepoRoot,
        trust: TrustLevel,
    ) -> Result<Vec<String>, GitReadError> {
        if trust.is_trusted() {
            return Ok(Vec::new());
        }
        // Sin --local: un filtro puede venir de un `include` o de la
        // configuracion global, y leerla no ejecuta nada.
        let command = GitCommand::new("config")
            .arg("-z")
            .arg("--name-only")
            .arg("--get-regexp")
            .arg("^filter\\.");
        let output = self.run(repo.as_path(), &command).await?;
        match output.code {
            Some(0) => Ok(neutralize(&String::from_utf8_lossy(&output.stdout))),
            // 1: ninguna clave coincide.
            Some(1) => Ok(Vec::new()),
            _ => Err(failed("config", &output)),
        }
    }

    /// Como `stdout`, apagando ademas los filtros si no se confia.
    pub(super) async fn filtered_stdout(
        &self,
        repo: &RepoRoot,
        trust: TrustLevel,
        command: &GitCommand,
    ) -> Result<Vec<u8>, GitReadError> {
        let overrides = self.filter_overrides(repo, trust).await?;
        let call = Invocation {
            untrusted: !trust.is_trusted(),
            config: &overrides,
            ..Invocation::new(Some(repo.as_path()), command)
        };
        let output = self.invoke(call).await?;
        if !output.success() {
            return Err(failed(&command.subcommand(), &output));
        }
        Ok(output.stdout)
    }
}

/// `filter.lfs.clean` y `filter.lfs.smudge` dan los cuatro `-c` de `lfs`, una
/// vez.
fn neutralize(names: &str) -> Vec<String> {
    let mut drivers: Vec<&str> = names
        .split('\0')
        .filter_map(|key| key.strip_prefix("filter."))
        .filter_map(|rest| rest.rsplit_once('.').map(|(driver, _)| driver))
        .filter(|driver| !driver.is_empty())
        .collect();
    drivers.sort_unstable();
    drivers.dedup();
    drivers
        .into_iter()
        .flat_map(|driver| {
            ["clean=", "smudge=", "process=", "required=false"]
                .map(|setting| format!("filter.{driver}.{setting}"))
        })
        .collect()
}

#[cfg(test)]
mod tests;
