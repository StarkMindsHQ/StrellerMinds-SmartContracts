use progress::{Progress, ProgressClient};
use serde::Serialize;
use soroban_sdk::{
    Address, Env, Symbol,
    testutils::{Address as _, Events as _},
};
use std::cmp::Ordering;
use std::collections::HashSet;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{Instant, SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone)]
struct LoadConfig {
    report_path: PathBuf,
    summary_path: PathBuf,
    peak_load: usize,
    load_multiplier: usize,
    student_pool: usize,
    course_count: usize,
    read_multiplier: usize,
    ci_mode: bool,
}

impl LoadConfig {
    fn from_env_and_args() -> Result<Self, String> {
        let mut report_path = PathBuf::from("target/load-test-report.json");
        let mut summary_path = PathBuf::from("target/load-test-summary.md");
        let mut ci_mode = false;

        let mut args = env::args().skip(1);
        while let Some(arg) = args.next() {
            match arg.as_str() {
                "--report" => {
                    let value = args.next().ok_or("missing value for --report")?;
                    report_path = PathBuf::from(value);
                }
                "--summary" => {
                    let value = args.next().ok_or("missing value for --summary")?;
                    summary_path = PathBuf::from(value);
                }
                "--ci" => {
                    ci_mode = true;
                }
                "-h" | "--help" => {
                    print_help();
                    std::process::exit(0);
                }
                _ => {
                    return Err(format!("unknown argument: {arg}"));
                }
            }
        }

        let default_peak = if ci_mode { 5 } else { 25 };
        let default_students = if ci_mode { 20 } else { 50 };
        let default_courses = if ci_mode { 4 } else { 8 };
        let default_reads = if ci_mode { 2 } else { 3 };

        let peak_load = read_env_usize("STRELLER_PEAK_LOAD", default_peak)?;
        let load_multiplier = read_env_usize("STRELLER_LOAD_MULTIPLIER", 10)?;
        let student_pool = read_env_usize("STRELLER_STUDENT_POOL", default_students)?;
        let course_count = read_env_usize("STRELLER_COURSE_COUNT", default_courses)?;
        let read_multiplier = read_env_usize("STRELLER_READ_MULTIPLIER", default_reads)?;

        if peak_load == 0
            || load_multiplier == 0
            || student_pool == 0
            || course_count == 0
            || read_multiplier == 0
        {
            return Err("all load configuration values must be greater than zero".to_string());
        }

        Ok(Self {
            report_path,
            summary_path,
            peak_load,
            load_multiplier,
            student_pool,
            course_count,
            read_multiplier,
            ci_mode,
        })
    }

    fn total_write_ops(&self) -> usize {
        self.peak_load.saturating_mul(self.load_multiplier)
    }

    fn total_read_ops(&self) -> usize {
        self.total_write_ops().saturating_mul(self.read_multiplier)
    }
}

#[derive(Debug, Serialize)]
struct LoadReport {
    generated_at_unix: u64,
    config: ReportConfig,
    scenarios: Vec<ScenarioMetrics>,
    summary: SummaryMetrics,
    bottlenecks: Vec<String>,
}

#[derive(Debug, Serialize)]
struct ReportConfig {
    peak_load: usize,
    load_multiplier: usize,
    simulated_peak_multiple: usize,
    total_write_ops: usize,
    total_read_ops: usize,
    student_pool: usize,
    course_count: usize,
    ci_mode: bool,
}

#[derive(Debug, Serialize)]
struct SummaryMetrics {
    total_scenarios: usize,
    total_operations: usize,
    total_failures: usize,
    overall_throughput_ops_per_sec: f64,
    worst_p95_ms: f64,
    max_memory_delta_bytes: Option<i64>,
}

#[derive(Debug, Serialize)]
struct ScenarioMetrics {
    name: String,
    operation_kind: String,
    target_operations: usize,
    succeeded_operations: usize,
    failed_operations: usize,
    total_duration_ms: f64,
    avg_latency_ms: f64,
    p50_latency_ms: f64,
    p95_latency_ms: f64,
    max_latency_ms: f64,
    throughput_ops_per_sec: f64,
    emitted_events: usize,
    estimated_state_writes: usize,
    peak_rss_delta_bytes: Option<i64>,
    gas_or_budget_metrics: CostMetrics,
}

