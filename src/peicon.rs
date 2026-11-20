//! Extracción del icono incrustado en un ejecutable PE (RT_GROUP_ICON / RT_ICON).
//! Implementado con std para no añadir crates.

use std::fs;
use std::path::Path;

fn u16at(buf: &[u8], at: usize) -> Option<u16> {
    let s = buf.get(at..at + 2)?;
    Some(u16::from_le_bytes([s[0], s[1]]))
}

fn u32at(buf: &[u8], at: usize) -> Option<u32> {
    let s = buf.get(at..at + 4)?;
    Some(u32::from_le_bytes([s[0], s[1], s[2], s[3]]))
}

#[derive(Clone, Copy, Debug)]
struct Section {
    va: u32,
    virtual_size: u32,
    raw_size: u32,
    raw_ptr: u32,
}

struct Pe {
    resource_rva: u32,
    sections: Vec<Section>,
}

impl Pe {
    fn rva_to_offset(&self, rva: u32) -> Option<usize> {
        for s in &self.sections {
            let span = s.virtual_size.max(s.raw_size);
            if rva >= s.va && (rva as u64) < s.va as u64 + span as u64 {
                let delta = rva - s.va;
                if delta >= s.raw_size {
                    return None;
                }
                return Some(s.raw_ptr as usize + delta as usize);
            }
        }
        None
    }

    /// Los offsets dentro del directorio de recursos son relativos al inicio
    /// del directorio, por eso se suma resource_rva antes de mapear.
    fn resource_offset(&self, relative: u32) -> Option<usize> {
        self.rva_to_offset(self.resource_rva.wrapping_add(relative))
    }

    fn resource_dir(&self, buf: &[u8], relative: u32) -> Option<Vec<(u32, u32)>> {
        let base = self.resource_offset(relative)?;
        let named = u16at(buf, base + 12)? as usize;
        let ids = u16at(buf, base + 14)? as usize;
        let mut out = Vec::with_capacity(named + ids);
        for i in 0..named + ids {
            let entry = base + 16 + i * 8;
            out.push((u32at(buf, entry)?, u32at(buf, entry + 4)?));
        }
        Some(out)
    }

    fn find_leaf(&self, buf: &[u8], type_id: u32, name_id: Option<u32>) -> Option<(u32, u32)> {
        let root = self.resource_dir(buf, 0)?;
        let mut target = root.into_iter().find(|(name, _)| *name == type_id)?.1;
        if target & 0x8000_0000 == 0 {
            return None;
        }
        let by_name = self.resource_dir(buf, target & 0x7fff_ffff)?;
        target = match name_id {
            Some(id) => by_name.into_iter().find(|(name, _)| *name == id)?.1,
            None => by_name.first()?.1,
        };
        if target & 0x8000_0000 != 0 {
            let languages = self.resource_dir(buf, target & 0x7fff_ffff)?;
            target = languages.first()?.1;
        }
        if target & 0x8000_0000 != 0 {
            return None;
        }
        let leaf = self.resource_offset(target)?;
        Some((u32at(buf, leaf)?, u32at(buf, leaf + 4)?))
    }

    fn bytes<'a>(&self, buf: &'a [u8], rva: u32, size: u32) -> Option<&'a [u8]> {
        let offset = self.rva_to_offset(rva)?;
        buf.get(offset..offset + size as usize)
    }
}

fn parse_pe(buf: &[u8]) -> Option<Pe> {
    if !buf.starts_with(b"MZ") {
        return None;
    }
    let pe = u32at(buf, 0x3c)? as usize;
    if buf.get(pe..pe + 4)? != b"PE\0\0" {
        return None;
    }
    let section_count = u16at(buf, pe + 6)? as usize;
    let optional_size = u16at(buf, pe + 20)? as usize;
    let optional = pe + 24;
    let magic = u16at(buf, optional)?;
    let directories = match magic {
        0x10b => optional + 96,  // PE32
        0x20b => optional + 112, // PE32+
        _ => return None,
    };
    // DataDirectory[2] = resource table
    let resource_rva = u32at(buf, directories + 16)?;
    let resource_size = u32at(buf, directories + 20)?;
    if resource_rva == 0 || resource_size == 0 {
        return None;
    }
    let section_table = optional + optional_size;
    let mut sections = Vec::with_capacity(section_count);
    for i in 0..section_count {
        let s = section_table + i * 40;
        sections.push(Section {
            virtual_size: u32at(buf, s + 8)?,
            va: u32at(buf, s + 12)?,
            raw_size: u32at(buf, s + 16)?,
            raw_ptr: u32at(buf, s + 20)?,
        });
    }
    Some(Pe {
        resource_rva,
        sections,
    })
}

