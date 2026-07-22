use serde::Serialize;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

const INSTALLER_MIN_AGE_DAYS: u64 = 7;
const CACHE_MIN_AGE_DAYS: u64 = 14;
const MIN_PLAN_BYTES: u64 = 250 * 1024 * 1024;
const PREVIEW_LIMIT: usize = 12;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RecoveryPlan {
    pub id: String,
    pub title: String,
    pub explanation: String,
    pub estimated_benefit: String,
    pub estimated_benefit_bytes: u64,
    pub estimated_time: String,
    pub interruption: String,
    pub confidence: f32,
    pub actions: Vec<CareActionSummary>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CareActionSummary {
    pub id: String,
    pub title: String,
    pub description: String,
    pub estimated_benefit: String,
    pub estimated_benefit_bytes: u64,
    pub interruption: String,
    pub risk: String,
    pub confidence: f32,
    pub preview_item_count: usize,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CareActionPreview {
    pub action_id: String,
    pub title: String,
    pub estimated_recovery: String,
    pub estimated_recovery_bytes: u64,
    pub files: Vec<PreviewFile>,
    pub omitted_count: usize,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PreviewFile {
    pub name: String,
    pub size: String,
    pub path: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CareActionExplanation {
    pub action_id: String,
    pub title: String,
    pub reason: String,
    pub expected_benefit: String,
    pub risk: String,
    pub interruption: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CareActionRunResult {
    pub action_id: String,
    pub title: String,
    pub success: bool,
    pub recovered: String,
    pub recovered_bytes: u64,
    pub current_free_space: String,
    pub current_free_space_bytes: u64,
    pub storage_health: String,
    pub verified: bool,
    pub errors: Vec<String>,
}

#[derive(Debug, Clone)]
struct RecoveryItem {
    name: String,
    path: PathBuf,
    display_path: String,
    size_bytes: u64,
}

#[derive(Debug, Clone)]
struct ActionEstimate {
    action_id: &'static str,
    title: &'static str,
    description: &'static str,
    reason: &'static str,
    risk: &'static str,
    interruption: &'static str,
    confidence: f32,
    items: Vec<RecoveryItem>,
}

trait CareAction {
    fn action_id(&self) -> &'static str;
    fn title(&self) -> &'static str;
    fn preview(&self) -> Result<CareActionPreview, String>;
    fn explain(&self) -> Result<CareActionExplanation, String>;
    fn estimate(&self) -> Result<ActionEstimate, String>;
    fn execute(&self) -> Result<CareActionRunResult, String>;
}

pub fn plan() -> Result<RecoveryPlan, String> {
    let estimates = collect_estimates()?;
    let actions = estimates
        .iter()
        .filter(|estimate| estimate_total_bytes(estimate) > 0)
        .map(action_summary)
        .collect::<Vec<_>>();
    let estimated_benefit_bytes = actions
        .iter()
        .map(|action| action.estimated_benefit_bytes)
        .sum::<u64>();

    if estimated_benefit_bytes < MIN_PLAN_BYTES {
        return Ok(RecoveryPlan {
            id: "storage-recovery-v1".to_string(),
            title: "Storage looks okay right now.".to_string(),
            explanation:
                "I checked Trash, downloaded installers, and conservative app caches. Nothing large enough needs a care moment."
                    .to_string(),
            estimated_benefit: format_bytes(estimated_benefit_bytes),
            estimated_benefit_bytes,
            estimated_time: "No care moment needed".to_string(),
            interruption: "None".to_string(),
            confidence: 0.82,
            actions: Vec::new(),
        });
    }

    let confidence = if actions.is_empty() {
        0.0
    } else {
        actions.iter().map(|action| action.confidence).sum::<f32>() / actions.len() as f32
    };

    Ok(RecoveryPlan {
        id: "storage-recovery-v1".to_string(),
        title: format!(
            "I found {} that appears safe to recover.",
            format_bytes(estimated_benefit_bytes)
        ),
        explanation:
            "This is space already in Trash, old downloaded installers, or conservative application caches. I will show exactly what changes before anything runs."
                .to_string(),
        estimated_benefit: format_bytes(estimated_benefit_bytes),
        estimated_benefit_bytes,
        estimated_time: "About 30 seconds".to_string(),
        interruption: "None".to_string(),
        confidence,
        actions,
    })
}

pub fn preview(action_id: &str) -> Result<CareActionPreview, String> {
    action_for_id(action_id)?.preview()
}

pub fn explain(action_id: &str) -> Result<CareActionExplanation, String> {
    action_for_id(action_id)?.explain()
}

pub fn run(action_id: &str) -> Result<CareActionRunResult, String> {
    action_for_id(action_id)?.execute()
}

fn collect_estimates() -> Result<Vec<ActionEstimate>, String> {
    Ok(vec![
        EmptyTrashAction.estimate()?,
        DeleteDownloadedInstallersAction.estimate()?,
        ClearObsoleteCachesAction.estimate()?,
    ])
}

fn action_for_id(action_id: &str) -> Result<Box<dyn CareAction>, String> {
    match action_id {
        "empty-trash" => Ok(Box::new(EmptyTrashAction)),
        "delete-downloaded-installers" => Ok(Box::new(DeleteDownloadedInstallersAction)),
        "clear-obsolete-caches" => Ok(Box::new(ClearObsoleteCachesAction)),
        _ => Err("System Pulse does not know that storage care action yet.".to_string()),
    }
}

struct EmptyTrashAction;
struct DeleteDownloadedInstallersAction;
struct ClearObsoleteCachesAction;

impl CareAction for EmptyTrashAction {
    fn action_id(&self) -> &'static str {
        "empty-trash"
    }

    fn title(&self) -> &'static str {
        "Empty Trash"
    }

    fn preview(&self) -> Result<CareActionPreview, String> {
        preview_from_estimate(self.estimate()?)
    }

    fn explain(&self) -> Result<CareActionExplanation, String> {
        explanation_from_estimate(self.estimate()?)
    }

    fn estimate(&self) -> Result<ActionEstimate, String> {
        let trash = home_path()?.join(".Trash");
        let items = direct_children(&trash)?;
        Ok(ActionEstimate {
            action_id: self.action_id(),
            title: self.title(),
            description: "Remove everything already in Trash.",
            reason: "Trash already contains items you have chosen to remove. Emptying it is the safest first storage recovery step because it does not touch working files.",
            risk: "Low. This permanently removes items that are already in Trash.",
            interruption: "None",
            confidence: 0.92,
            items,
        })
    }

    fn execute(&self) -> Result<CareActionRunResult, String> {
        execute_estimate(self.estimate()?)
    }

}

impl CareAction for DeleteDownloadedInstallersAction {
    fn action_id(&self) -> &'static str {
        "delete-downloaded-installers"
    }

    fn title(&self) -> &'static str {
        "Delete downloaded installers"
    }

    fn preview(&self) -> Result<CareActionPreview, String> {
        preview_from_estimate(self.estimate()?)
    }

    fn explain(&self) -> Result<CareActionExplanation, String> {
        explanation_from_estimate(self.estimate()?)
    }

    fn estimate(&self) -> Result<ActionEstimate, String> {
        let downloads = home_path()?.join("Downloads");
        let min_age = age_threshold_days("SYSTEM_PULSE_INSTALLER_MIN_AGE_DAYS", INSTALLER_MIN_AGE_DAYS);
        let items = installer_files(&downloads, min_age)?;
        Ok(ActionEstimate {
            action_id: self.action_id(),
            title: self.title(),
            description: "Remove old app installers from Downloads.",
            reason: "Downloaded installers are usually only needed once. I only include DMG, PKG, and ZIP files older than the configured age threshold.",
            risk: "Low. This removes installer files, not documents or app data.",
            interruption: "None",
            confidence: 0.84,
            items,
        })
    }

    fn execute(&self) -> Result<CareActionRunResult, String> {
        execute_estimate(self.estimate()?)
    }

}

impl CareAction for ClearObsoleteCachesAction {
    fn action_id(&self) -> &'static str {
        "clear-obsolete-caches"
    }

    fn title(&self) -> &'static str {
        "Clear obsolete app caches"
    }

    fn preview(&self) -> Result<CareActionPreview, String> {
        preview_from_estimate(self.estimate()?)
    }

    fn explain(&self) -> Result<CareActionExplanation, String> {
        explanation_from_estimate(self.estimate()?)
    }

    fn estimate(&self) -> Result<ActionEstimate, String> {
        let caches = home_path()?.join("Library").join("Caches");
        let min_age = age_threshold_days("SYSTEM_PULSE_CACHE_MIN_AGE_DAYS", CACHE_MIN_AGE_DAYS);
        let items = obsolete_cache_files(&caches, min_age)?;
        Ok(ActionEstimate {
            action_id: self.action_id(),
            title: self.title(),
            description: "Remove older cache files from conservative app cache folders.",
            reason: "Application caches can be rebuilt by the app. I skip browser profiles, preferences, documents, mail, photos, messages, and cloud data.",
            risk: "Low to medium. Some apps may rebuild cache files the next time they open.",
            interruption: "None",
            confidence: 0.72,
            items,
        })
    }

    fn execute(&self) -> Result<CareActionRunResult, String> {
        execute_estimate(self.estimate()?)
    }

}

