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
    should_auto_open_browser_with(config, is_dev_mode(), is_desktop_environment())
}

fn should_auto_open_browser_with(
    config: &AppConfig,
    dev_mode: bool,
    desktop_environment: bool,
) -> bool {
    config.server.open_browser && config.frontend.enabled && desktop_environment && !dev_mode
}

fn is_dev_mode() -> bool {
    matches!(
        env::var("SS_APP_DEV_MODE").ok().as_deref(),
        Some("1" | "true" | "TRUE" | "True" | "yes" | "YES" | "on" | "ON")
    )
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

#[cfg(test)]
mod tests {
    use super::should_auto_open_browser_with;
    use crate::config::{AppConfig, FrontendConfig, ServerConfig, StoreBackend, StoreConfig};
    use std::path::PathBuf;

    fn sample_config() -> AppConfig {
        AppConfig {
            server: ServerConfig {
                listen: "127.0.0.1:8080".to_owned(),
                open_browser: true,
            },
            store: StoreConfig {
                backend: StoreBackend::Memory,
                root: PathBuf::from("./unused"),
            },
            frontend: FrontendConfig {
                enabled: true,
                mount_path: "/".to_owned(),
                static_dir: None,
            },
        }
    }

    #[test]
    fn auto_open_is_disabled_in_dev_mode() {
        assert!(!should_auto_open_browser_with(&sample_config(), true, true));
    }

    #[test]
    fn auto_open_requires_desktop_and_frontend() {
        let config = sample_config();
        assert!(should_auto_open_browser_with(&config, false, true));
        assert!(!should_auto_open_browser_with(&config, false, false));

        let mut frontend_disabled = sample_config();
        frontend_disabled.frontend.enabled = false;
        assert!(!should_auto_open_browser_with(
            &frontend_disabled,
            false,
            true
        ));
    }
}
