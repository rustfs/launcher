//! Mach-O load commands for the RustFS binary the launcher ships and updates.
//!
//! The upstream Apple silicon build records
//! `LC_LOAD_DYLIB /opt/homebrew/opt/xz/lib/liblzma.5.dylib` (compatibility
//! version 14.0.0, current version 14.3.0). dyld aborts with "Library missing"
//! when that absolute file is absent. macOS DiagnosticReports often render the
//! same command as `/opt/homebrew/*/liblzma.5.dylib`; the asterisk is crash
//! reporter redaction, not the bytes in the file.
//!
//! `liblzma.5.dylib` is rewritten to `@loader_path/liblzma.5.dylib` and loaded
//! from a library shipped beside the binary. Any other non-system, non-relative
//! load command is rejected.

use crate::error::{Error, Result};
use std::path::Path;

pub(crate) const LIBLZMA_FILE: &str = "liblzma.5.dylib";
pub(crate) const LIBLZMA_LOAD: &str = "@loader_path/liblzma.5.dylib";

const MH_MAGIC_64: u32 = 0xFEED_FACF;
const FAT_MAGIC: u32 = 0xCAFE_BABE;
const FAT_CIGAM: u32 = 0xBEBA_FECA;

const LC_LOAD_DYLIB: u32 = 0xC;
const LC_ID_DYLIB: u32 = 0xD;
const LC_PREBOUND_DYLIB: u32 = 0x10;
const LC_LOAD_WEAK_DYLIB: u32 = 0x8000_0018;
const LC_RPATH: u32 = 0x8000_001C;
const LC_REEXPORT_DYLIB: u32 = 0x8000_001F;
const LC_LAZY_LOAD_DYLIB: u32 = 0x20;
const LC_LOAD_UPWARD_DYLIB: u32 = 0x8000_0023;

#[derive(Debug, Clone, PartialEq, Eq)]
struct Load {
    command: &'static str,
    cmd: u32,
    path: String,
    compatibility: u32,
    current: u32,
    name_at: usize,
    name_capacity: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Plan {
    /// Every `liblzma.5.dylib` dependency, including one that is already
    /// `@loader_path/liblzma.5.dylib`. The sibling file still has to exist.
    needs_liblzma: bool,
    /// `liblzma.5.dylib` commands whose path is not the vendored loader path.
    pub liblzma: Vec<String>,
    pub blocked: Vec<String>,
    required_compat: u32,
}

impl Plan {
    pub(crate) fn needs_liblzma(&self) -> bool {
        self.needs_liblzma
    }

