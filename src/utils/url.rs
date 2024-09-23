use url::Url;

#[derive(Debug)]
pub struct TcUrl {
    pub full_url: String,
    pub protocol: String,
    pub host: String,
    pub port: u16,
    pub app: String,
    pub instance: String,
}

pub fn parse_tc_url(tc_url: &str) -> std::io::Result<TcUrl> {
    let url = Url::parse(tc_url).map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidInput, e))?;

    let protocol = url.scheme().to_string();
    let host = url.host_str().ok_or(std::io::Error::new(std::io::ErrorKind::InvalidInput, "Failed to parse host"))?;
    let port = url.port().unwrap_or(1935);
    let path = url.path();
    let (app, instance) = if path.starts_with('/') {
        let mut parts = path[1..].splitn(2, '/');
        let app = parts.next().ok_or(std::io::Error::new(std::io::ErrorKind::InvalidInput, "Failed to parse app"))?;
        let instance = parts.next().unwrap_or("");
        (app.to_string(), instance.to_string())
    } else {
        (path.to_string(), "".to_string())
    };

    Ok(TcUrl {
        full_url: tc_url.to_string(),
        protocol,
        host: host.to_string(),
        port,
        app,
        instance,
    })
}