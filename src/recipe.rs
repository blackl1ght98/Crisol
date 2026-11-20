//! Parser y motor de recetas. DSL minimalista, sin dependencias externas,
//! para que el proyecto compile con un Rust vanilla recien instalado.

use std::cmp::Ordering;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

#[derive(Clone, Debug)]
pub enum Matcher {
    LogAny(Vec<String>),
    LogAll(Vec<String>),
    LogNot(Vec<String>),
    Detector(Vec<String>),
    Exit(Vec<i32>),
    FileMissing(Vec<String>),
}

#[derive(Clone, Debug)]
pub enum Action {
    Winetricks(Vec<String>),
    Override(Vec<(String, String)>),
    Env(Vec<(String, String)>),
    Engine(String),
    Retry,
    Reset(bool),
    SetExe(String),
}

#[derive(Clone, Debug)]
pub struct Recipe {
    pub id: String,
    pub confidence: f32,
    pub priority: i32,
    pub matchers: Vec<Matcher>,
    pub actions: Vec<Action>,
    pub message: Option<String>,
    pub source: String,
}

impl Recipe {
    fn new(id: String, source: &str) -> Self {
        Self {
            id,
            confidence: 0.5,
            priority: 0,
            matchers: Vec::new(),
            actions: Vec::new(),
            message: None,
            source: source.to_string(),
        }
    }

    fn push_winetricks(&mut self, vals: Vec<String>) {
        if let Some(Action::Winetricks(v)) = self
            .actions
            .iter_mut()
            .find(|a| matches!(a, Action::Winetricks(_)))
        {
            v.extend(vals);
        } else {
            self.actions.push(Action::Winetricks(vals));
        }
    }

    fn push_override(&mut self, vals: Vec<(String, String)>) {
        if let Some(Action::Override(v)) = self
            .actions
            .iter_mut()
            .find(|a| matches!(a, Action::Override(_)))
        {
            v.extend(vals);
        } else {
            self.actions.push(Action::Override(vals));
        }
    }

    fn push_env(&mut self, vals: Vec<(String, String)>) {
        if let Some(Action::Env(v)) = self
            .actions
            .iter_mut()
            .find(|a| matches!(a, Action::Env(_)))
        {
            v.extend(vals);
        } else {
            self.actions.push(Action::Env(vals));
        }
    }

    fn set(&mut self, key: &str, val: &str) {
        let lower = split_lower(val);
        let raw = split_raw(val);
        match key {
            "confidence" => {
                if let Ok(v) = val.trim().parse::<f32>() {
                    self.confidence = v.clamp(0.0, 1.0);
                }
            }
            "priority" => {
                if let Ok(v) = val.trim().parse::<i32>() {
                    self.priority = v;
                }
            }
            "match_log_contains" => {
                for v in lower {
                    self.matchers.push(Matcher::LogAll(vec![v]));
                }
            }
            "match_log_any" => self.matchers.push(Matcher::LogAny(lower)),
            "match_log_not" => self.matchers.push(Matcher::LogNot(lower)),
            "match_detector" => self.matchers.push(Matcher::Detector(lower)),
            "match_exit_code" => {
                if let Ok(code) = val.trim().parse::<i32>() {
                    if let Some(Matcher::Exit(v)) = self
                        .matchers
                        .iter_mut()
                        .find(|m| matches!(m, Matcher::Exit(_)))
                    {
                        v.push(code);
                    } else {
                        self.matchers.push(Matcher::Exit(vec![code]));
                    }
                }
            }
            "match_file_missing" => self.matchers.push(Matcher::FileMissing(lower)),
            "action_winetricks" => self.push_winetricks(raw),
            "action_override" => {
                let pairs: Vec<(String, String)> = raw
                    .iter()
                    .filter_map(|s| {
                        let mut it = s.splitn(2, '=');
                        let k = it.next()?.trim().to_lowercase();
                        let v = it.next().unwrap_or("native").trim().to_lowercase();
                        if k.is_empty() {
                            None
                        } else {
                            Some((k, v))
                        }
                    })
                    .collect();
                self.push_override(pairs);
            }
            "action_env" => {
                let mut parts = val.splitn(2, '=');
                if let (Some(key), Some(value)) = (parts.next(), parts.next()) {
                    let key = key.trim();
                    if !key.is_empty() {
                        self.push_env(vec![(key.to_string(), value.trim().to_string())]);
                    }
                }
            }
            "action_engine" => {
                if !val.trim().is_empty() {
                    self.actions.push(Action::Engine(val.trim().to_string()));
                }
            }
            "action_message" => self.message = Some(val.trim().to_string()),
            "action_retry" => {
                if parse_bool(val) {
                    self.actions.push(Action::Retry);
                }
            }
            "action_reset_prefix" => self.actions.push(Action::Reset(parse_bool(val))),
            "action_set_exe" => self
                .actions
                .push(Action::SetExe(val.trim().replace('\\', "/"))),
            _ => {}
        }
    }

