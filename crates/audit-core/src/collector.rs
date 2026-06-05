use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
};

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
        let mut package_name = None;
        let mut workspace_members = Vec::new();
        let mut dependencies = BTreeMap::new();
        let mut dev_dependencies = BTreeMap::new();
        let mut build_dependencies = BTreeMap::new();
        let mut section = String::new();

        for raw_line in content.lines() {
            let line = strip_comment(raw_line).trim();
            if line.is_empty() {
                continue;
            }
            if line.starts_with('[') && line.ends_with(']') {
                section = line.trim_matches(&['[', ']'][..]).to_string();
                continue;
            }
            if section == "package" && line.starts_with("name") {
                package_name = parse_value(line);
            } else if section == "workspace" && line.starts_with("members") {
                workspace_members.extend(parse_array(line));
            } else if is_dependency_section(&section) {
                if let Some((name, version)) = parse_dependency(line) {
                    match section.as_str() {
                        "dependencies" => {
                            dependencies.insert(name, version);
                        }
                        "dev-dependencies" => {
                            dev_dependencies.insert(name, version);
                        }
                        "build-dependencies" => {
                            build_dependencies.insert(name, version);
                        }
                        _ => {}
                    }
                }
            }
        }

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

fn strip_comment(line: &str) -> &str {
    line.split('#').next().unwrap_or(line)
}

fn parse_value(line: &str) -> Option<String> {
    line.split_once('=')
        .map(|(_, value)| value.trim().trim_matches('"').to_string())
        .filter(|value| !value.is_empty())
}

fn parse_array(line: &str) -> Vec<String> {
    line.split_once('=')
        .and_then(|(_, value)| value.split_once('[').map(|(_, rest)| rest))
        .and_then(|rest| rest.split_once(']').map(|(items, _)| items))
        .map(|items| {
            items
                .split(',')
                .map(|item| item.trim().trim_matches('"').to_string())
                .filter(|item| !item.is_empty())
                .collect()
        })
        .unwrap_or_default()
}

fn is_dependency_section(section: &str) -> bool {
    matches!(
        section,
        "dependencies" | "dev-dependencies" | "build-dependencies"
    )
}

fn parse_dependency(line: &str) -> Option<(String, String)> {
    let (name, raw_value) = line.split_once('=')?;
    let name = name.trim();
    if name.is_empty() {
        return None;
    }
    let version = raw_value
        .trim()
        .trim_matches('"')
        .trim_matches('{')
        .trim_matches('}')
        .to_string();
    Some((name.to_string(), version))
}
