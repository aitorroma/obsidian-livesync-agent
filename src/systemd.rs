use std::fs;
use std::io;
use std::path::Path;
use std::process::Command;

use anyhow::{bail, Context, Result};

pub fn install_user_service(config_path: &Path, interval_seconds: u64) -> Result<()> {
    if !config_path.exists() {
        bail!(
            "Config file does not exist: {}. Run `livesync-agent setup` first.",
            config_path.display()
        );
    }

    let home = std::env::var("HOME").context("HOME environment variable is not set")?;
    let service_dir = Path::new(&home).join(".config/systemd/user");
    fs::create_dir_all(&service_dir)?;

    let service_file = service_dir.join("livesync-agent.service");
    let current_exe =
        std::env::current_exe().context("failed to resolve current executable path")?;

    let unit = format!(
        "[Unit]\nDescription=LiveSync Agent (user service)\nAfter=network-online.target\nWants=network-online.target\n\n[Service]\nType=simple\nExecStart={} --config {} daemon --interval-seconds {}\nRestart=always\nRestartSec=5\n\n[Install]\nWantedBy=default.target\n",
        shell_escape_path(&current_exe),
        shell_escape_path(config_path),
        interval_seconds.max(5)
    );

    fs::write(&service_file, unit)?;

    if let Err(err) = run_systemctl_user(["daemon-reload"]) {
        if is_session_bus_error(&err) {
            print_manual_enable_help(&service_file);
            return Ok(());
        }
        return Err(err);
    }

    if let Err(err) = run_systemctl_user(["enable", "--now", "livesync-agent.service"]) {
        if is_session_bus_error(&err) {
            print_manual_enable_help(&service_file);
            return Ok(());
        }
        return Err(err);
    }

    println!("Installed and started systemd user service: livesync-agent.service");
    println!("Service file: {}", service_file.display());
    println!("Check status: systemctl --user status livesync-agent.service");
    println!("Follow logs: journalctl --user -u livesync-agent.service -f");

    Ok(())
}

fn run_systemctl_user<I, S>(args: I) -> Result<()>
where
    I: IntoIterator<Item = S>,
    S: AsRef<std::ffi::OsStr>,
{
    let args_vec: Vec<std::ffi::OsString> =
        args.into_iter().map(|s| s.as_ref().to_os_string()).collect();

    let mut cmd = Command::new("systemctl");
    cmd.arg("--user").args(&args_vec);
    populate_user_bus_env(&mut cmd);

    let output = cmd
        .output()
        .context("failed to execute systemctl --user")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        let args_rendered = args_vec
            .iter()
            .map(|s| s.to_string_lossy().to_string())
            .collect::<Vec<_>>()
            .join(" ");
        if stderr.is_empty() {
            bail!(
                "systemctl --user {} failed with status: {}",
                args_rendered,
                output.status
            );
        }
        bail!(
            "systemctl --user {} failed with status: {}: {}",
            args_rendered,
            output.status,
            stderr
        );
    }

    Ok(())
}

fn populate_user_bus_env(cmd: &mut Command) {
    let xdg_runtime_dir = std::env::var("XDG_RUNTIME_DIR")
        .ok()
        .or_else(|| {
            current_uid().ok().map(|uid| format!("/run/user/{uid}")).filter(|p| {
                std::fs::metadata(p)
                    .map(|m| m.is_dir())
                    .unwrap_or(false)
            })
        });

    if let Some(dir) = xdg_runtime_dir {
        if std::env::var_os("XDG_RUNTIME_DIR").is_none() {
            cmd.env("XDG_RUNTIME_DIR", &dir);
        }
        if std::env::var_os("DBUS_SESSION_BUS_ADDRESS").is_none() {
            let bus_path = format!("{dir}/bus");
            if std::path::Path::new(&bus_path).exists() {
                cmd.env("DBUS_SESSION_BUS_ADDRESS", format!("unix:path={bus_path}"));
            }
        }
    }
}

fn current_uid() -> io::Result<String> {
    let output = Command::new("id").arg("-u").output()?;
    if !output.status.success() {
        return Err(io::Error::new(io::ErrorKind::Other, "id -u failed"));
    }
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

fn is_session_bus_error(err: &anyhow::Error) -> bool {
    let msg = format!("{err:#}");
    msg.contains("$DBUS_SESSION_BUS_ADDRESS and $XDG_RUNTIME_DIR not defined")
        || msg.contains("Failed to connect to bus")
        || msg.contains("No medium found")
}

fn print_manual_enable_help(service_file: &Path) {
    println!("Service file installed: {}", service_file.display());
    println!("Could not auto-start user service in this non-interactive shell/session.");
    println!("Run after opening a normal login session:");
    println!("  systemctl --user daemon-reload");
    println!("  systemctl --user enable --now livesync-agent.service");
    println!("If this is a server without user login sessions, enable lingering once:");
    println!("  sudo loginctl enable-linger $USER");
}

fn shell_escape_path(path: &Path) -> String {
    // Conservative quoting for spaces/special chars in ExecStart.
    let raw = path.display().to_string();
    if raw
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || "/._-".contains(c))
    {
        raw
    } else {
        format!("\"{}\"", raw.replace('"', "\\\""))
    }
}