fn action_summary(estimate: &ActionEstimate) -> CareActionSummary {
    let estimated_benefit_bytes = estimate_total_bytes(estimate);
    CareActionSummary {
        id: estimate.action_id.to_string(),
        title: estimate.title.to_string(),
        description: estimate.description.to_string(),
        estimated_benefit: format_bytes(estimated_benefit_bytes),
        estimated_benefit_bytes,
        interruption: estimate.interruption.to_string(),
        risk: estimate.risk.to_string(),
        confidence: estimate.confidence,
        preview_item_count: estimate.items.len(),
    }
}

fn preview_from_estimate(mut estimate: ActionEstimate) -> Result<CareActionPreview, String> {
    estimate
        .items
        .sort_by(|left, right| right.size_bytes.cmp(&left.size_bytes));
    let total = estimate_total_bytes(&estimate);
    let omitted_count = estimate.items.len().saturating_sub(PREVIEW_LIMIT);
    let files = estimate
        .items
        .iter()
        .take(PREVIEW_LIMIT)
        .map(|item| PreviewFile {
            name: item.name.clone(),
            size: format_bytes(item.size_bytes),
            path: item.display_path.clone(),
        })
        .collect::<Vec<_>>();

    Ok(CareActionPreview {
        action_id: estimate.action_id.to_string(),
        title: estimate.title.to_string(),
        estimated_recovery: format_bytes(total),
        estimated_recovery_bytes: total,
        files,
        omitted_count,
    })
}

