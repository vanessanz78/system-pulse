use crate::models::{
    ApplicationImpact, BrowserSnapshot, DomainHealth, FocusContributor, FocusDomain,
    FocusPrediction, FocusState, HealthState, MenuBarState, PredictionStaleness,
    RecoveryCandidate, SafetyLevel, SessionPreservationRisk, StalenessStatus, SupportingMetric,
    SystemSnapshot, TodayPulse,
};

pub fn evaluate(snapshot: SystemSnapshot) -> TodayPulse {
    let cpu_score = score_cpu(&snapshot);
    let memory_score = score_memory(&snapshot);
    let storage_score = score_storage(&snapshot);
    let disk_score = score_disk_activity(&snapshot);
    let application_score = score_applications(&snapshot);
    let browser_score = score_browser(&snapshot);
    let renderer_score = score_renderers(&snapshot);
    let window_server_score = score_window_server(&snapshot);
    let system_score = weighted_score(
        cpu_score,
        memory_score,
        storage_score,
        disk_score,
        application_score,
        browser_score,
        renderer_score,
        window_server_score,
    );
    let health_state = health_state(system_score);
    let top_applications = application_impacts(&snapshot);
    let memory_health = memory_health(&snapshot, memory_score);
    let storage_health = storage_health(&snapshot, storage_score);
    let processor_health = processor_health(&snapshot, cpu_score.min(window_server_score));
    let browser_health = browser_health(&snapshot, browser_score);
    let application_health = application_health(&snapshot, application_score);
    let recommendation = primary_recommendation(
        &snapshot,
        cpu_score,
        memory_score,
        storage_score,
        disk_score,
        application_score,
        browser_score,
        renderer_score,
        window_server_score,
        system_score,
    );
    let focus_prediction = focus_prediction(
        &snapshot,
        system_score,
        cpu_score,
        memory_score,
        storage_score,
        disk_score,
        application_score,
        browser_score,
        renderer_score,
        window_server_score,
        &memory_health,
        &storage_health,
        &processor_health,
        &browser_health,
        &application_health,
        &top_applications,
    );
    let recovery_candidates = recovery_candidates(
        &snapshot,
        &recommendation,
        &top_applications,
        &storage_health,
        browser_score.min(renderer_score),
        storage_score,
    );

    TodayPulse {
        collected_at: snapshot.collected_at,
        platform: snapshot.platform,
        system_score,
        health_state,
        primary_explanation: recommendation.explanation,
        primary_recommendation: recommendation.text,
        estimated_additional_work_label: recommendation.estimated_additional_work_label,
        flow_remaining_label: format_flow_remaining(focus_prediction.remaining_minutes),
        flow_remaining_minutes: focus_prediction.remaining_minutes,
        memory_health,
        storage_health,
        processor_health,
        browser_health,
        application_health,
        top_applications,
        focus_prediction,
        recovery_candidates,
    }
}

pub fn sync_focus_contracts(pulse: &mut TodayPulse) {
    pulse.flow_remaining_minutes = pulse.focus_prediction.remaining_minutes;
    pulse.flow_remaining_label = format_flow_remaining(pulse.focus_prediction.remaining_minutes);
    pulse.focus_prediction.state = focus_state_for_minutes(pulse.focus_prediction.remaining_minutes);
    pulse.focus_prediction.menu_bar_state =
        menu_bar_state_for_minutes(pulse.focus_prediction.remaining_minutes);
    pulse.focus_prediction.primary_reducer = pulse
        .focus_prediction
        .contributors
        .iter()
        .filter(|contributor| contributor.risk >= 0.20)
        .max_by(|left, right| {
            left.risk
                .partial_cmp(&right.risk)
                .unwrap_or(std::cmp::Ordering::Equal)
        })
        .cloned();
}

pub fn cap_focus_prediction(pulse: &mut TodayPulse, cap_minutes: u32) {
    if pulse.focus_prediction.remaining_minutes > cap_minutes {
        pulse.focus_prediction.remaining_minutes = cap_minutes;
    }
    sync_focus_contracts(pulse);
}

struct Recommendation {
    text: String,
    explanation: String,
    estimated_additional_work_label: String,
}

struct FocusSignal {
    domain: FocusDomain,
    label: &'static str,
    score: u8,
    risk: f32,
    weight: f32,
    reason: String,
    supporting_metrics: Vec<SupportingMetric>,
    protected_work: bool,
    action_available: bool,
    weak_cap_minutes: Option<u32>,
    stability_penalty: f32,
}

#[allow(clippy::too_many_arguments)]
fn focus_prediction(
    snapshot: &SystemSnapshot,
    system_score: u8,
    cpu_score: u8,
    memory_score: u8,
    storage_score: u8,
    disk_score: u8,
    application_score: u8,
    browser_score: u8,
    renderer_score: u8,
    window_server_score: u8,
    memory_health: &DomainHealth,
    storage_health: &DomainHealth,
    processor_health: &DomainHealth,
    browser_health: &DomainHealth,
    application_health: &DomainHealth,
    top_applications: &[ApplicationImpact],
) -> FocusPrediction {
    let signals = focus_signals(
        snapshot,
        cpu_score,
        memory_score,
        storage_score,
        disk_score,
        application_score,
        browser_score,
        renderer_score,
        window_server_score,
        memory_health,
        storage_health,
        processor_health,
        browser_health,
        application_health,
        top_applications,
    );
    let sustained_pressure = sustained_pressure_penalty(snapshot, &signals);
    let rising_pressure = rising_pressure_penalty(&signals);
    let volatility = volatility_penalty(&signals);
    let weighted_risk = signals
        .iter()
        .map(|signal| signal.risk * signal.weight)
        .sum::<f32>()
        .clamp(0.0, 1.0);
    let prediction_risk =
        (weighted_risk + sustained_pressure + rising_pressure + volatility).clamp(0.0, 1.0);
    let mut remaining_minutes = (420.0 * (1.0 - prediction_risk).powf(2.2)).round() as u32;
    remaining_minutes = remaining_minutes.clamp(8, 420);
    if let Some(cap_minutes) = signals
        .iter()
        .filter_map(|signal| signal.weak_cap_minutes)
        .min()
    {
        remaining_minutes = remaining_minutes.min(cap_minutes);
    }

    let contributors = focus_contributors(&signals, prediction_risk);
    let primary_reducer = contributors
        .iter()
        .filter(|contributor| contributor.risk >= 0.20)
        .max_by(|left, right| {
            left.risk
                .partial_cmp(&right.risk)
                .unwrap_or(std::cmp::Ordering::Equal)
        })
        .cloned();
    let confidence = prediction_confidence(
        &[
            system_score,
            cpu_score,
            memory_score,
            storage_score,
            disk_score,
            application_score,
            browser_score,
            renderer_score,
            window_server_score,
        ],
        &signals,
        sustained_pressure + rising_pressure,
        volatility,
    );

    FocusPrediction {
        remaining_minutes,
        state: focus_state_for_minutes(remaining_minutes),
        confidence,
        primary_reducer,
        contributors,
        last_updated: snapshot.collected_at.clone(),
        staleness: PredictionStaleness {
            status: StalenessStatus::Fresh,
            age_seconds: 0,
        },
        menu_bar_state: menu_bar_state_for_minutes(remaining_minutes),
    }
}