#[derive(Debug, Serialize)]
struct CostMetrics {
    budget_tracking_supported: bool,
    execution_cost_note: String,
}

#[derive(Debug, Clone, Copy)]
struct SeedPlan {
    student_pool: usize,
    course_count: usize,
}

struct ScenarioSeed {
    env: Env,
    client: ProgressClient<'static>,
    students: Vec<Address>,
    courses: Vec<Symbol>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config =
        LoadConfig::from_env_and_args().map_err(|err| format!("load config error: {err}"))?;
    let report_dir = config
        .report_path
        .parent()
        .map(Path::to_path_buf)
        .unwrap_or_else(|| PathBuf::from("target"));
    fs::create_dir_all(&report_dir)?;
    if let Some(summary_dir) = config.summary_path.parent() {
        fs::create_dir_all(summary_dir)?;
    }

    let write_seed =
        SeedPlan { student_pool: config.student_pool, course_count: config.course_count };

    let hot_write =
        run_progress_write_scenario("progress-write-hot-path", "write", &config, write_seed, 1);
    let multi_course_write = run_progress_write_scenario(
        "progress-write-multi-course",
        "write",
        &config,
        write_seed,
        config.course_count,
    );
    let read_heavy = run_progress_read_scenario("progress-read-heavy", "read", &config, write_seed);

    let scenarios = vec![hot_write, multi_course_write, read_heavy];
    let summary = summarize(&scenarios);
    let bottlenecks = generate_bottlenecks(&scenarios);

    let generated_at_unix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or(0);

    let report = LoadReport {
        generated_at_unix,
        config: ReportConfig {
            peak_load: config.peak_load,
            load_multiplier: config.load_multiplier,
            simulated_peak_multiple: config.load_multiplier,
            total_write_ops: config.total_write_ops(),
            total_read_ops: config.total_read_ops(),
            student_pool: config.student_pool,
            course_count: config.course_count,
            ci_mode: config.ci_mode,
        },
        scenarios,
        summary,
        bottlenecks,
    };

    write_report(&config.report_path, &report)?;
    write_summary(&config.summary_path, &report)?;
    print_console_report(&report, &config.report_path, &config.summary_path);

    Ok(())
}

fn print_help() {
    println!("StrellerMinds load-test runner");
    println!();
    println!("Usage:");
    println!("  cargo load-test -- [--ci] [--report <path>] [--summary <path>]");
    println!();
    println!("Environment variables:");
    println!(
        "  STRELLER_PEAK_LOAD        Baseline peak load before multiplier (default: 25, CI: 5)"
    );
    println!("  STRELLER_LOAD_MULTIPLIER  Peak multiplier to simulate (default: 10)");
    println!("  STRELLER_STUDENT_POOL     Number of simulated students (default: 50, CI: 20)");
    println!("  STRELLER_COURSE_COUNT     Number of simulated courses (default: 8, CI: 4)");
    println!("  STRELLER_READ_MULTIPLIER  Reads per seeded write operation (default: 3, CI: 2)");
}

fn read_env_usize(name: &str, default: usize) -> Result<usize, String> {
    match env::var(name) {
        Ok(value) => {
            value.parse::<usize>().map_err(|_| format!("{name} must be a positive integer"))
        }
        Err(_) => Ok(default),
    }
}

fn seed_progress_environment(plan: SeedPlan) -> ScenarioSeed {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(Progress, ());
    let client = ProgressClient::new(&env, &contract_id);
    let admin = Address::generate(&env);
    client.initialize(&admin);

    let students = (0..plan.student_pool).map(|_| Address::generate(&env)).collect::<Vec<_>>();
    let courses =
        (0..plan.course_count).map(|idx| Symbol::new(&env, &format!("C{idx}"))).collect::<Vec<_>>();

    ScenarioSeed { env, client, students, courses }
}

