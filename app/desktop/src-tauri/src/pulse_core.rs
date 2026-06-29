use crate::models::{
    ApplicationImpact, BrowserSnapshot, DomainHealth, HealthState, SystemSnapshot, TodayPulse,
};

pub fn evaluate(snapshot: SystemSnapshot) -> TodayPulse {
    let memory_score = score_memory(&snapshot);
    let storage_score = score_storage(&snapshot);
    let application_score = score_applications(&snapshot);
    let browser_score = score_browser(&snapshot);
    let renderer_score = score_renderers(&snapshot);
    let window_server_score = score_window_server(&snapshot);
    let system_score = weighted_score(
        memory_score,
        storage_score,
        application_score,
        browser_score,
        renderer_score,
        window_server_score,
    );
    let health_state = health_state(system_score);
    let top_applications = application_impacts(&snapshot);
    let memory_health = memory_health(&snapshot, memory_score);
    let storage_health = storage_health(&snapshot, storage_score);
    let browser_health = browser_health(&snapshot, browser_score);
    let renderer_health = renderer_health(&snapshot, renderer_score);
    let window_server_health = window_server_health(&snapshot, window_server_score);
    let recommendation = primary_recommendation(
        &snapshot,
        memory_score,
        storage_score,
        application_score,
        browser_score,
        renderer_score,
        window_server_score,
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
        memory_health,
        storage_health,
        browser_health,
        renderer_health,
        window_server_health,
        top_applications,
    }
}

struct Recommendation {
    text: String,
    explanation: String,
    confidence: u8,
    expected_improvement: String,
}