/// Extrae el primer grupo de iconos y escribe un archivo .ico.
pub fn extract_icon(exe: &Path, out: &Path) -> Result<(), String> {
    let buf = fs::read(exe).map_err(|e| format!("No se pudo leer {}: {e}", exe.display()))?;
    let pe = parse_pe(&buf).ok_or_else(|| format!("{} no es un PE con recursos", exe.display()))?;
    let (group_rva, group_size) = pe
        .find_leaf(&buf, 14, None)
        .ok_or_else(|| "el PE no tiene RT_GROUP_ICON".to_string())?;
    let group = pe
        .bytes(&buf, group_rva, group_size)
        .ok_or_else(|| "no se pudo leer el grupo de iconos".to_string())?;
    if group.len() < 6 {
        return Err("grupo de iconos demasiado corto".to_string());
    }
    let count = u16at(group, 4).unwrap_or(0) as usize;
    if count == 0 {
        return Err("el grupo de iconos esta vacio".to_string());
    }

    let mut icons: Vec<(u8, u8, u16, u16, Vec<u8>)> = Vec::new();
    for i in 0..count {
        let entry = 6 + i * 14;
        if entry + 14 > group.len() {
            break;
        }
        let width = group[entry];
        let height = group[entry + 1];
        let planes = u16at(group, entry + 4).unwrap_or(1);
        let bits = u16at(group, entry + 6).unwrap_or(32);
        let id = u16at(group, entry + 12).unwrap_or(0) as u32;
        let (rva, size) = pe
            .find_leaf(&buf, 3, Some(id))
            .ok_or_else(|| format!("falta RT_ICON {id}"))?;
        let image = pe
            .bytes(&buf, rva, size)
            .ok_or_else(|| format!("no se pudo leer RT_ICON {id}"))?;
        icons.push((width, height, planes, bits, image.to_vec()));
    }

    let mut ico = Vec::new();
    ico.extend_from_slice(&[0, 0, 1, 0]); // ICONDIR: reservado, tipo=1
    ico.extend_from_slice(&(icons.len() as u16).to_le_bytes());
    let mut offset = 6 + 16 * icons.len() as u32;
    for (width, height, planes, bits, image) in &icons {
        ico.push(*width);
        ico.push(*height);
        ico.push(0); // paleta
        ico.push(0); // reservado
        ico.extend_from_slice(&planes.to_le_bytes());
        ico.extend_from_slice(&bits.to_le_bytes());
        ico.extend_from_slice(&(image.len() as u32).to_le_bytes());
        ico.extend_from_slice(&offset.to_le_bytes());
        offset += image.len() as u32;
    }
    for (_, _, _, _, image) in &icons {
        ico.extend_from_slice(image);
    }

    if let Some(parent) = out.parent() {
        fs::create_dir_all(parent).ok();
    }
    fs::write(out, ico).map_err(|e| format!("No se pudo escribir {}: {e}", out.display()))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn put16(buf: &mut [u8], at: usize, v: u16) {
        buf[at..at + 2].copy_from_slice(&v.to_le_bytes());
    }
    fn put32(buf: &mut [u8], at: usize, v: u32) {
        buf[at..at + 4].copy_from_slice(&v.to_le_bytes());
    }
    fn resource_dir(buf: &mut [u8], at: usize, count: u16) {
        put16(buf, at + 12, 0); // entradas con nombre
        put16(buf, at + 14, count);
    }
    fn resource_entry(buf: &mut [u8], at: usize, id: u32, target: u32) {
        put32(buf, at, id);
        put32(buf, at + 4, target);
    }

    /// PE sintetico con una seccion .rsrc que contiene un grupo con un icono.
    fn fake_pe() -> Vec<u8> {
        let mut b = vec![0u8; 0x400];
        b[0] = b'M';
        b[1] = b'Z';
        put32(&mut b, 0x3c, 0x40);
        b[0x40..0x44].copy_from_slice(b"PE\0\0");
        put16(&mut b, 0x44, 0x8664); // machine x64
        put16(&mut b, 0x46, 1); // una seccion
        put16(&mut b, 0x54, 0xf0); // tamaño del optional header (COFF+16)
        put16(&mut b, 0x58, 0x20b); // PE32+
        put32(&mut b, 0x58 + 112 + 16, 0x1000); // directorio de recursos
        put32(&mut b, 0x58 + 112 + 20, 0x200);
        let sec = 0x58 + 0xf0;
        b[sec..sec + 5].copy_from_slice(b".rsrc");
        put32(&mut b, sec + 8, 0x200); // virtual size
        put32(&mut b, sec + 12, 0x1000); // RVA
        put32(&mut b, sec + 16, 0x200); // raw size
        put32(&mut b, sec + 20, 0x200); // puntero a datos

        let r = 0x200; // offsets relativos al directorio de recursos
        resource_dir(&mut b, r, 2);
        resource_entry(&mut b, r + 16, 14, 0x100 | 0x8000_0000);
        resource_entry(&mut b, r + 24, 3, 0x180 | 0x8000_0000);

        resource_dir(&mut b, r + 0x100, 1);
        resource_entry(&mut b, r + 0x110, 1, 0x120 | 0x8000_0000);
        resource_dir(&mut b, r + 0x120, 1);
        resource_entry(&mut b, r + 0x130, 0x409, 0x140);
        put32(&mut b, r + 0x140, 0x1050); // RVA del grupo
        put32(&mut b, r + 0x144, 22);

        resource_dir(&mut b, r + 0x180, 1);
        resource_entry(&mut b, r + 0x190, 1, 0x1a0 | 0x8000_0000);
        resource_dir(&mut b, r + 0x1a0, 1);
        resource_entry(&mut b, r + 0x1b0, 0x409, 0x1c0);
        put32(&mut b, r + 0x1c0, 0x1100); // RVA del icono
        put32(&mut b, r + 0x1c4, 6);

        let g = r + 0x50;
        put16(&mut b, g + 2, 1); // tipo grupo
        put16(&mut b, g + 4, 1); // un icono
        b[g + 6] = 16; // ancho
        b[g + 7] = 16; // alto
        put16(&mut b, g + 10, 1); // planos
        put16(&mut b, g + 12, 32); // bits
        put32(&mut b, g + 14, 6); // bytes del recurso
        put16(&mut b, g + 18, 1); // nID

        let img = r + 0x100;
        b[img..img + 6].copy_from_slice(&[1, 2, 3, 4, 5, 6]);
        b
    }

    #[test]
    fn extracts_icon_from_synthetic_pe() {
        let dir = std::env::temp_dir().join("crisol_icon_test");
        fs::create_dir_all(&dir).unwrap();
        let exe = dir.join("fake.exe");
        let ico = dir.join("fake.ico");
        fs::write(&exe, fake_pe()).unwrap();
        extract_icon(&exe, &ico).unwrap();
        let data = fs::read(&ico).unwrap();
        assert_eq!(&data[..6], &[0, 0, 1, 0, 1, 0]);
        assert_eq!(data.len(), 6 + 16 + 6);
        assert_eq!(&data[data.len() - 6..], &[1, 2, 3, 4, 5, 6]);
        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn rejects_non_pe() {
        let dir = std::env::temp_dir().join("crisol_icon_test_bad");
        fs::create_dir_all(&dir).unwrap();
        let exe = dir.join("bad.exe");
        let ico = dir.join("bad.ico");
        fs::write(&exe, b"no soy un PE").unwrap();
        assert!(extract_icon(&exe, &ico).is_err());
        fs::remove_dir_all(&dir).unwrap();
    }
}
