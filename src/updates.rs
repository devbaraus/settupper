use anyhow::{Context, Result};
use semver::Version;
use serde::Deserialize;
use std::time::Duration;

pub const RELEASES_URL: &str = "https://github.com/devbaraus/settupper/releases";
pub const LATEST_RELEASE_API: &str =
    "https://api.github.com/repos/devbaraus/settupper/releases/latest";

#[derive(Deserialize)]
struct Release {
    tag_name: String,
}

pub fn check_latest(url: &str, current: &str) -> Result<Option<Version>> {
    let response = ureq::get(url)
        .set("User-Agent", concat!("settupper/", env!("CARGO_PKG_VERSION")))
        .set("Accept", "application/vnd.github+json")
        .timeout(Duration::from_secs(5))
        .call()
        .context("Failed to fetch latest release")?;
    let release: Release = serde_json::from_reader(response.into_reader())
        .context("Invalid release response")?;
    let latest = Version::parse(release.tag_name.strip_prefix('v').unwrap_or(&release.tag_name))
        .context("Invalid release version")?;
    let current = Version::parse(current).context("Invalid current version")?;
    Ok(latest.cmp_precedence(&current).is_gt().then_some(latest))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{Read, Write};
    use std::net::TcpListener;

    fn check_response(status: &str, body: &str, current: &str) -> Result<Option<Version>> {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let url = format!("http://{}/latest", listener.local_addr().unwrap());
        let response = format!(
            "HTTP/1.1 {status}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
            body.len()
        );
        let server = std::thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();
            stream.set_read_timeout(Some(Duration::from_secs(5))).unwrap();
            let mut request = Vec::new();
            let mut byte = [0];
            while !request.ends_with(b"\r\n\r\n") {
                stream.read_exact(&mut byte).unwrap();
                request.push(byte[0]);
            }
            let request = String::from_utf8(request).unwrap().to_lowercase();
            assert!(request.starts_with("get /latest http/1.1"));
            assert!(request.contains("user-agent: settupper/"));
            assert!(request.contains("accept: application/vnd.github+json"));
            stream.write_all(response.as_bytes()).unwrap();
        });
        let result = check_latest(&url, current);
        server.join().unwrap();
        result
    }

    #[test]
    fn compares_release_versions_by_semver_precedence() {
        for (tag, current, newer) in [
            ("v0.1.10", "0.1.6", true),
            ("1.0.0", "0.9.0", true),
            ("v0.1.6", "0.1.6", false),
            ("v0.1.5", "0.1.6", false),
            ("v0.1.6", "0.1.6-rc.1", true),
            ("v0.1.6-rc.1", "0.1.6", false),
            ("v0.1.6+build.2", "0.1.6+build.1", false),
        ] {
            let body = format!(r#"{{"tag_name":"{tag}"}}"#);
            let result = check_response("200 OK", &body, current).unwrap();
            assert_eq!(result.is_some(), newer, "{tag} vs {current}");
        }
    }

    #[test]
    fn rejects_failed_or_invalid_release_responses() {
        for (status, body) in [
            ("403 Forbidden", r#"{"message":"rate limited"}"#),
            ("200 OK", "not json"),
            ("200 OK", "{}"),
            ("200 OK", r#"{"tag_name":"invalid"}"#),
        ] {
            assert!(check_response(status, body, "0.1.6").is_err());
        }
    }
}
