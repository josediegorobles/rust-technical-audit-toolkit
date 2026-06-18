use std::{env, fs, path::PathBuf};

use rta_core::{
    audit_repository,
    json::{render_json, render_scorecard_json},
    model::AuditReport,
    pack::{
        render_evidence_json, render_methodology_markdown, render_review_questions_markdown,
        render_risk_register_json,
    },
    report::render_markdown,
};

#[derive(Debug, Clone, Copy)]
enum OutputFormat {
    Markdown,
    Json,
    Summary,
}

#[derive(Debug)]
struct Args {
    command: Command,
    path: PathBuf,
    format: OutputFormat,
    output: Option<PathBuf>,
}

#[derive(Debug, Clone, Copy)]
enum Command {
    Audit,
    Scorecard,
    AuditPack,
}

fn main() {
    match run() {
        Ok(()) => {}
        Err(err) => {
            eprintln!("error: {err}");
            eprintln!();
            eprintln!("{}", usage());
            std::process::exit(1);
        }
    }
}

fn run() -> Result<(), String> {
    let args = parse_args(env::args().skip(1))?;
    let report = audit_repository(&args.path)?;
    if matches!(args.command, Command::AuditPack) {
        let output_dir = args
            .output
            .as_deref()
            .ok_or_else(|| "audit-pack requires --output DIR".to_string())?;
        return write_audit_pack(&report, output_dir);
    }

    let rendered = match args.command {
        Command::Audit => match args.format {
            OutputFormat::Markdown => render_markdown(&report),
            OutputFormat::Json => render_json(&report),
            OutputFormat::Summary => render_summary(&report),
        },
        Command::Scorecard => match args.format {
            OutputFormat::Json => render_scorecard_json(&report),
            OutputFormat::Markdown | OutputFormat::Summary => {
                return Err("scorecard currently supports --json only".to_string());
            }
        },
        Command::AuditPack => unreachable!("audit-pack is handled before rendering"),
    };

    if let Some(path) = args.output {
        fs::write(&path, rendered)
            .map_err(|err| format!("failed to write {}: {err}", path.display()))?;
    } else {
        print!("{rendered}");
    }

    Ok(())
}

fn parse_args<I>(args: I) -> Result<Args, String>
where
    I: IntoIterator<Item = String>,
{
    let mut path = PathBuf::from(".");
    let mut format = OutputFormat::Markdown;
    let mut command = Command::Audit;
    let mut output = None;
    let mut positional_path_seen = false;
    let mut iter = args.into_iter();

    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "-h" | "--help" => return Err(usage().to_string()),
            "scorecard" if !positional_path_seen && matches!(command, Command::Audit) => {
                command = Command::Scorecard;
                format = OutputFormat::Json;
            }
            "audit-pack" if !positional_path_seen && matches!(command, Command::Audit) => {
                command = Command::AuditPack;
            }
            "--json" => format = OutputFormat::Json,
            "--markdown" => format = OutputFormat::Markdown,
            "--summary" => format = OutputFormat::Summary,
            "-o" | "--output" => {
                let value = iter
                    .next()
                    .ok_or_else(|| "--output requires a path".to_string())?;
                output = Some(PathBuf::from(value));
            }
            value if value.starts_with('-') => return Err(format!("unknown argument `{value}`")),
            value => {
                if positional_path_seen {
                    return Err(format!("unexpected positional argument `{value}`"));
                }
                path = PathBuf::from(value);
                positional_path_seen = true;
            }
        }
    }

    Ok(Args {
        command,
        path,
        format,
        output,
    })
}

fn write_audit_pack(report: &AuditReport, output_dir: &std::path::Path) -> Result<(), String> {
    fs::create_dir_all(output_dir)
        .map_err(|err| format!("failed to create {}: {err}", output_dir.display()))?;

    let files = [
        ("executive-report.md", render_markdown(report)),
        ("scorecard.json", render_scorecard_json(report)),
        ("evidence.json", render_evidence_json(report)),
        ("risk-register.json", render_risk_register_json(report)),
        (
            "review-questions.md",
            render_review_questions_markdown(report),
        ),
        ("methodology.md", render_methodology_markdown()),
    ];

    for (name, content) in files {
        let path = output_dir.join(name);
        fs::write(&path, content)
            .map_err(|err| format!("failed to write {}: {err}", path.display()))?;
    }

    Ok(())
}

fn render_summary(report: &AuditReport) -> String {
    format!(
        concat!(
            "Rust Technical Audit Toolkit\n",
            "Repository: {}\n",
            "Overall score: {}/100\n",
            "Crates: {}\n",
            "Dependencies: {} direct\n",
            "Maintainability: {}/100\n",
            "Architecture: {}/100\n",
            "Testing: {}/100\n",
            "Risks: {} finding(s)\n"
        ),
        report.repository_path,
        report.overall_score,
        report.overview.crate_count,
        report.dependencies.direct_dependencies,
        report.code_quality.score,
        report.architecture.score,
        report.testing.score,
        report.risks.findings.len()
    )
}

