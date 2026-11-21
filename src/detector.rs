use std::fs;
use std::path::Path;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InstallerKind {
    Msi,
    Nsis,
    Inno,
    InstallShield,
    Wix,
    PlainExe,
    Unknown,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Arch {
    X86,
    X64,
    Arm64,
    Unknown,
}

#[derive(Clone, Debug)]
pub struct Detection {
    pub kind: InstallerKind,
    pub arch: Arch,
    pub hints: Vec<String>,
}

impl Detection {
    /// Un PE sin firma de instalador es un portable candidato: conviene
    /// registrarlo y lanzarlo bajo demanda en vez de ejecutarlo como instalador.
    pub fn is_portable(&self) -> bool {
        matches!(self.kind, InstallerKind::PlainExe | InstallerKind::Unknown)
    }
}

fn contains_ci(haystack: &[u8], needle: &[u8]) -> bool {
    if needle.is_empty() {
        return true;
    }
    if needle.len() > haystack.len() {
        return false;
    }
    haystack
        .windows(needle.len())
        .any(|w| w.iter().zip(needle).all(|(a, b)| a.eq_ignore_ascii_case(b)))
}

fn u16le(buf: &[u8], at: usize) -> Option<u16> {
    let s = buf.get(at..at + 2)?;
    Some(u16::from_le_bytes([s[0], s[1]]))
}

fn u32le(buf: &[u8], at: usize) -> Option<u32> {
    let s = buf.get(at..at + 4)?;
    Some(u32::from_le_bytes([s[0], s[1], s[2], s[3]]))
}

pub fn detect(path: &Path) -> Result<Detection, String> {
    let buf = fs::read(path).map_err(|e| format!("No se pudo leer {}: {e}", path.display()))?;
    if buf.len() < 64 {
        return Err("El archivo es demasiado pequeño para ser un instalador".to_string());
    }

    let lower_name = path
        .file_name()
        .map(|s| s.to_string_lossy().to_lowercase())
        .unwrap_or_default();
    let is_msi = lower_name.ends_with(".msi")
        || buf.starts_with(&[0xd0, 0xcf, 0x11, 0xe0, 0xa1, 0xb1, 0x1a, 0xe1]);

    let mut hints: Vec<String> = Vec::new();
    let mut kind = if is_msi {
        InstallerKind::Msi
    } else if buf.starts_with(b"MZ") {
        InstallerKind::PlainExe
    } else {
        InstallerKind::Unknown
    };

    let arch = if is_msi {
        Arch::Unknown
    } else if buf.starts_with(b"MZ") {
        match u32le(&buf, 0x3c) {
            Some(pe) => {
                let pe = pe as usize;
                if buf.get(pe..pe + 4) == Some(b"PE\0\0") {
                    match u16le(&buf, pe + 4) {
                        Some(0x014c) => Arch::X86,
                        Some(0x8664) => Arch::X64,
                        Some(0xaa64) => Arch::Arm64,
                        _ => Arch::Unknown,
                    }
                } else {
                    Arch::Unknown
                }
            }
            None => Arch::Unknown,
        }
    } else {
        Arch::Unknown
    };

    let checks: [(&[u8], &str, InstallerKind); 4] = [
        (b"inno setup", "inno setup", InstallerKind::Inno),
        (b"nullsoft", "nsis", InstallerKind::Nsis),
        (b"nsis", "nsis", InstallerKind::Nsis),
        (
            b"installshield",
            "installshield",
            InstallerKind::InstallShield,
        ),
    ];
    for (needle, hint, detected) in checks {
        if contains_ci(&buf, needle) {
            hints.push(hint.to_string());
            if !is_msi {
                kind = detected;
            }
        }
    }
    if contains_ci(&buf, b"windows installer xml") || contains_ci(&buf, b"wix") {
        hints.push("wix".to_string());
        if !is_msi {
            kind = InstallerKind::Wix;
        }
    }
    if is_msi {
        hints.push("msi".to_string());
    }
    if kind == InstallerKind::PlainExe {
        hints.push("portable".to_string());
    }
    match arch {
        Arch::X86 => hints.push("x86".to_string()),
        Arch::X64 => hints.push("x64".to_string()),
        Arch::Arm64 => hints.push("arm64".to_string()),
        Arch::Unknown => {}
    }
    hints.sort();
    hints.dedup();

    if kind == InstallerKind::Unknown && !is_msi && !buf.starts_with(b"MZ") {
        return Err("No parece un ejecutable PE ni un MSI válido".to_string());
    }

    Ok(Detection { kind, arch, hints })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_msi_magic() {
        let dir = std::env::temp_dir().join("crisol_test_detect");
        fs::create_dir_all(&dir).unwrap();
        let p = dir.join("fake.msi");
        let mut data = vec![0u8; 128];
        data[..8].copy_from_slice(&[0xd0, 0xcf, 0x11, 0xe0, 0xa1, 0xb1, 0x1a, 0xe1]);
        fs::write(&p, data).unwrap();
        let d = detect(&p).unwrap();
        assert_eq!(d.kind, InstallerKind::Msi);
        assert!(d.hints.iter().any(|h| h == "msi"));
        fs::remove_file(&p).unwrap();
    }

    #[test]
    fn detects_plain_exe_as_portable() {
        let dir = std::env::temp_dir().join("crisol_test_portable");
        fs::create_dir_all(&dir).unwrap();
        let p = dir.join("app.exe");
        let mut data = vec![0u8; 256];
        data[..2].copy_from_slice(b"MZ");
        data[0x3c..0x40].copy_from_slice(&0x80u32.to_le_bytes());
        data[0x80..0x84].copy_from_slice(b"PE\0\0");
        data[0x84..0x86].copy_from_slice(&0x8664u16.to_le_bytes());
        fs::write(&p, data).unwrap();
        let d = detect(&p).unwrap();
        assert_eq!(d.kind, InstallerKind::PlainExe);
        assert_eq!(d.arch, Arch::X64);
        assert!(d.is_portable());
        assert!(d.hints.iter().any(|h| h == "portable"));
        fs::remove_file(&p).unwrap();
    }

    #[test]
    fn contains_is_case_insensitive() {
        assert!(contains_ci(b"NullSoft Installer", b"nullsoft"));
        assert!(!contains_ci(b"abc", b"abcd"));
    }
}