#[allow(clippy::too_many_arguments)]
fn focus_signals(
    snapshot: &SystemSnapshot,
    cpu_score: u8,
    memory_score: u8,
    storage_score: u8,
    disk_score: u8,
    application_score: u8,
    browser_score: u8,
    renderer_score: u8,
    window_server_score: u8,
    memory_health: &DomainHealth,
    storage_health: &DomainHealth,
    processor_health: &DomainHealth,
    browser_health: &DomainHealth,
    application_health: &DomainHealth,
    top_applications: &[ApplicationImpact],
) -> Vec<FocusSignal> {
    let browser_pressure_score = browser_score.min(renderer_score);
    let protected_work = top_applications.iter().any(|application| application.protected_work);
    let application_action_available = top_applications
        .iter()
        .any(|application| application.show_opportunity && application.action_kind != "none");
    let available_ratio = ratio(snapshot.memory.available_bytes, snapshot.memory.total_bytes);
    let swap_ratio = ratio(snapshot.memory.swap_used_bytes, snapshot.memory.total_bytes);
    let renderer_ratio = ratio(
        snapshot.renderers.total_memory_bytes,
        snapshot.memory.total_bytes,
    );
    let window_server_ratio = snapshot
        .window_server
        .as_ref()
        .map(|window_server| ratio(window_server.memory_bytes, snapshot.memory.total_bytes))
        .unwrap_or_default();

    vec![
        FocusSignal {
            domain: FocusDomain::Memory,
            label: "Memory",
            score: memory_score,
            risk: score_risk(memory_score)
                .max(ratio_risk(1.0 - available_ratio, 0.88, 0.98))
                .max(ratio_risk(swap_ratio, 0.05, 0.50)),
            weight: 0.25,
            reason: memory_health.detail.clone(),
            supporting_metrics: supporting_metrics(memory_health),
            protected_work,
            action_available: browser_pressure_score < 70,
            weak_cap_minutes: weak_cap_for_score(memory_score),
            stability_penalty: if swap_ratio >= 0.15 { 0.03 } else { 0.0 },
        },
        FocusSignal {
            domain: FocusDomain::Browser,
            label: "Browser",
            score: browser_pressure_score,
            risk: score_risk(browser_pressure_score)
                .max(browser_renderer_risk(snapshot))
                .max(ratio_risk(renderer_ratio, 0.14, 0.45)),
            weight: 0.20,
            reason: browser_health.detail.clone(),
            supporting_metrics: supporting_metrics(browser_health),
            protected_work: false,
            action_available: primary_browser(snapshot).is_some() && browser_pressure_score < 70,
            weak_cap_minutes: weak_cap_for_score(browser_pressure_score),
            stability_penalty: browser_stability_penalty(snapshot),
        },
        FocusSignal {
            domain: FocusDomain::Processor,
            label: "Processor",
            score: cpu_score,
            risk: score_risk(cpu_score)
                .max(ratio_risk((100.0 - snapshot.cpu.idle_percent) as f64, 65.0, 95.0)),
            weight: 0.20,
            reason: processor_health.detail.clone(),
            supporting_metrics: supporting_metrics(processor_health),
            protected_work,
            action_available: false,
            weak_cap_minutes: weak_cap_for_score(cpu_score),
            stability_penalty: if snapshot.cpu.idle_percent < 20.0 { 0.02 } else { 0.0 },
        },
        FocusSignal {
            domain: FocusDomain::Applications,
            label: "Applications",
            score: application_score,
            risk: score_risk(application_score),
            weight: 0.15,
            reason: application_health.detail.clone(),
            supporting_metrics: supporting_metrics(application_health),
            protected_work,
            action_available: application_action_available,
            weak_cap_minutes: weak_cap_for_score(application_score),
            stability_penalty: if protected_work { -0.02 } else { 0.0 },
        },
        FocusSignal {
            domain: FocusDomain::Desktop,
            label: "WindowServer activity",
            score: window_server_score,
            risk: score_risk(window_server_score)
                .max(ratio_risk(window_server_ratio, 0.05, 0.15)),
            weight: 0.08,
            reason: window_server_reason(snapshot),
            supporting_metrics: window_server_metrics(snapshot),
            protected_work: false,
            action_available: false,
            weak_cap_minutes: weak_cap_for_score(window_server_score),
            stability_penalty: if window_server_score < 70 { 0.02 } else { 0.0 },
        },
        FocusSignal {
            domain: FocusDomain::Disk,
            label: "Disk activity",
            score: disk_score,
            risk: score_risk(disk_score),
            weight: 0.07,
            reason: storage_health.detail.clone(),
            supporting_metrics: vec![SupportingMetric {
                label: "Disk activity".to_string(),
                value: format_disk_activity(snapshot.disk_activity.megabytes_per_second),
            }],
            protected_work: false,
            action_available: false,
            weak_cap_minutes: weak_cap_for_score(disk_score),
            stability_penalty: if disk_score < 70 { 0.03 } else { 0.0 },
        },
        FocusSignal {
            domain: FocusDomain::Storage,
            label: "Storage",
            score: storage_score,
            risk: score_risk(storage_score),
            weight: 0.05,
            reason: storage_health.detail.clone(),
            supporting_metrics: supporting_metrics(storage_health),
            protected_work: false,
            action_available: storage_score < 58,
            weak_cap_minutes: weak_cap_for_score(storage_score),
            stability_penalty: 0.0,
        },
    ]
}

