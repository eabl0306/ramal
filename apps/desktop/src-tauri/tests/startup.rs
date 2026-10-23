//! Test de arranque del composition root (ADR 0006): `assemble` monta los
//! plugins, el motor, los comandos y el `setup` sobre el runtime simulado de
//! Tauri, y el bucle de eventos simulado corre ese `setup` como lo haria la app
//! real al arrancar. Al ver `Ready`, el test cierra la ventana y el bucle acaba.

#![allow(
    unused_crate_dependencies,
    reason = "the lint runs per target and this test does not use every dependency"
)]
#![allow(
    clippy::expect_used,
    reason = "tests fail loudly on purpose; a panic here is a failed assertion"
)]

use std::sync::{Arc, Mutex, PoisonError};

use git_rudder_desktop::bootstrap::{assemble, build_engine};
use git_rudder_desktop::picker::{DialogFolderPicker, FolderPicker};
use git_rudder_engine::Engine;
use tauri::test::{MockRuntime, mock_builder};
use tauri::{AppHandle, Manager as _, RunEvent};

/// Lo que el test mira en cuanto la app esta lista.
#[derive(Debug, PartialEq, Eq)]
struct Started {
    engine: bool,
    folder_picker: bool,
    main_window: bool,
}

fn inspect(handle: &AppHandle<MockRuntime>) -> Started {
    Started {
        engine: handle.try_state::<Arc<Engine>>().is_some(),
        folder_picker: handle.try_state::<Arc<dyn FolderPicker>>().is_some(),
        main_window: handle.get_webview_window("main").is_some(),
    }
}

#[test]
fn the_setup_registers_the_native_folder_picker_and_opens_the_main_window() {
    let app = assemble(mock_builder(), Arc::new(build_engine()))
        .build(git_rudder_desktop::context())
        .expect("the app must build with the mock runtime");
    let seen = Arc::new(Mutex::new(None));
    let record = Arc::clone(&seen);

    app.run_return(move |handle, event| {
        if matches!(event, RunEvent::Ready) {
            *record.lock().unwrap_or_else(PoisonError::into_inner) = Some(inspect(handle));
            for window in handle.webview_windows().into_values() {
                window.destroy().expect("the mock window can be destroyed");
            }
        }
    });

    let started = seen.lock().unwrap_or_else(PoisonError::into_inner).take();
    let expected = Started {
        engine: true,
        folder_picker: true,
        main_window: true,
    };
    assert_eq!(started, Some(expected));
}

#[test]
fn the_native_picker_debug_output_hides_the_app_handle() {
    let app = mock_builder()
        .build(git_rudder_desktop::context())
        .expect("the app must build with the mock runtime");

    let picker = DialogFolderPicker::new(app.handle().clone());
    assert_eq!(format!("{picker:?}"), "DialogFolderPicker");
}
