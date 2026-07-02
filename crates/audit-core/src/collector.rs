use std::{
    collections::BTreeMap,
    fs,
    io::Read,
    path::{Path, PathBuf},
    process::{Command, Stdio},
    thread,
    time::{Duration, Instant},
};

use cargo_metadata::DependencyKind;
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

        let parsed_manifests = files
            .iter()
            .filter(|file| file.relative_path.ends_with("Cargo.toml"))
            .filter_map(|file| file.content.as_deref().map(|content| (file, content)))
            .map(|(file, content)| CargoManifest::parse(&file.relative_path, content))
            .collect::<Result<Vec<_>, _>>()?;
        let manifests = collect_metadata_manifests(&root).unwrap_or(parsed_manifests);

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

fn collect_metadata_manifests(root: &Path) -> Option<Vec<CargoManifest>> {
    if std::env::var_os("RTA_DISABLE_CARGO_METADATA").is_some() {
        return None;
    }
    let manifest_path = root.join("Cargo.toml");
    if !manifest_path.exists() {
        return None;
    }

    let metadata = cargo_metadata_with_timeout(root, &manifest_path)?;

    let workspace_members = metadata
        .workspace_members
        .iter()
        .filter_map(|member_id| {
            metadata
                .packages
                .iter()
                .find(|package| &package.id == member_id)
        })
        .filter_map(|package| relative_manifest_path(root, package.manifest_path.as_std_path()))
        .map(|relative| relative.trim_end_matches("/Cargo.toml").to_string())
        .collect::<Vec<_>>();
    let mut manifests = Vec::new();

    for package in metadata.packages {
        let relative_path = relative_manifest_path(root, package.manifest_path.as_std_path())?;
        let mut manifest = CargoManifest {
            relative_path,
            package_name: Some(package.name.to_string()),
            workspace_members: Vec::new(),
            dependencies: BTreeMap::new(),
            dev_dependencies: BTreeMap::new(),
            build_dependencies: BTreeMap::new(),
        };

        if package.manifest_path.as_std_path() == manifest_path {
            manifest.workspace_members = workspace_members.clone();
        }

        for dependency in package.dependencies {
            let value = dependency_value_from_metadata(&dependency);
            match dependency.kind {
                DependencyKind::Normal => {
                    manifest.dependencies.insert(dependency.name, value);
                }
                DependencyKind::Development => {
                    manifest.dev_dependencies.insert(dependency.name, value);
                }
                DependencyKind::Build => {
                    manifest.build_dependencies.insert(dependency.name, value);
                }
                _ => {}
            }
        }

        manifests.push(manifest);
    }

    Some(manifests)
}

fn cargo_metadata_with_timeout(
    root: &Path,
    manifest_path: &Path,
) -> Option<cargo_metadata::Metadata> {
    let mut child = Command::new("cargo")
        .arg("metadata")
        .arg("--no-deps")
        .arg("--format-version")
        .arg("1")
        .arg("--manifest-path")
        .arg(manifest_path)
        .current_dir(root)
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .ok()?;
    let started = Instant::now();
    loop {
        match child.try_wait().ok()? {
            Some(status) if status.success() => break,
            Some(_) => return None,
            None if started.elapsed() > Duration::from_secs(5) => {
                let _ = child.kill();
                let _ = child.wait();
                return None;
            }
            None => thread::sleep(Duration::from_millis(50)),
        }
    }

    let mut output = String::new();
    child.stdout.take()?.read_to_string(&mut output).ok()?;
    serde_json::from_str(&output).ok()
}

fn dependency_value_from_metadata(dependency: &cargo_metadata::Dependency) -> String {
    if let Some(path) = &dependency.path {
        return format!("path = {}", path);
    }
    if let Some(source) = &dependency.source {
        return format!("{} ({source})", dependency.req);
    }
    dependency.req.to_string()
}

fn relative_manifest_path(root: &Path, manifest_path: &Path) -> Option<String> {
    Some(
        manifest_path
            .strip_prefix(root)
            .ok()?
            .to_string_lossy()
            .replace('\\', "/"),
    )
}

impl CargoManifest {
    fn parse(relative_path: &str, content: &str) -> Result<Self, String> {
        let parsed = content
            .parse::<Value>()
            .map_err(|err| format!("failed to parse {relative_path}: {err}"))?;
        let package_name = parsed
            .get("package")
            .and_then(|package| package.get("name"))
            .and_then(Value::as_str)
            .map(str::to_string);
        let workspace_members = parsed
            .get("workspace")
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
        let dependencies = parse_dependency_table(&parsed, "dependencies");
        let dev_dependencies = parse_dependency_table(&parsed, "dev-dependencies");
        let build_dependencies = parse_dependency_table(&parsed, "build-dependencies");

        Ok(Self {
            relative_path: relative_path.to_string(),
            package_name,
            workspace_members,
            dependencies,
            dev_dependencies,
            build_dependencies,
        })
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

        let metadata = fs::symlink_metadata(&path)
            .map_err(|err| format!("failed to stat {}: {err}", path.display()))?;
        if metadata.file_type().is_symlink() {
            continue;
        }
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
        let content = if should_read_content(extension.as_deref()) {
            read_text_file(&path, metadata.len())
        } else {
            None
        };
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

fn should_read_content(extension: Option<&str>) -> bool {
    matches!(extension, Some("rs" | "toml"))
}

fn parse_dependency_table(manifest: &Value, section: &str) -> BTreeMap<String, String> {
    manifest
        .get(section)
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
        )
        .expect("workspace manifest should parse");

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
        )
        .expect("dependency manifest should parse");

        assert_eq!(manifest.package_name.as_deref(), Some("fixture"));
        assert_eq!(manifest.dependencies["serde"], "1");
        assert!(manifest.dependencies["local-helper"].contains("path"));
        assert!(manifest.dependencies["remote-helper"].contains("git"));
        assert!(manifest.dev_dependencies["axum"].contains("\"0.7\""));
        assert_eq!(manifest.build_dependencies["cc"], "1");
    }

    #[test]
    fn reports_invalid_toml() {
        let err = CargoManifest::parse("Cargo.toml", "[dependencies")
            .expect_err("invalid TOML should be rejected");

        assert!(err.contains("failed to parse Cargo.toml"));
    }
}