    fn required_compat(&self) -> u32 {
        self.required_compat
    }
}

pub(crate) fn inspect(bytes: &[u8]) -> Result<Option<Plan>> {
    if !looks_like_macho(bytes) {
        return Ok(None);
    }
    let loads = parse_loads(bytes).map_err(Error::RustFsUpdate)?;
    let mut liblzma = Vec::new();
    let mut blocked = Vec::new();
    let mut needs_liblzma = false;
    let mut required_compat = 0_u32;
    for load in &loads {
        if is_dependency(load.cmd) && load.cmd != LC_RPATH && file_name(&load.path) == LIBLZMA_FILE
        {
            needs_liblzma = true;
            required_compat = required_compat.max(load.compatibility);
            if load.path != LIBLZMA_LOAD {
                liblzma.push(describe(load));
            }
            continue;
        }
        if is_dependency(load.cmd) && !is_relocatable(&load.path) {
            blocked.push(describe(load));
        }
    }
    Ok(Some(Plan {
        needs_liblzma,
        liblzma,
        blocked,
        required_compat,
    }))
}

pub(crate) fn blocked_message(plan: &Plan) -> String {
    let mut lines = vec![
        "Refusing to use this RustFS binary because its Mach-O load commands are not relocatable."
            .to_string(),
    ];
    lines.extend(plan.blocked.iter().cloned());
    lines.extend(plan.liblzma.iter().cloned());
    lines.push(
        "dyld aborts at launch with \"Library missing\" unless that exact file exists. DiagnosticReports may print a middle path component as '*'; the command above is the path in the file."
            .to_string(),
    );
    lines.join("\n")
}

fn missing_liblzma_message(plan: &Plan) -> String {
    let mut lines = vec![
        "Refusing to use this RustFS binary because it loads liblzma.5.dylib from a path that is not shipped inside the launcher.".to_string(),
    ];
    lines.extend(plan.liblzma.iter().cloned());
    lines.push(
        "The launcher has no vendored liblzma.5.dylib to place beside the binary, so it cannot start on a Mac without Homebrew.".to_string(),
    );
    lines.join("\n")
}

pub(crate) fn vendor_liblzma(binary: &Path, vendored: Option<&Path>, plan: &Plan) -> Result<()> {
    let Some(source) = vendored.filter(|path| path.is_file()) else {
        return Err(Error::RustFsUpdate(missing_liblzma_message(plan)));
    };
    if !plan.blocked.is_empty() {
        return Err(Error::RustFsUpdate(blocked_message(plan)));
    }

    let mut dylib = std::fs::read(source).map_err(|error| {
        Error::RustFsUpdate(format!(
            "Could not read vendored {LIBLZMA_FILE} at {}: {error}",
            source.display()
        ))
    })?;
    let id_changed = rewrite_id(&mut dylib).map_err(Error::RustFsUpdate)?;
    ensure_vendored(&dylib, plan.required_compat()).map_err(Error::RustFsUpdate)?;

    let dest = binary
        .parent()
        .ok_or_else(|| Error::RustFsUpdate("RustFS binary has no parent directory".to_string()))?
        .join(LIBLZMA_FILE);
    if dest != source || id_changed {
        std::fs::write(&dest, &dylib).map_err(Error::Io)?;
        set_executable(&dest)?;
    }
    if id_changed {
        adhoc_sign(&dest)?;
    }

    let mut image = std::fs::read(binary).map_err(Error::Io)?;
    let rewritten = rewrite_liblzma_loads(&mut image).map_err(Error::RustFsUpdate)?;
    if rewritten > 0 {
        std::fs::write(binary, &image).map_err(Error::Io)?;
        set_executable(binary)?;
        adhoc_sign(binary)?;
    }
    Ok(())
}

fn ensure_vendored(dylib: &[u8], required_compat: u32) -> std::result::Result<(), String> {
    let loads = parse_loads(dylib)?;
    let ident = loads
        .iter()
        .find(|load| load.cmd == LC_ID_DYLIB)
        .ok_or_else(|| format!("vendored {LIBLZMA_FILE} has no LC_ID_DYLIB"))?;
    if ident.path != LIBLZMA_LOAD {
        return Err(format!(
            "vendored {LIBLZMA_FILE} install name is {}, expected {LIBLZMA_LOAD}",
            ident.path
        ));
    }
    if ident.compatibility < required_compat {
        return Err(format!(
            "vendored {LIBLZMA_FILE} compatibility version {} is older than the {} required by RustFS",
            format_version(ident.compatibility),
            format_version(required_compat)
        ));
    }
    let bad: Vec<_> = loads
        .iter()
        .filter(|load| is_dependency(load.cmd) && !is_relocatable(&load.path))
        .map(describe)
        .collect();
    if !bad.is_empty() {
        return Err(format!(
            "vendored {LIBLZMA_FILE} is itself not relocatable:\n{}",
            bad.join("\n")
        ));
    }
    Ok(())
}

fn looks_like_macho(bytes: &[u8]) -> bool {
    if bytes.len() < 4 {
        return false;
    }
    let le = u32_at(bytes, 0, true).unwrap_or(0);
    let be = u32_at(bytes, 0, false).unwrap_or(0);
    le == MH_MAGIC_64
        || be == FAT_MAGIC
        || be == FAT_CIGAM
        || matches!(le, 0xFEED_FACE | 0xCEFA_EDFE | 0xCFFA_EDFE)
}

fn is_relocatable(path: &str) -> bool {
    path.starts_with("@executable_path/")
        || path.starts_with("@loader_path/")
        || path.starts_with("@rpath/")
        || path.starts_with("/usr/lib/")
        || path.starts_with("/System/")
}

fn is_dependency(cmd: u32) -> bool {
    matches!(
        cmd,
        LC_LOAD_DYLIB
            | LC_LOAD_WEAK_DYLIB
            | LC_REEXPORT_DYLIB
            | LC_LOAD_UPWARD_DYLIB
            | LC_LAZY_LOAD_DYLIB
            | LC_PREBOUND_DYLIB
            | LC_RPATH
    )
}

fn file_name(path: &str) -> &str {
    path.rsplit('/').next().unwrap_or(path)
}

fn command_name(cmd: u32) -> Option<&'static str> {
    Some(match cmd {
        LC_LOAD_DYLIB => "LC_LOAD_DYLIB",
        LC_ID_DYLIB => "LC_ID_DYLIB",
        LC_PREBOUND_DYLIB => "LC_PREBOUND_DYLIB",
        LC_LOAD_WEAK_DYLIB => "LC_LOAD_WEAK_DYLIB",
        LC_RPATH => "LC_RPATH",
        LC_REEXPORT_DYLIB => "LC_REEXPORT_DYLIB",
        LC_LAZY_LOAD_DYLIB => "LC_LAZY_LOAD_DYLIB",
        LC_LOAD_UPWARD_DYLIB => "LC_LOAD_UPWARD_DYLIB",
        _ => return None,
    })
}