fn run_progress_write_scenario(
    scenario_name: &str,
    operation_kind: &str,
    config: &LoadConfig,
    plan: SeedPlan,
    active_course_count: usize,
) -> ScenarioMetrics {
    let seed = seed_progress_environment(plan);
    let mut latencies_ms = Vec::with_capacity(config.total_write_ops());
    let mut failed_operations = 0usize;
    let mut estimated_pairs = HashSet::new();
    let mut student_course_memberships = HashSet::new();
    let active_course_count = active_course_count.max(1).min(seed.courses.len());

    let rss_before = current_rss_bytes();
    let events_before = seed.env.events().all().len();
    let scenario_start = Instant::now();

    for idx in 0..config.total_write_ops() {
        let student_idx = idx % seed.students.len();
        let course_idx = idx % active_course_count;
        let progress_value = ((idx * 7) % 101) as u32;
        let start = Instant::now();
        let result = seed.client.try_record_progress(
            &seed.students[student_idx],
            &seed.courses[course_idx],
            &progress_value,
        );
        latencies_ms.push(duration_ms(start.elapsed()));

        if result.is_ok() {
            estimated_pairs.insert((student_idx, course_idx));
            student_course_memberships.insert((student_idx, course_idx));
        } else {
            failed_operations += 1;
        }
    }

    let total_duration = scenario_start.elapsed();
    let events_after = seed.env.events().all().len();
    let rss_after = current_rss_bytes();
    let succeeded_operations = config.total_write_ops().saturating_sub(failed_operations);

    ScenarioMetrics {
        name: scenario_name.to_string(),
        operation_kind: operation_kind.to_string(),
        target_operations: config.total_write_ops(),
        succeeded_operations,
        failed_operations,
        total_duration_ms: duration_ms(total_duration),
        avg_latency_ms: average(&latencies_ms),
        p50_latency_ms: percentile(&latencies_ms, 0.50),
        p95_latency_ms: percentile(&latencies_ms, 0.95),
        max_latency_ms: latencies_ms.iter().copied().fold(0.0, f64::max),
        throughput_ops_per_sec: throughput(succeeded_operations, total_duration),
        emitted_events: events_after.saturating_sub(events_before) as usize,
        estimated_state_writes: succeeded_operations * 2 + student_course_memberships.len(),
        peak_rss_delta_bytes: rss_after.zip(rss_before).map(|(after, before)| after - before),
        gas_or_budget_metrics: CostMetrics {
            budget_tracking_supported: false,
            execution_cost_note: format!(
                "Host-side Soroban test environment does not expose stable gas counters here; report tracks emitted events and estimated persistent writes for {} unique progress keys.",
                estimated_pairs.len()
            ),
        },
    }
}

fn run_progress_read_scenario(
    scenario_name: &str,
    operation_kind: &str,
    config: &LoadConfig,
    plan: SeedPlan,
) -> ScenarioMetrics {
    let seed = seed_progress_environment(plan);
    let mut seeded_pairs = Vec::with_capacity(config.total_write_ops());

    for idx in 0..config.total_write_ops() {
        let student_idx = idx % seed.students.len();
        let course_idx = idx % seed.courses.len();
        let progress_value = ((idx * 11) % 101) as u32;
        let _ = seed.client.try_record_progress(
            &seed.students[student_idx],
            &seed.courses[course_idx],
            &progress_value,
        );
        seeded_pairs.push((student_idx, course_idx));
    }

    let rss_before = current_rss_bytes();
    let events_before = seed.env.events().all().len();
    let scenario_start = Instant::now();
    let mut latencies_ms = Vec::with_capacity(config.total_read_ops());
    let mut failed_operations = 0usize;

    for idx in 0..config.total_read_ops() {
        let start = Instant::now();
        let success = if idx % 4 == 0 {
            let (student_idx, _) = seeded_pairs[idx % seeded_pairs.len()];
            seed.client.try_get_student_courses(&seed.students[student_idx]).is_ok()
        } else {
            let (student_idx, course_idx) = seeded_pairs[idx % seeded_pairs.len()];
            seed.client
                .try_get_progress(&seed.students[student_idx], &seed.courses[course_idx])
                .is_ok()
        };

        latencies_ms.push(duration_ms(start.elapsed()));
        if !success {
            failed_operations += 1;
        }
    }

    let total_duration = scenario_start.elapsed();
    let events_after = seed.env.events().all().len();
    let rss_after = current_rss_bytes();
    let succeeded_operations = config.total_read_ops().saturating_sub(failed_operations);

    ScenarioMetrics {
        name: scenario_name.to_string(),
        operation_kind: operation_kind.to_string(),
        target_operations: config.total_read_ops(),
        succeeded_operations,
        failed_operations,
        total_duration_ms: duration_ms(total_duration),
        avg_latency_ms: average(&latencies_ms),
        p50_latency_ms: percentile(&latencies_ms, 0.50),
        p95_latency_ms: percentile(&latencies_ms, 0.95),
        max_latency_ms: latencies_ms.iter().copied().fold(0.0, f64::max),
        throughput_ops_per_sec: throughput(succeeded_operations, total_duration),
        emitted_events: events_after.saturating_sub(events_before) as usize,
        estimated_state_writes: 0,
        peak_rss_delta_bytes: rss_after.zip(rss_before).map(|(after, before)| after - before),
        gas_or_budget_metrics: CostMetrics {
            budget_tracking_supported: false,
            execution_cost_note: "Read-heavy scenario uses host-side latency only; no persistent writes are expected during successful reads.".to_string(),
        },
    }
}