fn focus_contributors(signals: &[FocusSignal], prediction_risk: f32) -> Vec<FocusContributor> {
    let baseline_minutes = (420.0 * (1.0 - prediction_risk).powf(2.2)).round();
    let mut contributors = signals
        .iter()
        .map(|signal| {
            let without_signal_risk =
                (prediction_risk - (signal.risk * signal.weight)).clamp(0.0, 1.0);
            let without_signal_minutes =
                (420.0 * (1.0 - without_signal_risk).powf(2.2)).round();
            FocusContributor {
                domain: signal.domain,
                label: signal.label.to_string(),
                state: focus_state_for_score(signal.score),
                risk: signal.risk,
                impact_minutes: (without_signal_minutes - baseline_minutes).max(0.0) as i32,
                reason: signal.reason.clone(),
                supporting_metrics: signal.supporting_metrics.clone(),
                protected_work: signal.protected_work,
                action_available: signal.action_available && !signal.protected_work,
            }
        })
        .collect::<Vec<_>>();

    contributors.sort_by(|left, right| {
        right
            .risk
            .partial_cmp(&left.risk)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    contributors
}

fn score_risk(score: u8) -> f32 {
    (1.0 - (score as f32 / 100.0)).clamp(0.0, 1.0)
}

fn ratio_risk(value: f64, low: f64, high: f64) -> f32 {
    if high <= low {
        return 0.0;
    }
    ((value - low) / (high - low)).clamp(0.0, 1.0) as f32
}

fn weak_cap_for_score(score: u8) -> Option<u32> {
    if score <= 30 {
        Some(14)
    } else if score <= 42 {
        Some(30)
    } else if score <= 55 {
        Some(60)
    } else {
        None
    }
}

fn browser_renderer_risk(snapshot: &SystemSnapshot) -> f32 {
    let Some(browser) = primary_browser(snapshot) else {
        return 0.0;
    };
    let renderer_count_risk = ratio_risk(browser.renderer_count as f64, 24.0, 90.0);
    let largest_renderer_risk = ratio_risk(
        ratio(browser.largest_renderer_bytes, snapshot.memory.total_bytes),
        0.08,
        0.30,
    );
    renderer_count_risk.max(largest_renderer_risk)
}

fn browser_stability_penalty(snapshot: &SystemSnapshot) -> f32 {
    let Some(browser) = primary_browser(snapshot) else {
        return 0.0;
    };
    let long_session = browser
        .uptime_seconds
        .map(|seconds| seconds >= 8 * 60 * 60)
        .unwrap_or(false);
    if long_session && (browser.renderer_count >= 36 || browser.cpu_percent >= 35.0) {
        0.04
    } else if browser.renderer_count >= 52 {
        0.03
    } else {
        0.0
    }
}

fn sustained_pressure_penalty(snapshot: &SystemSnapshot, signals: &[FocusSignal]) -> f32 {
    let signal_pressure = signals
        .iter()
        .map(|signal| signal.stability_penalty)
        .sum::<f32>();
    let concurrent_pressure = signals
        .iter()
        .filter(|signal| signal.risk >= 0.45)
        .count();
    let concurrent_penalty = if concurrent_pressure >= 3 {
        0.05
    } else if concurrent_pressure >= 2 {
        0.03
    } else {
        0.0
    };
    let memory_browser_penalty = if score_risk(score_memory(snapshot)) >= 0.45
        && score_risk(score_browser(snapshot).min(score_renderers(snapshot))) >= 0.45
    {
        0.03
    } else {
        0.0
    };

    (signal_pressure + concurrent_penalty + memory_browser_penalty).clamp(0.0, 0.16)
}

fn rising_pressure_penalty(signals: &[FocusSignal]) -> f32 {
    let severe_count = signals.iter().filter(|signal| signal.risk >= 0.58).count();
    let moderate_count = signals.iter().filter(|signal| signal.risk >= 0.36).count();

    if severe_count >= 2 {
        0.05
    } else if severe_count == 1 && moderate_count >= 3 {
        0.04
    } else if moderate_count >= 4 {
        0.03
    } else {
        0.0
    }
}

fn volatility_penalty(signals: &[FocusSignal]) -> f32 {
    let highest_risk = signals
        .iter()
        .map(|signal| signal.risk)
        .fold(0.0_f32, f32::max);
    let lowest_risk = signals
        .iter()
        .map(|signal| signal.risk)
        .fold(1.0_f32, f32::min);
    let spread = highest_risk - lowest_risk;

    if spread >= 0.55 {
        0.04
    } else if spread >= 0.35 {
        0.02
    } else {
        0.0
    }
}

fn window_server_reason(snapshot: &SystemSnapshot) -> String {
    if let Some(window_server) = &snapshot.window_server {
        format!(
            "Desktop responsiveness is using {} memory and {} CPU.",
            format_bytes(window_server.memory_bytes),
            format_cpu(window_server.cpu_percent)
        )
    } else {
        "Desktop responsiveness was not reported in this check-in.".to_string()
    }
}

fn window_server_metrics(snapshot: &SystemSnapshot) -> Vec<SupportingMetric> {
    if let Some(window_server) = &snapshot.window_server {
        vec![
            SupportingMetric {
                label: "Memory".to_string(),
                value: format_bytes(window_server.memory_bytes),
            },
            SupportingMetric {
                label: "CPU".to_string(),
                value: format_cpu(window_server.cpu_percent),
            },
        ]
    } else {
        Vec::new()
    }
}

fn supporting_metrics(health: &DomainHealth) -> Vec<SupportingMetric> {
    let mut metrics = Vec::new();
    if !health.metric_label.is_empty() {
        metrics.push(SupportingMetric {
            label: "Current signal".to_string(),
            value: health.metric_label.clone(),
        });
    }
    if !health.metric_percent.is_empty() {
        metrics.push(SupportingMetric {
            label: "Relative load".to_string(),
            value: health.metric_percent.clone(),
        });
    }
    metrics
}

fn recovery_candidates(
    snapshot: &SystemSnapshot,
    recommendation: &Recommendation,
    top_applications: &[ApplicationImpact],
    storage_health: &DomainHealth,
    browser_pressure_score: u8,
    storage_score: u8,
) -> Vec<RecoveryCandidate> {
    let mut candidates = Vec::new();

    if let Some(application) = top_applications
        .iter()
        .find(|application| application.show_opportunity && application.action_kind != "none")
    {
        candidates.push(RecoveryCandidate {
            domain: FocusDomain::Applications,
            action_kind: application.action_kind.clone(),
            target: application.action_target.clone(),
            expected_gain_minutes: parse_minutes(&application.care_estimated_improvement),
            estimated_interruption_seconds: if application.action_kind == "restartFinder" {
                5
            } else {
                20
            },
            confidence: 0.55,
            safety_level: SafetyLevel::Caution,
            requires_confirmation: true,
            can_automate: true,
            session_preservation_risk: SessionPreservationRisk::Medium,
            reason: application.care_detail.clone(),
            trust_notes:
                "Only known safe application actions are exposed; protected active work stays blocked."
                    .to_string(),
        });
    }

    if browser_pressure_score < 70 {
        if let Some(browser) = primary_browser(snapshot) {
            candidates.push(RecoveryCandidate {
                domain: FocusDomain::Browser,
                action_kind: if browser.name == "Safari" {
                    "quitApp".to_string()
                } else {
                    "restartApp".to_string()
                },
                target: browser.name.clone(),
                expected_gain_minutes: 35,
                estimated_interruption_seconds: if browser.name == "Safari" { 10 } else { 20 },
                confidence: 0.58,
                safety_level: SafetyLevel::Caution,
                requires_confirmation: true,
                can_automate: true,
                session_preservation_risk: SessionPreservationRisk::Medium,
                reason: browser_recommendation_explanation(browser),
                trust_notes:
                    "Browser session restoration is likely but not guaranteed; confirmation remains required."
                        .to_string(),
            });
        }
    }

    if storage_score < 58 {
        candidates.push(RecoveryCandidate {
            domain: FocusDomain::Storage,
            action_kind: "openStorageSettings".to_string(),
            target: "Storage Settings".to_string(),
            expected_gain_minutes: 15,
            estimated_interruption_seconds: 180,
            confidence: 0.45,
            safety_level: SafetyLevel::Safe,
            requires_confirmation: false,
            can_automate: true,
            session_preservation_risk: SessionPreservationRisk::None,
            reason: storage_health.detail.clone(),
            trust_notes: "Opening Settings is non-destructive; cleanup remains a user decision.".to_string(),
        });
    }

    candidates.push(RecoveryCandidate {
        domain: FocusDomain::System,
        action_kind: "wait".to_string(),
        target: String::new(),
        expected_gain_minutes: parse_minutes(&recommendation.estimated_additional_work_label),
        estimated_interruption_seconds: 0,
        confidence: 0.50,
        safety_level: SafetyLevel::Safe,
        requires_confirmation: false,
        can_automate: false,
        session_preservation_risk: SessionPreservationRisk::None,
        reason: recommendation.explanation.clone(),
        trust_notes:
            "Guidance-only candidate preserves flow when no safe one-click action is appropriate."
                .to_string(),
    });

    candidates.sort_by(|left, right| {
        recovery_rank(right)
            .partial_cmp(&recovery_rank(left))
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    candidates
}

fn recovery_rank(candidate: &RecoveryCandidate) -> f32 {
    let safety_penalty = match candidate.safety_level {
        SafetyLevel::Safe => 0.0,
        SafetyLevel::Caution => 4.0,
        SafetyLevel::Restricted => 12.0,
        SafetyLevel::Blocked => 100.0,
    };
    let interruption_penalty = candidate.estimated_interruption_seconds as f32 / 30.0;
    (candidate.expected_gain_minutes as f32 * candidate.confidence)
        - interruption_penalty
        - safety_penalty
}

fn prediction_confidence(
    scores: &[u8],
    signals: &[FocusSignal],
    pressure_penalty: f32,
    volatility_penalty: f32,
) -> f32 {
    let weakest = scores.iter().min().copied().unwrap_or(50);
    let strongest = scores.iter().max().copied().unwrap_or(50);
    let spread = strongest.saturating_sub(weakest);
    let completeness = if signals.is_empty() { 0.0 } else { 1.0 };
    let missing_window_server = signals.iter().any(|signal| {
        signal.domain == FocusDomain::Desktop && signal.supporting_metrics.is_empty()
    });
    let completeness_penalty = if missing_window_server { 0.06 } else { 0.0 };
    let conflict_penalty = if spread >= 55 {
        0.10
    } else if spread >= 35 {
        0.06
    } else {
        0.02
    };
    let stability_score =
        1.0 - (pressure_penalty + volatility_penalty + conflict_penalty).clamp(0.0, 0.45);
    let base = 0.42
        + (weakest as f32 / 100.0 * 0.18)
        + (completeness * 0.12)
        + (stability_score * 0.20)
        - completeness_penalty;

    // Until persisted history exists, confidence stays deliberately capped.
    base.clamp(0.35, 0.85)
}

fn focus_state_for_score(score: u8) -> FocusState {
    if score >= 58 {
        FocusState::Green
    } else if score >= 40 {
        FocusState::Yellow
    } else if score >= 30 {
        FocusState::Orange
    } else {
        FocusState::Red
    }
}

fn focus_state_for_minutes(minutes: u32) -> FocusState {
    if minutes >= 61 {
        FocusState::Green
    } else if minutes >= 31 {
        FocusState::Yellow
    } else if minutes >= 15 {
        FocusState::Orange
    } else {
        FocusState::Red
    }
}

fn menu_bar_state_for_minutes(minutes: u32) -> MenuBarState {
    let state = focus_state_for_minutes(minutes);
    let (heart_color, shows_minutes, critical_pulse) = match state {
        FocusState::Green => ("green", false, false),
        FocusState::Yellow => ("yellow", true, false),
        FocusState::Orange => ("orange", true, false),
        FocusState::Red => ("red", true, true),
    };

    MenuBarState {
        state,
        heart_color: heart_color.to_string(),
        minutes_label: if shows_minutes {
            format_flow_remaining(minutes)
        } else {
            String::new()
        },
        shows_minutes,
        critical_pulse,
    }
}

fn parse_minutes(label: &str) -> i32 {
    let digits = label
        .chars()
        .filter(|character| character.is_ascii_digit())
        .collect::<String>();
    digits.parse::<i32>().unwrap_or(0)
}

fn weighted_score(
    cpu_score: u8,
    memory_score: u8,
    storage_score: u8,
    disk_score: u8,
    application_score: u8,
    browser_score: u8,
    renderer_score: u8,
    window_server_score: u8,
) -> u8 {
    let score = (memory_score as f64 * 0.28)
        + (cpu_score as f64 * 0.22)
        + (browser_score as f64 * 0.18)
        + (application_score as f64 * 0.12)
        + (disk_score as f64 * 0.08)
        + (storage_score as f64 * 0.06)
        + (renderer_score as f64 * 0.04)
        + (window_server_score as f64 * 0.02);
    score.round().clamp(0.0, 100.0) as u8
}

fn health_state(system_score: u8) -> HealthState {
    if system_score >= 58 {
        HealthState::Healthy
    } else if system_score >= 40 {
        HealthState::Attention
    } else {
        HealthState::Critical
    }
}

fn score_cpu(snapshot: &SystemSnapshot) -> u8 {
    let idle_percent = snapshot.cpu.idle_percent;
    let busy_percent = (snapshot.cpu.user_percent + snapshot.cpu.system_percent).clamp(0.0, 100.0);
    if idle_percent >= 55.0 && busy_percent <= 45.0 {
        96
    } else if idle_percent >= 35.0 && busy_percent <= 65.0 {
        84
    } else if idle_percent >= 20.0 && busy_percent <= 80.0 {
        70
    } else if idle_percent >= 10.0 && busy_percent <= 90.0 {
        55
    } else {
        34
    }
}

fn score_memory(snapshot: &SystemSnapshot) -> u8 {
    let available_ratio = ratio(snapshot.memory.available_bytes, snapshot.memory.total_bytes);
    let swap_ratio = ratio(snapshot.memory.swap_used_bytes, snapshot.memory.total_bytes);
    let compressed_ratio = ratio(snapshot.memory.compressed_bytes, snapshot.memory.total_bytes);

    if swap_ratio >= 0.50 || available_ratio < 0.03 {
        30
    } else if swap_ratio >= 0.30 || available_ratio < 0.05 {
        42
    } else if swap_ratio >= 0.15 || available_ratio < 0.08 || compressed_ratio >= 0.35 {
        55
    } else if swap_ratio >= 0.05 || available_ratio < 0.12 || compressed_ratio >= 0.25 {
        70
    } else if available_ratio >= 0.20 {
        96
    } else {
        84
    }
}

fn score_storage(snapshot: &SystemSnapshot) -> u8 {
    score_storage_space(snapshot).min(score_disk_activity(snapshot))
}

fn score_storage_space(snapshot: &SystemSnapshot) -> u8 {
    let available_ratio = ratio(snapshot.storage.available_bytes, snapshot.storage.total_bytes);
    if available_ratio >= 0.15 {
        96
    } else if available_ratio >= 0.08 {
        88
    } else if available_ratio >= 0.04 {
        72
    } else if available_ratio >= 0.02 {
        55
    } else {
        38
    }
}

fn score_disk_activity(snapshot: &SystemSnapshot) -> u8 {
    let throughput = snapshot.disk_activity.megabytes_per_second;
    if throughput >= 250.0 {
        38
    } else if throughput >= 120.0 {
        55
    } else if throughput >= 60.0 {
        70
    } else if throughput >= 25.0 {
        84
    } else {
        96
    }
}

fn score_applications(snapshot: &SystemSnapshot) -> u8 {
    snapshot
        .applications
        .iter()
        .map(|application| {
            score_application_load(
                ratio(application.memory_bytes, snapshot.memory.total_bytes),
                application.cpu_percent,
            )
        })
        .min()
        .unwrap_or(96)
}

fn score_application_load(memory_ratio: f64, cpu_percent: f32) -> u8 {
    if memory_ratio >= 0.50 || cpu_percent >= 80.0 {
        38
    } else if memory_ratio >= 0.35 || cpu_percent >= 55.0 {
        55
    } else if memory_ratio >= 0.25 || cpu_percent >= 35.0 {
        70
    } else if memory_ratio >= 0.15 || cpu_percent >= 20.0 {
        84
    } else {
        96
    }
}

fn score_browser(snapshot: &SystemSnapshot) -> u8 {
    let Some(browser) = primary_browser(snapshot) else {
        return 96;
    };
    let memory_ratio = ratio(browser.memory_bytes, snapshot.memory.total_bytes);
    let largest_renderer_ratio = ratio(browser.largest_renderer_bytes, snapshot.memory.total_bytes);

    if browser.renderer_count >= 90
        || memory_ratio >= 0.72
        || largest_renderer_ratio >= 0.30
        || browser.cpu_percent >= 120.0
    {
        30
    } else if browser.renderer_count >= 70
        || memory_ratio >= 0.58
        || largest_renderer_ratio >= 0.20
        || browser.cpu_percent >= 85.0
    {
        42
    } else if browser.renderer_count >= 52
        || memory_ratio >= 0.45
        || largest_renderer_ratio >= 0.12
        || browser.cpu_percent >= 60.0
    {
        55
    } else if browser.renderer_count >= 36
        || memory_ratio >= 0.34
        || largest_renderer_ratio >= 0.08
        || browser.cpu_percent >= 35.0
    {
        70
    } else if browser.renderer_count >= 24 || memory_ratio >= 0.24 || browser.cpu_percent >= 22.0 {
        84
    } else {
        96
    }
}

fn score_renderers(snapshot: &SystemSnapshot) -> u8 {
    let renderer_ratio = ratio(
        snapshot.renderers.total_memory_bytes,
        snapshot.memory.total_bytes,
    );

    if snapshot.renderers.total_count >= 90 || renderer_ratio >= 0.45 {
        38
    } else if snapshot.renderers.total_count >= 70 || renderer_ratio >= 0.34 {
        55
    } else if snapshot.renderers.total_count >= 48 || renderer_ratio >= 0.24 {
        70
    } else if snapshot.renderers.total_count >= 28 || renderer_ratio >= 0.14 {
        84
    } else {
        96
    }
}

fn score_window_server(snapshot: &SystemSnapshot) -> u8 {
    let Some(window_server) = &snapshot.window_server else {
        return 96;
    };
    let memory_ratio = ratio(window_server.memory_bytes, snapshot.memory.total_bytes);

    if memory_ratio >= 0.15 || window_server.cpu_percent >= 50.0 {
        38
    } else if memory_ratio >= 0.10 || window_server.cpu_percent >= 35.0 {
        55
    } else if memory_ratio >= 0.07 || window_server.cpu_percent >= 20.0 {
        70
    } else if memory_ratio >= 0.05 || window_server.cpu_percent >= 12.0 {
        82
    } else {
        96
    }
}

fn primary_recommendation(
    snapshot: &SystemSnapshot,
    cpu_score: u8,
    memory_score: u8,
    storage_score: u8,
    disk_score: u8,
    application_score: u8,
    browser_score: u8,
    renderer_score: u8,
    window_server_score: u8,
    system_score: u8,
) -> Recommendation {
    if system_score >= 58 {
        return Recommendation {
            text: "No action needed right now.".to_string(),
            explanation: "Nothing is close enough to reserve to interrupt your work. Smallest action: keep working. Expected interruption: none. Expected benefit: protect momentum.".to_string(),
            estimated_additional_work_label: "+0 minutes".to_string(),
        };
    }

    let browser_pressure_score = browser_score.min(renderer_score);

    if browser_pressure_score <= memory_score
        && browser_pressure_score <= application_score
        && browser_pressure_score <= cpu_score
    {
        if let Some(browser) = primary_browser(snapshot) {
            return Recommendation {
                text: format!("Restart {} at your next natural break.", browser.name),
                explanation: browser_recommendation_explanation(browser),
                estimated_additional_work_label: "+35 minutes".to_string(),
            };
        }
    }

    if application_score <= memory_score && application_score <= cpu_score {
        if let Some(application) = snapshot.applications.first() {
            if application.name.to_lowercase().contains("codex") {
                return Recommendation {
                    text: "Keep Codex running while this task is active.".to_string(),
                    explanation: "Codex is doing your current work. Smallest action: finish the thought first, then review inactive chats or windows. Expected interruption now: none. Expected benefit: avoid losing flow.".to_string(),
                    estimated_additional_work_label: "+0 minutes".to_string(),
                };
            }

            if application.name == "WindowServer" {
                return Recommendation {
                    text: "Let the desktop settle before opening more heavy apps.".to_string(),
                    explanation: "Desktop responsiveness is doing unusual work. Smallest action: pause window switching and close unused windows after the current task. Expected interruption: under 1 minute. Expected benefit: smoother app switching without restarting the Mac.".to_string(),
                    estimated_additional_work_label: "+10 minutes".to_string(),
                };
            }

            if safe_restart_target(&application.name) {
                return Recommendation {
                    text: format!("Restart {} when you are finished with it.", application.name),
                    explanation: format!(
                        "{} is using enough processor or memory to affect smoothness. Smallest action: restart only this app at a natural break. Expected interruption: about 20 seconds. Expected benefit: the next work block should feel lighter.",
                        application.name
                    ),
                    estimated_additional_work_label: "+25 minutes".to_string(),
                };
            }

            return Recommendation {
                text: format!("Let {} settle before opening another heavy app.", application.name),
                explanation: format!(
                    "{} is using enough processor or memory to explain sluggishness. Smallest action: wait for it to finish active work before adding more load. Expected interruption: none right now. Expected benefit: avoids interrupting work that may still be saving or processing.",
                    application.name
                ),
                estimated_additional_work_label: "+0 minutes".to_string(),
            };
        }
    }

    if memory_score <= cpu_score && memory_score <= storage_score {
        if browser_pressure_score < 70 {
            if let Some(browser) = primary_browser(snapshot) {
                return Recommendation {
                    text: format!("Restart {} when your current thought is safe.", browser.name),
                    explanation: format!(
                        "{} is the quickest safe way to free a meaningful amount of RAM. Smallest action: restart the browser only, not Codex or the Mac. Expected interruption: about 20 seconds. Expected benefit: a smoother next block.",
                        browser.name
                    ),
                    estimated_additional_work_label: "+35 minutes".to_string(),
                };
            }
        }

        Recommendation {
            text: "Free memory at your next natural break.".to_string(),
            explanation: "RAM and swap are close to reserve. Smallest action: close the heaviest safe app after this thought. Expected interruption: 30 seconds to 2 minutes. Expected benefit: more room for app switching.".to_string(),
            estimated_additional_work_label: "+20 minutes".to_string(),
        }
    } else if cpu_score <= disk_score && cpu_score <= storage_score {
        Recommendation {
            text: "Let the busy app settle before opening more work.".to_string(),
            explanation: "Processor reserve is low. Smallest action: pause new heavy apps until the current spike drops. Expected interruption: none. Expected benefit: fewer slow launches and less beachball risk.".to_string(),
            estimated_additional_work_label: "+15 minutes".to_string(),
        }
    } else if disk_score <= storage_score {
        Recommendation {
            text: "Let disk activity finish before changing apps.".to_string(),
            explanation: "The disk is busy right now. Smallest action: wait briefly rather than interrupting a write, sync, or indexing task. Expected interruption: under 1 minute. Expected benefit: smoother app launches after the disk settles.".to_string(),
            estimated_additional_work_label: "+10 minutes".to_string(),
        }
    } else if window_server_score < browser_score
        && window_server_score <= memory_score
        && window_server_score <= storage_score
    {
        Recommendation {
            text: "Finish this task, then tidy windows before restarting the Mac.".to_string(),
            explanation: "Desktop pressure can make the whole Mac feel heavy. Smallest action: close unused windows first and restart only if the Mac still feels stuck. Expected interruption: 1 to 5 minutes. Expected benefit: smoother desktop movement.".to_string(),
            estimated_additional_work_label: "+20 minutes".to_string(),
        }
    } else {
        Recommendation {
            text: "Review storage when you have a quiet moment.".to_string(),
            explanation: "Storage does not need to interrupt this second. Smallest action: use a quiet moment to clear safe temporary files or old downloads. Expected interruption: a few minutes. Expected benefit: protects caches, updates, and app reliability.".to_string(),
            estimated_additional_work_label: "+15 minutes".to_string(),
        }
    }
}

fn format_flow_remaining(minutes: u32) -> String {
    if minutes >= 60 {
        let hours = minutes / 60;
        let remaining_minutes = minutes % 60;
        format!("{hours}h {remaining_minutes:02}m")
    } else {
        format!("{minutes}m")
    }
}

fn memory_health(snapshot: &SystemSnapshot, score: u8) -> DomainHealth {
    let label = domain_label(score);
    let available = format_bytes(snapshot.memory.available_bytes);
    let headline = if score >= 90 {
        "Working memory has room."
    } else if score >= 58 {
        "Working memory is carrying weight."
    } else if score >= 40 {
        "Memory is close to reserve."
    } else {
        "Memory is on reserve."
    };
    let detail = if score >= 90 {
        format!("{available} available. You still have plenty of room for today's workload.")
    } else if score >= 58 {
        format!("{available} remains. Heavy apps are using RAM, but you can keep going for now.")
    } else if score >= 40 {
        format!("Only {available} remains. Swap is building up, so the Mac may start feeling less responsive.")
    } else {
        format!("Only {available} remains. Memory is tight enough to interrupt flow soon.")
    };
    let value = format!(
        "{} of {} RAM used, {} available, {}",
        format_bytes(snapshot.memory.used_bytes),
        format_bytes(snapshot.memory.total_bytes),
        format_bytes(snapshot.memory.available_bytes),
        format_swap_usage(snapshot.memory.swap_used_bytes, snapshot.memory.swap_total_bytes)
    );
    let metric_label = format!(
        "{} of {} used - {}",
        format_bytes(snapshot.memory.used_bytes),
        format_bytes(snapshot.memory.total_bytes),
        format_swap_usage(snapshot.memory.swap_used_bytes, snapshot.memory.swap_total_bytes)
    );
    let metric_percent = format!(
        "{}%",
        format_percent(snapshot.memory.used_bytes, snapshot.memory.total_bytes)
    );

    DomainHealth {
        label: label.to_string(),
        headline: headline.to_string(),
        detail,
        value,
        metric_label,
        metric_percent,
    }
}

fn processor_health(snapshot: &SystemSnapshot, score: u8) -> DomainHealth {
    let label = domain_label(score);
    let used_percent = (snapshot.cpu.user_percent + snapshot.cpu.system_percent).clamp(0.0, 100.0);
    let reserve_percent = snapshot.cpu.idle_percent.clamp(0.0, 100.0);
    let top_application = snapshot.applications.first();
    let top_application_name = top_application
        .map(|application| display_application_name(&application.name))
        .unwrap_or("No application");
    let top_application_cpu = top_application
        .map(|application| application.cpu_percent)
        .unwrap_or_default();

    let headline = if score >= 90 {
        "Processor has room.".to_string()
    } else if top_application_cpu >= 35.0 {
        format!("{top_application_name} is using active processor time.")
    } else if reserve_percent < 20.0 {
        "Processor reserve is getting low.".to_string()
    } else {
        "Processor is carrying today's workload.".to_string()
    };
    let detail = if score >= 90 {
        "No application is currently slowing the computer.".to_string()
    } else if top_application_cpu >= 35.0 {
        format!(
            "{top_application_name} is using significantly more processor time than normal."
        )
    } else if reserve_percent < 20.0 {
        "The Mac has less spare processor room, so opening heavy apps may feel slower.".to_string()
    } else {
        "Processor load is noticeable, but it does not need to interrupt you right now."
            .to_string()
    };
    let value = format!(
        "{} processor used, {} reserve",
        format_cpu(used_percent),
        format_cpu(reserve_percent)
    );
    let metric_label = format!(
        "{} used - {} reserve",
        format_cpu(used_percent),
        format_cpu(reserve_percent)
    );
    let metric_percent = format!("{}%", used_percent.round() as u8);

    DomainHealth {
        label: label.to_string(),
        headline,
        detail,
        value,
        metric_label,
        metric_percent,
    }
}

fn storage_health(snapshot: &SystemSnapshot, score: u8) -> DomainHealth {
    let label = domain_label(score);
    let space_score = score_storage_space(snapshot);
    let disk_score = score_disk_activity(snapshot);
    let storage_location = if snapshot.storage.mount_point == "/"
        || snapshot
            .storage
            .mount_point
            .chars()
            .all(|character| character.is_ascii_digit())
    {
        "your Mac"
    } else {
        snapshot.storage.mount_point.as_str()
    };
    let headline = if disk_score < 58 {
        "Disk is busy right now."
    } else if space_score >= 90 {
        "Disk space has room."
    } else if space_score >= 58 {
        "Storage is beginning to fill."
    } else if space_score >= 40 {
        "Storage is close to reserve."
    } else {
        "Disk space needs care soon."
    };
    let detail = if disk_score < 58 {
        "Heavy reads or writes can make apps feel slower for a short time."
    } else if space_score >= 90 {
        "There is enough free disk space for normal work."
    } else if space_score >= 58 {
        "Free disk space is lower than ideal, but not urgent."
    } else if space_score >= 40 {
        "Low space can make updates, caches, and app work less reliable."
    } else {
        "Free space is low enough to interrupt normal app work."
    };
    let value = format!(
        "{} of {} used on {}, {} free, {} disk activity",
        format_bytes(snapshot.storage.used_bytes),
        format_bytes(snapshot.storage.total_bytes),
        storage_location,
        format_bytes(snapshot.storage.available_bytes),
        format_disk_activity(snapshot.disk_activity.megabytes_per_second)
    );
    let metric_label = format!(
        "{} used of {} - {}",
        format_bytes(snapshot.storage.used_bytes),
        format_bytes(snapshot.storage.total_bytes),
        format_disk_activity(snapshot.disk_activity.megabytes_per_second)
    );
    let metric_percent = format!(
        "{}%",
        format_percent(snapshot.storage.used_bytes, snapshot.storage.total_bytes)
    );

    DomainHealth {
        label: label.to_string(),
        headline: headline.to_string(),
        detail: detail.to_string(),
        value,
        metric_label,
        metric_percent,
    }
}

fn browser_health(snapshot: &SystemSnapshot, score: u8) -> DomainHealth {
    let label = domain_label(score);
    if let Some(browser) = primary_browser(snapshot) {
        let headline = if score >= 90 {
            format!("{} looks steady.", browser.name)
        } else if score >= 58 {
            format!("{} is adding browser load.", browser.name)
        } else if score >= 40 {
            format!("{} is close to reserve.", browser.name)
        } else {
            format!("{} is likely affecting smoothness.", browser.name)
        };
        let detail = if score >= 90 {
            "Browser load is not standing out in this check-in.".to_string()
        } else if score >= 58 {
            format!(
                "{} is carrying noticeable tab and renderer work, but it can wait.",
                browser.name
            )
        } else if score >= 40 {
            format!(
                "{} has enough renderer load to make the next work block feel heavier.",
                browser.name
            )
        } else {
            format!(
                "{} is one of the likely reasons the Mac may feel slower.",
                browser.name
            )
        };
        let value = format!(
            "{}: {} RAM, {} CPU, {} processes, {} renderers",
            browser.name,
            format_bytes(browser.memory_bytes),
            format_cpu(browser.cpu_percent),
            browser.process_count,
            browser.renderer_count
        );
        let metric_label = format!(
            "{} of {} used - {} CPU - {} renderers",
            format_bytes(browser.memory_bytes),
            format_bytes(snapshot.memory.total_bytes),
            format_cpu(browser.cpu_percent),
            browser.renderer_count
        );
        let metric_percent = format!(
            "{}%",
            format_percent(browser.memory_bytes, snapshot.memory.total_bytes)
        );

        DomainHealth {
            label: label.to_string(),
            headline,
            detail,
            value,
            metric_label,
            metric_percent,
        }
    } else {
        DomainHealth {
            label: "Healthy".to_string(),
            headline: "No browser pressure detected.".to_string(),
            detail: "Browser activity is not standing out right now.".to_string(),
            value: "No browser pressure".to_string(),
            metric_label: "No browser pressure".to_string(),
            metric_percent: String::new(),
        }
    }
}

fn application_health(snapshot: &SystemSnapshot, score: u8) -> DomainHealth {
    let label = domain_label(score);
    let top_application = snapshot.applications.first();
    let top_application_name = top_application
        .map(|application| display_application_name(&application.name))
        .unwrap_or("No application");
    let headline = if score >= 90 {
        "Applications look steady.".to_string()
    } else if score >= 58 {
        format!("{top_application_name} may need care next.")
    } else if score >= 40 {
        format!("{top_application_name} is close to reserve.")
    } else {
        format!("{top_application_name} may interrupt momentum.")
    };
    let detail = if top_application.is_some() {
        if score >= 90 {
            format!(
                "{} is doing the most work, but it does not look disruptive.",
                top_application_name
            )
        } else if score >= 58 {
            format!(
                "{} is carrying noticeable work, but it can wait.",
                top_application_name
            )
        } else {
            format!(
                "{} is the main app to care about if your Mac feels heavy.",
                top_application_name
            )
        }
    } else {
        "No application is standing out right now.".to_string()
    };
    let value = top_application
        .map(|application| {
            format!(
                "{top_application_name} using {} RAM and {} CPU",
                format_bytes(application.memory_bytes),
                format_cpu(application.cpu_percent)
            )
        })
        .unwrap_or_else(|| "No standout application".to_string());
    let metric_label = top_application
        .map(|application| {
            format!(
                "{} of {} used - {} CPU",
                format_bytes(application.memory_bytes),
                format_bytes(snapshot.memory.total_bytes),
                format_cpu(application.cpu_percent)
            )
        })
        .unwrap_or_else(|| "No standout app".to_string());
    let metric_percent = top_application
        .map(|application| {
            format!(
                "{}%",
                format_percent(application.memory_bytes, snapshot.memory.total_bytes)
            )
        })
        .unwrap_or_default();

    DomainHealth {
        label: label.to_string(),
        headline,
        detail,
        value,
        metric_label,
        metric_percent,
    }
}

fn application_impacts(snapshot: &SystemSnapshot) -> Vec<ApplicationImpact> {
    snapshot
        .applications
        .iter()
        .enumerate()
        .map(|(index, application)| {
            let app_ratio = ratio(application.memory_bytes, snapshot.memory.total_bytes);
            let app_score = score_application_load(app_ratio, application.cpu_percent);
            let is_browser = is_observed_browser(snapshot, &application.name);
            let is_chrome = application.name.to_lowercase().contains("chrome");
            let is_codex = application.name.to_lowercase().contains("codex");
            let is_safari = application.name == "Safari";
            let is_finder = application.name == "Finder";
            let is_window_server = application.name == "WindowServer";
            let display_name = display_application_name(&application.name);
            let impact_label = if index == 0 {
                format!("{display_name} is shaping today's check-in.")
            } else if is_chrome {
                "Chrome is behaving normally for now.".to_string()
            } else if app_ratio >= 0.15 || application.cpu_percent >= 20.0 {
                format!("{display_name} is worth reviewing if things start to feel heavy.")
            } else {
                format!("{display_name} looks steady.")
            };
            let detail = if index == 0 && is_codex {
                "This is expected while you are actively building software.".to_string()
            } else if index == 0 && is_browser {
                "Browser tabs and background work are grouped here.".to_string()
            } else if index == 0 {
                "This can be normal while it is active in your current workload.".to_string()
            } else if is_chrome {
                "If your Mac starts feeling sluggish, this is the first application worth reviewing.".to_string()
            } else if is_browser {
                "Includes browser background work.".to_string()
            } else if application.name == "WindowServer" {
                "Supports desktop responsiveness and window movement.".to_string()
            } else if app_ratio >= 0.15 || application.cpu_percent >= 20.0 {
                "Doing noticeable work in this check-in.".to_string()
            } else {
                "Not standing out in the current check-in.".to_string()
            };
            let (
                care_label,
                care_detail,
                care_estimated_improvement,
                action_kind,
                action_target,
                action_label,
                show_opportunity,
                protected_work,
            ) = if is_codex {
                (
                    "No recommendation".to_string(),
                    "Codex is active work. Smallest action: keep it running and archive inactive conversations later if you need more breathing room. Expected interruption now: none.".to_string(),
                    "+0 minutes".to_string(),
                    "none".to_string(),
                    String::new(),
                    String::new(),
                    false,
                    true,
                )
            } else if index == 0 && is_browser {
                let action_kind = if is_safari { "quitApp" } else { "restartApp" };
                let action_label = if is_safari {
                    "Quit Safari".to_string()
                } else {
                    "Restart now".to_string()
                };
                let care_label = if is_safari {
                    "Quit Safari at your next break".to_string()
                } else {
                    format!("Restart {display_name} at your next break")
                };
                (
                    care_label,
                    browser_care_detail(display_name),
                    "+35 minutes".to_string(),
                    action_kind.to_string(),
                    display_name.to_string(),
                    action_label,
                    true,
                    false,
                )
            } else if index == 0 && is_finder && app_score < 58 {
                (
                    "Restart Finder at your next break".to_string(),
                    "Finder can make window movement and file browsing feel heavier. Smallest action: restart Finder only, not the Mac. Expected interruption: about 5 seconds.".to_string(),
                    "+5 minutes".to_string(),
                    "restartFinder".to_string(),
                    "Finder".to_string(),
                    "Restart Finder".to_string(),
                    true,
                    false,
                )
            } else if index == 0 && is_window_server && app_score < 58 {
                (
                    "No direct action yet".to_string(),
                    "Desktop responsiveness is doing unusual work. Smallest action: close unused windows after this task. Restart the Mac only if smaller steps do not help.".to_string(),
                    "+10 minutes".to_string(),
                    "none".to_string(),
                    String::new(),
                    String::new(),
                    false,
                    false,
                )
            } else if index == 0 && app_score < 58 && safe_restart_target(display_name) {
                (
                    format!("Restart {display_name} when finished"),
                    format!(
                        "{display_name} is carrying noticeable work. Smallest action: restart only this app after the current task. Expected interruption: about 20 seconds."
                    ),
                    "+25 minutes".to_string(),
                    "restartApp".to_string(),
                    display_name.to_string(),
                    "Restart now".to_string(),
                    true,
                    false,
                )
            } else if index == 0 && app_score < 58 {
                (
                    format!("Review {display_name} when this task is safe"),
                    format!(
                        "{display_name} is using enough RAM or processor time to affect smoothness. Smallest action: let it finish or close it manually when your work is safe."
                    ),
                    "+10 minutes".to_string(),
                    "none".to_string(),
                    String::new(),
                    String::new(),
                    false,
                    false,
                )
            } else if index == 0 {
                (
                    "No action needed today".to_string(),
                    "This looks normal for what you are doing right now.".to_string(),
                    "+0 minutes".to_string(),
                    "none".to_string(),
                    String::new(),
                    String::new(),
                    false,
                    false,
                )
            } else if app_ratio >= 0.15 || application.cpu_percent >= 20.0 {
                (
                    "Best reviewed at your next break".to_string(),
                    "Only needs care if your Mac starts to feel less responsive.".to_string(),
                    "+10 minutes".to_string(),
                    "none".to_string(),
                    String::new(),
                    String::new(),
                    false,
                    false,
                )
            } else {
                (
                    "No action needed today".to_string(),
                    "Not likely to interrupt your momentum.".to_string(),
                    "+0 minutes".to_string(),
                    "none".to_string(),
                    String::new(),
                    String::new(),
                    false,
                    false,
                )
            };
            ApplicationImpact {
                name: display_name.to_string(),
                memory_display: format_bytes(application.memory_bytes),
                cpu_display: format_cpu(application.cpu_percent),
                impact_label: impact_label.to_string(),
                detail,
                care_label,
                care_detail,
                care_estimated_improvement,
                action_kind,
                action_target,
                action_label,
                show_opportunity,
                protected_work,
            }
        })
        .collect()
}

fn primary_browser(snapshot: &SystemSnapshot) -> Option<&BrowserSnapshot> {
    if let Some(primary_name) = snapshot.renderers.primary_browser.as_deref() {
        if let Some(browser) = snapshot
            .browser
            .browsers
            .iter()
            .find(|browser| browser.name == primary_name)
        {
            return Some(browser);
        }
    }
    snapshot
        .browser
        .browsers
        .iter()
        .max_by_key(|browser| browser.memory_bytes)
}

fn is_observed_browser(snapshot: &SystemSnapshot, name: &str) -> bool {
    snapshot
        .browser
        .browsers
        .iter()
        .any(|browser| browser.name == name)
}

fn display_application_name(name: &str) -> &str {
    if name == "WindowServer" {
        "Desktop responsiveness"
    } else {
        name
    }
}

fn safe_restart_target(name: &str) -> bool {
    matches!(
        name,
        "Google Chrome" | "Microsoft Edge" | "Firefox" | "Safari" | "Finder"
    )
}

fn browser_care_detail(name: &str) -> String {
    if name == "Safari" {
        return "Safari is holding browser work in memory. Smallest action: quit Safari at a natural break. Expected interruption: about 10 seconds.".to_string();
    }

    format!(
        "{name} is carrying tab and renderer work. Smallest action: restart only the browser at a natural break. Expected interruption: about 20 seconds."
    )
}

fn browser_recommendation_explanation(browser: &BrowserSnapshot) -> String {
    let uptime = browser
        .uptime_seconds
        .map(format_duration)
        .map(|duration| format!(" It has been running for {duration}."))
        .unwrap_or_default();
    format!(
        "{} is carrying a lot of today's browser work.{} Smallest action: restart only {} at a natural break. Expected interruption: about 20 seconds. Expected benefit: likely smoother browsing and app switching.",
        browser.name, uptime, browser.name
    )
}

fn domain_label(score: u8) -> &'static str {
    if score >= 58 {
        "OK"
    } else if score >= 40 {
        "Later"
    } else {
        "Care"
    }
}

fn ratio(part: u64, whole: u64) -> f64 {
    if whole == 0 {
        0.0
    } else {
        part as f64 / whole as f64
    }
}

fn format_bytes(bytes: u64) -> String {
    let gib = bytes as f64 / 1_073_741_824.0;
    if gib >= 1.0 {
        format!("{gib:.1} GB")
    } else {
        let mib = bytes as f64 / 1_048_576.0;
        format!("{mib:.0} MB")
    }
}

fn format_percent(part: u64, whole: u64) -> String {
    if whole == 0 {
        "0".to_string()
    } else {
        format!("{:.0}", (part as f64 / whole as f64) * 100.0)
    }
}

fn format_disk_activity(megabytes_per_second: f32) -> String {
    if megabytes_per_second >= 10.0 {
        format!("{megabytes_per_second:.0} MB/s")
    } else if megabytes_per_second >= 1.0 {
        format!("{megabytes_per_second:.1} MB/s")
    } else {
        "<1 MB/s".to_string()
    }
}

fn format_swap_usage(used_bytes: u64, total_bytes: u64) -> String {
    if total_bytes > 0 {
        format!(
            "{} of {} swap",
            format_bytes(used_bytes),
            format_bytes(total_bytes)
        )
    } else {
        format!("{} swap", format_bytes(used_bytes))
    }
}

fn format_cpu(cpu_percent: f32) -> String {
    if cpu_percent >= 10.0 {
        format!("{cpu_percent:.0}%")
    } else if cpu_percent >= 1.0 {
        format!("{cpu_percent:.1}%")
    } else {
        "<1%".to_string()
    }
}

fn format_duration(seconds: u64) -> String {
    let days = seconds / 86_400;
    let hours = (seconds % 86_400) / 3_600;
    if days > 0 {
        format!("{days} day{}", if days == 1 { "" } else { "s" })
    } else if hours > 0 {
        format!("{hours} hour{}", if hours == 1 { "" } else { "s" })
    } else {
        "less than an hour".to_string()
    }
}