fn format_version(packed: u32) -> String {
    format!(
        "{}.{}.{}",
        packed >> 16,
        (packed >> 8) & 0xff,
        packed & 0xff
    )
}

fn describe(load: &Load) -> String {
    format!(
        "{} {} (compatibility version {}, current version {})",
        load.command,
        load.path,
        format_version(load.compatibility),
        format_version(load.current)
    )
}

fn u32_at(bytes: &[u8], offset: usize, little: bool) -> std::result::Result<u32, String> {
    let end = offset
        .checked_add(4)
        .filter(|end| *end <= bytes.len())
        .ok_or_else(|| format!("Mach-O truncated at {offset}"))?;
    let raw: [u8; 4] = bytes[offset..end]
        .try_into()
        .map_err(|_| format!("Mach-O truncated at {offset}"))?;
    Ok(if little {
        u32::from_le_bytes(raw)
    } else {
        u32::from_be_bytes(raw)
    })
}

fn slices(bytes: &[u8]) -> std::result::Result<Vec<(usize, usize)>, String> {
    if bytes.len() < 8 {
        return Err("not a Mach-O file".to_string());
    }
    let magic_le = u32_at(bytes, 0, true)?;
    let magic_be = u32_at(bytes, 0, false)?;
    if magic_be == FAT_MAGIC {
        let count = u32_at(bytes, 4, false)? as usize;
        let mut slices = Vec::with_capacity(count);
        let mut cursor = 8_usize;
        for _ in 0..count {
            if cursor + 20 > bytes.len() {
                return Err("fat header truncated".to_string());
            }
            let offset = u32_at(bytes, cursor + 8, false)? as usize;
            let size = u32_at(bytes, cursor + 12, false)? as usize;
            slices.push((offset, size));
            cursor += 20;
        }
        return Ok(slices);
    }
    if magic_be == FAT_CIGAM {
        return Err("little-endian fat Mach-O is not supported".to_string());
    }
    if magic_le == MH_MAGIC_64 {
        return Ok(vec![(0, bytes.len())]);
    }
    Err(
        "unsupported Mach-O encoding; refusing to bundle it without reading its load commands"
            .to_string(),
    )
}

