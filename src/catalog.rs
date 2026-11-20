use std::fs;
use std::path::Path;

#[derive(Clone, Debug)]
pub struct Entry {
    pub id: String,
    pub name: String,
    pub exe: String,
    pub args: String,
    pub prefix: String,
    pub installer: String,
    pub icon: String,
    pub engine: String,
    pub created: u64,
    pub last_status: String,
    /// IDs de las recetas aplicadas, separados por ';'. Lo usa el panel de feedback.
    pub recipes: String,
}

fn clean(s: &str) -> String {
    s.replace(['\t', '\n', '\r'], " ")
}

impl Entry {
    fn line(&self) -> String {
        [
            self.id.clone(),
            self.name.clone(),
            self.exe.clone(),
            self.args.clone(),
            self.prefix.clone(),
            self.installer.clone(),
            self.icon.clone(),
            self.engine.clone(),
            self.created.to_string(),
            self.last_status.clone(),
            self.recipes.clone(),
        ]
        .iter()
        .map(|s| clean(s))
        .collect::<Vec<_>>()
        .join("\t")
    }
}

pub fn load(path: &Path) -> Vec<Entry> {
    let Ok(text) = fs::read_to_string(path) else {
        return Vec::new();
    };
    text.lines()
        .filter_map(|line| {
            let f: Vec<&str> = line.split('\t').collect();
            if f.len() < 10 {
                return None;
            }
            Some(Entry {
                id: f[0].to_string(),
                name: f[1].to_string(),
                exe: f[2].to_string(),
                args: f[3].to_string(),
                prefix: f[4].to_string(),
                installer: f[5].to_string(),
                icon: f[6].to_string(),
                engine: f[7].to_string(),
                created: f[8].parse().unwrap_or(0),
                last_status: f[9].to_string(),
                recipes: f.get(10).map(|s| s.to_string()).unwrap_or_default(),
            })
        })
        .collect()
}

pub fn save(path: &Path, entries: &[Entry]) -> Result<(), String> {
    let mut text = String::new();
    for e in entries {
        text.push_str(&e.line());
        text.push('\n');
    }
    fs::write(path, text).map_err(|e| format!("No se pudo guardar el catalogo: {e}"))
}

pub fn upsert(path: &Path, entry: Entry) -> Result<(), String> {
    let mut entries = load(path);
    if let Some(old) = entries.iter_mut().find(|e| e.id == entry.id) {
        *old = entry;
    } else {
        entries.push(entry);
    }
    save(path, &entries)
}

pub fn remove(path: &Path, id: &str) -> Result<Option<Entry>, String> {
    let entries = load(path);
    let Some(found) = entries.iter().find(|e| e.id == id).cloned() else {
        return Ok(None);
    };
    let kept: Vec<Entry> = entries.into_iter().filter(|e| e.id != id).collect();
    save(path, &kept)?;
    Ok(Some(found))
}