fn explanation_from_estimate(estimate: ActionEstimate) -> Result<CareActionExplanation, String> {
    Ok(CareActionExplanation {
        action_id: estimate.action_id.to_string(),
        title: estimate.title.to_string(),
        reason: estimate.reason.to_string(),
        expected_benefit: format_bytes(estimate_total_bytes(&estimate)),
        risk: estimate.risk.to_string(),
        interruption: estimate.interruption.to_string(),
    })
}

fn execute_estimate(estimate: ActionEstimate) -> Result<CareActionRunResult, String> {
    let before_free_bytes = root_free_space_bytes().unwrap_or(0);
    let estimated_bytes = estimate_total_bytes(&estimate);
    let mut removed_bytes = 0;
    let mut errors = Vec::new();

    for item in &estimate.items {
        match remove_recovery_item(item) {
            Ok(()) => removed_bytes = removed_bytes.saturating_add(item.size_bytes),
            Err(error) => errors.push(format!("{}: {}", item.name, error)),
        }
    }

    if estimate.action_id == "clear-obsolete-caches" {
        let _ = remove_empty_cache_dirs();
    }

    let (current_free_space_bytes, verified) = verify_free_space(before_free_bytes, removed_bytes)?;
    let actual_delta = current_free_space_bytes.saturating_sub(before_free_bytes);
    let recovered_bytes = actual_delta.max(removed_bytes).min(estimated_bytes.max(removed_bytes));

    Ok(CareActionRunResult {
        action_id: estimate.action_id.to_string(),
        title: estimate.title.to_string(),
        success: errors.is_empty(),
        recovered: format_bytes(recovered_bytes),
        recovered_bytes,
        current_free_space: format_bytes(current_free_space_bytes),
        current_free_space_bytes,
        storage_health: storage_health_label(current_free_space_bytes),
        verified,
        errors,
    })
}

fn verify_free_space(before_free_bytes: u64, removed_bytes: u64) -> Result<(u64, bool), String> {
    let current_free_space_bytes = root_free_space_bytes()?;
    let verified = removed_bytes == 0 || current_free_space_bytes >= before_free_bytes;
    Ok((current_free_space_bytes, verified))
}

fn estimate_total_bytes(estimate: &ActionEstimate) -> u64 {
    estimate
        .items
        .iter()
        .map(|item| item.size_bytes)
        .sum::<u64>()
}

fn direct_children(root: &Path) -> Result<Vec<RecoveryItem>, String> {
    if !root.exists() {
        return Ok(Vec::new());
    }

    let mut items = Vec::new();
    for entry in fs::read_dir(root).map_err(|error| format!("Could not read {}: {error}", root.display()))? {
        let entry = match entry {
            Ok(entry) => entry,
            Err(_) => continue,
        };
        let path = entry.path();
        let size_bytes = path_size(&path);
        if size_bytes == 0 {
            continue;
        }
        items.push(recovery_item(path, size_bytes));
    }
    Ok(items)
}

