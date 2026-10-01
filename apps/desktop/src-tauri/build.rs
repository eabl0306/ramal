//! Genera el contexto de Tauri (configuracion, capacidades y recursos) y declara
//! los comandos del host en el `AppManifest`: asi cada uno necesita un permiso
//! explicito en `capabilities/default.json` y ninguna ventana puede invocar algo
//! que no se le haya concedido.

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let manifest = tauri_build::AppManifest::new().commands(&["rpc", "repo_pick_and_open"]);
    // Un error aqui sale como el error de main: cargo lo ensena y para el build.
    tauri_build::try_build(tauri_build::Attributes::new().app_manifest(manifest))?;
    Ok(())
}
