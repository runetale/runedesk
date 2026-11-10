//! Queries the local runetale daemon (runetaled) over its LocalAPI.
//!
//! The daemon serves HTTP on a unix socket (macOS/Linux) or a named pipe
//! (Windows). `GET /localapi/v0/status` returns, among others,
//! `runetale_ips`: the addresses peers use to reach this machine.

use hbb_common::log;
use std::{
    io::{Read, Write},
    sync::{Arc, Mutex, Once},
    time::Duration,
};

const LOCAL_API_HOST: &str = "local-runetaled.sock";
const STATUS_PATH: &str = "/localapi/v0/status";
const IO_TIMEOUT: Duration = Duration::from_secs(2);
const REFRESH_INTERVAL: Duration = Duration::from_secs(5);

#[cfg(target_os = "macos")]
const SOCKET_PATH: &str = "/var/run/runetaled.socket";
#[cfg(all(unix, not(target_os = "macos")))]
const SOCKET_PATH: &str = "/var/run/runetale/runetaled.sock";
#[cfg(windows)]
const SOCKET_PATH: &str = r"\\.\pipe\ProtectedPrefix\Administrators\Vision\runetaled";

lazy_static::lazy_static! {
    static ref CACHED: Arc<Mutex<Snapshot>> = Default::default();
}
static START_REFRESHER: Once = Once::new();

/// Sends a GET request to the daemon and returns the response body.
fn local_api_get(path: &str) -> std::io::Result<String> {
    // HTTP/1.0 so the daemon closes the connection and sends no chunked body.
    let request = format!("GET {path} HTTP/1.0\r\nHost: {LOCAL_API_HOST}\r\n\r\n");

    #[cfg(unix)]
    let mut stream = {
        let s = std::os::unix::net::UnixStream::connect(SOCKET_PATH)?;
        s.set_read_timeout(Some(IO_TIMEOUT))?;
        s.set_write_timeout(Some(IO_TIMEOUT))?;
        s
    };
    #[cfg(windows)]
    let mut stream = std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .open(SOCKET_PATH)?;

    stream.write_all(request.as_bytes())?;
    let mut response = String::new();
    stream.read_to_string(&mut response)?;

    let (head, body) = response.split_once("\r\n\r\n").ok_or_else(|| {
        std::io::Error::new(std::io::ErrorKind::InvalidData, "malformed HTTP response")
    })?;
    let status_line = head.lines().next().unwrap_or_default();
    if status_line.split_whitespace().nth(1) != Some("200") {
        return Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("runetaled LocalAPI: {status_line}"),
        ));
    }
    Ok(body.to_owned())
}

/// A peer this node may connect to, as reported by the daemon.
#[derive(Clone, Debug, PartialEq, Eq, serde_derive::Serialize)]
pub struct Peer {
    /// Short host name (first DNS label), e.g. `my-laptop`.
    pub name: String,
    /// Runetale IPv4 address to connect to.
    pub ip: String,
    pub os: String,
    pub online: bool,
}

/// What the UI needs from one `/localapi/v0/status` call.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Snapshot {
    /// This node's runetale IPv4 addresses. Empty when not running.
    pub ips: Vec<String>,
    pub peers: Vec<Peer>,
}

/// Seconds since the last WireGuard handshake after which a peer that does
/// not report `online` is treated as offline.
const HANDSHAKE_ONLINE_SECS: i64 = 180;

fn ipv4s(v: Option<&serde_json::Value>) -> Vec<String> {
    v.and_then(|ips| ips.as_array())
        .map(|ips| {
            ips.iter()
                .filter_map(|ip| ip.as_str())
                .map(|ip| ip.split('/').next().unwrap_or(ip))
                .filter(|ip| ip.parse::<std::net::Ipv4Addr>().is_ok())
                .map(|ip| ip.to_owned())
                .collect()
        })
        .unwrap_or_default()
}

fn parse_peer(p: &serde_json::Value, now_secs: i64) -> Option<Peer> {
    // Entries without a host name are bare WireGuard keys the policy does not
    // let us reach; skip them.
    let host = p.get("host_name")?.as_str()?.trim_end_matches('.');
    let name = host.split('.').next().unwrap_or(host);
    if name.is_empty() {
        return None;
    }
    let ip = ipv4s(p.get("runetale_ips")).into_iter().next()?;
    let online = match p.get("online").and_then(|o| o.as_bool()) {
        Some(o) => o,
        None => p
            .get("last_handshake")
            .and_then(|h| h.get("seconds"))
            .and_then(|s| s.as_i64())
            .map_or(false, |s| s > 0 && now_secs - s < HANDSHAKE_ONLINE_SECS),
    };
    Some(Peer {
        name: name.to_owned(),
        ip,
        os: p.get("os").and_then(|o| o.as_str()).unwrap_or_default().to_owned(),
        online,
    })
}

