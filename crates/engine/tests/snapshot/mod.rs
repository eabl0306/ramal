//! Snapshots del protocolo: la respuesta de un metodo, normalizada y guardada
//! como JSON en `tests/snapshots/`, para revisarla en el PR.
//!
//! Lo que cambia de una maquina a otra se sustituye por etiquetas estables: cada
//! sha por `<sha-N>` en el orden en que aparece (y su forma corta por
//! `<sha-N:short>`), la carpeta temporal por `<tmp>`, y el `repoId` y la huella
//! `snapshot` por su nombre. Asi el fichero es el mismo en Linux, macOS y
//! Windows.
//!
//! Con `GIT_RUDDER_UPDATE_SNAPSHOTS=1` se reescriben en vez de compararse.

#![allow(
    clippy::expect_used,
    clippy::panic,
    reason = "a snapshot that cannot be read or does not match must fail the test loudly"
)]

use std::path::{Path, PathBuf};

use serde_json::Value;

/// Claves cuyo valor depende de la ruta del repositorio, no de su historia.
const OPAQUE_KEYS: &[&str] = &["repoId", "snapshot"];

/// Normaliza `value` y lo compara con `tests/snapshots/<name>.json`.
pub fn assert_snapshot(name: &str, value: &Value, temp_roots: &[String]) {
    let normalized = normalize(value, temp_roots);
    let text = serde_json::to_string_pretty(&normalized).expect("a value always serializes") + "\n";
    let path = snapshot_path(name);
    if std::env::var_os("GIT_RUDDER_UPDATE_SNAPSHOTS").is_some() {
        std::fs::write(&path, &text).expect("could not write the snapshot");
        return;
    }
    let Ok(stored) = std::fs::read_to_string(&path) else {
        panic!(
            "missing snapshot {}; run with GIT_RUDDER_UPDATE_SNAPSHOTS=1 and review it",
            path.display()
        );
    };
    // Un checkout de Windows con autocrlf trae CRLF.
    let stored = stored.replace("\r\n", "\n");
    pretty_assertions::assert_eq!(stored, text, "snapshot {name} changed");
}

fn snapshot_path(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("snapshots")
        .join(format!("{name}.json"))
}

/// Copia de `value` con las etiquetas estables.
pub fn normalize(value: &Value, temp_roots: &[String]) -> Value {
    let mut shas = Vec::new();
    collect_shas(value, &mut shas);
    let labels = Labels { shas, temp_roots };
    labels.rewrite(value, None)
}

fn collect_shas(value: &Value, shas: &mut Vec<String>) {
    match value {
        Value::String(text) if is_full_sha(text) && !shas.contains(text) => {
            shas.push(text.clone());
        }
        Value::Array(items) => items.iter().for_each(|item| collect_shas(item, shas)),
        Value::Object(map) => map.values().for_each(|item| collect_shas(item, shas)),
        _ => {}
    }
}

fn is_full_sha(text: &str) -> bool {
    text.len() == 40 && text.bytes().all(|byte| byte.is_ascii_hexdigit())
}

struct Labels<'a> {
    shas: Vec<String>,
    temp_roots: &'a [String],
}

impl Labels<'_> {
    fn rewrite(&self, value: &Value, key: Option<&str>) -> Value {
        match value {
            Value::String(text) => Value::String(self.label(text, key)),
            Value::Array(items) => {
                Value::Array(items.iter().map(|item| self.rewrite(item, None)).collect())
            }
            Value::Object(map) => Value::Object(
                map.iter()
                    .map(|(name, item)| (name.clone(), self.rewrite(item, Some(name))))
                    .collect(),
            ),
            other => other.clone(),
        }
    }

    fn label(&self, text: &str, key: Option<&str>) -> String {
        if let Some(key) = key.filter(|key| OPAQUE_KEYS.contains(key)) {
            return format!("<{key}>");
        }
        if let Some(index) = self.shas.iter().position(|sha| sha == text) {
            return format!("<sha-{index}>");
        }
        let hex = text.len() >= 7 && text.bytes().all(|byte| byte.is_ascii_hexdigit());
        if let Some(index) = self
            .shas
            .iter()
            .position(|sha| hex && sha.starts_with(text))
        {
            return format!("<sha-{index}:short>");
        }
        self.temp_roots.iter().fold(text.to_owned(), |text, root| {
            text.replace(root.as_str(), "<tmp>")
        })
    }
}
