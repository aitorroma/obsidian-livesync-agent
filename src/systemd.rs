use std::fs;
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

    run_systemctl_user(["daemon-reload"])?;
    run_systemctl_user(["enable", "--now", "livesync-agent.service"])?;

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
    let status = Command::new("systemctl")
        .arg("--user")
        .args(args)
        .status()
        .context("failed to execute systemctl --user")?;

    if !status.success() {
        bail!("systemctl --user command failed with status: {status}");
    }

    Ok(())
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