    pub fn score(&self) -> f32 {
        self.confidence * 100.0 + self.priority as f32
    }

    pub fn matches(&self, ctx: &MatchCtx) -> bool {
        let log = ctx.log.to_lowercase();
        for m in &self.matchers {
            let ok = match m {
                Matcher::LogAny(vals) => vals.iter().any(|v| log.contains(v)),
                Matcher::LogAll(vals) => vals.iter().all(|v| log.contains(v)),
                Matcher::LogNot(vals) => vals.iter().all(|v| !log.contains(v)),
                Matcher::Detector(vals) => vals
                    .iter()
                    .any(|v| ctx.detector.iter().any(|h| h.contains(v) || v.contains(h))),
                Matcher::Exit(codes) => ctx.exit_code.map(|c| codes.contains(&c)).unwrap_or(false),
                Matcher::FileMissing(patterns) => {
                    patterns.iter().any(|p| !glob_exists(ctx.prefix, p))
                }
            };
            if !ok {
                return false;
            }
        }
        true
    }
}

pub struct MatchCtx<'a> {
    pub log: &'a str,
    pub detector: &'a [String],
    pub exit_code: Option<i32>,
    pub prefix: &'a Path,
}

fn parse_bool(v: &str) -> bool {
    matches!(v.trim().to_lowercase().as_str(), "1" | "true" | "yes" | "on" | "si")
}

fn split_lower(v: &str) -> Vec<String> {
    v.split(',')
        .map(|s| s.trim().to_lowercase())
        .filter(|s| !s.is_empty())
        .collect()
}

fn split_raw(v: &str) -> Vec<String> {
    v.split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect()
}

pub fn parse(text: &str, source: &str) -> Vec<Recipe> {
    let mut out = Vec::new();
    let mut current: Option<Recipe> = None;
    for raw in text.lines() {
        let line = raw.trim();
        if line.is_empty() || line.starts_with('#') || line.starts_with(';') {
            continue;
        }
        if line.starts_with('[') && line.ends_with(']') {
            if let Some(r) = current.take() {
                out.push(r);
            }
            let id = line[1..line.len() - 1].trim().to_string();
            if !id.is_empty() {
                current = Some(Recipe::new(id, source));
            }
            continue;
        }
        if let Some(pos) = line.find('=') {
            if let Some(r) = current.as_mut() {
                r.set(line[..pos].trim(), line[pos + 1..].trim());
            }
        }
    }
    if let Some(r) = current {
        out.push(r);
    }
    out
}

