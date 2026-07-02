use crate::models::{ApplicationSnapshot, HealthState, SystemSnapshot, TodayPulse};

const NOTICEABLE_APP_CPU: f32 = 35.0;
const NOTICEABLE_DESKTOP_CPU: f32 = 35.0;
const ATTENTION_SCORE_CAP: u8 = 57;
const ATTENTION_FLOW_MINUTES: u32 = 52;

pub fn apply(snapshot: &SystemSnapshot, pulse: &mut TodayPulse) {
    let top_pressure = top_non_browser_pressure(snapshot);
    let desktop_cpu = snapshot
        .window_server
        .as_ref()
        .map(|window_server| window_server.cpu_percent)
        .unwrap_or_default();

    let app_pressure = top_pressure
        .map(|application| application.cpu_percent >= NOTICEABLE_APP_CPU)
        .unwrap_or(false);
    let desktop_pressure = desktop_cpu >= NOTICEABLE_DESKTOP_CPU;

    if !app_pressure && !desktop_pressure {
        return;
    }

    if pulse.system_score > ATTENTION_SCORE_CAP {
        pulse.system_score = ATTENTION_SCORE_CAP;
        pulse.health_state = HealthState::Attention;
        pulse.flow_remaining_minutes = ATTENTION_FLOW_MINUTES;
        pulse.flow_remaining_label = "52m".to_string();
    }

    if let Some(application) = top_pressure {
        mark_application_pressure(pulse, application);
    } else if desktop_pressure {
        mark_desktop_pressure(pulse);
    }
}

fn top_non_browser_pressure<'a>(snapshot: &'a SystemSnapshot) -> Option<&'a ApplicationSnapshot> {
    snapshot
        .applications
        .iter()
        .filter(|application| !is_observed_browser(snapshot, &application.name))
        .max_by(|left, right| {
            pressure_score(left)
                .partial_cmp(&pressure_score(right))
                .unwrap_or(std::cmp::Ordering::Equal)
        })
}

fn pressure_score(application: &ApplicationSnapshot) -> f32 {
    application.cpu_percent + (application.memory_bytes as f32 / 1_073_741_824.0 * 4.0)
}

fn mark_application_pressure(pulse: &mut TodayPulse, application: &ApplicationSnapshot) {
    let display_name = display_application_name(&application.name);
    let is_codex = application.name.to_lowercase().contains("codex");

    pulse.application_health.label = "Later".to_string();
    pulse.application_health.headline = format!("{display_name} is using active CPU.");
    pulse.application_health.detail = if is_codex {
        "Codex is actively working. Opening Chrome can feel slow until this settles.".to_string()
    } else {
        format!("{display_name} is doing enough work to affect smooth app switching.")
    };
    pulse.primary_recommendation = if is_codex {
        "Let Codex finish before heavier apps.".to_string()
    } else {
        format!("Review {display_name} before opening another heavy app.")
    };
    pulse.primary_explanation =
        "Application CPU is high enough to explain the sluggishness you felt.".to_string();
    pulse.estimated_additional_work_label = "+0 minutes".to_string();

    if let Some(impact) = pulse
        .top_applications
        .iter_mut()
        .find(|impact| impact.name == display_name)
    {
        impact.care_label = if is_codex {
            "Let Codex finish before heavier apps".to_string()
        } else {
            format!("Review {display_name} in Activity Monitor")
        };
        impact.care_detail = if is_codex {
            "Codex is protected active work. Review it rather than restarting it mid-task."
                .to_string()
        } else {
            format!("{display_name} is using enough CPU to make app switching feel slower.")
        };
        impact.care_estimated_improvement = "+0 minutes".to_string();
        impact.action_kind = "openActivityMonitor".to_string();
        impact.action_target = "Activity Monitor".to_string();
        impact.action_label = "Review".to_string();
        impact.show_opportunity = true;
        impact.protected_work = is_codex;
    }
}

fn mark_desktop_pressure(pulse: &mut TodayPulse) {
    pulse.application_health.label = "Later".to_string();
    pulse.application_health.headline = "Desktop responsiveness is working hard.".to_string();
    pulse.application_health.detail =
        "Window movement and app opening can feel slower while desktop CPU is high.".to_string();
    pulse.primary_recommendation =
        "Let the desktop settle, then review Activity Monitor if it stays heavy.".to_string();
    pulse.primary_explanation =
        "Desktop responsiveness is under enough CPU pressure to explain sluggishness.".to_string();
    pulse.estimated_additional_work_label = "+0 minutes".to_string();

    if let Some(impact) = pulse
        .top_applications
        .iter_mut()
        .find(|impact| impact.name == "Desktop responsiveness")
    {
        impact.care_label = "Review desktop responsiveness".to_string();
        impact.care_detail =
            "Activity Monitor is the safest place to check whether the desktop is still working hard."
                .to_string();
        impact.care_estimated_improvement = "+0 minutes".to_string();
        impact.action_kind = "openActivityMonitor".to_string();
        impact.action_target = "Activity Monitor".to_string();
        impact.action_label = "Review".to_string();
        impact.show_opportunity = true;
        impact.protected_work = false;
    }
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
