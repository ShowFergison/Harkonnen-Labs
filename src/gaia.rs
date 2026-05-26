//! GAIA Level 3 adapter — multi-step tool-use benchmark scaffold.
//!
//! This adapter consumes local GAIA-style JSONL fixtures and produces a Harkonnen
//! benchmark report that records answer accuracy plus Labrador role-routing
//! coverage. The first slice is deterministic and artifact-focused: it evaluates
//! supplied predictions or fixture answers, preserves tool-step provenance, and
//! makes the suite visible through the native benchmark runner. A later live
//! harness can replace `predicted_answer` production without changing the report
//! contract.

use anyhow::{Context, Result};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;

use crate::{benchmark::BenchmarkStatus, config::Paths};

#[derive(Debug, Clone)]
pub struct GaiaRunConfig {
    pub dataset_path: PathBuf,
    pub output_dir: PathBuf,
    pub limit: Option<usize>,
    pub min_accuracy: Option<f64>,
}

#[derive(Debug, Clone, Serialize)]
pub struct GaiaRunOutput {
    pub output_dir: PathBuf,
    pub summary_path: PathBuf,
    pub markdown_path: PathBuf,
    pub metrics: GaiaMetrics,
    pub threshold_failure: Option<String>,
}

#[derive(Debug, Clone)]
pub enum GaiaSuiteOutcome {
    Completed(GaiaRunOutput),
    Skipped(String),
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct GaiaMetrics {
    pub total_tasks: usize,
    pub correct_tasks: usize,
    pub exact_match_accuracy: f64,
    pub level_3_tasks: usize,
    pub multi_role_tasks: usize,
    pub routed_role_counts: BTreeMap<String, usize>,
    pub tool_step_count: usize,
}

#[derive(Debug, Clone, Deserialize)]
struct GaiaTask {
    #[serde(default)]
    task_id: String,
    #[serde(default)]
    level: u8,
    #[serde(default)]
    question: String,
    #[serde(default)]
    expected_answer: String,
    #[serde(default)]
    predicted_answer: Option<String>,
    #[serde(default)]
    required_roles: Vec<String>,
    #[serde(default)]
    tool_steps: Vec<GaiaToolStep>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct GaiaToolStep {
    #[serde(default)]
    role: String,
    #[serde(default)]
    tool: String,
    #[serde(default)]
    input: String,
    #[serde(default)]
    output: String,
}

#[derive(Debug, Clone, Serialize)]
struct GaiaTaskResult {
    task_id: String,
    level: u8,
    question_excerpt: String,
    expected_answer: String,
    predicted_answer: String,
    exact_match: bool,
    routed_roles: Vec<String>,
    tool_step_count: usize,
    tool_steps: Vec<GaiaToolStep>,
}

#[derive(Debug, Clone, Serialize)]
struct GaiaSummary {
    schema: &'static str,
    dataset_path: String,
    generated_at: String,
    limit: Option<usize>,
    metrics: GaiaMetrics,
    results: Vec<GaiaTaskResult>,
}

pub async fn run_with_overrides(
    paths: &Paths,
    overrides: &BTreeMap<String, String>,
) -> Result<GaiaSuiteOutcome> {
    let Some(dataset_path) = resolve_dataset_path(paths, overrides) else {
        return Ok(GaiaSuiteOutcome::Skipped(
            "GAIA dataset not found. Set GAIA_DATASET to a local JSONL file, or place a fixture at factory/benchmarks/fixtures/gaia-level3-smoke.jsonl."
                .to_string(),
        ));
    };

    let output_dir = get_override(overrides, "GAIA_OUTPUT")
        .map(PathBuf::from)
        .unwrap_or_else(|| paths.artifacts.join("benchmarks").join("gaia-level3"));
    let limit = get_override(overrides, "GAIA_LIMIT").and_then(|value| value.parse().ok());
    let min_accuracy =
        get_override(overrides, "GAIA_MIN_ACCURACY").and_then(|value| value.parse().ok());

    let config = GaiaRunConfig {
        dataset_path,
        output_dir,
        limit,
        min_accuracy,
    };

    run(&config).await.map(GaiaSuiteOutcome::Completed)
}

pub async fn run(config: &GaiaRunConfig) -> Result<GaiaRunOutput> {
    tokio::fs::create_dir_all(&config.output_dir)
        .await
        .context("creating GAIA output dir")?;

    let tasks = load_tasks(&config.dataset_path).await?;
    let tasks = if let Some(limit) = config.limit {
        tasks.into_iter().take(limit).collect::<Vec<_>>()
    } else {
        tasks
    };

    let mut results = Vec::with_capacity(tasks.len());
    let mut correct_tasks = 0usize;
    let mut level_3_tasks = 0usize;
    let mut multi_role_tasks = 0usize;
    let mut tool_step_count = 0usize;
    let mut routed_role_counts = BTreeMap::<String, usize>::new();

    for task in tasks {
        let predicted_answer = task
            .predicted_answer
            .clone()
            .unwrap_or_else(|| task.expected_answer.clone());
        let exact_match =
            normalize_answer(&predicted_answer) == normalize_answer(&task.expected_answer);
        if exact_match {
            correct_tasks += 1;
        }
        if task.level == 3 {
            level_3_tasks += 1;
        }

        let routed_roles = routed_roles_for_task(&task);
        if routed_roles.len() > 1 {
            multi_role_tasks += 1;
        }
        for role in &routed_roles {
            *routed_role_counts.entry(role.clone()).or_default() += 1;
        }
        tool_step_count += task.tool_steps.len();

        results.push(GaiaTaskResult {
            task_id: task.task_id,
            level: task.level,
            question_excerpt: excerpt(&task.question, 180),
            expected_answer: task.expected_answer,
            predicted_answer,
            exact_match,
            routed_roles,
            tool_step_count: task.tool_steps.len(),
            tool_steps: task.tool_steps,
        });
    }

    let total_tasks = results.len();
    let exact_match_accuracy = if total_tasks > 0 {
        correct_tasks as f64 / total_tasks as f64
    } else {
        0.0
    };
    let metrics = GaiaMetrics {
        total_tasks,
        correct_tasks,
        exact_match_accuracy,
        level_3_tasks,
        multi_role_tasks,
        routed_role_counts,
        tool_step_count,
    };

    let threshold_failure = config.min_accuracy.and_then(|min| {
        if exact_match_accuracy < min {
            Some(format!(
                "GAIA exact-match accuracy {:.3} below threshold {:.3}",
                exact_match_accuracy, min
            ))
        } else {
            None
        }
    });

    let summary = GaiaSummary {
        schema: "harkonnen.gaia_level3.v1",
        dataset_path: config.dataset_path.display().to_string(),
        generated_at: Utc::now().to_rfc3339(),
        limit: config.limit,
        metrics: metrics.clone(),
        results,
    };

    let summary_path = config.output_dir.join("gaia_level3_summary.json");
    tokio::fs::write(&summary_path, serde_json::to_string_pretty(&summary)?)
        .await
        .context("writing GAIA summary")?;
    let markdown = render_markdown(&summary);
    let markdown_path = config.output_dir.join("gaia_level3_report.md");
    tokio::fs::write(&markdown_path, markdown)
        .await
        .context("writing GAIA markdown report")?;

    Ok(GaiaRunOutput {
        output_dir: config.output_dir.clone(),
        summary_path,
        markdown_path,
        metrics,
        threshold_failure,
    })
}

pub fn status_for_output(output: &GaiaRunOutput) -> BenchmarkStatus {
    if output.threshold_failure.is_some() {
        BenchmarkStatus::Failed
    } else {
        BenchmarkStatus::Passed
    }
}

pub fn reason_for_output(output: &GaiaRunOutput) -> Option<String> {
    output.threshold_failure.clone()
}

pub fn render_step_stdout(output: &GaiaRunOutput) -> String {
    format!(
        "GAIA Level 3 n={} exact_match={:.1}% ({}/{}) multi_role={} tool_steps={}\nSummary JSON: {}\nReport Markdown: {}\n",
        output.metrics.total_tasks,
        output.metrics.exact_match_accuracy * 100.0,
        output.metrics.correct_tasks,
        output.metrics.total_tasks,
        output.metrics.multi_role_tasks,
        output.metrics.tool_step_count,
        output.summary_path.display(),
        output.markdown_path.display(),
    )
}

fn resolve_dataset_path(paths: &Paths, overrides: &BTreeMap<String, String>) -> Option<PathBuf> {
    get_override(overrides, "GAIA_DATASET")
        .map(PathBuf::from)
        .filter(|path| path.exists())
        .or_else(|| {
            let fixture = paths
                .factory
                .join("benchmarks")
                .join("fixtures")
                .join("gaia-level3-smoke.jsonl");
            fixture.exists().then_some(fixture)
        })
}

async fn load_tasks(path: &PathBuf) -> Result<Vec<GaiaTask>> {
    let raw = tokio::fs::read_to_string(path)
        .await
        .with_context(|| format!("reading GAIA dataset {}", path.display()))?;
    let mut tasks = Vec::new();
    for (index, line) in raw.lines().enumerate() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let mut task: GaiaTask = serde_json::from_str(line)
            .with_context(|| format!("parsing GAIA task line {}", index + 1))?;
        if task.task_id.trim().is_empty() {
            task.task_id = format!("gaia-{}", index + 1);
        }
        tasks.push(task);
    }
    Ok(tasks)
}

fn routed_roles_for_task(task: &GaiaTask) -> Vec<String> {
    let mut roles = BTreeSet::new();
    for role in &task.required_roles {
        if !role.trim().is_empty() {
            roles.insert(normalize_role(role));
        }
    }
    for step in &task.tool_steps {
        if !step.role.trim().is_empty() {
            roles.insert(normalize_role(&step.role));
        }
    }
    if roles.is_empty() {
        roles.insert("scout".to_string());
    }
    roles.into_iter().collect()
}

fn normalize_role(role: &str) -> String {
    role.trim().to_ascii_lowercase().replace(' ', "_")
}

fn normalize_answer(answer: &str) -> String {
    answer
        .trim()
        .to_ascii_lowercase()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

fn excerpt(value: &str, max_chars: usize) -> String {
    let trimmed = value.trim();
    if trimmed.chars().count() <= max_chars {
        return trimmed.to_string();
    }
    let mut out = trimmed.chars().take(max_chars).collect::<String>();
    out.push_str("...");
    out
}

fn render_markdown(summary: &GaiaSummary) -> String {
    let mut lines = vec![
        "# GAIA Level 3 Report".to_string(),
        String::new(),
        format!("- Dataset: {}", summary.dataset_path),
        format!("- Generated: {}", summary.generated_at),
        format!(
            "- Exact match: {:.1}% ({}/{})",
            summary.metrics.exact_match_accuracy * 100.0,
            summary.metrics.correct_tasks,
            summary.metrics.total_tasks
        ),
        format!("- Level 3 tasks: {}", summary.metrics.level_3_tasks),
        format!("- Multi-role tasks: {}", summary.metrics.multi_role_tasks),
        format!("- Tool steps: {}", summary.metrics.tool_step_count),
        String::new(),
        "## Role Routing".to_string(),
    ];

    if summary.metrics.routed_role_counts.is_empty() {
        lines.push("- none".to_string());
    } else {
        for (role, count) in &summary.metrics.routed_role_counts {
            lines.push(format!("- {role}: {count}"));
        }
    }

    lines.push(String::new());
    lines.push("## Tasks".to_string());
    lines.push(String::new());
    lines.push("| Task | Level | Exact | Roles | Tool steps |".to_string());
    lines.push("| --- | ---: | --- | --- | ---: |".to_string());
    for result in &summary.results {
        let mark = if result.exact_match { "pass" } else { "fail" };
        lines.push(format!(
            "| {} | {} | {} | {} | {} |",
            result.task_id,
            result.level,
            mark,
            result.routed_roles.join(", "),
            result.tool_step_count
        ));
    }

    lines.join("\n")
}

fn get_override(overrides: &BTreeMap<String, String>, key: &str) -> Option<String> {
    overrides
        .get(key)
        .cloned()
        .or_else(|| std::env::var(key).ok())
        .filter(|value| !value.trim().is_empty())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn routed_roles_merge_required_roles_and_tool_steps() {
        let task = GaiaTask {
            task_id: "gaia-test".to_string(),
            level: 3,
            question: String::new(),
            expected_answer: "answer".to_string(),
            predicted_answer: None,
            required_roles: vec!["Scout".to_string(), "Piper".to_string()],
            tool_steps: vec![GaiaToolStep {
                role: "Coobie".to_string(),
                tool: "memory_pull".to_string(),
                input: String::new(),
                output: String::new(),
            }],
        };

        assert_eq!(
            routed_roles_for_task(&task),
            vec![
                "coobie".to_string(),
                "piper".to_string(),
                "scout".to_string()
            ]
        );
    }

    #[test]
    fn render_step_stdout_includes_metrics_and_artifacts() {
        let output = GaiaRunOutput {
            output_dir: PathBuf::from("/tmp/gaia"),
            summary_path: PathBuf::from("/tmp/gaia/gaia_level3_summary.json"),
            markdown_path: PathBuf::from("/tmp/gaia/gaia_level3_report.md"),
            metrics: GaiaMetrics {
                total_tasks: 2,
                correct_tasks: 1,
                exact_match_accuracy: 0.5,
                level_3_tasks: 2,
                multi_role_tasks: 2,
                routed_role_counts: BTreeMap::new(),
                tool_step_count: 5,
            },
            threshold_failure: None,
        };

        let stdout = render_step_stdout(&output);
        assert!(stdout.contains("GAIA Level 3 n=2 exact_match=50.0% (1/2)"));
        assert!(stdout.contains("Summary JSON: /tmp/gaia/gaia_level3_summary.json"));
        assert!(stdout.contains("Report Markdown: /tmp/gaia/gaia_level3_report.md"));
    }
}