pub fn parse_all(embedded: &str, user_dir: &Path) -> Vec<Recipe> {
    let mut by_id: HashMap<String, Recipe> = HashMap::new();
    for r in parse(embedded, "interna") {
        by_id.insert(r.id.clone(), r);
    }
    if let Ok(entries) = fs::read_dir(user_dir) {
        let mut files: Vec<_> = entries.flatten().map(|e| e.path()).collect();
        files.sort();
        for path in files {
            if path.extension().and_then(|s| s.to_str()) != Some("recipe") {
                continue;
            }
            if let Ok(text) = fs::read_to_string(&path) {
                let src = path.to_string_lossy().to_string();
                for r in parse(&text, &src) {
                    by_id.insert(r.id.clone(), r); // el usuario sobrescribe la receta interna
                }
            }
        }
    }
    // feedback.tsv: id<TAB>delta. Lo escribe el panel cuando el usuario dice
    // si la instalación funcionó. Se aplica después de las recetas del usuario.
    if let Some(parent) = user_dir.parent() {
        let feedback = parent.join("feedback.tsv");
        if let Ok(text) = fs::read_to_string(&feedback) {
            for line in text.lines() {
                let fields: Vec<&str> = line.split('\t').collect();
                if fields.len() != 2 {
                    continue;
                }
                if let (Ok(delta), Some(recipe)) = (fields[1].parse::<f32>(), by_id.get_mut(fields[0])) {
                    recipe.confidence = (recipe.confidence + delta).clamp(0.0, 1.0);
                }
            }
        }
    }

    let mut out: Vec<Recipe> = by_id.into_values().collect();
    out.sort_by(|a, b| {
        b.score()
            .partial_cmp(&a.score())
            .unwrap_or(Ordering::Equal)
            .then_with(|| a.id.cmp(&b.id))
    });
    out
}

pub fn select(recipes: &[Recipe], ctx: &MatchCtx, used: &[String], limit: usize) -> Vec<Recipe> {
    recipes
        .iter()
        .filter(|r| !used.contains(&r.id) && r.matches(ctx))
        .take(limit)
        .cloned()
        .collect()
}

fn wildcard_match(text: &str, pattern: &str) -> bool {
    fn go(t: &[char], p: &[char]) -> bool {
        if p.is_empty() {
            return t.is_empty();
        }
        match p[0] {
            '*' => go(t, &p[1..]) || (!t.is_empty() && go(&t[1..], p)),
            '?' => !t.is_empty() && go(&t[1..], &p[1..]),
            c => !t.is_empty() && t[0] == c && go(&t[1..], &p[1..]),
        }
    }
    let t: Vec<char> = text.chars().collect();
    let p: Vec<char> = pattern.chars().collect();
    go(&t, &p)
}

fn glob_exists(root: &Path, pattern: &str) -> bool {
    if !pattern.contains('*') && !pattern.contains('?') {
        return root.join(pattern).exists();
    }
    fn walk(root: &Path, dir: &Path, pattern: &str, depth: usize) -> bool {
        if depth > 10 {
            return false;
        }
        let Ok(entries) = fs::read_dir(dir) else {
            return false;
        };
        for entry in entries.flatten() {
            let Ok(kind) = entry.file_type() else {
                continue;
            };
            if kind.is_symlink() {
                continue;
            }
            let path = entry.path();
            if let Ok(rel) = path.strip_prefix(root) {
                let rel_s = rel.to_string_lossy().replace('\\', "/");
                if wildcard_match(&rel_s, pattern) {
                    return true;
                }
            }
            if kind.is_dir() && walk(root, &path, pattern, depth + 1) {
                return true;
            }
        }
        false
    }
    walk(root, root, pattern, 0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn parses_and_matches_recipe() {
        let text = "[r]\nconfidence = 0.9\nmatch_log_any = msvcp140.dll, vcruntime140.dll\naction_winetricks = vcrun2022\naction_message = Instalando VC++\n";
        let recipes = parse(text, "test");
        assert_eq!(recipes.len(), 1);
        assert_eq!(recipes[0].id, "r");
        assert_eq!(recipes[0].confidence, 0.9);
        assert_eq!(recipes[0].message.as_deref(), Some("Instalando VC++"));
        let ctx = MatchCtx {
            log: "error al cargar MSVCP140.DLL",
            detector: &[],
            exit_code: Some(1),
            prefix: &PathBuf::from("/tmp"),
        };
        assert!(recipes[0].matches(&ctx));
        let ctx2 = MatchCtx {
            log: "todo bien",
            detector: &[],
            exit_code: Some(0),
            prefix: &PathBuf::from("/tmp"),
        };
        assert!(!recipes[0].matches(&ctx2));
    }

    #[test]
    fn wildcard() {
        assert!(wildcard_match("drive_c/windows/system32/msvcp140.dll", "*/system32/*.dll"));
        assert!(!wildcard_match("drive_c/x/msvcp140.dll", "*/system32/*.dll"));
    }
}
