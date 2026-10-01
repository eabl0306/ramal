//! Comprobaciones de los `clippy.toml` del nucleo.
//!
//! Clippy usa el `clippy.toml` mas cercano sin fusionarlo con el de la raiz, asi
//! que los del nucleo repiten los umbrales comunes. Si alguien sube un umbral en
//! la raiz y se olvida de estos, el nucleo quedaria con el viejo sin que nadie lo
//! viera; si alguien borra una prohibicion, el nucleo podria leer la hora o
//! lanzar un proceso.

use std::collections::BTreeMap;
use std::path::Path;

/// Crates del nucleo con `clippy.toml` propio.
const CORE_CRATES: &[&str] = &["domain", "application"];

/// Umbrales que tienen que coincidir con la raiz.
const THRESHOLDS: &[&str] = &[
    "cognitive-complexity-threshold",
    "too-many-arguments-threshold",
    "too-many-lines-threshold",
    "type-complexity-threshold",
    "excessive-nesting-threshold",
    "avoid-breaking-exported-api",
];

/// Prohibiciones que cada nucleo tiene que declarar (seccion 5.3 de
/// docs/architecture.md).
const REQUIRED_PATHS: &[&str] = &[
    "std::process::Command",
    "std::fs::File",
    "std::net::TcpStream",
    "std::net::TcpListener",
    "std::net::UdpSocket",
    "std::process::Command::new",
    "std::fs::read",
    "std::fs::read_to_string",
    "std::fs::write",
    "std::env::var",
    "std::env::var_os",
    "std::time::SystemTime::now",
    "std::thread::spawn",
    "std::thread::sleep",
];

fn read(path: &Path) -> String {
    std::fs::read_to_string(path).expect("the clippy.toml should exist")
}

/// Pares `clave = valor` de primer nivel con valor en una sola linea.
fn scalar_settings(text: &str) -> BTreeMap<String, String> {
    text.lines()
        .map(str::trim)
        .filter(|line| !line.starts_with('#'))
        .filter_map(|line| line.split_once('='))
        .map(|(key, value)| (key.trim().to_owned(), value.trim().to_owned()))
        .collect()
}

/// Asegura que cada `clippy.toml` del nucleo repite los umbrales de la raiz y
/// declara todas las prohibiciones de IO.
pub fn assert_core_clippy_files() {
    let root = scalar_settings(&read(&super::repo_root().join("clippy.toml")));
    for core in CORE_CRATES {
        let path = super::api_root()
            .join("crates")
            .join(core)
            .join("clippy.toml");
        let text = read(&path);
        let settings = scalar_settings(&text);
        for key in THRESHOLDS {
            assert_eq!(
                settings.get(*key),
                root.get(*key),
                "{core}/clippy.toml has a different `{key}` than the root clippy.toml"
            );
        }
        for required in REQUIRED_PATHS {
            let quoted = format!("\"{required}\"");
            assert!(
                text.contains(&quoted),
                "{core}/clippy.toml does not forbid `{required}`"
            );
        }
    }
}
