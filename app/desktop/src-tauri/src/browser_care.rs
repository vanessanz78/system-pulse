use crate::collectors;
use crate::mission_engine::{
    MissionAction, MissionExplanation, MissionLifecycle, MissionPreview, MissionPreviewFile,
    MissionProvider, MissionResult, PulseMission,
};
use crate::models::{BrowserSnapshot, SystemSnapshot};
use std::process::Command;
use std::thread;
use std::time::{Duration, Instant};

const BROWSER_MISSION_ID: &str = "browser-care";
const RESTART_BROWSER_ACTION_ID: &str = "restart-browser";

const MIN_BROWSER_MEMORY_BYTES: u64 = 1_200 * 1024 * 1024;
const MIN_RENDERER_COUNT: u32 = 24;
const MIN_CPU_PERCENT: f32 = 18.0;
const LONG_UPTIME_SECONDS: u64 = 72 * 60 * 60;

pub struct BrowserCareMissionProvider;

impl MissionProvider for BrowserCareMissionProvider {
    fn mission_id(&self) -> &'static str {
        BROWSER_MISSION_ID
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
struct BrowserObservation {
    name: String,
    memory_bytes: u64,
    cpu_percent: f32,
    process_count: u32,
    renderer_count: u32,
    renderer_memory_bytes: u64,
    largest_renderer_bytes: u64,
    uptime_seconds: Option<u64>,
    window_count: Option<u32>,
    tab_count: Option<u32>,
}

#[derive(Debug, Clone)]
struct BrowserActionEstimate {
    browser: BrowserObservation,
    estimated_recovery_bytes: u64,
    confidence: &'static str,
    confidence_reason: String,
    reason: String,
    risk: String,
    interruption: String,
}

pub fn plan() -> Result<PulseMission, String> {
    let snapshot = collectors::collect_system_snapshot()?;
    let Some(estimate) = browser_action_estimate(&snapshot) else {
        return Ok(PulseMission {
            id: BROWSER_MISSION_ID.to_string(),
            category: "Browser".to_string(),
            mission_title: "Browser Care".to_string(),
            title: "Everything looks good.".to_string(),
            summary: "Your browser is not currently affecting performance.".to_string(),
            explanation:
                "System Pulse checked supported browsers and did not find enough browser pressure to recommend an action."
                    .to_string(),
            confidence: "High".to_string(),
            confidence_reason: "Only local process information was checked. Browsing history and website content were not read.".to_string(),
            status: MissionLifecycle::Ready.as_str().to_string(),
            priority: 8,
            estimated_benefit: "No recovery needed".to_string(),
            estimated_benefit_bytes: 0,
            expected_benefit: "No recovery needed".to_string(),
            expected_interruption: "None".to_string(),
            estimated_duration: "No care moment needed".to_string(),
            diagnosis: "Browser load is not standing out right now.".to_string(),
            recovery_plan: "No browser care action is useful right now.".to_string(),
            actions: Vec::new(),
        });
    };

    Ok(PulseMission {
        id: BROWSER_MISSION_ID.to_string(),
        category: "Browser".to_string(),
        mission_title: "Browser Care".to_string(),
        title: browser_title(&estimate.browser),
        summary: browser_summary(&estimate),
        explanation:
            "This mission checks browser memory, renderer processes, processor use, and session age. It does not read browsing history or website content."
                .to_string(),
        confidence: estimate.confidence.to_string(),
        confidence_reason: estimate.confidence_reason.clone(),
        status: MissionLifecycle::Ready.as_str().to_string(),
        priority: 92,
        estimated_benefit: format_bytes(estimate.estimated_recovery_bytes),
        estimated_benefit_bytes: estimate.estimated_recovery_bytes,
        expected_benefit: format!(
            "Around {} RAM",
            format_bytes(estimate.estimated_recovery_bytes)
        ),
        expected_interruption: estimate.interruption.clone(),
        estimated_duration: "About 20 seconds".to_string(),
        diagnosis: browser_diagnosis(&estimate.browser),
        recovery_plan: format!(
            "Restart {} to recover browser memory without restarting your Mac.",
            estimate.browser.name
        ),
        actions: vec![action_summary(&estimate)],
    })
}

pub fn preview(action_id: &str) -> Result<MissionPreview, String> {
    ensure_action(action_id)?;
    let estimate = current_estimate()?;
    Ok(preview_from_estimate(&estimate))
}

pub fn explain(action_id: &str) -> Result<MissionExplanation, String> {
    ensure_action(action_id)?;
    let estimate = current_estimate()?;
    Ok(explanation_from_estimate(&estimate))
}

pub fn run(action_id: &str) -> Result<MissionResult, String> {
    ensure_action(action_id)?;
    let estimate = current_estimate()?;
    execute_estimate(estimate)
}

fn current_estimate() -> Result<BrowserActionEstimate, String> {
    let snapshot = collectors::collect_system_snapshot()?;
    browser_action_estimate(&snapshot)
        .ok_or_else(|| "Browser Care does not have a safe action to run right now.".to_string())
}

fn browser_action_estimate(snapshot: &SystemSnapshot) -> Option<BrowserActionEstimate> {
    let browser = snapshot
        .browser
        .browsers
        .iter()
        .filter(|browser| is_supported_browser(&browser.name))
        .max_by_key(|browser| browser_pressure_key(browser))?;

    if !needs_browser_care(browser) {
        return None;
    }

    let observation = observe_browser(browser);
    let estimated_recovery_bytes = estimate_recoverable_memory(&observation);
    if estimated_recovery_bytes < 350 * 1024 * 1024 {
        return None;
    }

    let (confidence, confidence_reason) = browser_confidence(&observation);
    Some(BrowserActionEstimate {
        reason: browser_reason(&observation),
        risk: "Low to medium. The browser will close and reopen. Tabs usually restore, but unsaved form text may not be preserved.".to_string(),
        interruption: "The browser will close and reopen. Tabs may reload when you return to them.".to_string(),
        browser: observation,
        estimated_recovery_bytes,
        confidence,
        confidence_reason,
    })
}

fn action_summary(estimate: &BrowserActionEstimate) -> MissionAction {
    MissionAction {
        id: mission_action_id(RESTART_BROWSER_ACTION_ID),
        title: "Reduce browser memory use".to_string(),
        description: format!(
            "Restart {} to release memory from old browser processes.",
            estimate.browser.name
        ),
        confidence: estimate.confidence.to_string(),
        confidence_reason: estimate.confidence_reason.clone(),
        why_recommended: estimate.reason.clone(),
        estimated_benefit: format_bytes(estimate.estimated_recovery_bytes),
        estimated_benefit_bytes: estimate.estimated_recovery_bytes,
        interruption: estimate.interruption.clone(),
        risk: estimate.risk.clone(),
        preview_item_count: technical_detail_count(&estimate.browser),
        status: MissionLifecycle::Ready.as_str().to_string(),
    }
}

fn preview_from_estimate(estimate: &BrowserActionEstimate) -> MissionPreview {
    MissionPreview {
        action_id: mission_action_id(RESTART_BROWSER_ACTION_ID),
        title: "Reduce browser memory use".to_string(),
        what_i_found: browser_diagnosis(&estimate.browser),
        why_selected: estimate.reason.clone(),
        confidence: estimate.confidence.to_string(),
        risk: estimate.risk.clone(),
        interruption: estimate.interruption.clone(),
        estimated_recovery: format_bytes(estimate.estimated_recovery_bytes),
        estimated_recovery_bytes: estimate.estimated_recovery_bytes,
        files: browser_preview_details(&estimate.browser),
        omitted_count: 0,
    }
}

fn explanation_from_estimate(estimate: &BrowserActionEstimate) -> MissionExplanation {
    MissionExplanation {
        action_id: mission_action_id(RESTART_BROWSER_ACTION_ID),
        title: "Reduce browser memory use".to_string(),
        reason: format!(
            "{} creates separate processes to keep tabs stable and secure. Over time those processes can use more memory. Restarting the browser is the safest automatic action available right now.",
            estimate.browser.name
        ),
        confidence: estimate.confidence.to_string(),
        confidence_reason: estimate.confidence_reason.clone(),
        expected_benefit: format_bytes(estimate.estimated_recovery_bytes),
        risk: estimate.risk.clone(),
        interruption: estimate.interruption.clone(),
    }
}

fn execute_estimate(estimate: BrowserActionEstimate) -> Result<MissionResult, String> {
    let started_at = Instant::now();
    let before_memory_bytes = estimate.browser.memory_bytes;
    let mut errors = Vec::new();

    if let Err(error) = restart_browser(&estimate.browser.name) {
        errors.push(error);
    }

    let after = read_browser_after_restart(&estimate.browser.name);
    let after_memory_bytes = after.as_ref().map(|browser| browser.memory_bytes).unwrap_or(0);
    let recovered_bytes = before_memory_bytes.saturating_sub(after_memory_bytes);
    let opened_again = after.is_some();
    let success = errors.is_empty() && opened_again;
    let duration_seconds = started_at.elapsed().as_secs();

    Ok(MissionResult {
        action_id: mission_action_id(RESTART_BROWSER_ACTION_ID),
        title: "Reduce browser memory use".to_string(),
        success,
        completed: success,
        skipped: false,
        failed: !errors.is_empty(),
        storage_before: format_bytes(before_memory_bytes),
        storage_before_bytes: before_memory_bytes,
        storage_after: if opened_again {
            format_bytes(after_memory_bytes)
        } else {
            "Not running".to_string()
        },
        storage_after_bytes: after_memory_bytes,
        recovered: format_bytes(recovered_bytes),
        recovered_bytes,
        recovered_space: format_bytes(recovered_bytes),
        recovered_space_bytes: recovered_bytes,
        current_free_space: if opened_again {
            format!("{} remains open", estimate.browser.name)
        } else {
            format!("{} did not reopen", estimate.browser.name)
        },
        current_free_space_bytes: after_memory_bytes,
        storage_health: if success {
            "Browser refreshed".to_string()
        } else {
            "Needs review".to_string()
        },
        duration: format_duration(duration_seconds),
        duration_seconds,
        actions_completed: if success { 1 } else { 0 },
        skipped_items: 0,
        verified: opened_again,
        verification: if opened_again && recovered_bytes > 0 {
            format!(
                "System Pulse restarted {} and measured browser memory again.",
                estimate.browser.name
            )
        } else if opened_again {
            format!(
                "System Pulse restarted {}. macOS did not report lower browser memory yet.",
                estimate.browser.name
            )
        } else {
            format!(
                "System Pulse tried to restart {}, but could not confirm it reopened.",
                estimate.browser.name
            )
        },
        errors,
    })
}

fn observe_browser(browser: &BrowserSnapshot) -> BrowserObservation {
    let (window_count, tab_count) = collect_browser_tabs(&browser.name).unwrap_or((None, None));
    BrowserObservation {
        name: browser.name.clone(),
        memory_bytes: browser.memory_bytes,
        cpu_percent: browser.cpu_percent,
        process_count: browser.process_count,
        renderer_count: browser.renderer_count,
        renderer_memory_bytes: browser.renderer_memory_bytes,
        largest_renderer_bytes: browser.largest_renderer_bytes,
        uptime_seconds: browser.uptime_seconds,
        window_count,
        tab_count,
    }
}

fn read_browser_after_restart(browser_name: &str) -> Option<BrowserObservation> {
    thread::sleep(Duration::from_millis(1_800));
    let snapshot = collectors::collect_system_snapshot().ok()?;
    snapshot
        .browser
        .browsers
        .iter()
        .find(|browser| browser.name == browser_name)
        .map(observe_browser)
}

fn restart_browser(browser_name: &str) -> Result<(), String> {
    ensure_supported_browser(browser_name)?;
    quit_browser(browser_name)?;
    thread::sleep(Duration::from_millis(1_200));
    open_browser(browser_name)?;
    Ok(())
}

fn quit_browser(browser_name: &str) -> Result<(), String> {
    let script = format!(
        "tell application \"{}\" to quit",
        escape_applescript(browser_name)
    );
    run_status(
        "osascript",
        &["-e", &script],
        &format!("Could not ask {browser_name} to quit"),
    )
}

fn open_browser(browser_name: &str) -> Result<(), String> {
    run_status(
        "open",
        &["-a", browser_name],
        &format!("Could not reopen {browser_name}"),
    )
}

fn run_status(command: &str, args: &[&str], context: &str) -> Result<(), String> {
    let status = Command::new(command)
        .args(args)
        .status()
        .map_err(|error| format!("{context}: {error}"))?;

    if status.success() {
        Ok(())
    } else {
        Err(format!("{context}."))
    }
}

fn collect_browser_tabs(browser_name: &str) -> Result<(Option<u32>, Option<u32>), String> {
    ensure_supported_browser(browser_name)?;
    let script = format!(
        r#"tell application "{}"
set windowCount to count windows
set tabCount to 0
repeat with browserWindow in windows
    set tabCount to tabCount + (count tabs of browserWindow)
end repeat
return (windowCount as text) & "," & (tabCount as text)
end tell"#,
        escape_applescript(browser_name)
    );

    let output = Command::new("osascript")
        .args(["-e", &script])
        .output()
        .map_err(|error| format!("Could not count browser tabs: {error}"))?;

    if !output.status.success() {
        return Ok((None, None));
    }

    let text = String::from_utf8(output.stdout)
        .map_err(|error| format!("Browser tab count was not readable: {error}"))?;
    let mut parts = text.trim().split(',');
    let windows = parts.next().and_then(|value| value.parse::<u32>().ok());
    let tabs = parts.next().and_then(|value| value.parse::<u32>().ok());
    Ok((windows, tabs))
}

fn browser_preview_details(browser: &BrowserObservation) -> Vec<MissionPreviewFile> {
    let mut details = vec![
        preview_detail(
            "Browser memory",
            "Memory",
            format_bytes(browser.memory_bytes),
            &browser.name,
            "This is the memory macOS reports for the browser and its helper processes.",
        ),
        preview_detail(
            "Processor use",
            "CPU",
            format_cpu(browser.cpu_percent),
            &browser.name,
            "This is how much processor work the browser is doing right now.",
        ),
        preview_detail(
            "Browser processes",
            "Processes",
            format!("{} processes", browser.process_count),
            &browser.name,
            "Browsers split work into separate helper processes for stability.",
        ),
        preview_detail(
            "Renderer processes",
            "Processes",
            format!("{} renderers", browser.renderer_count),
            &browser.name,
            "Browsers use separate processes to keep tabs stable and secure.",
        ),
        preview_detail(
            "Largest renderer",
            "Memory",
            format_bytes(browser.largest_renderer_bytes),
            &browser.name,
            "This is the largest single browser helper process macOS reported.",
        ),
        preview_detail(
            "Browser uptime",
            "Session age",
            browser
                .uptime_seconds
                .map(format_duration)
                .unwrap_or_else(|| "Unknown".to_string()),
            &browser.name,
            "Long browser sessions can hold onto memory from earlier work.",
        ),
    ];

    if let Some(tab_count) = browser.tab_count {
        details.push(preview_detail(
            "Open tabs",
            "Tabs",
            format!("{tab_count} tabs"),
            &browser.name,
            "Only the number of open tabs was counted. Page titles, URLs, and browsing history were not read.",
        ));
    }

    if let (Some(window_count), Some(tab_count)) = (browser.window_count, browser.tab_count) {
        let inactive_tabs = tab_count.saturating_sub(window_count);
        details.push(preview_detail(
            "Inactive tabs",
            "Tabs",
            format!("{inactive_tabs} inactive"),
            &browser.name,
            "This is an estimate based only on tab and window counts.",
        ));
        details.push(preview_detail(
            "Background windows",
            "Windows",
            format!("{} background", window_count.saturating_sub(1)),
            &browser.name,
            "Windows behind the front browser window can keep tabs active.",
        ));
    }

    details.push(preview_detail(
        "Duplicate tabs",
        "Privacy",
        "Not inspected",
        &browser.name,
        "System Pulse does not read URLs or page content, so duplicate tabs are not checked in this build.",
    ));

    details
}

fn preview_detail(
    name: &str,
    item_kind: &str,
    size: String,
    path: &str,
    reason: &str,
) -> MissionPreviewFile {
    MissionPreviewFile {
        name: name.to_string(),
        item_kind: item_kind.to_string(),
        size,
        path: path.to_string(),
        reason: reason.to_string(),
        confidence: "Medium".to_string(),
        expected_benefit: "Recover browser memory".to_string(),
        interruption: "The browser will close and reopen.".to_string(),
    }
}

fn needs_browser_care(browser: &BrowserSnapshot) -> bool {
    browser.memory_bytes >= MIN_BROWSER_MEMORY_BYTES
        || browser.renderer_count >= MIN_RENDERER_COUNT
        || browser.cpu_percent >= MIN_CPU_PERCENT
        || browser
            .uptime_seconds
            .map(|uptime| {
                uptime >= LONG_UPTIME_SECONDS && browser.memory_bytes >= 800 * 1024 * 1024
            })
            .unwrap_or(false)
}

fn browser_pressure_key(browser: &BrowserSnapshot) -> u64 {
    browser
        .memory_bytes
        .saturating_add(browser.renderer_memory_bytes / 2)
        .saturating_add((browser.renderer_count as u64).saturating_mul(40 * 1024 * 1024))
        .saturating_add((browser.cpu_percent.max(0.0) as u64).saturating_mul(25 * 1024 * 1024))
}

fn estimate_recoverable_memory(browser: &BrowserObservation) -> u64 {
    let renderer_estimate = browser.renderer_memory_bytes.saturating_mul(45) / 100;
    let total_estimate = browser.memory_bytes.saturating_mul(30) / 100;
    renderer_estimate.max(total_estimate).min(browser.memory_bytes)
}

fn browser_confidence(browser: &BrowserObservation) -> (&'static str, String) {
    if browser.name == "Safari" {
        (
            "Medium",
            "Safari can be restarted safely, but macOS session restoration depends on your current Safari settings.".to_string(),
        )
    } else {
        (
            "Medium",
            format!(
                "{} usually restores tabs after restart, but unsaved form text may not be preserved.",
                browser.name
            ),
        )
    }
}

fn browser_reason(browser: &BrowserObservation) -> String {
    if browser.renderer_count >= MIN_RENDERER_COUNT {
        return format!(
            "{} currently has many tabs or renderer processes open and is using more memory than usual.",
            browser.name
        );
    }
    if browser
        .uptime_seconds
        .map(|uptime| uptime >= LONG_UPTIME_SECONDS)
        .unwrap_or(false)
    {
        return format!(
            "{} has been running for a long time. Some memory can usually be recovered by restarting it.",
            browser.name
        );
    }
    if browser.cpu_percent >= MIN_CPU_PERCENT {
        return format!(
            "{} is doing more processor work than usual. Restarting it is the safest automatic browser action available.",
            browser.name
        );
    }
    format!(
        "{} is using more memory than usual. Restarting it can release memory from older browser processes.",
        browser.name
    )
}

fn browser_title(browser: &BrowserObservation) -> String {
    format!("{} is using more memory than usual.", browser.name)
}

fn browser_summary(estimate: &BrowserActionEstimate) -> String {
    format!(
        "{} Recommended: reduce browser memory use. You'll recover around {} RAM.",
        estimate.browser.name,
        format_bytes(estimate.estimated_recovery_bytes)
    )
}

fn browser_diagnosis(browser: &BrowserObservation) -> String {
    if browser.renderer_count >= MIN_RENDERER_COUNT {
        return format!(
            "{} currently has many tabs open and is using more memory than usual.",
            browser.name
        );
    }
    if browser
        .uptime_seconds
        .map(|uptime| uptime >= LONG_UPTIME_SECONDS)
        .unwrap_or(false)
    {
        return format!(
            "{} has been running for {}. Some memory can usually be recovered by restarting it.",
            browser.name,
            format_duration(browser.uptime_seconds.unwrap_or(0))
        );
    }
    format!(
        "{} is currently your busiest browser and is using more memory than usual.",
        browser.name
    )
}

fn technical_detail_count(browser: &BrowserObservation) -> usize {
    browser_preview_details(browser).len()
}

fn mission_action_id(action_id: &str) -> String {
    format!("{BROWSER_MISSION_ID}:{action_id}")
}

fn ensure_action(action_id: &str) -> Result<(), String> {
    match browser_action_id(action_id) {
        RESTART_BROWSER_ACTION_ID => Ok(()),
        _ => Err("System Pulse does not know that browser care action yet.".to_string()),
    }
}

fn browser_action_id(action_id: &str) -> &str {
    action_id.strip_prefix("browser-care:").unwrap_or(action_id)
}

fn ensure_supported_browser(browser_name: &str) -> Result<(), String> {
    if is_supported_browser(browser_name) {
        Ok(())
    } else {
        Err("Browser Care is currently wired for Chrome, Edge, and Safari.".to_string())
    }
}

fn is_supported_browser(browser_name: &str) -> bool {
    matches!(browser_name, "Google Chrome" | "Microsoft Edge" | "Safari")
}

fn escape_applescript(value: &str) -> String {
    value.replace('\\', "\\\\").replace('"', "\\\"")
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

fn format_cpu(percent: f32) -> String {
    if percent >= 10.0 {
        format!("{}%", percent.round() as u32)
    } else {
        format!("{percent:.1}%")
    }
}

fn format_duration(seconds: u64) -> String {
    if seconds < 60 {
        return "Under 1 minute".to_string();
    }
    let days = seconds / 86_400;
    if days >= 1 {
        return if days == 1 {
            "1 day".to_string()
        } else {
            format!("{days} days")
        };
    }
    let hours = seconds / 3_600;
    if hours >= 1 {
        return if hours == 1 {
            "1 hour".to_string()
        } else {
            format!("{hours} hours")
        };
    }
    let minutes = seconds / 60;
    if minutes == 1 {
        "1 minute".to_string()
    } else {
        format!("{minutes} minutes")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn browser(memory_bytes: u64, renderer_count: u32, cpu_percent: f32) -> BrowserSnapshot {
        BrowserSnapshot {
            name: "Google Chrome".to_string(),
            memory_bytes,
            cpu_percent,
            process_count: 30,
            renderer_count,
            renderer_memory_bytes: memory_bytes / 2,
            largest_renderer_bytes: memory_bytes / 8,
            uptime_seconds: Some(60),
        }
    }

    #[test]
    fn browser_care_requires_meaningful_pressure() {
        assert!(!needs_browser_care(&browser(200 * 1024 * 1024, 4, 1.0)));
        assert!(needs_browser_care(&browser(
            2_u64 * 1024 * 1024 * 1024,
            18,
            2.0
        )));
        assert!(needs_browser_care(&browser(800 * 1024 * 1024, 30, 2.0)));
        assert!(needs_browser_care(&browser(800 * 1024 * 1024, 8, 24.0)));
    }

    #[test]
    fn browser_action_ids_accept_prefixed_actions() {
        assert!(ensure_action("browser-care:restart-browser").is_ok());
        assert!(ensure_action("restart-browser").is_ok());
        assert!(ensure_action("browser-care:fake").is_err());
    }
}
