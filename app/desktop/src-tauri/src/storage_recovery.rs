use crate::mission_engine::{
    CareAction as MissionCareAction, MissionAction, MissionEstimate, MissionExplanation,
    MissionLifecycle, MissionPreview, MissionPreviewFile, MissionProvider, MissionResult,
    MissionVerification, PulseMission,
};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant, SystemTime};

const INSTALLER_MIN_AGE_DAYS: u64 = 7;
const CACHE_MIN_AGE_DAYS: u64 = 14;
const MIN_PLAN_BYTES: u64 = 250 * 1024 * 1024;
const PREVIEW_LIMIT: usize = 12;

const STORAGE_MISSION_ID: &str = "storage-recovery";

pub type RecoveryPlan = PulseMission;
pub type CareActionSummary = MissionAction;
pub type CareActionPreview = MissionPreview;
pub type PreviewFile = MissionPreviewFile;
pub type CareActionExplanation = MissionExplanation;
pub type CareActionRunResult = MissionResult;

pub struct StorageMissionProvider;

impl MissionProvider for StorageMissionProvider {
    fn mission_id(&self) -> &'static str {
        STORAGE_MISSION_ID
    }

    fn load(&self) -> Result<Option<PulseMission>, String> {
        Ok(Some(plan()?))
    }

    fn preview(&self, action_id: &str) -> Result<MissionPreview, String> {
        preview(action_id)
    }

    fn explain(&self, action_id: &str) -> Result<MissionExplanation, String> {
        explain(action_id)
    }

    fn execute(&self, action_id: &str) -> Result<MissionResult, String> {
        run(action_id)
    }
}

#[derive(Debug, Clone)]
struct RecoveryItem {
    name: String,
    item_kind: &'static str,
    reason: String,
    confidence: &'static str,
    interruption: &'static str,
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
    confidence: &'static str,
    confidence_reason: &'static str,
    items: Vec<RecoveryItem>,
}

trait StorageCareAction {
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
            id: STORAGE_MISSION_ID.to_string(),
            category: "Storage".to_string(),
            mission_title: "Storage Mission".to_string(),
            title: "Storage looks okay right now.".to_string(),
            summary: "I checked the safest places first and did not find enough recoverable space to need a mission.".to_string(),
            explanation:
                "I checked Trash, old installers, and temporary application files. Nothing large enough needs a care moment."
                    .to_string(),
            confidence: "High".to_string(),
            confidence_reason: "These checks avoid personal documents and only look in places that are normally safe to tidy.".to_string(),
            status: MissionLifecycle::Ready.as_str().to_string(),
            priority: 10,
            estimated_benefit: format_bytes(estimated_benefit_bytes),
            expected_benefit: format_bytes(estimated_benefit_bytes),
            estimated_benefit_bytes,
            expected_interruption: "None".to_string(),
            estimated_duration: "No care moment needed".to_string(),
            diagnosis: "The safest storage locations do not currently contain enough recoverable space to need attention.".to_string(),
            recovery_plan: "No care action is useful right now.".to_string(),
            actions: Vec::new(),
        });
    }

    let (confidence, confidence_reason) = plan_confidence(&actions);

    Ok(RecoveryPlan {
        id: STORAGE_MISSION_ID.to_string(),
        category: "Storage".to_string(),
        mission_title: "Storage Mission".to_string(),
        title: format!(
            "I found {} that appears safe to recover.",
            format_bytes(estimated_benefit_bytes)
        ),
        summary: storage_plan_summary(&actions, estimated_benefit_bytes),
        explanation:
            "This mission focuses on storage that is already discarded, old installer files, or temporary files your applications can recreate. Personal documents are not included."
                .to_string(),
        confidence,
        confidence_reason,
        status: MissionLifecycle::Ready.as_str().to_string(),
        priority: 80,
        estimated_benefit: format_bytes(estimated_benefit_bytes),
        expected_benefit: format!(
            "Recover {} without affecting personal files.",
            format_bytes(estimated_benefit_bytes)
        ),
        estimated_benefit_bytes,
        expected_interruption: "None".to_string(),
        estimated_duration: "About 40 seconds".to_string(),
        diagnosis: "Recoverable storage exists in low-risk locations: Trash, old installers, or temporary app files.".to_string(),
        recovery_plan: "Choose the one useful cleaning action you want System Pulse to handle.".to_string(),
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
        StorageCareAction::estimate(&EmptyTrashAction)?,
        StorageCareAction::estimate(&DeleteDownloadedInstallersAction)?,
        StorageCareAction::estimate(&ClearObsoleteCachesAction)?,
    ])
}

