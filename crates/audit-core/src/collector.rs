use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
};

use toml::Value;

#[derive(Debug, Clone)]
pub struct RepositorySnapshot {
    pub root: PathBuf,
    pub files: Vec<FileSnapshot>,
    pub manifests: Vec<CargoManifest>,
}

#[derive(Debug, Clone)]
pub struct FileSnapshot {
    pub path: PathBuf,
    pub relative_path: String,
    pub extension: Option<String>,
    pub bytes: u64,
    pub lines: usize,
    pub content: Option<String>,
}

#[derive(Debug, Clone)]
pub struct CargoManifest {
    pub relative_path: String,
    pub package_name: Option<String>,
    pub workspace_members: Vec<String>,
    pub dependencies: BTreeMap<String, String>,
    pub dev_dependencies: BTreeMap<String, String>,
    pub build_dependencies: BTreeMap<String, String>,
}

impl RepositorySnapshot {
    pub fn collect(root: &Path) -> Result<Self, String> {
        let root = root
            .canonicalize()
            .map_err(|err| format!("failed to resolve {}: {err}", root.display()))?;
        let mut files = Vec::new();
        walk(&root, &root, &mut files)?;

        let manifests = files
            .iter()
            .filter(|file| file.relative_path.ends_with("Cargo.toml"))
            .filter_map(|file| file.content.as_deref().map(|content| (file, content)))
            .map(|(file, content)| CargoManifest::parse(&file.relative_path, content))
            .collect();

        Ok(Self {
            root,
            files,
            manifests,
        })
    }

    pub fn rust_files(&self) -> impl Iterator<Item = &FileSnapshot> {
        self.files
            .iter()
            .filter(|file| file.extension.as_deref() == Some("rs"))
    }
}

impl CargoManifest {
    fn parse(relative_path: &str, content: &str) -> Self {
        let parsed = content.parse::<Value>().ok();
        let package_name = parsed
            .as_ref()
            .and_then(|manifest| manifest.get("package"))
            .and_then(|package| package.get("name"))
            .and_then(Value::as_str)
            .map(str::to_string);
        let workspace_members = parsed
            .as_ref()
            .and_then(|manifest| manifest.get("workspace"))
            .and_then(|workspace| workspace.get("members"))
            .and_then(Value::as_array)
            .map(|members| {
                members
                    .iter()
                    .filter_map(Value::as_str)
                    .map(str::to_string)
                    .collect()
            })
            .unwrap_or_default();
        let dependencies = parse_dependency_table(parsed.as_ref(), "dependencies");
        let dev_dependencies = parse_dependency_table(parsed.as_ref(), "dev-dependencies");
        let build_dependencies = parse_dependency_table(parsed.as_ref(), "build-dependencies");

        Self {
            relative_path: relative_path.to_string(),
            package_name,
            workspace_members,
            dependencies,
            dev_dependencies,
            build_dependencies,
        }
    }
}

fn walk(root: &Path, current: &Path, files: &mut Vec<FileSnapshot>) -> Result<(), String> {
    let entries = fs::read_dir(current)
        .map_err(|err| format!("failed to read {}: {err}", current.display()))?;

    for entry in entries {
        let entry = entry.map_err(|err| format!("failed to read directory entry: {err}"))?;
        let path = entry.path();
        let name = entry.file_name();
        let name = name.to_string_lossy();

        if should_skip(&name) {
            continue;
        }

        let metadata = entry
            .metadata()
            .map_err(|err| format!("failed to stat {}: {err}", path.display()))?;
        if metadata.is_dir() {
            walk(root, &path, files)?;
            continue;
        }
        if !metadata.is_file() {
            continue;
        }

        let relative_path = path
            .strip_prefix(root)
            .unwrap_or(&path)
            .to_string_lossy()
            .replace('\\', "/");
        let extension = path
            .extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| ext.to_ascii_lowercase());
        let content = read_text_file(&path, metadata.len());
        let lines = content
            .as_deref()
            .map(|content| content.lines().count())
            .unwrap_or(0);

        files.push(FileSnapshot {
            path,
            relative_path,
            extension,
            bytes: metadata.len(),
            lines,
            content,
        });
    }

    Ok(())
}

fn should_skip(name: &str) -> bool {
    matches!(
        name,
        ".git" | "target" | "node_modules" | ".idea" | ".vscode" | ".DS_Store"
    )
}

fn read_text_file(path: &Path, bytes: u64) -> Option<String> {
    if bytes > 1_000_000 {
        return None;
    }
    fs::read_to_string(path).ok()
}

fn parse_dependency_table(manifest: Option<&Value>, section: &str) -> BTreeMap<String, String> {
    manifest
        .and_then(|manifest| manifest.get(section))
        .and_then(Value::as_table)
        .map(|dependencies| {
            dependencies
                .iter()
                .map(|(name, value)| (name.clone(), dependency_value(value)))
                .collect()
        })
        .unwrap_or_default()
}

fn dependency_value(value: &Value) -> String {
    value
        .as_str()
        .map(str::to_string)
        .unwrap_or_else(|| value.to_string())
}

#[cfg(test)]
mod tests {
    use super::CargoManifest;

    #[test]
    fn parses_multiline_workspace_members() {
        let manifest = CargoManifest::parse(
            "Cargo.toml",
            r#"
[workspace]
members = [
    "crates/audit-core",
    "crates/audit-cli",
]
"#,
        );

        assert_eq!(
            manifest.workspace_members,
            ["crates/audit-core", "crates/audit-cli"]
        );
    }

    #[test]
    fn parses_dependency_tables_without_losing_path_or_git_signals() {
        let manifest = CargoManifest::parse(
            "Cargo.toml",
            r#"
[package]
name = "fixture"

[dependencies]
serde = "1"
local-helper = { path = "../local-helper" }
remote-helper = { git = "https://example.test/repo.git", rev = "abc123" }

[dev-dependencies]
axum = { version = "0.7", features = ["json"] }

[build-dependencies]
cc = "1"
"#,
        );

        assert_eq!(manifest.package_name.as_deref(), Some("fixture"));
        assert_eq!(manifest.dependencies["serde"], "1");
        assert!(manifest.dependencies["local-helper"].contains("path"));
        assert!(manifest.dependencies["remote-helper"].contains("git"));
        assert!(manifest.dev_dependencies["axum"].contains("\"0.7\""));
        assert_eq!(manifest.build_dependencies["cc"], "1");
    }
}
