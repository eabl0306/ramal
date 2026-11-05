//! `git config --local --list --show-origin -z`: lo que se mira antes de confiar.
//!
//! Con `-z` cada entrada es `<origen>\0<clave>\n<valor>\0`. Solo interesan origen
//! y clave: el valor de una clave que lanza un comando no se ensena (puede ser
//! largo o llevar un secreto), basta con decir que esta.

use git_rudder_core_domain::trust::ConfigEntry;

/// Lee la salida completa.
#[must_use]
pub fn parse_config_list(stdout: &[u8]) -> Vec<ConfigEntry> {
    let text = String::from_utf8_lossy(stdout);
    let mut fields = text.split('\0');
    let mut entries = Vec::new();
    while let (Some(origin), Some(pair)) = (fields.next(), fields.next()) {
        let key = pair.split_once('\n').map_or(pair, |(key, _)| key);
        if !key.is_empty() {
            entries.push(ConfigEntry {
                key: key.to_owned(),
                origin: origin.to_owned(),
            });
        }
    }
    entries
}

#[cfg(test)]
mod tests;
