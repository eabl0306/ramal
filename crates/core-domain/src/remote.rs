//! Remotos y el enlace web que se deriva de su URL.
//!
//! La barra lateral ensena "Abrir en el navegador" para cada remoto. La URL de git
//! puede ser https, ssh o la forma corta de scp (`git@host:propietario/repo`), y
//! el enlace es siempre la pagina https del repositorio, sin credenciales, sin
//! puerto de ssh y sin el `.git` final.

/// Un remoto con sus URLs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Remote {
    /// Nombre (`origin`).
    pub name: String,
    /// URL de fetch.
    pub fetch_url: Option<String>,
    /// URL de push, si es distinta de la de fetch.
    pub push_url: Option<String>,
}

impl Remote {
    /// Pagina web del repositorio, si la URL de fetch (o la de push) la tiene.
    #[must_use]
    pub fn web_url(&self) -> Option<String> {
        self.fetch_url
            .as_deref()
            .and_then(web_url)
            .or_else(|| self.push_url.as_deref().and_then(web_url))
    }
}

/// Host de ssh de Azure DevOps, con su forma de ruta propia (`v3/org/proyecto/repo`).
const AZURE_SSH_HOST: &str = "ssh.dev.azure.com";

/// Enlace web de una URL de git, o `None` si es una ruta local o no se reconoce.
#[must_use]
pub fn web_url(url: &str) -> Option<String> {
    let url = url.trim();
    let (scheme, host, path) = split_url(url)?;
    let host = host.to_ascii_lowercase();
    let path = path
        .trim_matches('/')
        .trim_end_matches(".git")
        .trim_end_matches('/');
    if path.is_empty() || host.is_empty() {
        return None;
    }
    if host == AZURE_SSH_HOST {
        return azure_web_url(path);
    }
    Some(format!("{scheme}://{host}/{path}"))
}

/// Esquema web, host (con puerto si es http) y ruta.
fn split_url(url: &str) -> Option<(&'static str, &str, &str)> {
    if let Some((scheme, rest)) = url.split_once("://") {
        let (authority, path) = rest.split_once('/').unwrap_or((rest, ""));
        let host_port = authority
            .rsplit_once('@')
            .map_or(authority, |(_, host)| host);
        return match scheme.to_ascii_lowercase().as_str() {
            "https" => Some(("https", host_port, path)),
            "http" => Some(("http", host_port, path)),
            "ssh" | "git" | "git+ssh" | "ssh+git" => Some(("https", without_port(host_port), path)),
            _ => None,
        };
    }
    scp_like(url).map(|(host, path)| ("https", host, path))
}

/// `usuario@host:ruta` sin esquema. No lo es una ruta de Windows (`C:\x`) ni una
/// ruta local con dos puntos despues de una barra.
fn scp_like(url: &str) -> Option<(&str, &str)> {
    let (before, path) = url.split_once(':')?;
    let odd = |text: &str| text.contains(['/', '\\']) || text.contains(char::is_whitespace);
    // `ext::cmd` y `fd::3` son transportes de git, no un host.
    if odd(before) || path.starts_with([':', '\\']) || path.contains(char::is_whitespace) {
        return None;
    }
    let host = before.rsplit_once('@').map_or(before, |(_, host)| host);
    (host.len() > 1).then_some((host, path))
}

fn without_port(host_port: &str) -> &str {
    host_port
        .rsplit_once(':')
        .filter(|(_, port)| port.bytes().all(|byte| byte.is_ascii_digit()))
        .map_or(host_port, |(host, _)| host)
}

/// `v3/org/proyecto/repo` a `https://dev.azure.com/org/proyecto/_git/repo`.
fn azure_web_url(path: &str) -> Option<String> {
    let mut parts = path.split('/');
    let (Some("v3"), Some(org), Some(project), Some(repo), None) = (
        parts.next(),
        parts.next(),
        parts.next(),
        parts.next(),
        parts.next(),
    ) else {
        return None;
    };
    Some(format!("https://dev.azure.com/{org}/{project}/_git/{repo}"))
}

#[cfg(test)]
mod tests;
