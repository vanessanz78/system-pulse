use serde::Serialize;

#[derive(Debug, Clone)]
pub struct SystemSnapshot {
    pub collected_at: String,
    pub platform: String,
    pub cpu: CpuSnapshot,
    pub memory: MemorySnapshot,
    pub storage: StorageSnapshot,
    pub disk_activity: DiskActivitySnapshot,
    pub applications: Vec<ApplicationSnapshot>,
    pub browser: BrowserHealthSnapshot,
    pub renderers: RendererHealthSnapshot,
    pub window_server: Option<WindowServerSnapshot>,
}

#[derive(Debug, Clone)]
pub struct CpuSnapshot {
    pub user_percent: f32,
    pub system_percent: f32,
    pub idle_percent: f32,
}

#[derive(Debug, Clone)]
pub struct MemorySnapshot {
    pub total_bytes: u64,
    pub available_bytes: u64,
    pub used_bytes: u64,
    pub compressed_bytes: u64,
    pub swap_total_bytes: u64,
    pub swap_used_bytes: u64,
}

#[derive(Debug, Clone)]
pub struct StorageSnapshot {
    pub mount_point: String,
    pub total_bytes: u64,
    pub available_bytes: u64,
    pub used_bytes: u64,
}

#[derive(Debug, Clone)]
pub struct DiskActivitySnapshot {
    pub megabytes_per_second: f32,
}

#[derive(Debug, Clone)]
pub struct ApplicationSnapshot {
    pub name: String,
    pub memory_bytes: u64,
    pub cpu_percent: f32,
}

#[derive(Debug, Clone)]
pub struct BrowserHealthSnapshot {
    pub browsers: Vec<BrowserSnapshot>,
}

#[derive(Debug, Clone)]
pub struct BrowserSnapshot {
    pub name: String,
    pub memory_bytes: u64,
    pub cpu_percent: f32,
    pub process_count: u32,
    pub renderer_count: u32,
    pub renderer_memory_bytes: u64,
    pub largest_renderer_bytes: u64,
    pub uptime_seconds: Option<u64>,
}

#[derive(Debug, Clone)]
pub struct RendererHealthSnapshot {
    pub total_count: u32,
    pub total_memory_bytes: u64,
    pub largest_renderer_name: Option<String>,
    pub largest_renderer_memory_bytes: u64,
    pub primary_browser: Option<String>,
    pub primary_browser_renderer_count: u32,
}

#[derive(Debug, Clone)]
pub struct WindowServerSnapshot {
    pub memory_bytes: u64,
    pub cpu_percent: f32,
    pub uptime_seconds: Option<u64>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TodayPulse {
    pub collected_at: String,
    pub platform: String,
    pub system_score: u8,
    pub health_state: HealthState,
    pub primary_explanation: String,
    pub primary_recommendation: String,
    pub estimated_additional_work_label: String,
    pub flow_remaining_label: String,
    pub flow_remaining_minutes: u32,
    pub memory_health: DomainHealth,
    pub storage_health: DomainHealth,
    pub processor_health: DomainHealth,
    pub browser_health: DomainHealth,
    pub application_health: DomainHealth,
    pub top_applications: Vec<ApplicationImpact>,
    pub focus_prediction: FocusPrediction,
    pub recovery_candidates: Vec<RecoveryCandidate>,
}

#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum HealthState {
    Healthy,
    Attention,
    Critical,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DomainHealth {
    pub label: String,
    pub headline: String,
    pub detail: String,
    pub value: String,
    pub metric_label: String,
    pub metric_percent: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ApplicationImpact {
    pub name: String,
    pub memory_display: String,
    pub cpu_display: String,
    pub impact_label: String,
    pub detail: String,
    pub care_label: String,
    pub care_detail: String,
    pub care_estimated_improvement: String,
    pub action_kind: String,
    pub action_target: String,
    pub action_label: String,
    pub show_opportunity: bool,
    pub protected_work: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FocusPrediction {
    pub remaining_minutes: u32,
    pub state: FocusState,
    pub confidence: f32,
    pub primary_reducer: Option<FocusContributor>,
    pub contributors: Vec<FocusContributor>,
    pub last_updated: String,
    pub staleness: PredictionStaleness,
    pub menu_bar_state: MenuBarState,
}

#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum FocusState {
    Green,
    Yellow,
    Orange,
    Red,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FocusContributor {
    pub domain: FocusDomain,
    pub label: String,
    pub state: FocusState,
    pub risk: f32,
    pub impact_minutes: i32,
    pub reason: String,
    pub supporting_metrics: Vec<SupportingMetric>,
    pub protected_work: bool,
    pub action_available: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum FocusDomain {
    Applications,
    Memory,
    Processor,
    Browser,
    Storage,
    Disk,
    Desktop,
    System,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SupportingMetric {
    pub label: String,
    pub value: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PredictionStaleness {
    pub status: StalenessStatus,
    pub age_seconds: u32,
}

#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum StalenessStatus {
    Fresh,
    Stale,
    Unknown,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RecoveryCandidate {
    pub domain: FocusDomain,
    pub action_kind: String,
    pub target: String,
    pub expected_gain_minutes: i32,
    pub estimated_interruption_seconds: u32,
    pub confidence: f32,
    pub safety_level: SafetyLevel,
    pub requires_confirmation: bool,
    pub can_automate: bool,
    pub session_preservation_risk: SessionPreservationRisk,
    pub reason: String,
    pub trust_notes: String,
}

#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum SafetyLevel {
    Safe,
    Caution,
    Restricted,
    Blocked,
}

#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum SessionPreservationRisk {
    None,
    Low,
    Medium,
    High,
    Unknown,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MenuBarState {
    pub state: FocusState,
    pub heart_color: String,
    pub minutes_label: String,
    pub shows_minutes: bool,
    pub critical_pulse: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ActionResult {
    pub action_kind: String,
    pub target: String,
    pub started_at: String,
    pub completed_at: Option<String>,
    pub success: bool,
    pub interruption_seconds: u32,
    pub before_prediction: Option<FocusPrediction>,
    pub after_prediction: Option<FocusPrediction>,
    pub actual_gain_minutes: Option<i32>,
    pub errors: Vec<String>,
    pub user_cancelled: bool,
}