fn parse_slice(bytes: &[u8], base: usize, size: usize) -> std::result::Result<Vec<Load>, String> {
    if size < 32 || base + 32 > bytes.len() {
        return Err("Mach-O slice truncated".to_string());
    }
    if u32_at(bytes, base, true)? != MH_MAGIC_64 {
        return Err("Mach-O slice is not 64-bit little-endian".to_string());
    }
    let ncmds = u32_at(bytes, base + 16, true)? as usize;
    let sizeofcmds = u32_at(bytes, base + 20, true)? as usize;
    if sizeofcmds > size - 32 || base + 32 + sizeofcmds > bytes.len() {
        return Err("Mach-O load commands extend past the slice".to_string());
    }
    let mut loads = Vec::new();
    let mut cursor = base + 32;
    let end = cursor + sizeofcmds;
    for _ in 0..ncmds {
        if cursor + 8 > end {
            return Err("Mach-O load command truncated".to_string());
        }
        let cmd = u32_at(bytes, cursor, true)?;
        let cmdsize = u32_at(bytes, cursor + 4, true)? as usize;
        if cmdsize < 8 || cursor + cmdsize > end {
            return Err(format!("Mach-O load command size {cmdsize} is invalid"));
        }
        if let Some(command) = command_name(cmd) {
            let name_off = u32_at(bytes, cursor + 8, true)? as usize;
            if name_off < 8 || name_off >= cmdsize {
                return Err("Mach-O dylib name offset is outside the command".to_string());
            }
            let name_at = cursor + name_off;
            let raw = &bytes[name_at..cursor + cmdsize];
            let nul = raw
                .iter()
                .position(|byte| *byte == 0)
                .ok_or_else(|| "Mach-O dylib path is not NUL-terminated".to_string())?;
            let path = std::str::from_utf8(&raw[..nul])
                .map_err(|_| "Mach-O dylib path is not UTF-8".to_string())?
                .to_string();
            let versioned = matches!(
                cmd,
                LC_LOAD_DYLIB
                    | LC_ID_DYLIB
                    | LC_LOAD_WEAK_DYLIB
                    | LC_REEXPORT_DYLIB
                    | LC_LOAD_UPWARD_DYLIB
                    | LC_LAZY_LOAD_DYLIB
            );
            let (current, compatibility) = if versioned {
                (
                    u32_at(bytes, cursor + 16, true)?,
                    u32_at(bytes, cursor + 20, true)?,
                )
            } else {
                (0, 0)
            };
            loads.push(Load {
                command,
                cmd,
                path,
                compatibility,
                current,
                name_at,
                name_capacity: cmdsize - name_off,
            });
        }
        cursor += cmdsize;
    }
    Ok(loads)
}

fn parse_loads(bytes: &[u8]) -> std::result::Result<Vec<Load>, String> {
    let mut loads = Vec::new();
    for (offset, size) in slices(bytes)? {
        loads.extend(parse_slice(bytes, offset, size)?);
    }
    Ok(loads)
}

fn rewrite_bytes(data: &mut [u8], load: &Load, new_path: &str) -> std::result::Result<(), String> {
    let encoded = new_path.as_bytes();
    if encoded.len() + 1 > load.name_capacity {
        return Err(format!(
            "cannot replace {} with {new_path}: the load command has room for {} bytes",
            load.path,
            load.name_capacity - 1
        ));
    }
    data[load.name_at..load.name_at + load.name_capacity].fill(0);
    data[load.name_at..load.name_at + encoded.len()].copy_from_slice(encoded);
    Ok(())
}

fn rewrite_liblzma_loads(data: &mut [u8]) -> std::result::Result<usize, String> {
    let loads = parse_loads(data)?;
    let targets: Vec<_> = loads
        .into_iter()
        .filter(|load| {
            is_dependency(load.cmd)
                && load.cmd != LC_RPATH
                && file_name(&load.path) == LIBLZMA_FILE
                && load.path != LIBLZMA_LOAD
        })
        .collect();
    for load in &targets {
        rewrite_bytes(data, load, LIBLZMA_LOAD)?;
    }
    Ok(targets.len())
}

fn rewrite_id(data: &mut [u8]) -> std::result::Result<bool, String> {
    let loads = parse_loads(data)?;
    let Some(ident) = loads.into_iter().find(|load| load.cmd == LC_ID_DYLIB) else {
        return Err(format!("vendored {LIBLZMA_FILE} has no LC_ID_DYLIB"));
    };
    if ident.path == LIBLZMA_LOAD {
        return Ok(false);
    }
    rewrite_bytes(data, &ident, LIBLZMA_LOAD)?;
    Ok(true)
}

fn set_executable(path: &Path) -> Result<()> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut permissions = std::fs::metadata(path).map_err(Error::Io)?.permissions();
        permissions.set_mode(permissions.mode() | 0o755);
        std::fs::set_permissions(path, permissions).map_err(Error::Io)?;
    }
    #[cfg(not(unix))]
    {
        let _ = path;
    }
    Ok(())
}

