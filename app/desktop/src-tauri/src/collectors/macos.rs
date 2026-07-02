use crate::models::{
    ApplicationSnapshot, BrowserHealthSnapshot, BrowserSnapshot, CpuSnapshot, DiskActivitySnapshot,
    MemorySnapshot, RendererHealthSnapshot, StorageSnapshot, SystemSnapshot, WindowServerSnapshot,
};
use std::collections::HashMap;
use std::path::Path;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

pub fn collect_system_snapshot() -> Result<SystemSnapshot, String> {
    let cpu = collect_cpu().unwrap_or_default();
    let memory = collect_memory()?;
    let storage = collect_storage()?;
    let disk_activity = collect_disk_activity().unwrap_or_default();
    let processes = collect_processes()?;
    let applications = collect_top_applications(memory.total_bytes, &processes);
    let browser = collect_browser_health(&processes);
    let renderers = collect_renderer_health(&browser);
    let window_server = collect_window_server(&processes);

    Ok(SystemSnapshot {
        collected_at: collected_at()?,
        platform: "macOS".to_string(),
        cpu,
        memory,
        storage,
        disk_activity,
        applications,
        browser,
        renderers,
        window_server,
    })
}

impl Default for CpuSnapshot {
    fn default() -> Self {
        Self {
            user_percent: 0.0,
            system_percent: 0.0,
            idle_percent: 100.0,
        }
    }
}

impl Default for DiskActivitySnapshot {
    fn default() -> Self {
        Self {
            megabytes_per_second: 0.0,
        }
    }
}

#[derive(Debug)]
struct ProcessInfo {
    rss_bytes: u64,
    cpu_percent: f32,
    elapsed_seconds: Option<u64>,
    user_name: String,
    command_name: String,
}

fn collect_cpu() -> Result<CpuSnapshot, String> {
    let output = run_command("top", &["-l", "1", "-n", "0"])?;
    parse_cpu_snapshot(&output).ok_or_else(|| "Could not read CPU reserve.".to_string())
}

fn collect_memory() -> Result<MemorySnapshot, String> {
    let total_bytes = run_command("sysctl", &["-n", "hw.memsize"])?
        .trim()
        .parse::<u64>()
        .map_err(|error| format!("Could not parse total memory: {error}"))?;

    let vm_stat = run_command("vm_stat", &[])?;
    let page_size = parse_page_size(&vm_stat).unwrap_or(16_384);
    let free_pages = parse_vm_pages(&vm_stat, "Pages free");
    let inactive_pages = parse_vm_pages(&vm_stat, "Pages inactive");
    let speculative_pages = parse_vm_pages(&vm_stat, "Pages speculative");
    let compressed_pages = parse_vm_pages(&vm_stat, "Pages occupied by compressor");
    let (swap_total_bytes, swap_used_bytes) = collect_swap_usage().unwrap_or((0, 0));

    let available_bytes = free_pages
        .saturating_add(inactive_pages)
        .saturating_add(speculative_pages)
        .saturating_mul(page_size);
    let used_bytes = total_bytes.saturating_sub(available_bytes);

    Ok(MemorySnapshot {
        total_bytes,
        available_bytes,
        used_bytes,
        compressed_bytes: compressed_pages.saturating_mul(page_size),
        swap_total_bytes,
        swap_used_bytes,
    })
}

fn collect_storage() -> Result<StorageSnapshot, String> {
    let output = run_command("df", &["-k", "/"])?;
    let line = output
        .lines()
        .nth(1)
        .ok_or_else(|| "Could not read root storage information.".to_string())?;
    let fields = line.split_whitespace().collect::<Vec<_>>();

    if fields.len() < 6 {
        return Err("Root storage information was incomplete.".to_string());
    }

    let total_bytes = parse_kib(fields[1], "total storage")?;
    let used_bytes = parse_kib(fields[2], "used storage")?;
    let available_bytes = parse_kib(fields[3], "available storage")?;

    Ok(StorageSnapshot {
        mount_point: fields[5].to_string(),
        total_bytes,
        available_bytes,
        used_bytes,
    })
}

fn collect_disk_activity() -> Result<DiskActivitySnapshot, String> {
    let output = run_command("iostat", &["-d", "disk0", "1", "2"])?;
    let megabytes_per_second = output
        .lines()
        .filter_map(parse_iostat_mbps)
        .last()
        .unwrap_or(0.0);

    Ok(DiskActivitySnapshot {
        megabytes_per_second,
    })
}

