use std::io::Write;
use std::path::PathBuf;
use std::sync::{Arc, Mutex, PoisonError};

use git_rudder_engine::{Engine, EngineBuilder, EngineConfig};

use super::{build_engine, init_tracing, log_engine_info};

/// Lo que escribe el log mientras dura `body`, en un buffer en memoria.
fn captured_log(body: impl FnOnce()) -> String {
    let buffer = Arc::new(Mutex::new(Vec::new()));
    let writer = Arc::clone(&buffer);
    let subscriber = tracing_subscriber::fmt()
        .with_writer(move || BufferWriter(Arc::clone(&writer)))
        .with_ansi(false)
        .finish();
    tracing::subscriber::with_default(subscriber, body);
    let bytes = buffer
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
        .clone();
    String::from_utf8_lossy(&bytes).into_owned()
}

struct BufferWriter(Arc<Mutex<Vec<u8>>>);

impl Write for BufferWriter {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        self.0
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .extend_from_slice(bytes);
        Ok(bytes.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

fn startup_log(engine: &Engine) -> String {
    captured_log(|| tauri::async_runtime::block_on(log_engine_info(engine)))
}

#[test]
fn tracing_can_be_initialised_more_than_once() {
    init_tracing();
    init_tracing();
    assert!(tracing::dispatcher::has_been_set());
}

#[test]
fn the_startup_log_says_the_engine_is_ready_when_git_works() {
    let log = startup_log(&build_engine());
    assert!(log.contains("engine ready"), "{log}");
}

#[test]
fn the_startup_log_warns_when_git_is_missing() {
    let config = EngineConfig {
        git_path: Some(PathBuf::from("/nonexistent/git-rudder/git")),
    };
    let log = startup_log(&EngineBuilder::new(config).build());
    assert!(log.contains("engine started without a usable git"), "{log}");
}