fn action_for_id(action_id: &str) -> Result<Box<dyn StorageCareAction>, String> {
    match storage_action_id(action_id) {
        "empty-trash" => Ok(Box::new(EmptyTrashAction)),
        "delete-downloaded-installers" => Ok(Box::new(DeleteDownloadedInstallersAction)),
        "clear-obsolete-caches" => Ok(Box::new(ClearObsoleteCachesAction)),
        _ => Err("System Pulse does not know that storage care action yet.".to_string()),
    }
}

struct EmptyTrashAction;
struct DeleteDownloadedInstallersAction;
struct ClearObsoleteCachesAction;

impl StorageCareAction for EmptyTrashAction {
    fn action_id(&self) -> &'static str {
        "empty-trash"
    }

    fn title(&self) -> &'static str {
        "Empty Trash"
    }

    fn preview(&self) -> Result<CareActionPreview, String> {
        preview_from_estimate(StorageCareAction::estimate(self)?)
    }

    fn explain(&self) -> Result<CareActionExplanation, String> {
        explanation_from_estimate(StorageCareAction::estimate(self)?)
    }

    fn estimate(&self) -> Result<ActionEstimate, String> {
        let trash = home_path()?.join(".Trash");
        let items = direct_children(&trash)?;
        Ok(ActionEstimate {
            action_id: StorageCareAction::action_id(self),
            title: StorageCareAction::title(self),
            description: "Remove everything already in Trash.",
            reason: "Trash already contains items you have chosen to remove. Emptying it is the safest first storage recovery step because it does not touch working files.",
            risk: "Low. This permanently removes items that are already in Trash.",
            interruption: "None",
            confidence: "High",
            confidence_reason: "The items are already in Trash, which means they have already been removed from active work.",
            items,
        })
    }

    fn execute(&self) -> Result<CareActionRunResult, String> {
        execute_estimate(StorageCareAction::estimate(self)?)
    }

}

impl StorageCareAction for DeleteDownloadedInstallersAction {
    fn action_id(&self) -> &'static str {
        "delete-downloaded-installers"
    }

    fn title(&self) -> &'static str {
        "Remove old installers"
    }

    fn preview(&self) -> Result<CareActionPreview, String> {
        preview_from_estimate(StorageCareAction::estimate(self)?)
    }

    fn explain(&self) -> Result<CareActionExplanation, String> {
        explanation_from_estimate(StorageCareAction::estimate(self)?)
    }

    fn estimate(&self) -> Result<ActionEstimate, String> {
        let downloads = home_path()?.join("Downloads");
        let min_age = age_threshold_days("SYSTEM_PULSE_INSTALLER_MIN_AGE_DAYS", INSTALLER_MIN_AGE_DAYS);
        let items = installer_files(&downloads, min_age)?;
        Ok(ActionEstimate {
            action_id: StorageCareAction::action_id(self),
            title: StorageCareAction::title(self),
            description: "Remove old app installers from Downloads.",
            reason: "Installers are usually only needed once. I only include older app installer files, and removing them does not remove the apps themselves.",
            risk: "Low. This removes installer files, not documents or app data.",
            interruption: "None",
            confidence: "High",
            confidence_reason: "Installed applications, documents, and app data are not touched.",
            items,
        })
    }

    fn execute(&self) -> Result<CareActionRunResult, String> {
        execute_estimate(StorageCareAction::estimate(self)?)
    }

}

impl StorageCareAction for ClearObsoleteCachesAction {
    fn action_id(&self) -> &'static str {
        "clear-obsolete-caches"
    }

    fn title(&self) -> &'static str {
        "Clean temporary files"
    }

    fn preview(&self) -> Result<CareActionPreview, String> {
        preview_from_estimate(StorageCareAction::estimate(self)?)
    }

    fn explain(&self) -> Result<CareActionExplanation, String> {
        explanation_from_estimate(StorageCareAction::estimate(self)?)
    }

    fn estimate(&self) -> Result<ActionEstimate, String> {
        let caches = home_path()?.join("Library").join("Caches");
        let min_age = age_threshold_days("SYSTEM_PULSE_CACHE_MIN_AGE_DAYS", CACHE_MIN_AGE_DAYS);
        let items = obsolete_cache_files(&caches, min_age)?;
        Ok(ActionEstimate {
            action_id: StorageCareAction::action_id(self),
            title: StorageCareAction::title(self),
            description: "Remove temporary files your applications can recreate automatically.",
            reason: "I found temporary files that your applications can recreate automatically. I skip browser profiles, preferences, documents, mail, photos, messages, and cloud data.",
            risk: "Low to medium. Nothing personal will be removed.",
            interruption: "Applications may open slightly slower the first time after cleaning.",
            confidence: "Medium",
            confidence_reason: "These files are rebuildable, but some apps may briefly recreate them after cleanup.",
            items,
        })
    }

    fn execute(&self) -> Result<CareActionRunResult, String> {
        execute_estimate(StorageCareAction::estimate(self)?)
    }

}

