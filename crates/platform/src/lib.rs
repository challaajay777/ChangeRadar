use std::path::Path;
use std::process::Command;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum PlatformError {
    #[error("Failed to open file in system explorer: {0}")]
    OpenFailed(#[from] std::io::Error),
}

/// Reveal a file or folder in the platform's native file explorer.
pub fn reveal_in_explorer<P: AsRef<Path>>(path: P) -> Result<(), PlatformError> {
    let p = path.as_ref();
    #[cfg(target_os = "windows")]
    {
        if p.is_dir() {
            Command::new("explorer").arg(p).spawn()?;
        } else {
            Command::new("explorer")
                .arg(format!("/select,{}", p.display()))
                .spawn()?;
        }
    }

    #[cfg(target_os = "macos")]
    {
        Command::new("open").arg("-R").arg(p).spawn()?;
    }

    #[cfg(target_os = "linux")]
    {
        Command::new("xdg-open")
            .arg(p.parent().unwrap_or(p))
            .spawn()?;
    }

    Ok(())
}

/// Return human-readable operating system platform identity.
pub fn get_platform_info() -> String {
    format!("{} ({})", std::env::consts::OS, std::env::consts::ARCH)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_platform_info() {
        let info = get_platform_info();
        assert!(!info.is_empty());
        assert!(info.contains(std::env::consts::OS));
        assert!(info.contains(std::env::consts::ARCH));
    }
}