fn collect_swap_usage() -> Result<(u64, u64), String> {
    let output = run_command("sysctl", &["vm.swapusage"])?;
    let total = parse_swap_bytes(&output, "total").unwrap_or(0);
    let used = parse_swap_bytes(&output, "used").unwrap_or(0);
    Ok((total, used))
}

fn collect_processes() -> Result<Vec<ProcessInfo>, String> {
    let output = run_command("ps", &["-axo", "pid=,rss=,pcpu=,etime=,user=,comm="])?;
    let mut processes = Vec::new();

    for line in output.lines() {
        let mut parts = line.split_whitespace();
        let _pid = parts.next();
        let Some(rss_kib) = parts.next() else {
            continue;
        };
        let Some(cpu_percent) = parts.next() else {
            continue;
        };
        let Some(elapsed) = parts.next() else {
            continue;
        };
        let Some(user_name) = parts.next() else {
            continue;
        };
        let command_name = parts.collect::<Vec<_>>().join(" ");
        if command_name.is_empty() {
            continue;
        }

        let rss_bytes = parse_kib(rss_kib, "process memory").unwrap_or(0);
        let cpu_percent = cpu_percent.parse::<f32>().unwrap_or(0.0);
        processes.push(ProcessInfo {
            rss_bytes,
            cpu_percent,
            elapsed_seconds: parse_elapsed_seconds(elapsed),
            user_name: user_name.to_string(),
            command_name,
        });
    }

    Ok(processes)
}

fn collect_top_applications(
    total_memory_bytes: u64,
    processes: &[ProcessInfo],
) -> Vec<ApplicationSnapshot> {
    let mut grouped: HashMap<String, (u64, f32)> = HashMap::new();

    for process in processes {
        if process.rss_bytes == 0 {
            continue;
        }
        if !belongs_to_current_user(process) {
            continue;
        }
        if browser_name(&process.command_name).is_some() {
            continue;
        }
        if is_system_pulse_process(&process.command_name) {
            continue;
        }
        let name = normalize_application_name(&process.command_name);
        let entry = grouped.entry(name).or_insert((0, 0.0));
        entry.0 = entry.0.saturating_add(process.rss_bytes);
        entry.1 += process.cpu_percent;
    }

    let mut applications = grouped
        .into_iter()
        .filter(|(_, (memory_bytes, cpu_percent))| {
            total_memory_bytes == 0
                || (*memory_bytes as f64 / total_memory_bytes as f64) >= 0.005
                || *cpu_percent >= 1.0
        })
        .map(|(name, (memory_bytes, cpu_percent))| ApplicationSnapshot {
            name,
            memory_bytes,
            cpu_percent,
        })
        .collect::<Vec<_>>();

    applications.sort_by(|left, right| {
        right
            .cpu_percent
            .partial_cmp(&left.cpu_percent)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then(right.memory_bytes.cmp(&left.memory_bytes))
    });
    applications.truncate(8);
    applications
}

fn belongs_to_current_user(process: &ProcessInfo) -> bool {
    std::env::var("USER")
        .map(|user_name| process.user_name == user_name)
        .unwrap_or(true)
}

fn is_system_pulse_process(command_name: &str) -> bool {
    let raw_name = command_name.to_lowercase();
    let normalized_name = normalize_application_name(command_name).to_lowercase();

    raw_name.contains("system-pulse")
        || raw_name.contains("system pulse")
        || normalized_name.contains("system-pulse")
        || normalized_name.contains("system pulse")
}

#[derive(Default)]
struct BrowserAccumulator {
    memory_bytes: u64,
    cpu_percent: f32,
    process_count: u32,
    renderer_count: u32,
    renderer_memory_bytes: u64,
    largest_renderer_bytes: u64,
    uptime_seconds: Option<u64>,
}

