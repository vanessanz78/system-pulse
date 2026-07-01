use serde::Serialize;

#[derive(Debug, Clone)]
pub struct SystemSnapshot {
    pub collected_at: String,
    pub platform: String,
    pub memory: MemorySnapshot,
    pub storage: StorageSnapshot,
    pub applications: Vec<ApplicationSnapshot>,
    pub browser: BrowserHealthSnapshot,
    pub renderers: RendererHealthSnapshot,
    pub window_server: Option<WindowServerSnapshot>,
}

#[derive(Debug, Clone)]
pub struct MemorySnapshot {
    pub total_bytes: u64,
    pub available_bytes: u64,
    pub used_bytes: u64,
    pub compressed_bytes: u64,
}

#[derive(Debug, Clone)]
pub struct StorageSnapshot {
    pub mount_point: String,
    pub total_bytes: u64,
    pub available_bytes: u64,
    pub used_bytes: u64,
}

#[derive(Debug, Clone)]
pub struct ApplicationSnapshot {
    pub name: String,
    pub memory_bytes: u64,
}

#[derive(Debug, Clone)]
pub struct BrowserHealthSnapshot {
    pub browsers: Vec<BrowserSnapshot>,
}

#[derive(Debug, Clone)]
pub struct BrowserSnapshot {
    pub name: String,
    pub memory_bytes: u64,
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
    pub browser_health: DomainHealth,
    pub application_health: DomainHealth,
    pub top_applications: Vec<ApplicationImpact>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum HealthState {
    Healthy,
    Attention,
    Critical,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DomainHealth {
    pub label: String,
    pub headline: String,
    pub detail: String,
    pub value: String,
    pub metric_label: String,
    pub metric_percent: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ApplicationImpact {
    pub name: String,
    pub memory_display: String,
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