fn summarize(scenarios: &[ScenarioMetrics]) -> SummaryMetrics {
    let total_duration_secs =
        scenarios.iter().map(|scenario| scenario.total_duration_ms).sum::<f64>() / 1000.0;
    let total_operations = scenarios.iter().map(|scenario| scenario.target_operations).sum();
    let total_failures = scenarios.iter().map(|scenario| scenario.failed_operations).sum();
    let total_successes =
        scenarios.iter().map(|scenario| scenario.succeeded_operations).sum::<usize>();
    let worst_p95_ms = scenarios.iter().map(|scenario| scenario.p95_latency_ms).fold(0.0, f64::max);
    let max_memory_delta_bytes =
        scenarios.iter().filter_map(|scenario| scenario.peak_rss_delta_bytes).max();

    SummaryMetrics {
        total_scenarios: scenarios.len(),
        total_operations,
        total_failures,
        overall_throughput_ops_per_sec: if total_duration_secs > 0.0 {
            total_successes as f64 / total_duration_secs
        } else {
            0.0
        },
        worst_p95_ms,
        max_memory_delta_bytes,
    }
}

fn generate_bottlenecks(scenarios: &[ScenarioMetrics]) -> Vec<String> {
    let mut notes = Vec::new();

    for scenario in scenarios {
        if scenario.failed_operations > 0 {
            notes.push(format!(
                "{} recorded {} failed operations under load.",
                scenario.name, scenario.failed_operations
            ));
        }
        if scenario.p95_latency_ms > scenario.avg_latency_ms * 2.0 && scenario.avg_latency_ms > 0.0
        {
            notes.push(format!(
                "{} shows high tail latency (p95 {:.3}ms vs avg {:.3}ms).",
                scenario.name, scenario.p95_latency_ms, scenario.avg_latency_ms
            ));
        }
        if scenario.estimated_state_writes > scenario.succeeded_operations {
            notes.push(format!(
                "{} is storage-heavy with {} estimated writes across {} successful operations.",
                scenario.name, scenario.estimated_state_writes, scenario.succeeded_operations
            ));
        }
    }

    let mut write_scenarios =
        scenarios.iter().filter(|scenario| scenario.operation_kind == "write");
    let read_scenario = scenarios.iter().find(|scenario| scenario.operation_kind == "read");
    if let (Some(first_write), Some(read_scenario)) = (write_scenarios.next(), read_scenario) {
        if first_write.throughput_ops_per_sec > 0.0
            && read_scenario.throughput_ops_per_sec < first_write.throughput_ops_per_sec
        {
            notes.push(format!(
                "Read throughput ({:.2} ops/sec) is lower than write throughput ({:.2} ops/sec); inspect repeated course-list reads and symbol lookups.",
                read_scenario.throughput_ops_per_sec, first_write.throughput_ops_per_sec
            ));
        }
    }

    if notes.is_empty() {
        notes.push("No bottlenecks exceeded the current heuristic thresholds; keep baselining p95 latency and throughput over time.".to_string());
    }

    notes
}

fn write_report(path: &Path, report: &LoadReport) -> Result<(), Box<dyn std::error::Error>> {
    let json = serde_json::to_string_pretty(report)?;
    fs::write(path, json)?;
    Ok(())
}

