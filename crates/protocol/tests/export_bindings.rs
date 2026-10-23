//! Escribe `methods.ts` e `index.ts` en la carpeta de ts-rs
//! (`TS_RS_EXPORT_DIR`, que `.cargo/config.toml` fija en
//! `packages/bridge/src/generated`).
//!
//! ts-rs genera un fichero por tipo, pero no valores ni el mapa de metodos. Este
//! test los saca de `define_methods!` y del trait `TS`: los nombres de los tipos
//! nunca se escriben a mano. Su nombre empieza por `export_bindings` para que
//! `just protocol-gen` (que filtra por ese prefijo) lo ejecute junto a los de
//! ts-rs. Tambien exporta las raices con todas sus dependencias, asi funciona
//! aunque se lance solo.

#![allow(
    unused_crate_dependencies,
    reason = "the lint runs per target and this test only needs ts-rs and the crate"
)]
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "a failed export must stop the test loudly"
)]

use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::path::{Path, PathBuf};

use git_rudder_protocol::methods::{MethodVisitor, PROTOCOL_VERSION, RpcMethod, visit_methods};
use git_rudder_protocol::{RpcNotification, RpcRequest, RpcResponse};
use ts_rs::{Config, TS, TypeVisitor};

const NOTE: &str = "// Generado por crates/protocol/tests/export_bindings.rs (just protocol-gen). \
                    No editar a mano.\n";

/// Tipos exportables alcanzables, por fichero de salida.
type TypeFiles = BTreeMap<PathBuf, String>;

/// Recorre las dependencias como lo hace ts-rs al exportar: solo los tipos con
/// fichero propio, y cada uno una vez (`JsonValue` es recursivo).
struct Collector<'a> {
    cfg: &'a Config,
    types: &'a mut TypeFiles,
}

impl TypeVisitor for Collector<'_> {
    fn visit<T: TS + 'static + ?Sized>(&mut self) {
        let Some(path) = T::output_path() else {
            return;
        };
        if self.types.insert(path, T::ident(self.cfg)).is_none() {
            T::visit_dependencies(self);
        }
    }
}

fn export_root<T: TS + 'static>(cfg: &Config, types: &mut TypeFiles) {
    T::export_all(cfg).expect("ts-rs could not export the type");
    Collector { cfg, types }.visit::<T>();
}

/// Lineas de `EngineMethods` y las listas de metodos, en orden de declaracion.
struct MethodTable<'a> {
    cfg: &'a Config,
    types: &'a mut TypeFiles,
    entries: Vec<String>,
    all: Vec<&'static str>,
    host_only: Vec<&'static str>,
}

impl MethodVisitor for MethodTable<'_> {
    fn visit<M: RpcMethod>(&mut self) {
        export_root::<M::Params>(self.cfg, self.types);
        export_root::<M::Result>(self.cfg, self.types);
        self.entries.push(format!(
            "  {:?}: {{ params: {}; result: {} }};",
            M::NAME,
            M::Params::ident(self.cfg),
            M::Result::ident(self.cfg),
        ));
        self.all.push(M::NAME);
        if M::HOST_ONLY {
            self.host_only.push(M::NAME);
        }
    }
}

fn module_path(path: &Path) -> String {
    let without_extension = path.with_extension("");
    format!(
        "./{}",
        without_extension.to_string_lossy().replace('\\', "/")
    )
}

fn literal_list(names: &[&str]) -> String {
    let quoted: Vec<String> = names.iter().map(|name| format!("{name:?}")).collect();
    format!("[{}]", quoted.join(", "))
}

fn imports_ts(cfg: &Config) -> String {
    let mut method_types = TypeFiles::new();
    visit_methods(&mut ImportVisitor {
        cfg,
        types: &mut method_types,
    });
    let imports: BTreeMap<&String, String> = method_types
        .iter()
        .map(|(path, name)| (name, module_path(path)))
        .collect();
    let mut out = String::new();
    for (name, path) in imports {
        writeln!(out, "import type {{ {name} }} from {path:?};").unwrap();
    }
    out
}

fn methods_ts(cfg: &Config, table: &MethodTable<'_>) -> String {
    let imports = imports_ts(cfg);
    let entries = table.entries.join("\n");
    let all = literal_list(&table.all);
    let host_only = literal_list(&table.host_only);
    format!(
        "{NOTE}{imports}
/** Version del protocolo del motor. */
export const PROTOCOL_VERSION = {PROTOCOL_VERSION};

/** Cada metodo del motor con sus parametros y su resultado. */
export type EngineMethods = {{
{entries}
}};

/** Nombre de un metodo del motor. */
export type EngineMethod = keyof EngineMethods;

/** Todos los metodos: la lista blanca del extension host. */
export const ENGINE_METHODS = {all} as const satisfies readonly EngineMethod[];

/** Metodos que solo puede llamar un host: nunca un webview. */
export const HOST_METHODS = {host_only} as const satisfies readonly EngineMethod[];
"
    )
}

/// Solo los tipos que nombra `EngineMethods` (parametros y resultados).
struct ImportVisitor<'a> {
    cfg: &'a Config,
    types: &'a mut TypeFiles,
}

impl MethodVisitor for ImportVisitor<'_> {
    fn visit<M: RpcMethod>(&mut self) {
        for (path, name) in [
            (M::Params::output_path(), M::Params::ident(self.cfg)),
            (M::Result::output_path(), M::Result::ident(self.cfg)),
        ] {
            if let Some(path) = path {
                self.types.insert(path, name);
            }
        }
    }
}

fn index_ts(types: &TypeFiles) -> String {
    let mut out = String::from(NOTE);
    for path in types.keys() {
        writeln!(out, "export type * from {:?};", module_path(path)).unwrap();
    }
    writeln!(out, "export * from \"./methods\";").unwrap();
    out
}

#[test]
fn export_bindings_methods_and_index() {
    let cfg = Config::from_env();
    let dir = cfg.out_dir().to_path_buf();
    std::fs::create_dir_all(&dir).unwrap();

    let mut types = TypeFiles::new();
    export_root::<RpcRequest>(&cfg, &mut types);
    export_root::<RpcResponse>(&cfg, &mut types);
    export_root::<RpcNotification>(&cfg, &mut types);
    let mut table = MethodTable {
        cfg: &cfg,
        types: &mut types,
        entries: Vec::new(),
        all: Vec::new(),
        host_only: Vec::new(),
    };
    visit_methods(&mut table);

    let methods = methods_ts(&cfg, &table);
    std::fs::write(dir.join("methods.ts"), methods).unwrap();
    std::fs::write(dir.join("index.ts"), index_ts(&types)).unwrap();

    for path in types.keys() {
        assert!(
            dir.join(path).is_file(),
            "{} was not exported",
            path.display()
        );
    }
}