/// Parses a `/localapi/v0/status` body. Peers are sorted online-first, then
/// by name.
fn parse_status(body: &str, now_secs: i64) -> Snapshot {
    let Ok(v) = serde_json::from_str::<serde_json::Value>(body) else {
        return Snapshot::default();
    };
    if v.get("backend_state").and_then(|s| s.as_str()) != Some("Running") {
        return Snapshot::default();
    }
    let mut peers: Vec<Peer> = v
        .get("peer")
        .and_then(|p| p.as_object())
        .map(|m| m.values().filter_map(|p| parse_peer(p, now_secs)).collect())
        .unwrap_or_default();
    peers.sort_by(|a, b| b.online.cmp(&a.online).then_with(|| a.name.cmp(&b.name)));
    peers.dedup_by(|a, b| a.ip == b.ip);
    Snapshot {
        ips: ipv4s(v.get("runetale_ips")),
        peers,
    }
}

fn now_secs() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| d.as_secs() as i64)
}

/// Asks the daemon for the current status.
pub fn query_snapshot() -> Snapshot {
    match local_api_get(STATUS_PATH) {
        Ok(body) => parse_status(&body, now_secs()),
        Err(e) => {
            log::debug!("runetaled LocalAPI unavailable: {e}");
            Snapshot::default()
        }
    }
}

/// Returns the last known status without blocking.
///
/// A background thread refreshes it every few seconds, so UI code can call
/// this on every redraw.
pub fn snapshot() -> Snapshot {
    START_REFRESHER.call_once(|| {
        *CACHED.lock().unwrap() = query_snapshot();
        std::thread::spawn(|| loop {
            std::thread::sleep(REFRESH_INTERVAL);
            let s = query_snapshot();
            *CACHED.lock().unwrap() = s;
        });
    });
    CACHED.lock().unwrap().clone()
}

/// Normalizes an IPv4-mapped IPv6 address (`::ffff:a.b.c.d`) to IPv4.
fn unmap(ip: std::net::IpAddr) -> std::net::IpAddr {
    match ip {
        std::net::IpAddr::V6(v6) => v6
            .to_ipv4_mapped()
            .map(std::net::IpAddr::V4)
            .unwrap_or(std::net::IpAddr::V6(v6)),
        v4 => v4,
    }
}

/// Finds the peer that owns `ip` in a `/localapi/v0/status` body.
fn whois_in_status(body: &str, ip: std::net::IpAddr) -> Option<String> {
    let v = serde_json::from_str::<serde_json::Value>(body).ok()?;
    if v.get("backend_state").and_then(|s| s.as_str()) != Some("Running") {
        return None;
    }
    let ip = unmap(ip);
    v.get("peer")?.as_object()?.values().find_map(|p| {
        // Only peers the policy lets us see carry a host name and addresses.
        let host = p.get("host_name")?.as_str()?.trim_end_matches('.');
        let owns = p
            .get("runetale_ips")?
            .as_array()?
            .iter()
            .filter_map(|a| a.as_str())
            .filter_map(|a| a.split('/').next()?.parse::<std::net::IpAddr>().ok())
            .any(|a| unmap(a) == ip);
        if !owns {
            return None;
        }
        let name = host.split('.').next().unwrap_or(host);
        Some(if name.is_empty() { ip.to_string() } else { name.to_owned() })
    })
}

/// Returns the name of the runetale peer that owns `ip`, asking the daemon
/// directly (not the UI cache). `None` if `ip` is not a peer this node can
/// see, or the daemon is not reachable.
///
/// A connection whose source is a runetale peer address has already been
/// authenticated (WireGuard binds the source address to the peer's key) and
/// authorized (the runetale ACL let the packets through).
pub fn whois(ip: std::net::IpAddr) -> Option<String> {
    match local_api_get(STATUS_PATH) {
        Ok(body) => whois_in_status(&body, ip),
        Err(e) => {
            log::warn!("runetaled LocalAPI unavailable for whois {ip}: {e}");
            None
        }
    }
}