fn installer_files(root: &Path, min_age_days: u64) -> Result<Vec<RecoveryItem>, String> {
    if !root.exists() {
        return Ok(Vec::new());
    }

    let mut items = Vec::new();
    for entry in fs::read_dir(root).map_err(|error| format!("Could not read {}: {error}", root.display()))? {
        let entry = match entry {
            Ok(entry) => entry,
            Err(_) => continue,
        };
        let path = entry.path();
        if !is_regular_file(&path) || !is_allowed_installer(&path) || !is_older_than(&path, min_age_days) {
            continue;
        }
        let size_bytes = path_size(&path);
        if size_bytes == 0 {
            continue;
        }
        items.push(recovery_item(path, size_bytes));
    }
    Ok(items)
}

fn obsolete_cache_files(root: &Path, min_age_days: u64) -> Result<Vec<RecoveryItem>, String> {
    if !root.exists() {
        return Ok(Vec::new());
    }

    let mut items = Vec::new();
    for entry in fs::read_dir(root).map_err(|error| format!("Could not read {}: {error}", root.display()))? {
        let entry = match entry {
            Ok(entry) => entry,
            Err(_) => continue,
        };
        let cache_dir = entry.path();
        if !is_real_directory(&cache_dir) || !is_safe_cache_dir(&cache_dir) {
            continue;
        }
        collect_old_files(&cache_dir, min_age_days, &mut items);
    }
    Ok(items)
}

fn collect_old_files(root: &Path, min_age_days: u64, items: &mut Vec<RecoveryItem>) {
    let Ok(entries) = fs::read_dir(root) else {
        return;
    };

    for entry in entries.flatten() {
        let path = entry.path();
        let Ok(metadata) = fs::symlink_metadata(&path) else {
            continue;
        };
        let file_type = metadata.file_type();
        if file_type.is_symlink() {
            continue;
        }
        if file_type.is_dir() {
            collect_old_files(&path, min_age_days, items);
            continue;
        }
        if !file_type.is_file() || !is_older_than(&path, min_age_days) {
            continue;
        }
        let size_bytes = metadata.len();
        if size_bytes > 0 {
            items.push(recovery_item(path, size_bytes));
        }
    }
}

fn recovery_item(path: PathBuf, size_bytes: u64) -> RecoveryItem {
    let name = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("Item")
        .to_string();
    let display_path = user_facing_path(&path);
    RecoveryItem {
        name,
        path,
        display_path,
        size_bytes,
    }
}

fn remove_recovery_item(item: &RecoveryItem) -> Result<(), String> {
    let metadata = fs::symlink_metadata(&item.path)
        .map_err(|error| format!("Could not inspect item before removing it: {error}"))?;
    let file_type = metadata.file_type();

    if file_type.is_symlink() || file_type.is_file() {
        fs::remove_file(&item.path).map_err(|error| format!("Could not remove file: {error}"))
    } else if file_type.is_dir() {
        fs::remove_dir_all(&item.path).map_err(|error| format!("Could not remove folder: {error}"))
    } else {
        Err("System Pulse skipped this item because it is not a regular file or folder.".to_string())
    }
}

fn remove_empty_cache_dirs() -> Result<(), String> {
    let root = home_path()?.join("Library").join("Caches");
    if !root.exists() {
        return Ok(());
    }
    let entries = fs::read_dir(&root).map_err(|error| format!("Could not read cache folders: {error}"))?;
    for entry in entries.flatten() {
        let path = entry.path();
        if is_real_directory(&path) && is_safe_cache_dir(&path) {
            remove_empty_dirs_under(&path);
            let _ = fs::remove_dir(&path);
        }
    }
    Ok(())
}

fn remove_empty_dirs_under(root: &Path) {
    let Ok(entries) = fs::read_dir(root) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if !is_real_directory(&path) {
            continue;
        }
        remove_empty_dirs_under(&path);
        let _ = fs::remove_dir(&path);
    }
}

fn path_size(path: &Path) -> u64 {
    let Ok(metadata) = fs::symlink_metadata(path) else {
        return 0;
    };
    let file_type = metadata.file_type();
    if file_type.is_symlink() || file_type.is_file() {
        return metadata.len();
    }
    if !file_type.is_dir() {
        return 0;
    }

    let mut total = 0_u64;
    let Ok(entries) = fs::read_dir(path) else {
        return 0;
    };
    for entry in entries.flatten() {
        total = total.saturating_add(path_size(&entry.path()));
    }
    total
}

