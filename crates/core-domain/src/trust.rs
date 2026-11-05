//! Confianza en un repositorio (ADR 0009 y ADR 0020).
//!
//! Abrir una carpeta ya ejecuta codigo si su configuracion lo pide: `git status`
//! lanza `core.fsmonitor` y los filtros, y un diff lanza `diff.external` y los
//! `textconv`. Antes del primer comando, el motor lee la configuracion local sin
//! ejecutarla y mira los hooks; aqui se decide que de eso obliga a preguntar.
//!
//! Es una lista de lo que obliga a preguntar, no de lo prohibido: una clave que no
//! este no se bloquea, solo no dispara la pregunta. Se mantiene a mano y crece con
//! cada version de git que traiga una clave nueva que lance un comando.

/// Si las lecturas pueden dejar que la configuracion del repositorio ejecute
/// comandos.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TrustLevel {
    /// El usuario confio en el repositorio, o no hay nada que pueda ejecutar
    /// codigo: git corre con su configuracion.
    Trusted,
    /// Solo se lee, con `core.fsmonitor`, el diff externo y los `textconv`
    /// apagados en la linea de comandos.
    #[default]
    Untrusted,
}

impl TrustLevel {
    /// El repositorio es de confianza.
    #[must_use]
    pub const fn is_trusted(self) -> bool {
        matches!(self, Self::Trusted)
    }
}

/// Una entrada de `git config --local --list --show-origin`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConfigEntry {
    /// Clave tal como la escribe git: seccion y nombre en minusculas, la
    /// subseccion como este (`diff.Word.textconv` sale `diff.Word.textconv`).
    pub key: String,
    /// De donde sale (`file:.git/config`).
    pub origin: String,
}

/// Algo del repositorio que puede ejecutar codigo y obliga a preguntar.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum TrustFinding {
    /// Una clave de configuracion que lanza un comando.
    ConfigKey {
        /// La clave.
        key: String,
        /// El fichero que la define.
        origin: String,
    },
    /// Un hook que no es un `.sample`.
    Hook {
        /// Nombre del fichero dentro de la carpeta de hooks.
        name: String,
    },
}

/// Lo que se encontro al mirar el repositorio sin ejecutar nada.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct TrustInspection {
    /// Configuracion local.
    pub config: Vec<ConfigEntry>,
    /// Ficheros de la carpeta de hooks.
    pub hooks: Vec<String>,
}

impl TrustInspection {
    /// Lo que obliga a preguntar, ordenado y sin repetir.
    #[must_use]
    pub fn findings(&self) -> Vec<TrustFinding> {
        let keys = self
            .config
            .iter()
            .filter(|entry| is_risky_key(&entry.key))
            .map(|entry| TrustFinding::ConfigKey {
                key: entry.key.clone(),
                origin: entry.origin.clone(),
            });
        let hooks = self
            .hooks
            .iter()
            .filter(|name| is_active_hook(name))
            .map(|name| TrustFinding::Hook { name: name.clone() });
        let mut findings: Vec<TrustFinding> = keys.chain(hooks).collect();
        findings.sort();
        findings.dedup();
        findings
    }
}

/// Claves exactas (seccion y nombre, en minusculas) que lanzan un comando.
const EXACT_KEYS: &[&str] = &[
    "core.fsmonitor",
    "core.hookspath",
    "core.sshcommand",
    "core.pager",
    "core.editor",
    "sequence.editor",
    "diff.external",
    "credential.helper",
];

/// La clave lanza un comando o carga otra configuracion que podria hacerlo.
///
/// Cubre la lista del ADR 0009: las exactas, `diff.*.textconv`, `diff.*.command`,
/// `filter.*`, `merge.*.driver`, `credential.*.helper`, `gpg*.program`,
/// `uploadpack.*` e `include*`.
#[must_use]
pub fn is_risky_key(key: &str) -> bool {
    let lower = key.to_ascii_lowercase();
    let Some((section, rest)) = lower.split_once('.') else {
        return false;
    };
    let name = rest.rsplit('.').next().unwrap_or(rest);
    let has_subsection = rest.contains('.');
    EXACT_KEYS.contains(&lower.as_str())
        || matches!(section, "filter" | "uploadpack" | "include" | "includeif")
        || (section == "diff" && has_subsection && matches!(name, "textconv" | "command"))
        || (section == "merge" && has_subsection && name == "driver")
        || (section == "credential" && name == "helper")
        || (section.starts_with("gpg") && name == "program")
}

/// git solo ejecuta los hooks sin la extension `.sample` que trae `git init`.
fn is_active_hook(name: &str) -> bool {
    !name.ends_with(".sample") && !name.starts_with('.')
}

#[cfg(test)]
mod tests;