/// This node's runetale IPv4 addresses. Empty means runetale is not running
/// or not logged in.
pub fn get_runetale_ips() -> Vec<String> {
    snapshot().ips
}

/// Peers this node can see, as JSON for the UI.
pub fn get_runetale_peers_json() -> String {
    serde_json::to_string(&snapshot().peers).unwrap_or_else(|_| "[]".to_owned())
}

#[cfg(test)]
mod tests {
    use super::*;

    const NOW: i64 = 1_800_000_000;

    #[test]
    fn parses_ipv4_from_status() {
        let body = r#"{"backend_state":"Running","runetale_ips":["100.107.0.35","fd63:9365:78fa::23"]}"#;
        assert_eq!(parse_status(body, NOW).ips, vec!["100.107.0.35".to_owned()]);
    }

    #[test]
    fn strips_prefix_length() {
        let body = r#"{"backend_state":"Running","runetale_ips":["100.107.0.35/32"]}"#;
        assert_eq!(parse_status(body, NOW).ips, vec!["100.107.0.35".to_owned()]);
    }

    #[test]
    fn empty_when_not_running() {
        let body = r#"{"backend_state":"NeedsLogin","runetale_ips":["100.107.0.35"],
            "peer":{"k":{"host_name":"a.x.","runetale_ips":["100.107.0.2/32"],"online":true}}}"#;
        assert_eq!(parse_status(body, NOW), Snapshot::default());
    }

    #[test]
    fn empty_on_bad_json() {
        assert_eq!(parse_status("not json", NOW), Snapshot::default());
    }

    #[test]
    fn whois_finds_peer() {
        let body = r#"{"backend_state":"Running","peer":{
            "k1":{"host_name":"shinta76c2.runep9anv.rune.host.","runetale_ips":["100.107.0.54/32","fd63:9365:78fa::36/128"]},
            "k2":{"public_key":"bare"}}}"#;
        let v4: std::net::IpAddr = "100.107.0.54".parse().unwrap();
        let mapped: std::net::IpAddr = "::ffff:100.107.0.54".parse().unwrap();
        let v6: std::net::IpAddr = "fd63:9365:78fa::36".parse().unwrap();
        assert_eq!(whois_in_status(body, v4).as_deref(), Some("shinta76c2"));
        assert_eq!(whois_in_status(body, mapped).as_deref(), Some("shinta76c2"));
        assert_eq!(whois_in_status(body, v6).as_deref(), Some("shinta76c2"));
        let lan: std::net::IpAddr = "10.211.55.4".parse().unwrap();
        assert_eq!(whois_in_status(body, lan), None);
    }

    #[test]
    fn whois_none_when_not_running() {
        let body = r#"{"backend_state":"Stopped","peer":{"k1":{"host_name":"a.x.","runetale_ips":["100.107.0.54/32"]}}}"#;
        assert_eq!(whois_in_status(body, "100.107.0.54".parse().unwrap()), None);
    }

    #[test]
    fn parses_peers() {
        let body = format!(
            r#"{{"backend_state":"Running","runetale_ips":["100.107.0.35"],"peer":{{
            "k1":{{"host_name":"shinta76c2.runep9anv.rune.host.","os":"windows",
                   "runetale_ips":["100.107.0.54/32","fd63:9365:78fa::36/128"],
                   "last_handshake":{{"seconds":{recent}}}}},
            "k2":{{"host_name":"dev.runep9anv.rune.host.","os":"linux",
                   "runetale_ips":["100.107.0.1/32"],"online":false}},
            "k3":{{"host_name":"old.runep9anv.rune.host.","os":"linux",
                   "runetale_ips":["100.107.0.9/32"],"last_handshake":{{"seconds":{old}}}}},
            "k4":{{"public_key":"bare-key-only"}},
            "k5":{{"host_name":"v6only.x.","runetale_ips":["fd63:9365:78fa::40/128"]}}
            }}}}"#,
            recent = NOW - 10,
            old = NOW - 3600,
        );
        let peers = parse_status(&body, NOW).peers;
        assert_eq!(
            peers,
            vec![
                Peer { name: "shinta76c2".into(), ip: "100.107.0.54".into(), os: "windows".into(), online: true },
                Peer { name: "dev".into(), ip: "100.107.0.1".into(), os: "linux".into(), online: false },
                Peer { name: "old".into(), ip: "100.107.0.9".into(), os: "linux".into(), online: false },
            ]
        );
    }
}
