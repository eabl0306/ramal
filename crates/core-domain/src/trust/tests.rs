use super::{ConfigEntry, TrustFinding, TrustInspection, TrustLevel, is_risky_key};

fn entry(key: &str) -> ConfigEntry {
    ConfigEntry {
        key: key.to_owned(),
        origin: "file:.git/config".to_owned(),
    }
}

#[test]
fn every_key_of_the_adr_list_asks_for_trust() {
    let risky = [
        "core.fsmonitor",
        "core.hooksPath",
        "core.sshCommand",
        "core.pager",
        "core.editor",
        "sequence.editor",
        "diff.external",
        "diff.word.textconv",
        "diff.Tool.command",
        "filter.lfs.clean",
        "filter.lfs.process",
        "merge.ours.driver",
        "credential.helper",
        "credential.https://example.com.helper",
        "gpg.program",
        "gpg.ssh.program",
        "gpg.x509.program",
        "uploadpack.packObjectsHook",
        "include.path",
        "includeIf.gitdir:~/work/.path",
    ];
    for key in risky {
        assert!(is_risky_key(key), "{key} should ask for trust");
    }
}

#[test]
fn ordinary_keys_do_not_ask() {
    let ordinary = [
        "core.bare",
        "core.filemode",
        "remote.origin.url",
        "branch.main.remote",
        "diff.renames",
        "merge.conflictstyle",
        "user.name",
        "credential.username",
        "nosection",
    ];
    for key in ordinary {
        assert!(!is_risky_key(key), "{key} should not ask for trust");
    }
}

#[test]
fn findings_list_risky_keys_and_active_hooks_once() {
    let inspection = TrustInspection {
        config: vec![
            entry("core.bare"),
            entry("core.fsmonitor"),
            entry("core.fsmonitor"),
        ],
        hooks: vec![
            "pre-commit.sample".to_owned(),
            "post-checkout".to_owned(),
            ".DS_Store".to_owned(),
        ],
    };

    assert_eq!(
        inspection.findings(),
        vec![
            TrustFinding::ConfigKey {
                key: "core.fsmonitor".to_owned(),
                origin: "file:.git/config".to_owned(),
            },
            TrustFinding::Hook {
                name: "post-checkout".to_owned()
            },
        ]
    );
}

#[test]
fn an_inspection_without_risks_finds_nothing() {
    assert_eq!(TrustInspection::default().findings(), Vec::new());
}

#[test]
fn untrusted_is_the_safe_default() {
    assert_eq!(TrustLevel::default(), TrustLevel::Untrusted);
    assert!(TrustLevel::Trusted.is_trusted());
    assert!(!TrustLevel::Untrusted.is_trusted());
}
