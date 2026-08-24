use anyhow::{Context, Result, anyhow};
use std::io::Write;
use std::path::Path;
use std::process::{Command, Stdio};

// Some operations related disks moved here.

pub const LUKS_PASSWORD_FILE: &str = "/run/xeonitte-luks.key";

pub fn write_file(path: &str, contents: &str) -> Result<()> {
    if let Some(parent) = Path::new(path).parent() {
        Command::new("pkexec")
            .args(["mkdir", "-p", &parent.to_string_lossy()])
            .status()
            .context("pkexec mkdir -p failed")?;
    }

    let mut child = Command::new("pkexec")
        .args(["tee", path])
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .spawn()
        .context("pkexec tee failed to spawn")?;
    child
        .stdin
        .as_mut()
        .context("Failed to open tee stdin")?
        .write_all(contents.as_bytes())?;
    if !child.wait()?.success() {
        return Err(anyhow!("pkexec tee exited non-zero for {path}"));
    }
    Ok(())
}

pub fn write_luks_key(path: &str, passphrase: &str) -> Result<()> {
    let mut child = Command::new("pkexec")
        .args(["tee", path])
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .spawn()
        .context("pkexec tee (luks key) failed to spawn")?;
    child
        .stdin
        .as_mut()
        .context("Failed to open tee stdin")?
        .write_all(passphrase.as_bytes())?;
    if !child.wait()?.success() {
        return Err(anyhow!("pkexec tee exited non-zero for luks key"));
    }
    Command::new("pkexec")
        .args(["chmod", "0100", path])
        .status()
        .context("pkexec chmod on luks key failed")?;
    Ok(())
}

pub fn unmount(tmpdir: &str) -> Result<()> {
    Command::new("pkexec")
        .args(["umount", "-R", "-f", tmpdir])
        .status()
        .context("pkexec umount failed")?;
    Ok(())
}
