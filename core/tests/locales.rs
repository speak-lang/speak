//! Every locale must define every key that en.yml defines.
//! Missing keys fail the test; extra keys and differing %{placeholders} only warn.
use serde_yaml::Value;
use std::collections::BTreeMap;
use std::fs;

fn flatten(prefix: &str, v: &Value, out: &mut BTreeMap<String, String>) {
    match v {
        Value::Mapping(m) => {
            for (k, v) in m {
                // keys like `true:` or `in:` may parse as non-strings
                let k = match k {
                    Value::String(s) => s.clone(),
                    Value::Bool(b) => b.to_string(),
                    other => format!("{:?}", other),
                };
                let path = if prefix.is_empty() { k } else { format!("{prefix}.{k}") };
                flatten(&path, v, out);
            }
        }
        Value::String(s) => { out.insert(prefix.into(), s.clone()); }
        Value::Bool(b) => { out.insert(prefix.into(), b.to_string()); }
        Value::Number(n) => { out.insert(prefix.into(), n.to_string()); }
        _ => { out.insert(prefix.into(), String::new()); }
    }
}

fn load(path: &std::path::Path) -> BTreeMap<String, String> {
    let text = fs::read_to_string(path).unwrap_or_else(|e| panic!("cannot read {}: {e}", path.display()));
    let v: Value = serde_yaml::from_str(&text).unwrap_or_else(|e| panic!("{} is not valid YAML: {e}", path.display()));
    let mut out = BTreeMap::new();
    flatten("", &v, &mut out);
    out
}

fn placeholders(s: &str) -> Vec<String> {
    let mut found: Vec<String> = s.split("%{").skip(1).filter_map(|r| r.split_once('}')).map(|(n, _)| format!("%{{{n}}}")).collect();
    found.sort();
    found
}

#[test]
fn locales_have_every_english_key() {
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("locales");
    let en = load(&dir.join("en.yml"));
    let mut report = String::new();

    let mut files: Vec<_> = fs::read_dir(&dir).unwrap().map(|e| e.unwrap().path())
        .filter(|p| matches!(p.extension().and_then(|e| e.to_str()), Some("yml" | "yaml")))
        .collect();
    files.sort();
    for path in files {
        let name = path.file_name().unwrap().to_string_lossy().to_string();
        if name == "en.yml" { continue; }
        let tr = load(&path);
        for (k, en_val) in &en {
            match tr.get(k) {
                None => report.push_str(&format!(
                    "{name} is missing the key \"{k}\". Add a line for it, translating the English \"{en_val}\".\n")),
                Some(t) if placeholders(t) != placeholders(en_val) => eprintln!(
                    "warning: {name}, key \"{k}\": placeholders {:?} differ from English {:?}", placeholders(t), placeholders(en_val)),
                _ => {}
            }
        }
        for k in tr.keys().filter(|k| !en.contains_key(*k)) {
            eprintln!("warning: {name} has the key \"{k}\" which en.yml does not have");
        }
    }
    assert!(report.is_empty(), "\nLocale files are incomplete:\n{report}");
}