fn is_allowed_installer(path: &Path) -> bool {
    let Some(extension) = path.extension().and_then(|value| value.to_str()) else {
        return false;
    };
    matches!(extension.to_ascii_lowercase().as_str(), "dmg" | "pkg" | "zip")
}

fn is_regular_file(path: &Path) -> bool {
    fs::symlink_metadata(path)
        .map(|metadata| metadata.file_type().is_file())
        .unwrap_or(false)
}

fn is_real_directory(path: &Path) -> bool {
    fs::symlink_metadata(path)
        .map(|metadata| metadata.file_type().is_dir())
        .unwrap_or(false)
}

fn is_safe_cache_dir(path: &Path) -> bool {
    let Some(name) = path.file_name().and_then(|value| value.to_str()) else {
        return false;
    };
    let name = name.to_ascii_lowercase();
    let protected_terms = [
        "safari",
        "chrome",
        "chromium",
        "firefox",
        "mozilla",
        "edge",
        "brave",
        "arc",
        "opera",
        "vivaldi",
        "mail",
        "message",
        "notes",
        "photos",
        "icloud",
        "clouddocs",
        "addressbook",
        "preference",
        "password",
        "keychain",
    ];
    !protected_terms.iter().any(|term| name.contains(term))
}

fn is_older_than(path: &Path, min_age_days: u64) -> bool {
    let Ok(metadata) = fs::metadata(path) else {
        return false;
    };
    let Ok(modified) = metadata.modified() else {
        return false;
    };
    let Ok(age) = SystemTime::now().duration_since(modified) else {
        return false;
    };
    age >= Duration::from_secs(min_age_days.saturating_mul(86_400))
}

fn age_threshold_days(env_name: &str, default_days: u64) -> u64 {
    env::var(env_name)
        .ok()
        .and_then(|value| value.parse::<u64>().ok())
        .filter(|value| *value > 0)
        .unwrap_or(default_days)
}

fn home_path() -> Result<PathBuf, String> {
    env::var_os("HOME")
        .map(PathBuf::from)
        .ok_or_else(|| "System Pulse could not find your home folder.".to_string())
}

fn user_facing_path(path: &Path) -> String {
    if let Ok(home) = home_path() {
        if let Ok(relative) = path.strip_prefix(&home) {
            return format!("~/{}", relative.display());
        }
    }
    path.display().to_string()
}

fn root_free_space_bytes() -> Result<u64, String> {
    let output = std::process::Command::new("df")
        .args(["-k", "/"])
        .output()
        .map_err(|error| format!("Could not measure free space: {error}"))?;
    if !output.status.success() {
        return Err("Could not measure free space.".to_string());
    }
    let text = String::from_utf8(output.stdout)
        .map_err(|error| format!("Free-space output was not readable: {error}"))?;
    let line = text
        .lines()
        .nth(1)
        .ok_or_else(|| "Free-space output was incomplete.".to_string())?;
    let fields = line.split_whitespace().collect::<Vec<_>>();
    if fields.len() < 4 {
        return Err("Free-space output was incomplete.".to_string());
    }
    fields[3]
        .parse::<u64>()
        .map(|kib| kib.saturating_mul(1024))
        .map_err(|error| format!("Could not parse free space: {error}"))
}

fn storage_health_label(current_free_space_bytes: u64) -> String {
    if current_free_space_bytes >= 40 * 1024 * 1024 * 1024 {
        "Healthy".to_string()
    } else if current_free_space_bytes >= 20 * 1024 * 1024 * 1024 {
        "Good".to_string()
    } else if current_free_space_bytes >= 10 * 1024 * 1024 * 1024 {
        "Getting tight".to_string()
    } else {
        "Needs care".to_string()
    }
}

fn format_bytes(bytes: u64) -> String {
    const GIB: f64 = 1024.0 * 1024.0 * 1024.0;
    const MIB: f64 = 1024.0 * 1024.0;
    if bytes >= 1024 * 1024 * 1024 {
        format!("{:.1} GB", bytes as f64 / GIB)
    } else if bytes >= 1024 * 1024 {
        format!("{:.0} MB", bytes as f64 / MIB)
    } else if bytes >= 1024 {
        format!("{:.0} KB", bytes as f64 / 1024.0)
    } else {
        format!("{bytes} B")
    }
}