fn collect_browser_health(processes: &[ProcessInfo]) -> BrowserHealthSnapshot {
    let mut browsers: HashMap<String, BrowserAccumulator> = HashMap::new();

    for process in processes {
        let Some(browser_name) = browser_name(&process.command_name) else {
            continue;
        };
        let accumulator = browsers.entry(browser_name).or_default();
        accumulator.memory_bytes = accumulator.memory_bytes.saturating_add(process.rss_bytes);
        accumulator.cpu_percent += process.cpu_percent;
        accumulator.process_count = accumulator.process_count.saturating_add(1);
        accumulator.uptime_seconds =
            max_optional(accumulator.uptime_seconds, process.elapsed_seconds);

        if renderer_browser_name(&process.command_name).is_some() {
            accumulator.renderer_count = accumulator.renderer_count.saturating_add(1);
            accumulator.renderer_memory_bytes = accumulator
                .renderer_memory_bytes
                .saturating_add(process.rss_bytes);
            accumulator.largest_renderer_bytes =
                accumulator.largest_renderer_bytes.max(process.rss_bytes);
        }
    }

    let mut browsers = browsers
        .into_iter()
        .map(|(name, accumulator)| BrowserSnapshot {
            name,
            memory_bytes: accumulator.memory_bytes,
            cpu_percent: accumulator.cpu_percent,
            process_count: accumulator.process_count,
            renderer_count: accumulator.renderer_count,
            renderer_memory_bytes: accumulator.renderer_memory_bytes,
            largest_renderer_bytes: accumulator.largest_renderer_bytes,
            uptime_seconds: accumulator.uptime_seconds,
        })
        .collect::<Vec<_>>();

    browsers.sort_by(|left, right| right.memory_bytes.cmp(&left.memory_bytes));
    BrowserHealthSnapshot { browsers }
}

fn collect_renderer_health(browser_health: &BrowserHealthSnapshot) -> RendererHealthSnapshot {
    let total_count = browser_health
        .browsers
        .iter()
        .map(|browser| browser.renderer_count)
        .sum();
    let total_memory_bytes = browser_health
        .browsers
        .iter()
        .map(|browser| browser.renderer_memory_bytes)
        .sum();
    let primary = browser_health
        .browsers
        .iter()
        .max_by(|left, right| {
            left.renderer_count
                .cmp(&right.renderer_count)
                .then(left.memory_bytes.cmp(&right.memory_bytes))
        });

    RendererHealthSnapshot {
        total_count,
        total_memory_bytes,
        largest_renderer_name: primary.map(|browser| browser.name.clone()),
        largest_renderer_memory_bytes: primary
            .map(|browser| browser.largest_renderer_bytes)
            .unwrap_or(0),
        primary_browser: primary.map(|browser| browser.name.clone()),
        primary_browser_renderer_count: primary
            .map(|browser| browser.renderer_count)
            .unwrap_or(0),
    }
}

fn collect_window_server(processes: &[ProcessInfo]) -> Option<WindowServerSnapshot> {
    processes.iter().find_map(|process| {
        let name = process.command_name.to_lowercase();
        if name.ends_with("windowserver") || name.contains("/windowserver") {
            Some(WindowServerSnapshot {
                memory_bytes: process.rss_bytes,
                cpu_percent: process.cpu_percent,
                uptime_seconds: process.elapsed_seconds,
            })
        } else {
            None
        }
    })
}

fn normalize_application_name(command_name: &str) -> String {
    let lower = command_name.to_lowercase();
    if lower.contains("google chrome") {
        return "Google Chrome".to_string();
    }
    if lower.contains("visual studio code") || lower.contains("code helper") {
        return "Visual Studio Code".to_string();
    }
    if lower.contains("cursor") {
        return "Cursor".to_string();
    }
    if lower.contains("codex") {
        return "Codex".to_string();
    }
    if lower.contains("slack") {
        return "Slack".to_string();
    }
    if lower.contains("discord") {
        return "Discord".to_string();
    }
    if lower.contains("spotify") {
        return "Spotify".to_string();
    }
    if lower.contains("safari") {
        return "Safari".to_string();
    }
    if lower.ends_with("windowserver") || lower.contains("/windowserver") {
        return "WindowServer".to_string();
    }

    Path::new(command_name)
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or(command_name)
        .trim()
        .trim_end_matches(".app")
        .to_string()
}

fn browser_name(command_name: &str) -> Option<String> {
    let lower = command_name.to_lowercase();
    if lower.contains("google chrome") {
        return Some("Google Chrome".to_string());
    }
    if lower.contains("microsoft edge") {
        return Some("Microsoft Edge".to_string());
    }
    if lower.contains("firefox") {
        return Some("Firefox".to_string());
    }
    if lower.contains("safari") || lower.contains("webkit.webcontent") {
        return Some("Safari".to_string());
    }
    None
}

fn renderer_browser_name(command_name: &str) -> Option<String> {
    let lower = command_name.to_lowercase();
    if lower.contains("google chrome helper") && lower.contains("renderer") {
        return Some("Google Chrome".to_string());
    }
    if lower.contains("microsoft edge helper") && lower.contains("renderer") {
        return Some("Microsoft Edge".to_string());
    }
    if lower.contains("webkit.webcontent") || lower.contains("safari web content") {
        return Some("Safari".to_string());
    }
    if lower.contains("firefox") && (lower.contains("web content") || lower.contains("plugin-container")) {
        return Some("Firefox".to_string());
    }
    None
}