fn usage() -> &'static str {
    "Usage:\n  rta [PATH] [--markdown|--json|--summary] [--output FILE]\n  rta scorecard [PATH] --json [--output FILE]\n  rta audit-pack [PATH] --output DIR\n\nExamples:\n  rta . --summary\n  rta ./service --json\n  rta ./service --markdown --output audit-report.md\n  rta scorecard ./service --json\n  rta audit-pack ./service --output audit-pack"
}

#[cfg(test)]
mod tests {
    use std::{
        fs,
        path::{Path, PathBuf},
        time::{SystemTime, UNIX_EPOCH},
    };

    use rta_core::audit_repository;
    use serde_json::Value;

    use super::{parse_args, write_audit_pack, Command, OutputFormat};

    #[test]
    fn parses_json_output() {
        let args = match parse_args(["repo".to_string(), "--json".to_string()]) {
            Ok(args) => args,
            Err(err) => panic!("args parse failed: {err}"),
        };
        assert!(matches!(args.format, OutputFormat::Json));
        assert_eq!(args.path.to_string_lossy(), "repo");
    }

    #[test]
    fn parses_scorecard_command() {
        let args = match parse_args([
            "scorecard".to_string(),
            "repo".to_string(),
            "--json".to_string(),
        ]) {
            Ok(args) => args,
            Err(err) => panic!("args parse failed: {err}"),
        };
        assert!(matches!(args.command, Command::Scorecard));
        assert!(matches!(args.format, OutputFormat::Json));
        assert_eq!(args.path.to_string_lossy(), "repo");
    }

    #[test]
    fn parses_audit_pack_command() {
        let args = match parse_args([
            "audit-pack".to_string(),
            "repo".to_string(),
            "--output".to_string(),
            "pack".to_string(),
        ]) {
            Ok(args) => args,
            Err(err) => panic!("args parse failed: {err}"),
        };
        assert!(matches!(args.command, Command::AuditPack));
        assert_eq!(args.path.to_string_lossy(), "repo");
        assert_eq!(
            args.output
                .as_ref()
                .expect("audit-pack output should parse")
                .to_string_lossy(),
            "pack"
        );
    }

    #[test]
    fn writes_audit_pack_files_with_valid_json_and_markdown() {
        let report = match audit_repository(&sample_service_path()) {
            Ok(report) => report,
            Err(err) => panic!("sample audit failed: {err}"),
        };
        let output_dir = unique_output_dir();
        if output_dir.exists() {
            fs::remove_dir_all(&output_dir).expect("stale test output should be removable");
        }

        write_audit_pack(&report, &output_dir).expect("audit pack should write");

        for file in [
            "executive-report.md",
            "scorecard.json",
            "evidence.json",
            "risk-register.json",
            "review-questions.md",
            "methodology.md",
        ] {
            assert!(
                output_dir.join(file).is_file(),
                "audit pack should create {file}"
            );
        }

        let scorecard = assert_valid_json(&output_dir.join("scorecard.json"));
        assert_eq!(scorecard["schema_version"], "rta.scorecard.v1");
        let evidence = assert_valid_json(&output_dir.join("evidence.json"));
        assert_eq!(evidence["schema_version"], "rta.evidence.v1");
        let risk_register = assert_valid_json(&output_dir.join("risk-register.json"));
        assert_eq!(risk_register["schema_version"], "rta.risk-register.v1");

        let executive_report =
            fs::read_to_string(output_dir.join("executive-report.md")).expect("read report");
        assert!(executive_report.contains("## Executive Summary"));
        assert!(executive_report.contains("## Architecture"));
        assert!(executive_report.contains("## Dependency Health"));
        assert!(executive_report.contains("## Testing"));
        assert!(executive_report.contains("## Risks"));

        let review_questions =
            fs::read_to_string(output_dir.join("review-questions.md")).expect("read questions");
        assert!(review_questions.contains("# Review Questions"));
        assert!(review_questions.contains("Testing scored"));

        let methodology =
            fs::read_to_string(output_dir.join("methodology.md")).expect("read methodology");
        assert!(methodology.contains("# Methodology"));
        assert!(methodology.contains("No AI analysis"));

        fs::remove_dir_all(&output_dir).expect("test output should be removable");
    }

    fn assert_valid_json(path: &Path) -> Value {
        let content = fs::read_to_string(path).expect("JSON file should be readable");
        serde_json::from_str(&content).expect("JSON file should parse")
    }

    fn sample_service_path() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../examples/sample-rust-service")
    }

    fn unique_output_dir() -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock should be after UNIX_EPOCH")
            .as_nanos();
        std::env::temp_dir().join(format!(
            "rta-audit-pack-test-{}-{nanos}",
            std::process::id()
        ))
    }
}
