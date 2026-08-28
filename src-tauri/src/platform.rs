use crate::error::{Error, Result};

/// Desktop targets the launcher ships installers for. Linux is intentionally
/// absent: upstream RustFS publishes gnu/musl variants that need a per-distro
/// decision the launcher cannot make, and no Linux bundle is released.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Platform {
    MacOsAarch64,
    MacOsX86_64,
    WindowsX86_64,
}

impl Platform {
    pub fn detect() -> Result<Self> {
        Self::from_target(std::env::consts::OS, std::env::consts::ARCH)
    }

    pub fn from_target(os: &str, arch: &str) -> Result<Self> {
        match (os, arch) {
            ("macos", "aarch64") => Ok(Self::MacOsAarch64),
            ("macos", "x86_64") => Ok(Self::MacOsX86_64),
            // Windows on ARM runs the x86_64 build under emulation because
            // upstream does not publish a native aarch64 Windows binary.
            ("windows", "x86_64") | ("windows", "aarch64") => Ok(Self::WindowsX86_64),
            _ => Err(Error::UnsupportedPlatform(format!("{os}-{arch}"))),
        }
    }

    /// File name of the RustFS binary as staged by the build scripts.
    pub fn binary_name(self) -> &'static str {
        match self {
            Self::MacOsAarch64 => "rustfs-macos-aarch64",
            Self::MacOsX86_64 => "rustfs-macos-x86_64",
            Self::WindowsX86_64 => "rustfs-windows-x86_64.exe",
        }
    }

    /// Prefix of the upstream release archive, without the version suffix.
    pub fn asset_slug(self) -> &'static str {
        match self {
            Self::MacOsAarch64 => "rustfs-macos-aarch64",
            Self::MacOsX86_64 => "rustfs-macos-x86_64",
            Self::WindowsX86_64 => "rustfs-windows-x86_64",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::MacOsAarch64 => "macOS (Apple Silicon)",
            Self::MacOsX86_64 => "macOS (Intel)",
            Self::WindowsX86_64 => "Windows (x86_64)",
        }
    }
}

pub fn current_label() -> String {
    Platform::detect()
        .map(|platform| platform.label().to_string())
        .unwrap_or_else(|_| {
            format!(
                "{}-{} (unsupported)",
                std::env::consts::OS,
                std::env::consts::ARCH
            )
        })
}

#[cfg(test)]
mod tests {
    use super::Platform;
    use crate::error::Error;

    #[test]
    fn maps_supported_desktop_targets() {
        assert_eq!(
            Platform::from_target("macos", "aarch64")
                .unwrap()
                .binary_name(),
            "rustfs-macos-aarch64"
        );
        assert_eq!(
            Platform::from_target("macos", "x86_64")
                .unwrap()
                .binary_name(),
            "rustfs-macos-x86_64"
        );
        assert_eq!(
            Platform::from_target("windows", "x86_64")
                .unwrap()
                .binary_name(),
            "rustfs-windows-x86_64.exe"
        );
        assert_eq!(
            Platform::from_target("windows", "aarch64")
                .unwrap()
                .binary_name(),
            "rustfs-windows-x86_64.exe"
        );
    }

    #[test]
    fn rejects_unsupported_targets_with_the_target_triple() {
        let error = Platform::from_target("linux", "x86_64").unwrap_err();
        assert!(matches!(error, Error::UnsupportedPlatform(_)));
        assert!(error.to_string().contains("linux-x86_64"));

        assert!(Platform::from_target("macos", "riscv64").is_err());
    }

    #[test]
    fn asset_slug_drops_the_windows_extension() {
        assert_eq!(
            Platform::from_target("windows", "x86_64")
                .unwrap()
                .asset_slug(),
            "rustfs-windows-x86_64"
        );
        assert_eq!(
            Platform::from_target("macos", "aarch64")
                .unwrap()
                .asset_slug(),
            "rustfs-macos-aarch64"
        );
    }
}