fn max_optional(left: Option<u64>, right: Option<u64>) -> Option<u64> {
    match (left, right) {
        (Some(left), Some(right)) => Some(left.max(right)),
        (Some(left), None) => Some(left),
        (None, Some(right)) => Some(right),
        (None, None) => None,
    }
}

fn run_command(command: &str, args: &[&str]) -> Result<String, String> {
    let output = Command::new(command)
        .args(args)
        .output()
        .map_err(|error| format!("Could not run {command}: {error}"))?;

    if !output.status.success() {
        return Err(format!("{command} returned a non-zero status."));
    }

    String::from_utf8(output.stdout).map_err(|error| format!("{command} output was not UTF-8: {error}"))
}

fn parse_page_size(vm_stat: &str) -> Option<u64> {
    let first_line = vm_stat.lines().next()?;
    let start = first_line.find("page size of ")? + "page size of ".len();
    let end = first_line[start..].find(" bytes")? + start;
    first_line[start..end].parse::<u64>().ok()
}

fn parse_cpu_snapshot(output: &str) -> Option<CpuSnapshot> {
    let line = output
        .lines()
        .find(|line| line.trim_start().starts_with("CPU usage:"))?;

    Some(CpuSnapshot {
        user_percent: parse_cpu_component(line, "user")?,
        system_percent: parse_cpu_component(line, "sys")?,
        idle_percent: parse_cpu_component(line, "idle")?,
    })
}

fn parse_cpu_component(line: &str, label: &str) -> Option<f32> {
    let marker = format!("% {label}");
    let marker_index = line.find(&marker)?;
    let before_marker = &line[..marker_index];
    let start = before_marker
        .rfind(|character: char| character == ' ' || character == ':')
        .map(|index| index + 1)
        .unwrap_or(0);
    before_marker[start..].trim().parse::<f32>().ok()
}

fn parse_vm_pages(vm_stat: &str, label: &str) -> u64 {
    vm_stat
        .lines()
        .find(|line| line.starts_with(label))
        .and_then(|line| line.split(':').nth(1))
        .map(|value| {
            value
                .chars()
                .filter(|character| character.is_ascii_digit())
                .collect::<String>()
        })
        .and_then(|digits| digits.parse::<u64>().ok())
        .unwrap_or(0)
}

fn parse_swap_bytes(output: &str, label: &str) -> Option<u64> {
    let marker = format!("{label} = ");
    let start = output.find(&marker)? + marker.len();
    let rest = output[start..].trim_start();
    let number_end = rest
        .find(|character: char| !character.is_ascii_digit() && character != '.')
        .unwrap_or(rest.len());
    let number = rest[..number_end].parse::<f64>().ok()?;
    let unit = rest[number_end..].trim_start().chars().next().unwrap_or('M');
    let multiplier = match unit {
        'G' | 'g' => 1_073_741_824.0,
        'M' | 'm' => 1_048_576.0,
        'K' | 'k' => 1024.0,
        _ => 1.0,
    };
    Some((number * multiplier).round() as u64)
}

fn parse_iostat_mbps(line: &str) -> Option<f32> {
    let fields = line.split_whitespace().collect::<Vec<_>>();
    fields.last()?.parse::<f32>().ok()
}

fn parse_kib(value: &str, label: &str) -> Result<u64, String> {
    value
        .parse::<u64>()
        .map(|kib| kib.saturating_mul(1024))
        .map_err(|error| format!("Could not parse {label}: {error}"))
}

fn parse_elapsed_seconds(value: &str) -> Option<u64> {
    let (days, rest) = if let Some((days, rest)) = value.split_once('-') {
        (days.parse::<u64>().ok()?, rest)
    } else {
        (0, value)
    };
    let parts = rest
        .split(':')
        .map(|part| part.parse::<u64>().ok())
        .collect::<Option<Vec<_>>>()?;

    let seconds = match parts.as_slice() {
        [minutes, seconds] => minutes.saturating_mul(60).saturating_add(*seconds),
        [hours, minutes, seconds] => hours
            .saturating_mul(3_600)
            .saturating_add(minutes.saturating_mul(60))
            .saturating_add(*seconds),
        _ => return None,
    };

    Some(days.saturating_mul(86_400).saturating_add(seconds))
}

fn collected_at() -> Result<String, String> {
    let seconds = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|error| format!("System clock error: {error}"))?
        .as_secs();
    Ok(format!("Unix {seconds}"))
}