macro_rules! impl_mission_care_action {
    ($action:ty, $confidence:expr, $interruption:expr) => {
        impl MissionCareAction for $action {
            fn id(&self) -> &'static str {
                StorageCareAction::action_id(self)
            }

            fn title(&self) -> &'static str {
                StorageCareAction::title(self)
            }

            fn confidence(&self) -> &'static str {
                $confidence
            }

            fn interruption(&self) -> &'static str {
                $interruption
            }

            fn preview(&self) -> Result<MissionPreview, String> {
                StorageCareAction::preview(self)
            }

            fn explain(&self) -> Result<MissionExplanation, String> {
                StorageCareAction::explain(self)
            }

            fn estimate(&self) -> Result<MissionEstimate, String> {
                let estimate = StorageCareAction::estimate(self)?;
                Ok(MissionEstimate {
                    action_id: mission_action_id(estimate.action_id),
                    title: estimate.title.to_string(),
                    estimated_benefit: format_bytes(estimate_total_bytes(&estimate)),
                    estimated_benefit_bytes: estimate_total_bytes(&estimate),
                    confidence: estimate.confidence.to_string(),
                    interruption: estimate.interruption.to_string(),
                })
            }

            fn execute(&self) -> Result<MissionResult, String> {
                StorageCareAction::execute(self)
            }

            fn verify(&self, result: &MissionResult) -> MissionVerification {
                MissionVerification {
                    verified: result.verified,
                    verification: result.verification.clone(),
                }
            }
        }
    };
}

impl_mission_care_action!(EmptyTrashAction, "High", "None");
impl_mission_care_action!(DeleteDownloadedInstallersAction, "High", "None");
impl_mission_care_action!(
    ClearObsoleteCachesAction,
    "Medium",
    "Applications may open slightly slower the first time after cleaning."
);

fn action_summary(estimate: &ActionEstimate) -> CareActionSummary {
    let estimated_benefit_bytes = estimate_total_bytes(estimate);
    CareActionSummary {
        id: mission_action_id(estimate.action_id),
        title: estimate.title.to_string(),
        description: estimate.description.to_string(),
        confidence: estimate.confidence.to_string(),
        confidence_reason: estimate.confidence_reason.to_string(),
        why_recommended: estimate.reason.to_string(),
        estimated_benefit: format_bytes(estimated_benefit_bytes),
        estimated_benefit_bytes,
        interruption: estimate.interruption.to_string(),
        risk: estimate.risk.to_string(),
        preview_item_count: estimate.items.len(),
        status: MissionLifecycle::Ready.as_str().to_string(),
    }
}

fn plan_confidence(actions: &[CareActionSummary]) -> (String, String) {
    if actions.is_empty() {
        return (
            "High".to_string(),
            "System Pulse only checked conservative storage locations and found no large recovery opportunity."
                .to_string(),
        );
    }

    let has_medium = actions.iter().any(|action| action.confidence == "Medium");
    if has_medium {
        (
            "Medium".to_string(),
            "The mission includes rebuildable application caches, which are usually safe but may briefly affect the next launch of an app."
                .to_string(),
        )
    } else {
        (
            "High".to_string(),
            "The mission is limited to items already in Trash or old downloaded installers, and it does not touch personal documents."
                .to_string(),
        )
    }
}

fn storage_plan_summary(actions: &[CareActionSummary], total_bytes: u64) -> String {
    let largest = actions
        .iter()
        .max_by_key(|action| action.estimated_benefit_bytes)
        .map(|action| action.title.as_str())
        .unwrap_or("safe storage");

    format!(
        "Recover {} safely. Most of today's recoverable storage comes from {} rather than personal documents.",
        format_bytes(total_bytes),
        largest.to_lowercase()
    )
}

