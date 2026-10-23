#![allow(
    clippy::expect_used,
    reason = "tests fail loudly on purpose; a panic here is a failed assertion"
)]

use pretty_assertions::assert_eq;
use ramal_protocol::{ErrorKind, METHODS, MethodInfo, error_code};

use super::{CONTROL_NOTIFICATIONS, MAIN_WINDOW, SURFACES, authorize, methods_for};

#[test]
fn the_main_window_can_ask_for_the_graph_window() {
    assert_eq!(authorize(MAIN_WINDOW, "graph/window", false), Ok(()));
}

#[test]
fn a_host_method_is_answered_as_if_it_did_not_exist() {
    let error = authorize(MAIN_WINDOW, "repo/open", false)
        .expect_err("repo/open must be rejected for a webview");
    assert_eq!(error.code, error_code::METHOD_NOT_FOUND);
    assert_eq!(error.data.map(|data| data.kind), Some(ErrorKind::Internal));
}

#[test]
fn a_method_outside_the_window_list_is_rejected() {
    assert!(authorize(MAIN_WINDOW, "engine/info", false).is_err());
}

#[test]
fn an_unknown_method_is_rejected() {
    let error = authorize(MAIN_WINDOW, "stash/list", false)
        .expect_err("an unknown method must be rejected");
    assert_eq!(error.code, error_code::METHOD_NOT_FOUND);
}

#[test]
fn an_unknown_window_cannot_ask_for_anything() {
    let methods = methods_for("settings");
    assert!(methods.is_empty(), "{methods:?}");
    assert!(authorize("settings", "graph/window", false).is_err());
}

#[test]
fn only_control_notifications_are_let_through() {
    assert_eq!(authorize(MAIN_WINDOW, "$/cancel", true), Ok(()));
    assert!(authorize(MAIN_WINDOW, "graph/window", true).is_err());
}

#[test]
fn no_surface_lists_a_host_method_or_an_unknown_one() {
    for surface in SURFACES {
        for method in surface.methods {
            let info = MethodInfo::find(method).expect("every listed method is in define_methods!");
            assert!(
                !info.host_only,
                "{} lists the host method {method}",
                surface.window
            );
        }
    }
}

#[test]
fn every_host_method_stays_out_of_every_window() {
    let host_methods = METHODS.iter().filter(|method| method.host_only);
    for method in host_methods {
        for surface in SURFACES {
            assert!(authorize(surface.window, method.name, false).is_err());
        }
    }
}

#[test]
fn control_notifications_are_not_engine_methods() {
    for notification in CONTROL_NOTIFICATIONS {
        assert!(notification.starts_with("$/"));
        assert_eq!(MethodInfo::find(notification), None);
    }
}
