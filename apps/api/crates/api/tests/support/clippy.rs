//! Comprobaciones de los `clippy.toml` del nucleo.
//!
//! Clippy usa el `clippy.toml` mas cercano sin fusionarlo con el de la raiz, asi
//! que los del nucleo repiten los umbrales comunes. Si alguien sube un umbral en
//! la raiz y se olvida de estos, el nucleo quedaria con el viejo sin que nadie lo
//! viera; si alguien borra una prohibicion, el nucleo podria leer la hora o
//! lanzar un proceso.
//!
//! Las prohibiciones se leen por seccion y sin comentarios: una entrada
//! comentada, o puesta en `disallowed-types` cuando es un metodo, no prohibe
//! nada aunque su texto siga en el fichero. La misma lista vive en
//! crates/engine/tests/core_purity.rs, que ademas comprueba que los cuatro
//! nucleos prohiben lo mismo.

use std::collections::{BTreeMap, BTreeSet};
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

/// Tipos prohibidos (seccion 5.3 de docs/architecture.md).
const DISALLOWED_TYPES: &[&str] = &[
    "std::process::Command",
    "std::fs::File",
    "std::fs::OpenOptions",
    "std::fs::DirBuilder",
    "std::net::TcpStream",
    "std::net::TcpListener",
    "std::net::UdpSocket",
    "std::net::ToSocketAddrs",
    "std::os::unix::net::UnixStream",
    "std::os::unix::net::UnixListener",
    "std::os::unix::net::UnixDatagram",
    "std::time::SystemTime",
    "std::thread::Builder",
];

/// Funciones y metodos prohibidos. El servidor anade los relojes de chrono.
const DISALLOWED_METHODS: &[&str] = &[
    "std::process::Command::new",
    "std::process::exit",
    "std::process::abort",
    "std::fs::read",
    "std::fs::read_to_string",
    "std::fs::write",
    "std::fs::canonicalize",
    "std::fs::copy",
    "std::fs::create_dir",
    "std::fs::create_dir_all",
    "std::fs::exists",
    "std::fs::hard_link",
    "std::fs::metadata",
    "std::fs::read_dir",
    "std::fs::read_link",
    "std::fs::remove_dir",
    "std::fs::remove_dir_all",
    "std::fs::remove_file",
    "std::fs::rename",
    "std::fs::set_permissions",
    "std::fs::symlink_metadata",
    "std::path::Path::canonicalize",
    "std::path::Path::exists",
    "std::path::Path::try_exists",
    "std::path::Path::is_dir",
    "std::path::Path::is_file",
    "std::path::Path::is_symlink",
    "std::path::Path::metadata",
    "std::path::Path::symlink_metadata",
    "std::path::Path::read_dir",
    "std::path::Path::read_link",
    "std::env::var",
    "std::env::var_os",
    "std::env::vars",
    "std::env::vars_os",
    "std::env::args",
    "std::env::args_os",
    "std::env::current_dir",
    "std::env::current_exe",
    "std::env::home_dir",
    "std::env::temp_dir",
    "std::env::set_current_dir",
    "std::net::ToSocketAddrs::to_socket_addrs",
    "std::io::stdin",
    "std::io::stdout",
    "std::io::stderr",
    "std::time::SystemTime::now",
    "std::time::SystemTime::elapsed",
    "chrono::Utc::now",
    "chrono::Local::now",
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

/// Rutas `path = "..."` de un array de clippy.toml (`disallowed-types = [`),
/// sin las lineas comentadas ni lo que va detras de un `#`.
pub fn declared_paths(text: &str, key: &str) -> BTreeSet<String> {
    let opening = format!("{key} = [");
    text.lines()
        .map(str::trim)
        .skip_while(|line| *line != opening)
        .skip(1)
        .take_while(|line| *line != "]")
        .filter_map(|line| line.split('#').next())
        .filter_map(|line| line.split("path = \"").nth(1))
        .filter_map(|rest| rest.split('"').next())
        .map(str::to_owned)
        .collect()
}

/// Asegura que cada `clippy.toml` del nucleo repite los umbrales de la raiz y
/// declara todas las prohibiciones de IO, cada una en su seccion.
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
        for (section, required) in [
            ("disallowed-types", DISALLOWED_TYPES),
            ("disallowed-methods", DISALLOWED_METHODS),
        ] {
            let declared = declared_paths(&text, section);
            for path in required {
                assert!(
                    declared.contains(*path),
                    "{core}/clippy.toml does not forbid `{path}` in {section}"
                );
            }
        }
    }
}
