use serde::Serialize;

#[derive(Debug, Clone)]
pub struct SystemSnapshot {
    pub collected_at: String,
    pub platform: String,
    pub memory: MemorySnapshot,
    pub storage: StorageSnapshot,
    pub applications: Vec<ApplicationSnapshot>,
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

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TodayPulse {
    pub collected_at: String,
    pub platform: String,
    pub system_score: u8,
    pub health_state: HealthState,
    pub primary_explanation: String,
    pub primary_recommendation: String,
    pub confidence: u8,
    pub expected_improvement: String,
    pub memory_health: DomainHealth,
    pub storage_health: DomainHealth,
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
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ApplicationImpact {
    pub name: String,
    pub memory_display: String,
    pub impact_label: String,
    pub detail: String,
}