fn adhoc_sign(path: &Path) -> Result<()> {
    #[cfg(target_os = "macos")]
    {
        let output = std::process::Command::new("/usr/bin/codesign")
            .arg("--force")
            .arg("--sign")
            .arg("-")
            .arg(path)
            .output()
            .map_err(|error| {
                Error::RustFsUpdate(format!("Could not codesign {}: {error}", path.display()))
            })?;
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(Error::RustFsUpdate(format!(
                "codesign failed for {}: {stderr}",
                path.display()
            )));
        }
    }
    #[cfg(not(target_os = "macos"))]
    {
        let _ = path;
    }
    Ok(())
}

#[cfg(test)]
pub(crate) fn fixture(paths: &[&str], ident: Option<&str>, compat: u32, current: u32) -> Vec<u8> {
    let mut commands: Vec<(u32, String)> = Vec::new();
    if let Some(ident) = ident {
        commands.push((LC_ID_DYLIB, ident.to_string()));
    }
    for path in paths {
        commands.push((LC_LOAD_DYLIB, (*path).to_string()));
    }
    let mut encoded = Vec::new();
    for (cmd, path) in &commands {
        let raw = path.as_bytes();
        let mut cmdsize = 24 + raw.len() + 1;
        cmdsize = (cmdsize + 7) & !7;
        let mut blob = vec![0_u8; cmdsize];
        blob[0..4].copy_from_slice(&cmd.to_le_bytes());
        blob[4..8].copy_from_slice(&(cmdsize as u32).to_le_bytes());
        blob[8..12].copy_from_slice(&24_u32.to_le_bytes());
        blob[12..16].copy_from_slice(&2_u32.to_le_bytes());
        blob[16..20].copy_from_slice(&current.to_le_bytes());
        blob[20..24].copy_from_slice(&compat.to_le_bytes());
        blob[24..24 + raw.len()].copy_from_slice(raw);
        encoded.extend(blob);
    }
    let mut header = Vec::with_capacity(32);
    header.extend_from_slice(&MH_MAGIC_64.to_le_bytes());
    header.extend_from_slice(&0x0100_000C_u32.to_le_bytes());
    header.extend_from_slice(&0_u32.to_le_bytes());
    header.extend_from_slice(&2_u32.to_le_bytes());
    header.extend_from_slice(&(commands.len() as u32).to_le_bytes());
    header.extend_from_slice(&(encoded.len() as u32).to_le_bytes());
    header.extend_from_slice(&0x0020_0085_u32.to_le_bytes());
    header.extend_from_slice(&0_u32.to_le_bytes());
    header.extend(encoded);
    header
}

#[cfg(test)]
mod tests {
    use super::*;

    const HOMEBREW: &str = "/opt/homebrew/opt/xz/lib/liblzma.5.dylib";
    const COMPAT_14: u32 = 14 << 16;
    const CURRENT_14_3: u32 = (14 << 16) | (3 << 8);

    #[test]
    fn system_load_commands_are_relocatable() {
        let bytes = fixture(
            &[
                "/usr/lib/libSystem.B.dylib",
                "/System/Library/Frameworks/Foundation.framework/Foundation",
            ],
            None,
            1 << 16,
            1 << 16,
        );
        let plan = inspect(&bytes).unwrap().unwrap();
        assert!(!plan.needs_liblzma());
        assert!(plan.blocked.is_empty());
    }

    #[test]
    fn homebrew_liblzma_is_rewritten_to_the_vendored_path() {
        let mut bytes = fixture(
            &[HOMEBREW, "/usr/lib/libSystem.B.dylib"],
            None,
            COMPAT_14,
            CURRENT_14_3,
        );
        let plan = inspect(&bytes).unwrap().unwrap();
        assert!(plan.needs_liblzma());
        assert!(plan.blocked.is_empty(), "{:?}", plan.blocked);
        assert!(plan.liblzma.iter().any(|line| line.contains(HOMEBREW)));
        assert!(plan.liblzma.iter().any(|line| line.contains("14.0.0")));
        assert!(plan.liblzma.iter().any(|line| line.contains("14.3.0")));

        assert_eq!(rewrite_liblzma_loads(&mut bytes).unwrap(), 1);
        let rewritten = inspect(&bytes).unwrap().unwrap();
        assert!(rewritten.needs_liblzma());
        assert!(rewritten.liblzma.is_empty());
        assert!(rewritten.blocked.is_empty());
        let loads = parse_loads(&bytes).unwrap();
        assert!(loads.iter().any(|load| load.path == LIBLZMA_LOAD));
    }

