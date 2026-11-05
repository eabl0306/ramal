use git_rudder_core_domain::trust::ConfigEntry;
use pretty_assertions::assert_eq;

use super::parse_config_list;

#[test]
fn keys_keep_their_origin_and_values_are_dropped() {
    let stdout =
        b"file:.git/config\0core.bare\nfalse\0file:.git/config\0core.fsmonitor\nrm -rf ~\0";

    assert_eq!(
        parse_config_list(stdout),
        vec![
            ConfigEntry {
                key: "core.bare".to_owned(),
                origin: "file:.git/config".to_owned(),
            },
            ConfigEntry {
                key: "core.fsmonitor".to_owned(),
                origin: "file:.git/config".to_owned(),
            },
        ]
    );
    assert_eq!(parse_config_list(b""), Vec::new());
}
