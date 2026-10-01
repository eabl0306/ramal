//! Pureza del nucleo del cliente (docs/architecture.md 5.3).
//!
//! Los `clippy.toml` de los nucleos repiten los umbrales de la raiz (clippy no
//! los fusiona) y prohiben todas las puertas de IO de la biblioteca estandar, no
//! solo la mas comun. Las listas se leen por seccion y sin comentarios: una
//! entrada comentada o puesta en la seccion equivocada no prohibe nada, y antes
//! pasaba porque bastaba con que el texto apareciera en el fichero. Como red de
//! seguridad, el codigo del nucleo tampoco puede nombrar los modulos de IO.
//!
//! La misma lista vive en apps/api/crates/api/tests/support/clippy.rs para el
//! nucleo del servidor; este test comprueba ademas que los cuatro nucleos
//! prohiben lo mismo y revisa el codigo de los cuatro.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "tests fail loudly on purpose; a panic here is the assertion"
)]
#![allow(
    unused_crate_dependencies,
    reason = "the lint runs per target and this test only reads files"
)]

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

const CLIENT_CORES: &[&str] = &["crates/core-domain", "crates/core-application"];
const SERVER_CORES: &[&str] = &["apps/api/crates/domain", "apps/api/crates/application"];

/// Umbrales de clippy que los nucleos repiten de la raiz.
const THRESHOLDS: &[&str] = &[
    "cognitive-complexity-threshold",
    "too-many-arguments-threshold",
    "too-many-lines-threshold",
    "type-complexity-threshold",
    "excessive-nesting-threshold",
    "avoid-breaking-exported-api",
];

/// Tipos de la biblioteca estandar que hacen IO o leen la hora de pared.
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

/// Funciones y metodos de la biblioteca estandar que hacen IO, leen el entorno,
/// la hora de pared o manejan hilos y el proceso.
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
    "std::thread::spawn",
    "std::thread::sleep",
];

/// Modulos de IO que el codigo del nucleo no puede ni nombrar.
const FORBIDDEN_MODULES: &[&str] = &["fs", "env", "net", "process", "os"];

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn read(relative: &str) -> String {
    std::fs::read_to_string(root().join(relative)).expect("the file should exist")
}