fn preview_found_label(estimate: &ActionEstimate) -> String {
    match estimate.action_id {
        "empty-trash" => "I found items already waiting in Trash.".to_string(),
        "delete-downloaded-installers" => {
            "I found old installers in Downloads.".to_string()
        }
        "clear-obsolete-caches" => {
            "I found temporary files that your applications can recreate automatically.".to_string()
        }
        _ => "I found recoverable storage.".to_string(),
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
            item_kind: item.item_kind.to_string(),
            size: format_bytes(item.size_bytes),
            path: item.display_path.clone(),
            reason: item.reason.clone(),
            confidence: item.confidence.to_string(),
            expected_benefit: format!("Recover {}", format_bytes(item.size_bytes)),
            interruption: item.interruption.to_string(),
        })
        .collect::<Vec<_>>();

    Ok(CareActionPreview {
        action_id: mission_action_id(estimate.action_id),
        title: estimate.title.to_string(),
        what_i_found: preview_found_label(&estimate),
        why_selected: estimate.reason.to_string(),
        confidence: estimate.confidence.to_string(),
        risk: estimate.risk.to_string(),
        interruption: estimate.interruption.to_string(),
        estimated_recovery: format_bytes(total),
        estimated_recovery_bytes: total,
        files,
        omitted_count,
    })
}

fn explanation_from_estimate(estimate: ActionEstimate) -> Result<CareActionExplanation, String> {
    Ok(CareActionExplanation {
        action_id: mission_action_id(estimate.action_id),
        title: estimate.title.to_string(),
        reason: estimate.reason.to_string(),
        confidence: estimate.confidence.to_string(),
        confidence_reason: estimate.confidence_reason.to_string(),
        expected_benefit: format_bytes(estimate_total_bytes(&estimate)),
        risk: estimate.risk.to_string(),
        interruption: estimate.interruption.to_string(),
    })
}

fn execute_estimate(estimate: ActionEstimate) -> Result<CareActionRunResult, String> {
    let before_free_bytes = root_free_space_bytes().unwrap_or(0);
    let started_at = Instant::now();
    let estimated_bytes = estimate_total_bytes(&estimate);
    let mut removed_bytes: u64 = 0;
    let mut errors = Vec::new();
    let mut actions_completed = 0;

    for item in &estimate.items {
        match remove_recovery_item(item) {
            Ok(()) => {
                removed_bytes = removed_bytes.saturating_add(item.size_bytes);
                actions_completed += 1;
            }
            Err(error) => errors.push(format!("{}: {}", item.name, error)),
        }
    }

    if estimate.action_id == "clear-obsolete-caches" {
        let _ = remove_empty_cache_dirs();
    }

    let (current_free_space_bytes, verified) = verify_free_space(before_free_bytes, removed_bytes)?;
    let actual_delta = current_free_space_bytes.saturating_sub(before_free_bytes);
    let recovered_bytes = actual_delta.max(removed_bytes).min(estimated_bytes.max(removed_bytes));
    let duration_seconds = started_at.elapsed().as_secs();

    Ok(CareActionRunResult {
        action_id: mission_action_id(estimate.action_id),
        title: estimate.title.to_string(),
        success: errors.is_empty(),
        completed: errors.is_empty(),
        skipped: estimate.items.is_empty(),
        failed: !errors.is_empty(),
        storage_before: format_bytes(before_free_bytes),
        storage_before_bytes: before_free_bytes,
        storage_after: format_bytes(current_free_space_bytes),
        storage_after_bytes: current_free_space_bytes,
        recovered: format_bytes(recovered_bytes),
        recovered_bytes,
        recovered_space: format_bytes(recovered_bytes),
        recovered_space_bytes: recovered_bytes,
        current_free_space: format_bytes(current_free_space_bytes),
        current_free_space_bytes,
        storage_health: storage_health_label(current_free_space_bytes),
        duration: format_duration(duration_seconds),
        duration_seconds,
        actions_completed,
        skipped_items: errors.len(),
        verified,
        verification: if verified {
            "System Pulse measured your free space again after the action.".to_string()
        } else {
            "System Pulse ran the action, but macOS has not reported a free-space increase yet.".to_string()
        },
        errors,
    })
}

fn mission_action_id(action_id: &str) -> String {
    format!("{STORAGE_MISSION_ID}:{action_id}")
}

