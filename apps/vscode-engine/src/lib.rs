//! Sidecar de la extension de VS Code: adaptador driving (bucle JSON-RPC 2.0 sobre
//! stdio con marcos `Content-Length`) y raiz de composicion exterior del cliente
//! de VS Code.
//!
//! stdout es solo protocolo. Todo el log va a stderr, que el extension host vuelca
//! en el canal "Ramal Engine"; por eso `print_stdout` esta denegado en clippy. El
//! proceso termina cuando se cierra stdin: si el extension host muere, el motor no
//! se queda huerfano.

pub mod args;
pub mod bootstrap;
pub mod framing;
pub mod message;
pub mod server;

use std::ffi::OsString;
use std::io::Write as _;
use std::process::ExitCode;
use std::sync::Arc;
use std::time::Duration;

use args::{Args, Command};
use message::EngineHandler;

/// Codigo de salida con argumentos invalidos, como hacen las herramientas de linea
/// de comandos.
const USAGE_EXIT: u8 = 2;

/// Arranca el sidecar con los argumentos del proceso (el primero es el programa).
#[must_use]
pub fn run<I: IntoIterator<Item = OsString>>(arguments: I) -> ExitCode {
    let args = match Args::parse(arguments.into_iter().skip(1)) {
        Ok(Command::Serve(args)) => args,
        Ok(Command::Version) => return print_version(),
        Err(error) => {
            let _ = writeln!(
                std::io::stderr(),
                "ramal-vscode-engine: {error}\n{}",
                args::USAGE
            );
            return ExitCode::from(USAGE_EXIT);
        }
    };
    init_tracing(&args.log_level);
    serve_stdio(&args)
}

fn serve_stdio(args: &Args) -> ExitCode {
    let runtime = match tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
    {
        Ok(runtime) => runtime,
        Err(error) => {
            tracing::error!(%error, "could not start the async runtime");
            return ExitCode::FAILURE;
        }
    };
    let handler = Arc::new(EngineHandler::new(bootstrap::build_engine(args)));
    tracing::info!(
        version = ramal_engine::ENGINE_VERSION,
        "engine ready on stdio"
    );
    let result = runtime.block_on(server::serve(
        handler,
        tokio::io::stdin(),
        tokio::io::stdout(),
    ));
    // La lectura de stdin de tokio vive en un hilo bloqueante; sin un limite, cerrar
    // el runtime esperaria a una lectura que no va a llegar.
    runtime.shutdown_timeout(Duration::from_millis(200));
    match result {
        Ok(()) => {
            tracing::info!("stdin closed; engine stopped");
            ExitCode::SUCCESS
        }
        Err(error) => {
            tracing::error!(%error, "connection closed by a protocol error");
            ExitCode::FAILURE
        }
    }
}

fn print_version() -> ExitCode {
    let line = format!("ramal-vscode-engine {}", ramal_engine::ENGINE_VERSION);
    match writeln!(std::io::stdout(), "{line}") {
        Ok(()) => ExitCode::SUCCESS,
        Err(_) => ExitCode::FAILURE,
    }
}

/// Log a stderr, sin colores (el canal de salida de VS Code no los pinta). Si ya
/// hay un subscriber (en los tests), se respeta.
fn init_tracing(level: &str) {
    let filter = tracing_subscriber::EnvFilter::try_new(level)
        .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info"));
    let _ = tracing_subscriber::fmt()
        .with_writer(std::io::stderr)
        .with_ansi(false)
        .with_env_filter(filter)
        .try_init();
}

// `unused_crate_dependencies` se evalua por target, y el de los tests unitarios
// recibe tambien las dev-dependencies que solo usan los tests de `tests/`.
#[cfg(test)]
use {pretty_assertions as _, tempfile as _};
