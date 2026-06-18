use std::{env, fs, path::PathBuf};

use rta_core::{
    audit_repository,
    json::{render_json, render_scorecard_json},
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

fn render_summary(report: &rta_core::model::AuditReport) -> String {
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
    "Usage:\n  rta [PATH] [--markdown|--json|--summary] [--output FILE]\n  rta scorecard [PATH] --json [--output FILE]\n\nExamples:\n  rta . --summary\n  rta ./service --json\n  rta ./service --markdown --output audit-report.md\n  rta scorecard ./service --json"
}

#[cfg(test)]
mod tests {
    use super::{parse_args, Command, OutputFormat};

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
}