/// Rutas `path = "..."` de un array de clippy.toml (`disallowed-types = [`),
/// sin las lineas comentadas ni lo que va detras de un `#`.
fn declared_paths(text: &str, key: &str) -> BTreeSet<String> {
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

fn thresholds(text: &str) -> BTreeMap<String, String> {
    text.lines()
        .filter(|line| !line.trim_start().starts_with('#'))
        .filter_map(|line| line.split_once('='))
        .map(|(key, value)| (key.trim().to_owned(), value.trim().to_owned()))
        .filter(|(key, _)| THRESHOLDS.contains(&key.as_str()))
        .collect()
}

fn std_paths(text: &str, key: &str) -> BTreeSet<String> {
    declared_paths(text, key)
        .into_iter()
        .filter(|path| path.starts_with("std::"))
        .collect()
}

#[test]
fn the_core_clippy_files_repeat_the_root_thresholds() {
    let expected = thresholds(&read("clippy.toml"));
    assert_eq!(
        expected.len(),
        THRESHOLDS.len(),
        "the root clippy.toml lost a threshold"
    );
    for core in CLIENT_CORES {
        let file = format!("{core}/clippy.toml");
        assert_eq!(
            thresholds(&read(&file)),
            expected,
            "{file} drifted from the root clippy.toml"
        );
    }
}

#[test]
fn the_core_clippy_files_forbid_every_io_entry_point() {
    for core in CLIENT_CORES {
        let file = format!("{core}/clippy.toml");
        let text = read(&file);
        for (key, required) in [
            ("disallowed-types", DISALLOWED_TYPES),
            ("disallowed-methods", DISALLOWED_METHODS),
        ] {
            let declared = declared_paths(&text, key);
            for path in required {
                assert!(
                    declared.contains(*path),
                    "{file} does not forbid {path} in {key}"
                );
            }
        }
    }
}

#[test]
fn the_four_cores_forbid_the_same_standard_library_paths() {
    let reference = read("crates/core-domain/clippy.toml");
    for core in CLIENT_CORES.iter().chain(SERVER_CORES) {
        let text = read(&format!("{core}/clippy.toml"));
        for key in ["disallowed-types", "disallowed-methods"] {
            assert_eq!(
                std_paths(&text, key),
                std_paths(&reference, key),
                "{core}/clippy.toml forbids a different set of std paths in {key}"
            );
        }
    }
}

#[test]
fn a_commented_or_misplaced_entry_forbids_nothing() {
    let text = r#"
disallowed-types = [
  { path = "std::fs::File", reason = "x" },
  { path = "std::env::var", reason = "x" },
]
disallowed-methods = [
  # { path = "std::env::var", reason = "x" },
  { path = "std::fs::read", reason = "x" }, # { path = "std::fs::write" }
]
"#;
    let methods = declared_paths(text, "disallowed-methods");
    assert_eq!(methods, BTreeSet::from(["std::fs::read".to_owned()]));
    assert!(declared_paths(text, "disallowed-types").contains("std::env::var"));
}

/// Los modulos de IO que nombra un fuente: `std::fs::...` directo o dentro de un
/// `use std::{...}` agrupado, aunque este anidado. Los comentarios de linea no
/// cuentan.
fn named_io_modules(source: &str) -> BTreeSet<String> {
    let code = source
        .lines()
        .map(|line| line.split("//").next().unwrap_or_default())
        .collect::<Vec<_>>()
        .join("\n");
    code.split("std::")
        .skip(1)
        .flat_map(|rest| identifiers(scope_after_std(rest)))
        .filter(|name| FORBIDDEN_MODULES.contains(&name.as_str()))
        .collect()
}

/// Lo que nombra un `std::`: el primer segmento, o el grupo `{...}` entero.
fn scope_after_std(rest: &str) -> &str {
    let Some(group) = rest.strip_prefix('{') else {
        let end = rest
            .find(|c: char| !(c.is_alphanumeric() || c == '_'))
            .unwrap_or(rest.len());
        return rest.get(..end).unwrap_or_default();
    };
    let mut depth = 1_usize;
    for (index, char) in group.char_indices() {
        depth = match char {
            '{' => depth + 1,
            '}' => depth - 1,
            _ => depth,
        };
        if depth == 0 {
            return group.get(..index).unwrap_or_default();
        }
    }
    group
}

fn identifiers(text: &str) -> Vec<String> {
    text.split(|c: char| !(c.is_alphanumeric() || c == '_'))
        .filter(|token| !token.is_empty())
        .map(str::to_owned)
        .collect()
}

fn rust_files(dir: &Path, found: &mut Vec<PathBuf>) {
    for entry in std::fs::read_dir(dir).expect("the source folder should exist") {
        let path = entry.expect("the entry should be readable").path();
        if path.is_dir() {
            rust_files(&path, found);
        } else if path.extension().is_some_and(|ext| ext == "rs") {
            found.push(path);
        }
    }
}

#[test]
fn the_core_sources_never_name_io_modules() {
    for core in CLIENT_CORES.iter().chain(SERVER_CORES) {
        let mut files = Vec::new();
        rust_files(&root().join(core).join("src"), &mut files);
        assert!(!files.is_empty(), "{core}/src has no Rust files");
        for file in files {
            let source = std::fs::read_to_string(&file).unwrap();
            let named = named_io_modules(&source);
            assert!(
                named.is_empty(),
                "{} names std IO modules {named:?}",
                file.display()
            );
        }
    }
}

#[test]
fn the_source_scan_sees_direct_and_grouped_imports() {
    let source = "use std::fmt;\nuse std::{collections::HashMap, io::{self, Read}, fs, env::var};\n\
                  // std::net\nfn a() { std::process::exit(0) }\n";
    let expected = ["env", "fs", "process"].map(str::to_owned);
    assert_eq!(named_io_modules(source), BTreeSet::from(expected));
    let clean = "use std::sync::Arc;\nlet fs = 1;\nuse std::path::Path;\nlet env = 2;\n";
    assert!(named_io_modules(clean).is_empty());
}