fn write_summary(path: &Path, report: &LoadReport) -> Result<(), Box<dyn std::error::Error>> {
    let mut markdown = String::new();
    markdown.push_str("# Load Test Summary\n\n");
    markdown.push_str(&format!(
        "- Simulated load: {}x peak (peak={}, total writes={}, total reads={})\n",
        report.config.simulated_peak_multiple,
        report.config.peak_load,
        report.config.total_write_ops,
        report.config.total_read_ops
    ));
    markdown.push_str(&format!(
        "- Overall throughput: {:.2} ops/sec\n",
        report.summary.overall_throughput_ops_per_sec
    ));
    markdown.push_str(&format!("- Worst p95 latency: {:.3} ms\n", report.summary.worst_p95_ms));
    markdown.push_str(&format!("- Total failures: {}\n\n", report.summary.total_failures));

    markdown.push_str("## Scenarios\n\n");
    markdown.push_str("| Scenario | Ops | Failures | Throughput (ops/sec) | Avg ms | P95 ms | Events | Estimated Writes |\n");
    markdown.push_str("| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |\n");
    for scenario in &report.scenarios {
        markdown.push_str(&format!(
            "| {} | {} | {} | {:.2} | {:.3} | {:.3} | {} | {} |\n",
            scenario.name,
            scenario.target_operations,
            scenario.failed_operations,
            scenario.throughput_ops_per_sec,
            scenario.avg_latency_ms,
            scenario.p95_latency_ms,
            scenario.emitted_events,
            scenario.estimated_state_writes
        ));
    }

    markdown.push_str("\n## Bottleneck Notes\n\n");
    for note in &report.bottlenecks {
        markdown.push_str(&format!("- {note}\n"));
    }

    fs::write(path, markdown)?;
    Ok(())
}

fn print_console_report(report: &LoadReport, report_path: &Path, summary_path: &Path) {
    println!("=========================================");
    println!(" StrellerMinds Load Test Report");
    println!("=========================================");
    println!(
        "Simulated load: {}x peak (peak={} -> writes={}, reads={})",
        report.config.simulated_peak_multiple,
        report.config.peak_load,
        report.config.total_write_ops,
        report.config.total_read_ops
    );
    println!(
        "Student pool: {} | Courses: {} | CI mode: {}",
        report.config.student_pool, report.config.course_count, report.config.ci_mode
    );
    println!();

    for scenario in &report.scenarios {
        println!(
            "[{}] throughput={:.2} ops/sec | avg={:.3}ms | p95={:.3}ms | failures={} | events={} | est_writes={}",
            scenario.name,
            scenario.throughput_ops_per_sec,
            scenario.avg_latency_ms,
            scenario.p95_latency_ms,
            scenario.failed_operations,
            scenario.emitted_events,
            scenario.estimated_state_writes
        );
    }

    println!();
    println!(
        "Overall throughput: {:.2} ops/sec | Worst p95: {:.3}ms | Total failures: {}",
        report.summary.overall_throughput_ops_per_sec,
        report.summary.worst_p95_ms,
        report.summary.total_failures
    );
    println!("Report written to: {}", report_path.display());
    println!("Summary written to: {}", summary_path.display());
    println!();
    println!("Bottleneck notes:");
    for note in &report.bottlenecks {
        println!("  - {note}");
    }
}

fn duration_ms(duration: std::time::Duration) -> f64 {
    duration.as_secs_f64() * 1000.0
}

fn average(values: &[f64]) -> f64 {
    if values.is_empty() { 0.0 } else { values.iter().sum::<f64>() / values.len() as f64 }
}

fn percentile(values: &[f64], quantile: f64) -> f64 {
    if values.is_empty() {
        return 0.0;
    }

    let mut sorted = values.to_vec();
    sorted.sort_by(|left, right| left.partial_cmp(right).unwrap_or(Ordering::Equal));

    let index = ((sorted.len() - 1) as f64 * quantile).round() as usize;
    sorted[index.min(sorted.len() - 1)]
}

fn throughput(successful_ops: usize, duration: std::time::Duration) -> f64 {
    let seconds = duration.as_secs_f64();
    if seconds > 0.0 { successful_ops as f64 / seconds } else { 0.0 }
}

fn current_rss_bytes() -> Option<i64> {
    #[cfg(target_os = "linux")]
    {
        let status = fs::read_to_string("/proc/self/status").ok()?;
        let rss_kib = status
            .lines()
            .find(|line| line.starts_with("VmRSS:"))?
            .split_whitespace()
            .nth(1)?
            .parse::<i64>()
            .ok()?;
        return Some(rss_kib * 1024);
    }

    #[cfg(not(target_os = "linux"))]
    {
        None
    }
}
