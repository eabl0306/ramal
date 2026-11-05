use super::{Remote, web_url};

#[test]
fn https_ssh_and_scp_urls_lead_to_the_same_page() {
    let page = Some("https://github.com/acme/rudder".to_owned());
    for url in [
        "https://github.com/acme/rudder.git",
        "https://github.com/acme/rudder/",
        "https://ana:token@github.com/acme/rudder.git",
        "ssh://git@github.com/acme/rudder.git",
        "ssh://git@github.com:22/acme/rudder.git",
        "git@github.com:acme/rudder.git",
        "github.com:acme/rudder",
        "git://GitHub.com/acme/rudder",
    ] {
        assert_eq!(web_url(url), page, "{url}");
    }
}

#[test]
fn an_http_port_is_kept_and_nested_groups_survive() {
    assert_eq!(
        web_url("http://git.local:3000/team/sub/repo.git").as_deref(),
        Some("http://git.local:3000/team/sub/repo")
    );
    assert_eq!(
        web_url("git@gitlab.com:group/subgroup/project.git").as_deref(),
        Some("https://gitlab.com/group/subgroup/project")
    );
}

#[test]
fn azure_devops_ssh_urls_get_their_web_form() {
    assert_eq!(
        web_url("git@ssh.dev.azure.com:v3/acme/Rudder/engine").as_deref(),
        Some("https://dev.azure.com/acme/Rudder/_git/engine")
    );
    assert_eq!(web_url("git@ssh.dev.azure.com:v2/acme/x"), None);
}

#[test]
fn local_paths_and_unknown_schemes_have_no_page() {
    for url in [
        "/srv/git/repo.git",
        "../sibling",
        "C:\\repos\\x",
        "file:///srv/git/repo.git",
        "https://github.com",
        "",
        "ext::ssh -i key host %S",
    ] {
        assert_eq!(web_url(url), None, "{url:?}");
    }
}

#[test]
fn a_remote_falls_back_to_its_push_url() {
    let remote = Remote {
        name: "origin".to_owned(),
        fetch_url: Some("/local/mirror".to_owned()),
        push_url: Some("git@github.com:acme/rudder.git".to_owned()),
    };
    assert_eq!(
        remote.web_url().as_deref(),
        Some("https://github.com/acme/rudder")
    );
}
