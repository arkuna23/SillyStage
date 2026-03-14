use std::env;
use std::io;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr};
use std::process::Command;
use std::time::Duration;

use tracing::{debug, info, warn};

use crate::config::AppConfig;

pub(crate) fn spawn_browser_if_desktop(config: &AppConfig, listen: SocketAddr) {
    if !should_auto_open_browser(config) {
        return;
    }

    let url = browser_url(listen, &config.frontend.mount_path);
    tokio::spawn(async move {
        tokio::time::sleep(Duration::from_millis(250)).await;
        let open_url = url.clone();
        match tokio::task::spawn_blocking(move || open_browser(&open_url)).await {
            Ok(Ok(())) => info!(%url, "opened browser"),
            Ok(Err(source)) => warn!(%url, %source, "failed to open browser"),
            Err(source) => warn!(%url, %source, "browser opener task failed"),
        }
    });
}

fn should_auto_open_browser(config: &AppConfig) -> bool {
    config.server.open_browser && config.frontend.enabled && is_desktop_environment()
}

fn is_desktop_environment() -> bool {
    if env::var_os("CI").is_some() {
        return false;
    }

    match env::consts::OS {
        "linux" => {
            env::var_os("DISPLAY").is_some()
                || env::var_os("WAYLAND_DISPLAY").is_some()
                || env::var_os("XDG_CURRENT_DESKTOP").is_some()
        }
        "windows" | "macos" => true,
        _ => false,
    }
}

fn browser_url(listen: SocketAddr, mount_path: &str) -> String {
    let host = browser_host(listen.ip());
    let path = if mount_path == "/" { "" } else { mount_path };
    format!("http://{host}:{}{path}", listen.port())
}

fn browser_host(ip: IpAddr) -> String {
    match ip {
        IpAddr::V4(ip) if ip.is_unspecified() => Ipv4Addr::LOCALHOST.to_string(),
        IpAddr::V6(ip) if ip.is_unspecified() => Ipv6Addr::LOCALHOST.to_string(),
        IpAddr::V4(ip) => ip.to_string(),
        IpAddr::V6(ip) => format!("[{ip}]"),
    }
}

fn open_browser(url: &str) -> io::Result<()> {
    debug!(%url, "opening browser");

    match env::consts::OS {
        "windows" => Command::new("cmd")
            .args(["/C", "start", "", url])
            .spawn()
            .map(|_| ()),
        "macos" => Command::new("open").arg(url).spawn().map(|_| ()),
        _ => Command::new("xdg-open").arg(url).spawn().map(|_| ()),
    }
}