    #[test]
    fn a_literal_asterisk_liblzma_install_name_is_rewritten() {
        let starred = "/opt/homebrew/*/liblzma.5.dylib";
        let mut bytes = fixture(&[starred], None, COMPAT_14, CURRENT_14_3);
        let plan = inspect(&bytes).unwrap().unwrap();
        assert!(plan.liblzma.iter().any(|line| line.contains(starred)));
        rewrite_liblzma_loads(&mut bytes).unwrap();
        assert!(parse_loads(&bytes)
            .unwrap()
            .iter()
            .any(|load| load.path == LIBLZMA_LOAD));
    }

    #[test]
    fn other_non_relative_paths_fail_the_guard_and_quote_the_command() {
        let starred = "/opt/homebrew/*/libfoo.1.dylib";
        let openssl = "/usr/local/opt/openssl@3/lib/libssl.3.dylib";
        let bytes = fixture(&[HOMEBREW, openssl, starred], None, COMPAT_14, CURRENT_14_3);
        let plan = inspect(&bytes).unwrap().unwrap();
        let message = blocked_message(&plan);
        assert!(message.contains(openssl), "{message}");
        assert!(message.contains(starred), "{message}");
        assert!(message.contains(HOMEBREW), "{message}");
        assert!(message.contains("not relocatable"), "{message}");
    }

    #[test]
    #[cfg(not(target_os = "macos"))]
    fn vendored_liblzma_is_copied_beside_the_binary() {
        let dir = tempfile::tempdir().unwrap();
        let binary = dir.path().join("rustfs-macos-aarch64");
        std::fs::write(&binary, fixture(&[HOMEBREW], None, COMPAT_14, CURRENT_14_3)).unwrap();
        let dylib_path = dir.path().join("source-liblzma.5.dylib");
        std::fs::write(
            &dylib_path,
            fixture(
                &["/usr/lib/libSystem.B.dylib"],
                Some("/usr/local/lib/liblzma.5.dylib"),
                COMPAT_14,
                (14 << 16) | (4 << 8),
            ),
        )
        .unwrap();
        let plan = inspect(&std::fs::read(&binary).unwrap()).unwrap().unwrap();
        vendor_liblzma(&binary, Some(&dylib_path), &plan).unwrap();

        let rewritten = parse_loads(&std::fs::read(&binary).unwrap()).unwrap();
        assert!(rewritten.iter().any(|load| load.path == LIBLZMA_LOAD));
        let sibling = dir.path().join(LIBLZMA_FILE);
        let sibling_loads = parse_loads(&std::fs::read(&sibling).unwrap()).unwrap();
        assert!(sibling_loads
            .iter()
            .any(|load| load.cmd == LC_ID_DYLIB && load.path == LIBLZMA_LOAD));
    }

    #[test]
    fn missing_or_old_vendored_liblzma_is_rejected() {
        let dir = tempfile::tempdir().unwrap();
        let binary = dir.path().join("rustfs");
        std::fs::write(&binary, fixture(&[HOMEBREW], None, COMPAT_14, CURRENT_14_3)).unwrap();
        let plan = inspect(&std::fs::read(&binary).unwrap()).unwrap().unwrap();
        let missing = vendor_liblzma(&binary, None, &plan).unwrap_err();
        assert!(missing.to_string().contains(HOMEBREW), "{missing}");

        let old = dir.path().join("old.dylib");
        std::fs::write(
            &old,
            fixture(
                &["/usr/lib/libSystem.B.dylib"],
                Some(LIBLZMA_LOAD),
                6 << 16,
                6 << 16,
            ),
        )
        .unwrap();
        let error = vendor_liblzma(&binary, Some(&old), &plan).unwrap_err();
        assert!(error.to_string().contains("6.0.0"), "{error}");
        assert!(error.to_string().contains("14.0.0"), "{error}");
    }

    #[test]
    fn non_macho_bytes_are_left_alone_and_truncated_macho_is_rejected() {
        assert!(inspect(b"not-an-executable").unwrap().is_none());
        let error = inspect(&[0xcf, 0xfa, 0xed, 0xfe, 0, 0, 0, 0]).unwrap_err();
        assert!(error.to_string().contains("Mach-O"), "{error}");
    }
}
