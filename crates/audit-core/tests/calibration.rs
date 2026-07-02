use std::{
    fs,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

use rta_core::audit_repository;

#[test]
fn calibration_orders_repositories_by_structural_maturity() {
    let fixture_root = unique_fixture_root("ordering");
    let toy = fixture_root.join("toy-clean");
    let monolith = fixture_root.join("healthy-monolith");
    let workspace = fixture_root.join("mature-workspace");
    let disaster = fixture_root.join("disaster");

    write_toy_clean(&toy);
    write_healthy_monolith(&monolith);
    write_mature_workspace(&workspace);
    write_disaster(&disaster);

    let toy_report = audit_repository(&toy).expect("toy fixture should audit");
    let monolith_report = audit_repository(&monolith).expect("monolith fixture should audit");
    let workspace_report = audit_repository(&workspace).expect("workspace fixture should audit");
    let disaster_report = audit_repository(&disaster).expect("disaster fixture should audit");
    let toy_score = toy_report.overall_score;
    let monolith_score = monolith_report.overall_score;
    let workspace_score = workspace_report.overall_score;
    let disaster_score = disaster_report.overall_score;

    eprintln!(
        "calibration scores: disaster={disaster_score}, toy={toy_score}, monolith={monolith_score}, workspace={workspace_score}"
    );
    eprintln!(
        "toy components: dep={}, code={}, arch={}, test={}, risk={}",
        toy_report.dependencies.score,
        toy_report.code_quality.score,
        toy_report.architecture.score,
        toy_report.testing.score,
        toy_report.risks.score
    );
    eprintln!(
        "monolith components: dep={}, code={}, arch={}, test={}, risk={}",
        monolith_report.dependencies.score,
        monolith_report.code_quality.score,
        monolith_report.architecture.score,
        monolith_report.testing.score,
        monolith_report.risks.score
    );
    eprintln!(
        "workspace components: dep={}, code={}, arch={}, test={}, risk={}",
        workspace_report.dependencies.score,
        workspace_report.code_quality.score,
        workspace_report.architecture.score,
        workspace_report.testing.score,
        workspace_report.risks.score
    );

    assert!(
        disaster_score < toy_score,
        "disaster={disaster_score}, toy={toy_score}"
    );
    assert!(
        toy_score <= monolith_score,
        "toy={toy_score}, monolith={monolith_score}"
    );
    assert!(
        monolith_score <= workspace_score,
        "monolith={monolith_score}, workspace={workspace_score}"
    );
    assert!(
        (65..=85).contains(&workspace_score),
        "mature workspace should land in CTO-review band, got {workspace_score}"
    );

    fs::remove_dir_all(&fixture_root).expect("fixture cleanup should succeed");
}

fn write_toy_clean(root: &Path) {
    write_file(
        root.join("Cargo.toml"),
        r#"
[package]
name = "toy-clean"
version = "0.1.0"
edition = "2021"
"#,
    );
    write_file(
        root.join("src/lib.rs"),
        "pub mod parser;\npub mod runner;\n",
    );
    write_file(
        root.join("src/parser.rs"),
        "pub fn parse(input: &str) -> usize { input.len() }\n",
    );
    write_file(
        root.join("src/runner.rs"),
        "pub fn run() -> usize { crate::parser::parse(\"ok\") }\n",
    );
}

fn write_healthy_monolith(root: &Path) {
    write_file(
        root.join("Cargo.toml"),
        r#"
[package]
name = "healthy-monolith"
version = "0.1.0"
edition = "2021"
"#,
    );
    let mut lib = String::new();
    for index in 0..12 {
        lib.push_str(&format!("pub mod module_{index};\n"));
        write_file(
            root.join(format!("src/module_{index}.rs")),
            generated_module(18, 8, &format!("module_{index}")),
        );
    }
    lib.push_str("#[cfg(test)]\nmod tests {\n#[test]\nfn unit() {}\n}\n");
    write_file(root.join("src/lib.rs"), lib);
    write_file(
        root.join("tests/smoke.rs"),
        "#[test]\nfn smoke() { assert!(true); }\n",
    );
}

fn write_mature_workspace(root: &Path) {
    write_file(
        root.join("Cargo.toml"),
        r#"
[workspace]
members = ["crates/core", "crates/runtime", "crates/net", "crates/macros"]
resolver = "2"
"#,
    );

    for crate_name in ["core", "runtime", "net", "macros"] {
        write_file(
            root.join(format!("crates/{crate_name}/Cargo.toml")),
            format!(
                r#"
[package]
name = "mature-{crate_name}"
version = "0.1.0"
edition = "2021"
"#
            ),
        );
        let mut lib = String::new();
        for index in 0..6 {
            lib.push_str(&format!("pub mod module_{index};\n"));
            write_file(
                root.join(format!("crates/{crate_name}/src/module_{index}.rs")),
                generated_module(22, 12, &format!("{crate_name}_{index}")),
            );
        }
        lib.push_str("#[cfg(test)]\nmod tests {\n#[test]\nfn unit() {}\n}\n");
        write_file(root.join(format!("crates/{crate_name}/src/lib.rs")), lib);
    }
    write_file(
        root.join("tests/workspace_smoke.rs"),
        "#[test]\nfn smoke() { assert!(true); }\n",
    );
}

fn write_disaster(root: &Path) {
    write_file(
        root.join("Cargo.toml"),
        r#"
[package]
name = "disaster"
version = "0.1.0"
edition = "2021"

[dependencies]
serde = "0.1"
tokio = "*"
local-helper = { path = "../missing-helper" }
"#,
    );

    let mut lib = String::new();
    for index in 0..30 {
        lib.push_str(&format!("pub mod module_{index};\n"));
    }
    write_file(root.join("src/lib.rs"), lib);
    write_file(
        root.join("src/module_0.rs"),
        format!(
            "use crate::module_1;\n{}",
            generated_module(90, 20, "bad_0")
        ),
    );
    write_file(
        root.join("src/module_1.rs"),
        format!(
            "use crate::module_0;\n{}",
            generated_module(90, 20, "bad_1")
        ),
    );
    for index in 2..30 {
        write_file(
            root.join(format!("src/module_{index}.rs")),
            generated_module(45, 16, &format!("bad_{index}")),
        );
    }
}

fn generated_module(functions: usize, branches_per_function: usize, prefix: &str) -> String {
    let mut content = String::new();
    for function in 0..functions {
        content.push_str(&format!(
            "pub fn {prefix}_{function}(value: usize) -> usize {{\n"
        ));
        content.push_str("    let mut out = value;\n");
        for branch in 0..branches_per_function {
            content.push_str(&format!(
                "    if out % {} == 0 {{ out += {}; }}\n",
                branch + 2,
                branch + 1
            ));
        }
        content.push_str("    out\n}\n\n");
    }
    content
}

fn write_file(path: PathBuf, content: impl AsRef<str>) {
    let parent = path.parent().expect("fixture path should have a parent");
    fs::create_dir_all(parent).expect("fixture directory should be created");
    fs::write(path, content.as_ref()).expect("fixture file should be written");
}

fn unique_fixture_root(name: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock should be after epoch")
        .as_nanos();
    std::env::temp_dir().join(format!(
        "rta-calibration-{name}-{}-{nanos}",
        std::process::id()
    ))
}