fn storage_action_id(action_id: &str) -> &str {
    action_id.strip_prefix("storage-recovery:").unwrap_or(action_id)
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
    for entry in
        fs::read_dir(root).map_err(|error| format!("Could not read {}: {error}", root.display()))?
    {
        let entry = match entry {
            Ok(entry) => entry,
            Err(_) => continue,
        };
        let path = entry.path();
        let size_bytes = path_size(&path);
        if size_bytes == 0 {
            continue;
        }
        items.push(recovery_item(
            path,
            size_bytes,
            "Trash item",
            "This item is already in Trash, so it has already been removed from active work.",
            "High",
            "None",
        ));
    }
    Ok(items)
}

fn installer_files(root: &Path, min_age_days: u64) -> Result<Vec<RecoveryItem>, String> {
    if !root.exists() {
        return Ok(Vec::new());
    }

    let mut items = Vec::new();
    for entry in
        fs::read_dir(root).map_err(|error| format!("Could not read {}: {error}", root.display()))?
    {
        let entry = match entry {
            Ok(entry) => entry,
            Err(_) => continue,
        };
        let path = entry.path();
        if !is_regular_file(&path)
            || !is_allowed_installer(&path)
            || !is_older_than(&path, min_age_days)
        {
            continue;
        }
        let size_bytes = path_size(&path);
        if size_bytes == 0 {
            continue;
        }
        let reason = installer_reason(&path);
        items.push(recovery_item(
            path,
            size_bytes,
            "Downloaded installer",
            &reason,
            "High",
            "None",
        ));
    }
    Ok(items)
}

fn installer_reason(path: &Path) -> String {
    let appears_installed = application_appears_installed(path);
    if appears_installed {
        "The app appears to already be installed. Keeping this file is only useful if you want an offline reinstall copy.".to_string()
    } else {
        "This is an old installer or archive. Removing it will not remove an installed app or your documents.".to_string()
    }
}

fn application_appears_installed(path: &Path) -> bool {
    let Some(stem) = path.file_stem().and_then(|value| value.to_str()) else {
        return false;
    };
    let normalized_stem = normalize_app_name(stem);
    if normalized_stem.is_empty() {
        return false;
    }

    [Path::new("/Applications"), Path::new("/System/Applications")]
        .iter()
        .filter_map(|root| fs::read_dir(root).ok())
        .flat_map(|entries| entries.flatten())
        .map(|entry| entry.path())
        .filter(|path| path.extension().and_then(|value| value.to_str()) == Some("app"))
        .filter_map(|path| {
            path.file_stem()
                .and_then(|value| value.to_str())
                .map(normalize_app_name)
        })
        .any(|app_name| {
            app_name == normalized_stem
                || normalized_stem.contains(&app_name)
                || app_name.contains(&normalized_stem)
        })
}

fn normalize_app_name(value: &str) -> String {
    let lower = value.to_ascii_lowercase();
    let without_version = lower
        .split(|character: char| character.is_ascii_digit())
        .next()
        .unwrap_or(&lower);
    without_version
        .chars()
        .filter(|character| character.is_ascii_alphanumeric())
        .collect::<String>()
}

fn obsolete_cache_files(root: &Path, min_age_days: u64) -> Result<Vec<RecoveryItem>, String> {
    if !root.exists() {
        return Ok(Vec::new());
    }

    let mut items = Vec::new();
    for entry in
        fs::read_dir(root).map_err(|error| format!("Could not read {}: {error}", root.display()))?
    {
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
            items.push(recovery_item(
                path,
                size_bytes,
                "Temporary file",
                "Your application can recreate this automatically if it needs it again.",
                "Medium",
                "The app may open slightly slower the first time after cleaning.",
            ));
        }
    }
}

fn recovery_item(
    path: PathBuf,
    size_bytes: u64,
    item_kind: &'static str,
    reason: &str,
    confidence: &'static str,
    interruption: &'static str,
) -> RecoveryItem {
    let name = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("Item")
        .to_string();
    let display_path = user_facing_path(&path);
    RecoveryItem {
        name,
        item_kind,
        reason: reason.to_string(),
        confidence,
        interruption,
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

fn format_duration(seconds: u64) -> String {
    if seconds == 0 {
        "Under 1 second".to_string()
    } else if seconds == 1 {
        "1 second".to_string()
    } else if seconds < 60 {
        format!("{seconds} seconds")
    } else {
        let minutes = seconds / 60;
        let remaining_seconds = seconds % 60;
        if remaining_seconds == 0 {
            format!("{minutes} minutes")
        } else {
            format!("{minutes} minutes {remaining_seconds} seconds")
        }
    }
}
