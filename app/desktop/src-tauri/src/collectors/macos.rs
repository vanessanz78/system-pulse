use crate::models::{ApplicationSnapshot, MemorySnapshot, StorageSnapshot, SystemSnapshot};
use std::collections::HashMap;
use std::path::Path;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

pub fn collect_system_snapshot() -> Result<SystemSnapshot, String> {
    let memory = collect_memory()?;
    let storage = collect_storage()?;
    let applications = collect_top_applications(memory.total_bytes)?;

    Ok(SystemSnapshot {
        collected_at: collected_at()?,
        platform: "macOS".to_string(),
        memory,
        storage,
        applications,
    })
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

fn collect_top_applications(total_memory_bytes: u64) -> Result<Vec<ApplicationSnapshot>, String> {
    let output = run_command("ps", &["-axo", "pid=,rss=,comm="])?;
    let mut grouped: HashMap<String, u64> = HashMap::new();

    for line in output.lines() {
        let mut parts = line.split_whitespace();
        let _pid = parts.next();
        let Some(rss_kib) = parts.next() else {
            continue;
        };
        let command_name = parts.collect::<Vec<_>>().join(" ");
        if command_name.is_empty() {
            continue;
        }
        let memory_bytes = parse_kib(rss_kib, "application memory").unwrap_or(0);
        if memory_bytes == 0 {
            continue;
        }
        let name = normalize_application_name(&command_name);
        *grouped.entry(name).or_insert(0) += memory_bytes;
    }

    let mut applications = grouped
        .into_iter()
        .filter(|(_, memory_bytes)| {
            total_memory_bytes == 0 || (*memory_bytes as f64 / total_memory_bytes as f64) >= 0.005
        })
        .map(|(name, memory_bytes)| ApplicationSnapshot { name, memory_bytes })
        .collect::<Vec<_>>();

    applications.sort_by(|left, right| right.memory_bytes.cmp(&left.memory_bytes));
    applications.truncate(6);
    Ok(applications)
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

    Path::new(command_name)
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or(command_name)
        .trim()
        .trim_end_matches(".app")
        .to_string()
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

fn parse_kib(value: &str, label: &str) -> Result<u64, String> {
    value
        .parse::<u64>()
        .map(|kib| kib.saturating_mul(1024))
        .map_err(|error| format!("Could not parse {label}: {error}"))
}

fn collected_at() -> Result<String, String> {
    let seconds = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|error| format!("System clock error: {error}"))?
        .as_secs();
    Ok(format!("Unix {seconds}"))
}
