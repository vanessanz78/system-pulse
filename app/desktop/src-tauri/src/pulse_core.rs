use crate::models::{
    ApplicationImpact, DomainHealth, HealthState, SystemSnapshot, TodayPulse,
};

pub fn evaluate(snapshot: SystemSnapshot) -> TodayPulse {
    let memory_score = score_memory(&snapshot);
    let storage_score = score_storage(&snapshot);
    let application_score = score_applications(&snapshot);
    let system_score = weighted_score(memory_score, storage_score, application_score);
    let health_state = health_state(system_score);
    let top_applications = application_impacts(&snapshot);
    let recommendation = primary_recommendation(
        &snapshot,
        memory_score,
        storage_score,
        application_score,
        system_score,
    );

    TodayPulse {
        collected_at: snapshot.collected_at,
        platform: snapshot.platform,
        system_score,
        health_state,
        primary_explanation: recommendation.explanation,
        primary_recommendation: recommendation.text,
        confidence: recommendation.confidence,
        expected_improvement: recommendation.expected_improvement,
        memory_health: memory_health(&snapshot, memory_score),
        storage_health: storage_health(&snapshot, storage_score),
        top_applications,
    }
}

struct Recommendation {
    text: String,
    explanation: String,
    confidence: u8,
    expected_improvement: String,
}

fn weighted_score(memory_score: u8, storage_score: u8, application_score: u8) -> u8 {
    let score = (memory_score as f64 * 0.45)
        + (storage_score as f64 * 0.25)
        + (application_score as f64 * 0.30);
    score.round().clamp(0.0, 100.0) as u8
}

fn health_state(system_score: u8) -> HealthState {
    if system_score >= 90 {
        HealthState::Healthy
    } else if system_score >= 70 {
        HealthState::Attention
    } else {
        HealthState::Critical
    }
}

fn score_memory(snapshot: &SystemSnapshot) -> u8 {
    let available_ratio = ratio(snapshot.memory.available_bytes, snapshot.memory.total_bytes);
    if available_ratio >= 0.25 {
        96
    } else if available_ratio >= 0.15 {
        88
    } else if available_ratio >= 0.08 {
        74
    } else {
        60
    }
}

fn score_storage(snapshot: &SystemSnapshot) -> u8 {
    let available_ratio = ratio(snapshot.storage.available_bytes, snapshot.storage.total_bytes);
    if available_ratio >= 0.20 {
        96
    } else if available_ratio >= 0.10 {
        84
    } else if available_ratio >= 0.05 {
        72
    } else {
        58
    }
}

fn score_applications(snapshot: &SystemSnapshot) -> u8 {
    let top_ratio = snapshot
        .applications
        .first()
        .map(|application| ratio(application.memory_bytes, snapshot.memory.total_bytes))
        .unwrap_or(0.0);

    if top_ratio < 0.08 {
        96
    } else if top_ratio < 0.15 {
        86
    } else if top_ratio < 0.25 {
        74
    } else {
        62
    }
}

fn primary_recommendation(
    snapshot: &SystemSnapshot,
    memory_score: u8,
    storage_score: u8,
    application_score: u8,
    system_score: u8,
) -> Recommendation {
    if system_score >= 90 {
        return Recommendation {
            text: "Nothing needs your attention today.".to_string(),
            explanation: "Memory, storage and application impact all look steady. You can get on with work.".to_string(),
            confidence: 92,
            expected_improvement: "+0".to_string(),
        };
    }

    if application_score <= memory_score && application_score <= storage_score {
        if let Some(application) = snapshot.applications.first() {
            return Recommendation {
                text: format!("Restart {} when you have a natural break.", application.name),
                explanation: format!(
                    "{} is the largest memory user right now. Restarting it may recover memory and improve responsiveness.",
                    application.name
                ),
                confidence: 88,
                expected_improvement: "+6".to_string(),
            };
        }
    }

    if memory_score <= storage_score {
        Recommendation {
            text: "Close or restart the heaviest app when you are ready.".to_string(),
            explanation: "Available memory is lower than ideal. Reducing the largest app's memory use would likely make the computer feel smoother.".to_string(),
            confidence: 84,
            expected_improvement: "+5".to_string(),
        }
    } else {
        Recommendation {
            text: "Review storage when you have a quiet moment.".to_string(),
            explanation: "Available storage is getting tighter. Choosing files to remove later can help protect performance without interrupting your work now.".to_string(),
            confidence: 82,
            expected_improvement: "+4".to_string(),
        }
    }
}

fn memory_health(snapshot: &SystemSnapshot, score: u8) -> DomainHealth {
    let available_ratio = ratio(snapshot.memory.available_bytes, snapshot.memory.total_bytes);
    let label = domain_label(score);
    let headline = if score >= 90 {
        "Memory looks steady."
    } else if score >= 70 {
        "Memory is working harder than usual."
    } else {
        "Memory needs attention."
    };
    let detail = format!(
        "{} available. {} is currently in use.",
        format_bytes(snapshot.memory.available_bytes),
        format_bytes(snapshot.memory.used_bytes)
    );

    DomainHealth {
        label: label.to_string(),
        headline: headline.to_string(),
        detail,
        value: format!("{:.0}% available", available_ratio * 100.0),
    }
}

fn storage_health(snapshot: &SystemSnapshot, score: u8) -> DomainHealth {
    let available_ratio = ratio(snapshot.storage.available_bytes, snapshot.storage.total_bytes);
    let label = domain_label(score);
    let headline = if score >= 90 {
        "Storage has room to breathe."
    } else if score >= 70 {
        "Storage is beginning to fill."
    } else {
        "Storage needs attention soon."
    };
    let detail = format!(
        "{} available on {}. {} is currently used.",
        format_bytes(snapshot.storage.available_bytes),
        snapshot.storage.mount_point,
        format_bytes(snapshot.storage.used_bytes)
    );

    DomainHealth {
        label: label.to_string(),
        headline: headline.to_string(),
        detail,
        value: format!("{:.0}% available", available_ratio * 100.0),
    }
}

fn application_impacts(snapshot: &SystemSnapshot) -> Vec<ApplicationImpact> {
    snapshot
        .applications
        .iter()
        .map(|application| {
            let app_ratio = ratio(application.memory_bytes, snapshot.memory.total_bytes);
            let impact_label = if app_ratio >= 0.15 {
                "High"
            } else if app_ratio >= 0.08 {
                "Medium"
            } else {
                "Low"
            };
            ApplicationImpact {
                name: application.name.clone(),
                memory_display: format_bytes(application.memory_bytes),
                impact_label: impact_label.to_string(),
                detail: "Using local memory right now.".to_string(),
            }
        })
        .collect()
}

fn domain_label(score: u8) -> &'static str {
    if score >= 90 {
        "Healthy"
    } else if score >= 70 {
        "Working hard"
    } else {
        "Needs attention"
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
