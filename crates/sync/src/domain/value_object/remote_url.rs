#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RemoteUrl {
    original: String,
    network: String,
}

#[derive(Debug, PartialEq, Eq, thiserror::Error)]
#[error("unsupported remote url `{0}`")]
pub struct UnsupportedRemoteUrlError(pub String);

impl RemoteUrl {
    pub fn parse(raw: &str) -> Result<Self, UnsupportedRemoteUrlError> {
        network_url_of(raw)
            .map(|network| Self {
                original: raw.to_string(),
                network,
            })
            .ok_or_else(|| UnsupportedRemoteUrlError(raw.to_string()))
    }

    pub fn as_str(&self) -> &str {
        &self.original
    }

    pub fn network_url(&self) -> &str {
        &self.network
    }

    pub fn repository_name(&self) -> String {
        let last_segment = self
            .network
            .trim_end_matches('/')
            .rsplit('/')
            .next()
            .unwrap_or_default();
        last_segment
            .strip_suffix(".git")
            .unwrap_or(last_segment)
            .to_string()
    }
}

fn network_url_of(raw: &str) -> Option<String> {
    if raw.is_empty() || raw.chars().any(char::is_whitespace) {
        return None;
    }
    if let Some(rest) = raw.strip_prefix("https://").or(raw.strip_prefix("http://")) {
        return split_host_and_path(rest).map(|_| raw.to_string());
    }
    if let Some(rest) = raw.strip_prefix("file://") {
        return (!rest.is_empty()).then(|| raw.to_string());
    }
    if let Some(rest) = raw.strip_prefix("ssh://") {
        let (authority, path) = split_host_and_path(rest)?;
        return https_url(host_of(authority), path);
    }
    if raw.contains("://") {
        return None;
    }
    let (authority, path) = raw.split_once(':')?;
    https_url(without_user(authority), path)
}

fn split_host_and_path(rest: &str) -> Option<(&str, &str)> {
    rest.split_once('/')
        .filter(|(host, path)| !host.is_empty() && !path.is_empty())
}

fn host_of(authority: &str) -> &str {
    let host_and_port = without_user(authority);
    host_and_port.split(':').next().unwrap_or(host_and_port)
}

fn without_user(authority: &str) -> &str {
    authority
        .split_once('@')
        .map_or(authority, |(_, host)| host)
}

fn https_url(host: &str, path: &str) -> Option<String> {
    (!host.is_empty() && !path.is_empty()).then(|| format!("https://{host}/{path}"))
}
