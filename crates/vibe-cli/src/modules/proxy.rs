#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProxySettings {
    pub source: &'static str,
    pub url: String,
}

pub fn resolve_proxy<F>(lookup: F) -> Option<ProxySettings>
where
    F: Fn(&str) -> Option<String>,
{
    lookup("HTTPS_PROXY")
        .map(|url| ProxySettings {
            source: "HTTPS_PROXY",
            url,
        })
        .or_else(|| {
            lookup("HTTP_PROXY").map(|url| ProxySettings {
                source: "HTTP_PROXY",
                url,
            })
        })
}

pub fn current_proxy() -> Option<ProxySettings> {
    resolve_proxy(|key| std::env::var(key).ok())
}

pub fn proxy_env() -> Option<String> {
    current_proxy().map(|proxy| proxy.url)
}

#[cfg(test)]
mod tests {
    use super::resolve_proxy;

    #[test]
    fn prefers_https_proxy_over_http_proxy() {
        let proxy = resolve_proxy(|key| match key {
            "HTTPS_PROXY" => Some("https://proxy.example".into()),
            "HTTP_PROXY" => Some("http://proxy.example".into()),
            _ => None,
        })
        .unwrap();
        assert_eq!(proxy.source, "HTTPS_PROXY");
        assert_eq!(proxy.url, "https://proxy.example");
    }
}
