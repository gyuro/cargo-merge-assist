use std::collections::BTreeSet;

use toml::Value;

#[derive(Debug, Clone)]
pub struct MergeConflict {
    pub path: String,
    pub base: Option<Value>,
    pub ours: Option<Value>,
    pub theirs: Option<Value>,
}

impl std::fmt::Display for MergeConflict {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "semantic conflict at `{}`\n  base  : {}\n  ours  : {}\n  theirs: {}",
            if self.path.is_empty() {
                "<root>"
            } else {
                &self.path
            },
            render_value(self.base.as_ref()),
            render_value(self.ours.as_ref()),
            render_value(self.theirs.as_ref())
        )
    }
}

impl std::error::Error for MergeConflict {}

fn render_value(v: Option<&Value>) -> String {
    match v {
        Some(value) => value.to_string(),
        None => "<deleted>".to_string(),
    }
}

pub fn merge_manifest_texts(
    base_text: &str,
    ours_text: &str,
    theirs_text: &str,
) -> Result<String, MergeConflict> {
    let base: Value = toml::from_str(base_text).map_err(|_| MergeConflict {
        path: "<parse:base>".to_string(),
        base: None,
        ours: None,
        theirs: None,
    })?;
    let ours: Value = toml::from_str(ours_text).map_err(|_| MergeConflict {
        path: "<parse:ours>".to_string(),
        base: None,
        ours: None,
        theirs: None,
    })?;
    let theirs: Value = toml::from_str(theirs_text).map_err(|_| MergeConflict {
        path: "<parse:theirs>".to_string(),
        base: None,
        ours: None,
        theirs: None,
    })?;

    let merged = merge_value("", Some(&base), Some(&ours), Some(&theirs))?
        .expect("root merge always returns a document");

    let mut output = toml::to_string_pretty(&merged).map_err(|_| MergeConflict {
        path: "<serialize>".to_string(),
        base: None,
        ours: None,
        theirs: None,
    })?;

    if !output.ends_with('\n') {
        output.push('\n');
    }

    Ok(output)
}

fn merge_value(
    path: &str,
    base: Option<&Value>,
    ours: Option<&Value>,
    theirs: Option<&Value>,
) -> Result<Option<Value>, MergeConflict> {
    if ours == theirs {
        return Ok(ours.cloned());
    }

    if ours == base {
        return Ok(theirs.cloned());
    }

    if theirs == base {
        return Ok(ours.cloned());
    }

    match (ours, theirs) {
        (Some(Value::Table(ours_table)), Some(Value::Table(theirs_table))) => {
            let mut keys = BTreeSet::new();
            keys.extend(ours_table.keys().cloned());
            keys.extend(theirs_table.keys().cloned());

            if let Some(Value::Table(base_table)) = base {
                keys.extend(base_table.keys().cloned());
            }

            let mut out = toml::map::Map::new();

            for key in keys {
                let key_path = join_path(path, &key);
                let base_child = base
                    .and_then(|v| v.as_table())
                    .and_then(|table| table.get(&key));
                let ours_child = ours_table.get(&key);
                let theirs_child = theirs_table.get(&key);

                if let Some(value) = merge_value(&key_path, base_child, ours_child, theirs_child)? {
                    out.insert(key, value);
                }
            }

            Ok(Some(Value::Table(out)))
        }
        _ => Err(MergeConflict {
            path: path.to_string(),
            base: base.cloned(),
            ours: ours.cloned(),
            theirs: theirs.cloned(),
        }),
    }
}

fn join_path(base: &str, key: &str) -> String {
    if base.is_empty() {
        key.to_string()
    } else {
        format!("{base}.{key}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn merges_independent_dependency_changes() {
        let base = r#"
[dependencies]
serde = "1"
"#;
        let ours = r#"
[dependencies]
serde = "1"
clap = "4"
"#;
        let theirs = r#"
[dependencies]
serde = "1"
anyhow = "1"
"#;

        let merged = merge_manifest_texts(base, ours, theirs).expect("merge should succeed");
        assert!(merged.contains("clap = \"4\""));
        assert!(merged.contains("anyhow = \"1\""));
        assert!(merged.contains("serde = \"1\""));
    }

    #[test]
    fn keeps_single_side_change() {
        let base = r#"
[package]
name = "demo"
version = "0.1.0"
"#;
        let ours = r#"
[package]
name = "demo"
version = "0.2.0"
"#;
        let theirs = base;

        let merged = merge_manifest_texts(base, ours, theirs).expect("merge should succeed");
        assert!(merged.contains("version = \"0.2.0\""));
    }

    #[test]
    fn reports_conflict_on_same_key_different_change() {
        let base = r#"
[dependencies]
serde = "1"
"#;
        let ours = r#"
[dependencies]
serde = "1.0.200"
"#;
        let theirs = r#"
[dependencies]
serde = "1.0.199"
"#;

        let err = merge_manifest_texts(base, ours, theirs).expect_err("merge must conflict");
        assert_eq!(err.path, "dependencies.serde");
    }
}
