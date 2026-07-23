use serde::Serialize;
use std::time::Instant;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MissionLifecycle {
    Observed,
    Diagnosed,
    Ready,
    Previewed,
    Approved,
    Running,
    Verifying,
    Completed,
    Deferred,
    Unavailable,
    Failed,
}

impl MissionLifecycle {
    pub fn as_str(self) -> &'static str {
        match self {
            MissionLifecycle::Observed => "Observed",
            MissionLifecycle::Diagnosed => "Diagnosed",
            MissionLifecycle::Ready => "Ready",
            MissionLifecycle::Previewed => "Previewed",
            MissionLifecycle::Approved => "Approved",
            MissionLifecycle::Running => "Running",
            MissionLifecycle::Verifying => "Verifying",
            MissionLifecycle::Completed => "Completed",
            MissionLifecycle::Deferred => "Deferred",
            MissionLifecycle::Unavailable => "Unavailable",
            MissionLifecycle::Failed => "Failed",
        }
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PulseMission {
    pub id: String,
    pub category: String,
    pub mission_title: String,
    pub title: String,
    pub summary: String,
    pub explanation: String,
    pub confidence: String,
    pub confidence_reason: String,
    pub status: String,
    pub priority: u32,
    pub estimated_benefit: String,
    pub estimated_benefit_bytes: u64,
    pub expected_benefit: String,
    pub expected_interruption: String,
    pub estimated_duration: String,
    pub diagnosis: String,
    pub recovery_plan: String,
    pub actions: Vec<MissionAction>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MissionAction {
    pub id: String,
    pub title: String,
    pub description: String,
    pub confidence: String,
    pub confidence_reason: String,
    pub why_recommended: String,
    pub estimated_benefit: String,
    pub estimated_benefit_bytes: u64,
    pub interruption: String,
    pub risk: String,
    pub preview_item_count: usize,
    pub status: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MissionPreview {
    pub action_id: String,
    pub title: String,
    pub what_i_found: String,
    pub why_selected: String,
    pub confidence: String,
    pub risk: String,
    pub interruption: String,
    pub estimated_recovery: String,
    pub estimated_recovery_bytes: u64,
    pub files: Vec<MissionPreviewFile>,
    pub omitted_count: usize,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MissionPreviewFile {
    pub name: String,
    pub item_kind: String,
    pub size: String,
    pub path: String,
    pub reason: String,
    pub confidence: String,
    pub expected_benefit: String,
    pub interruption: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MissionExplanation {
    pub action_id: String,
    pub title: String,
    pub reason: String,
    pub confidence: String,
    pub confidence_reason: String,
    pub expected_benefit: String,
    pub risk: String,
    pub interruption: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MissionResult {
    pub action_id: String,
    pub title: String,
    pub success: bool,
    pub completed: bool,
    pub skipped: bool,
    pub failed: bool,
    pub storage_before: String,
    pub storage_before_bytes: u64,
    pub storage_after: String,
    pub storage_after_bytes: u64,
    pub recovered: String,
    pub recovered_bytes: u64,
    pub recovered_space: String,
    pub recovered_space_bytes: u64,
    pub current_free_space: String,
    pub current_free_space_bytes: u64,
    pub storage_health: String,
    pub duration: String,
    pub duration_seconds: u64,
    pub actions_completed: usize,
    pub skipped_items: usize,
    pub verified: bool,
    pub verification: String,
    pub errors: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MissionEstimate {
    pub action_id: String,
    pub title: String,
    pub estimated_benefit: String,
    pub estimated_benefit_bytes: u64,
    pub confidence: String,
    pub interruption: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MissionVerification {
    pub verified: bool,
    pub verification: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MissionRegistrySnapshot {
    pub top_mission: Option<PulseMission>,
    pub other_opportunities: Vec<PulseMission>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AskPulseRoute {
    pub route: String,
    pub category: String,
    pub mission_id: Option<String>,
    pub reason: String,
}

pub trait CareAction {
    fn id(&self) -> &'static str;
    fn title(&self) -> &'static str;
    fn confidence(&self) -> &'static str;
    fn interruption(&self) -> &'static str;
    fn preview(&self) -> Result<MissionPreview, String>;
    fn explain(&self) -> Result<MissionExplanation, String>;
    fn estimate(&self) -> Result<MissionEstimate, String>;
    fn execute(&self) -> Result<MissionResult, String>;
    fn verify(&self, result: &MissionResult) -> MissionVerification;
}

pub trait MissionProvider {
    fn mission_id(&self) -> &'static str;
    fn load(&self) -> Result<Option<PulseMission>, String>;
    fn preview(&self, action_id: &str) -> Result<MissionPreview, String>;
    fn explain(&self, action_id: &str) -> Result<MissionExplanation, String>;
    fn execute(&self, action_id: &str) -> Result<MissionResult, String>;
}

#[derive(Default)]
pub struct MissionRegistry {
    providers: Vec<Box<dyn MissionProvider>>,
}

impl MissionRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register<P>(&mut self, provider: P)
    where
        P: MissionProvider + 'static,
    {
        self.providers.push(Box::new(provider));
    }

    pub fn snapshot(&self) -> Result<MissionRegistrySnapshot, String> {
        let mut missions = self
            .providers
            .iter()
            .filter_map(|provider| provider.load().transpose())
            .collect::<Result<Vec<_>, _>>()?;

        MissionPlanner::rank(&mut missions);
        let top_mission = missions.first().cloned();
        let other_opportunities = missions.into_iter().skip(1).collect::<Vec<_>>();

        Ok(MissionRegistrySnapshot {
            top_mission,
            other_opportunities,
        })
    }

    pub fn preview(&self, action_id: &str) -> Result<MissionPreview, String> {
        self.find_provider(action_id)?.preview(action_id)
    }

    pub fn explain(&self, action_id: &str) -> Result<MissionExplanation, String> {
        self.find_provider(action_id)?.explain(action_id)
    }

    pub fn execute(&self, action_id: &str) -> Result<MissionResult, String> {
        self.find_provider(action_id)?.execute(action_id)
    }

    fn find_provider(&self, action_id: &str) -> Result<&dyn MissionProvider, String> {
        let mission_id = action_id
            .split(':')
            .next()
            .filter(|value| !value.is_empty())
            .ok_or_else(|| "System Pulse could not understand that care action.".to_string())?;

        self.providers
            .iter()
            .find(|provider| provider.mission_id() == mission_id)
            .map(|provider| provider.as_ref())
            .ok_or_else(|| "System Pulse does not know that mission yet.".to_string())
    }
}

pub struct MissionPlanner;

impl MissionPlanner {
    pub fn rank(missions: &mut [PulseMission]) {
        missions.sort_by(|left, right| {
            mission_score(right)
                .cmp(&mission_score(left))
                .then_with(|| left.title.cmp(&right.title))
        });
    }
}

pub struct MissionLifecycleMachine {
    state: MissionLifecycle,
}

impl MissionLifecycleMachine {
    pub fn new() -> Self {
        Self {
            state: MissionLifecycle::Observed,
        }
    }

    pub fn state(&self) -> MissionLifecycle {
        self.state
    }

    pub fn transition(&mut self, next: MissionLifecycle) -> Result<(), String> {
        if is_allowed_transition(self.state, next) {
            self.state = next;
            Ok(())
        } else {
            Err(format!(
                "Mission cannot move from {} to {}.",
                self.state.as_str(),
                next.as_str()
            ))
        }
    }
}

pub struct LocalMissionTelemetry {
    events: Vec<LocalMissionEvent>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LocalMissionEvent {
    pub mission_id: String,
    pub action_id: Option<String>,
    pub event: String,
    pub duration_ms: Option<u128>,
    pub verification: Option<String>,
}

impl LocalMissionTelemetry {
    pub fn new() -> Self {
        Self { events: Vec::new() }
    }

    pub fn started(&mut self, mission_id: &str, action_id: &str) -> Instant {
        self.events.push(LocalMissionEvent {
            mission_id: mission_id.to_string(),
            action_id: Some(action_id.to_string()),
            event: "Mission started".to_string(),
            duration_ms: None,
            verification: None,
        });
        Instant::now()
    }

    pub fn completed(&mut self, mission_id: &str, action_id: &str, started_at: Instant, verification: &str) {
        self.events.push(LocalMissionEvent {
            mission_id: mission_id.to_string(),
            action_id: Some(action_id.to_string()),
            event: "Mission completed".to_string(),
            duration_ms: Some(started_at.elapsed().as_millis()),
            verification: Some(verification.to_string()),
        });
    }

    pub fn cancelled(&mut self, mission_id: &str, action_id: &str) {
        self.record(mission_id, Some(action_id), "Mission cancelled");
    }

    pub fn deferred(&mut self, mission_id: &str) {
        self.record(mission_id, None, "Mission deferred");
    }

    pub fn events(&self) -> &[LocalMissionEvent] {
        &self.events
    }

    fn record(&mut self, mission_id: &str, action_id: Option<&str>, event: &str) {
        self.events.push(LocalMissionEvent {
            mission_id: mission_id.to_string(),
            action_id: action_id.map(str::to_string),
            event: event.to_string(),
            duration_ms: None,
            verification: None,
        });
    }
}

pub fn route_ask_pulse(query: &str, snapshot: &MissionRegistrySnapshot) -> AskPulseRoute {
    let normalized = query.to_ascii_lowercase();
    if normalized.contains("space")
        || normalized.contains("storage")
        || normalized.contains("trash")
        || normalized.contains("download")
        || normalized.contains("cache")
    {
        return AskPulseRoute {
            route: "mission".to_string(),
            category: "Storage".to_string(),
            mission_id: Some("storage-recovery".to_string()),
            reason: "That question is about recoverable storage, so Ask Pulse can route it to the Storage Mission without becoming a chatbot.".to_string(),
        };
    }

    if normalized.contains("browser")
        || normalized.contains("chrome")
        || normalized.contains("safari")
        || normalized.contains("edge")
        || normalized.contains("tab")
    {
        return AskPulseRoute {
            route: "mission".to_string(),
            category: "Browser".to_string(),
            mission_id: Some("browser-care".to_string()),
            reason: "That question is about browser performance, so Ask Pulse can route it to Browser Care without becoming a chatbot.".to_string(),
        };
    }

    if let Some(mission) = &snapshot.top_mission {
        return AskPulseRoute {
            route: "mission".to_string(),
            category: mission.category.clone(),
            mission_id: Some(mission.id.clone()),
            reason: "Ask Pulse can start with today's highest-value mission and keep the answer grounded in PulseCore output.".to_string(),
        };
    }

    AskPulseRoute {
        route: "today".to_string(),
        category: "Health".to_string(),
        mission_id: None,
        reason: "No specific mission is ready, so Ask Pulse should answer from Today's local health picture.".to_string(),
    }
}

fn mission_score(mission: &PulseMission) -> u64 {
    let benefit_score = (mission.estimated_benefit_bytes / (100 * 1024 * 1024)).min(100);
    let confidence_score = match mission.confidence.as_str() {
        "High" => 30,
        "Medium" => 18,
        "Low" => 6,
        _ => 10,
    };
    let interruption_penalty = match mission.expected_interruption.as_str() {
        "None" => 0,
        "Low" => 6,
        _ => 12,
    };
    mission
        .priority
        .saturating_add(benefit_score as u32)
        .saturating_add(confidence_score)
        .saturating_sub(interruption_penalty) as u64
}

fn is_allowed_transition(current: MissionLifecycle, next: MissionLifecycle) -> bool {
    use MissionLifecycle::*;
    matches!(
        (current, next),
        (Observed, Diagnosed)
            | (Diagnosed, Ready)
            | (Diagnosed, Unavailable)
            | (Ready, Previewed)
            | (Ready, Deferred)
            | (Ready, Approved)
            | (Ready, Unavailable)
            | (Previewed, Approved)
            | (Previewed, Deferred)
            | (Previewed, Ready)
            | (Approved, Running)
            | (Running, Verifying)
            | (Verifying, Completed)
            | (Verifying, Failed)
            | (Running, Failed)
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestProvider {
        mission: PulseMission,
    }

    impl MissionProvider for TestProvider {
        fn mission_id(&self) -> &'static str {
            "test"
        }

        fn load(&self) -> Result<Option<PulseMission>, String> {
            Ok(Some(self.mission.clone()))
        }

        fn preview(&self, _action_id: &str) -> Result<MissionPreview, String> {
            Ok(MissionPreview {
                action_id: "test:action".to_string(),
                title: "Preview".to_string(),
                what_i_found: "Found something.".to_string(),
                why_selected: "It helps.".to_string(),
                confidence: "High".to_string(),
                risk: "Low".to_string(),
                interruption: "None".to_string(),
                estimated_recovery: "1 GB".to_string(),
                estimated_recovery_bytes: 1_000_000_000,
                files: Vec::new(),
                omitted_count: 0,
            })
        }

        fn explain(&self, _action_id: &str) -> Result<MissionExplanation, String> {
            Ok(MissionExplanation {
                action_id: "test:action".to_string(),
                title: "Explain".to_string(),
                reason: "A safe reason.".to_string(),
                confidence: "High".to_string(),
                confidence_reason: "Known fixture.".to_string(),
                expected_benefit: "1 GB".to_string(),
                risk: "Low".to_string(),
                interruption: "None".to_string(),
            })
        }

        fn execute(&self, _action_id: &str) -> Result<MissionResult, String> {
            Ok(MissionResult {
                action_id: "test:action".to_string(),
                title: "Run".to_string(),
                success: true,
                completed: true,
                skipped: false,
                failed: false,
                storage_before: "10 GB".to_string(),
                storage_before_bytes: 10,
                storage_after: "11 GB".to_string(),
                storage_after_bytes: 11,
                recovered: "1 GB".to_string(),
                recovered_bytes: 1,
                recovered_space: "1 GB".to_string(),
                recovered_space_bytes: 1,
                current_free_space: "11 GB".to_string(),
                current_free_space_bytes: 11,
                storage_health: "Healthy".to_string(),
                duration: "Under 1 second".to_string(),
                duration_seconds: 0,
                actions_completed: 1,
                skipped_items: 0,
                verified: true,
                verification: "Verified".to_string(),
                errors: Vec::new(),
            })
        }
    }

    #[test]
    fn registry_registers_and_exposes_ranked_top_mission() {
        let mut registry = MissionRegistry::new();
        registry.register(TestProvider {
            mission: mission_fixture("test-a", 10, 100),
        });
        registry.register(TestProvider {
            mission: mission_fixture("test-b", 80, 2_000_000_000),
        });

        let snapshot = registry.snapshot().unwrap();
        assert_eq!(snapshot.top_mission.unwrap().id, "test-b");
        assert_eq!(snapshot.other_opportunities.len(), 1);
    }

    #[test]
    fn lifecycle_allows_only_standard_states() {
        let mut lifecycle = MissionLifecycleMachine::new();
        assert_eq!(lifecycle.state(), MissionLifecycle::Observed);
        lifecycle.transition(MissionLifecycle::Diagnosed).unwrap();
        lifecycle.transition(MissionLifecycle::Ready).unwrap();
        lifecycle.transition(MissionLifecycle::Previewed).unwrap();
        assert!(lifecycle.transition(MissionLifecycle::Completed).is_err());
        lifecycle.transition(MissionLifecycle::Approved).unwrap();
        lifecycle.transition(MissionLifecycle::Running).unwrap();
        lifecycle.transition(MissionLifecycle::Verifying).unwrap();
        lifecycle.transition(MissionLifecycle::Completed).unwrap();
    }

    #[test]
    fn registry_executes_and_verifies_actions_through_provider() {
        let mut registry = MissionRegistry::new();
        registry.register(TestProvider {
            mission: mission_fixture("test", 10, 100),
        });

        let preview = registry.preview("test:action").unwrap();
        let result = registry.execute("test:action").unwrap();

        assert_eq!(preview.title, "Preview");
        assert!(result.completed);
        assert!(result.verified);
        assert_eq!(result.verification, "Verified");
    }

    #[test]
    fn telemetry_records_cancellation_and_deferral_locally() {
        let mut telemetry = LocalMissionTelemetry::new();
        telemetry.cancelled("test", "test:action");
        telemetry.deferred("test");

        assert_eq!(telemetry.events().len(), 2);
        assert_eq!(telemetry.events()[0].event, "Mission cancelled");
        assert_eq!(telemetry.events()[1].event, "Mission deferred");
    }

    #[test]
    fn ask_pulse_routes_storage_questions_to_storage_mission() {
        let snapshot = MissionRegistrySnapshot {
            top_mission: None,
            other_opportunities: Vec::new(),
        };

        let route = route_ask_pulse("Free up space", &snapshot);

        assert_eq!(route.route, "mission");
        assert_eq!(route.category, "Storage");
        assert_eq!(route.mission_id, Some("storage-recovery".to_string()));
    }

    fn mission_fixture(id: &str, priority: u32, bytes: u64) -> PulseMission {
        PulseMission {
            id: id.to_string(),
            category: "Storage".to_string(),
            mission_title: "Storage Mission".to_string(),
            title: id.to_string(),
            summary: "Summary".to_string(),
            explanation: "Explanation".to_string(),
            confidence: "High".to_string(),
            confidence_reason: "Fixture".to_string(),
            status: MissionLifecycle::Ready.as_str().to_string(),
            priority,
            estimated_benefit: "1 GB".to_string(),
            estimated_benefit_bytes: bytes,
            expected_benefit: "Recover 1 GB".to_string(),
            expected_interruption: "None".to_string(),
            estimated_duration: "About 1 second".to_string(),
            diagnosis: "Recoverable space exists.".to_string(),
            recovery_plan: "Use the smallest safe action.".to_string(),
            actions: Vec::new(),
        }
    }
}