fn weighted_score(
    memory_score: u8,
    storage_score: u8,
    application_score: u8,
    browser_score: u8,
    renderer_score: u8,
    window_server_score: u8,
) -> u8 {
    let score = (memory_score as f64 * 0.25)
        + (storage_score as f64 * 0.15)
        + (application_score as f64 * 0.15)
        + (browser_score as f64 * 0.25)
        + (renderer_score as f64 * 0.10)
        + (window_server_score as f64 * 0.10);
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

fn score_browser(snapshot: &SystemSnapshot) -> u8 {
    let Some(browser) = primary_browser(snapshot) else {
        return 96;
    };
    let memory_ratio = ratio(browser.memory_bytes, snapshot.memory.total_bytes);
    let largest_renderer_ratio = ratio(browser.largest_renderer_bytes, snapshot.memory.total_bytes);

    if browser.renderer_count >= 45 || memory_ratio >= 0.30 || largest_renderer_ratio >= 0.10 {
        62
    } else if browser.renderer_count >= 28 || memory_ratio >= 0.20 || largest_renderer_ratio >= 0.06
    {
        74
    } else if browser.renderer_count >= 16 || memory_ratio >= 0.12 {
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

    if snapshot.renderers.total_count >= 55 || renderer_ratio >= 0.28 {
        62
    } else if snapshot.renderers.total_count >= 34 || renderer_ratio >= 0.18 {
        74
    } else if snapshot.renderers.total_count >= 18 || renderer_ratio >= 0.10 {
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

    if memory_ratio >= 0.08 || window_server.cpu_percent >= 20.0 {
        70
    } else if memory_ratio >= 0.05 || window_server.cpu_percent >= 10.0 {
        82
    } else {
        96
    }
}

fn primary_recommendation(
    snapshot: &SystemSnapshot,
    memory_score: u8,
    storage_score: u8,
    application_score: u8,
    browser_score: u8,
    renderer_score: u8,
    window_server_score: u8,
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

    if (browser_score <= application_score
        && browser_score <= memory_score
        && browser_score <= storage_score)
        || renderer_score < 80
    {
        if let Some(browser) = primary_browser(snapshot) {
            let confidence = browser_confidence(snapshot, browser);
            let expected_improvement =
                expected_improvement(system_score, if confidence >= 94 { 98 } else { 94 });
            return Recommendation {
                text: format!("Restart {} when you have a natural break.", browser.name),
                explanation: browser_recommendation_explanation(browser),
                confidence,
                expected_improvement,
            };
        }
    }

    if window_server_score < browser_score
        && window_server_score <= memory_score
        && window_server_score <= storage_score
    {
        return Recommendation {
            text: "Give the desktop a planned restart if it still feels sluggish.".to_string(),
            explanation: "WindowServer is using more resources than usual. That can line up with slow screenshots, laggy window movement or Mission Control delays. Restart only when it will not interrupt your work.".to_string(),
            confidence: 78,
            expected_improvement: expected_improvement(system_score, 92),
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
            expected_improvement: expected_improvement(system_score, 92),
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

fn browser_health(snapshot: &SystemSnapshot, score: u8) -> DomainHealth {
    let label = domain_label(score);
    if let Some(browser) = primary_browser(snapshot) {
        let headline = if score >= 90 {
            "Browser activity looks steady."
        } else if score >= 70 {
            "Browser activity may be affecting responsiveness."
        } else {
            "Browser activity is likely affecting responsiveness."
        };
        let detail = format!(
            "{} is using {} across {} processes.",
            browser.name,
            format_bytes(browser.memory_bytes),
            browser.process_count
        );

        DomainHealth {
            label: label.to_string(),
            headline: headline.to_string(),
            detail,
            value: format!("{} renderers", browser.renderer_count),
        }
    } else {
        DomainHealth {
            label: "Healthy".to_string(),
            headline: "No browser pressure detected.".to_string(),
            detail: "Browser activity is not standing out right now.".to_string(),
            value: "No browser pressure".to_string(),
        }
    }
}

fn renderer_health(snapshot: &SystemSnapshot, score: u8) -> DomainHealth {
    let label = domain_label(score);
    let headline = if score >= 90 {
        "Renderer activity looks steady."
    } else if score >= 70 {
        "Renderer processes are accumulating."
    } else {
        "Renderer processes are likely affecting responsiveness."
    };
    let primary = snapshot
        .renderers
        .primary_browser
        .as_deref()
        .unwrap_or("The browser");
    let detail = if snapshot.renderers.total_count == 0 {
        "No browser renderer pressure is visible right now.".to_string()
    } else {
        format!(
            "{} has the highest renderer count. Renderer processes often explain browser-related sluggishness better than overall memory alone.",
            primary
        )
    };

    DomainHealth {
        label: label.to_string(),
        headline: headline.to_string(),
        detail,
        value: format!("{} renderers", snapshot.renderers.total_count),
    }
}

fn window_server_health(snapshot: &SystemSnapshot, score: u8) -> Option<DomainHealth> {
    let window_server = snapshot.window_server.as_ref()?;
    let headline = if score >= 90 {
        "Desktop responsiveness looks steady."
    } else if score >= 70 {
        "Desktop responsiveness may be working harder than usual."
    } else {
        "Desktop responsiveness needs attention."
    };

    Some(DomainHealth {
        label: domain_label(score).to_string(),
        headline: headline.to_string(),
        detail: "WindowServer can correlate with slow screenshots, window movement and Mission Control delays.".to_string(),
        value: format!(
            "{} memory, {:.1}% CPU",
            format_bytes(window_server.memory_bytes),
            window_server.cpu_percent
        ),
    })
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
            let detail = if is_observed_browser(snapshot, &application.name) {
                "Includes browser helper and renderer processes.".to_string()
            } else if application.name == "WindowServer" {
                "Desktop responsiveness support process.".to_string()
            } else {
                "Using local memory right now.".to_string()
            };
            ApplicationImpact {
                name: application.name.clone(),
                memory_display: format_bytes(application.memory_bytes),
                impact_label: impact_label.to_string(),
                detail,
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

fn browser_confidence(snapshot: &SystemSnapshot, browser: &BrowserSnapshot) -> u8 {
    let memory_ratio = ratio(browser.memory_bytes, snapshot.memory.total_bytes);
    if browser.renderer_count >= 28 && memory_ratio >= 0.18 {
        96
    } else if browser.renderer_count >= 16 || memory_ratio >= 0.12 {
        90
    } else {
        84
    }
}

fn browser_recommendation_explanation(browser: &BrowserSnapshot) -> String {
    let uptime = browser
        .uptime_seconds
        .map(format_duration)
        .map(|duration| format!(" It has been running for {duration}."))
        .unwrap_or_default();
    format!(
        "{} appears to be the largest contributor to your computer's current resource usage. Most of this comes from browser renderer processes that can accumulate during long browsing sessions.{} Restarting {} would likely improve responsiveness.",
        browser.name, uptime, browser.name
    )
}

fn expected_improvement(current_score: u8, expected_score: u8) -> String {
    let improvement = expected_score.saturating_sub(current_score).max(1);
    format!("+{improvement}")
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
