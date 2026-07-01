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
    let application_health = application_health(&snapshot, application_score);
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
    let flow = flow_remaining_estimate(
        system_score,
        memory_score,
        storage_score,
        application_score,
        browser_score,
        renderer_score,
        window_server_score,
    );

    TodayPulse {
        collected_at: snapshot.collected_at,
        platform: snapshot.platform,
        system_score,
        health_state,
        primary_explanation: recommendation.explanation,
        primary_recommendation: recommendation.text,
        estimated_additional_work_label: recommendation.estimated_additional_work_label,
        flow_remaining_label: flow.label,
        flow_remaining_minutes: flow.minutes,
        memory_health,
        storage_health,
        browser_health,
        application_health,
        top_applications,
    }
}

struct Recommendation {
    text: String,
    explanation: String,
    estimated_additional_work_label: String,
}

struct FlowEstimate {
    label: String,
    minutes: u32,
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
            text: "No action needed right now.".to_string(),
            explanation: "Nothing is likely to interrupt your work in this check-in.".to_string(),
            estimated_additional_work_label: "+0 minutes".to_string(),
        };
    }

    if (browser_score <= application_score
        && browser_score <= memory_score
        && browser_score <= storage_score)
        || renderer_score < 80
    {
        if let Some(browser) = primary_browser(snapshot) {
            return Recommendation {
                text: format!("Restart {} at your next natural break.", browser.name),
                explanation: browser_recommendation_explanation(browser),
                estimated_additional_work_label: "+35 minutes".to_string(),
            };
        }
    }

    if window_server_score < browser_score
        && window_server_score <= memory_score
        && window_server_score <= storage_score
    {
        return Recommendation {
            text: "Finish this task, then restart your Mac if it still feels heavy.".to_string(),
            explanation: "The lowest-interruption choice is to keep working now and only restart after your active work is safe.".to_string(),
            estimated_additional_work_label: "+45 minutes".to_string(),
        };
    }

    if application_score <= memory_score && application_score <= storage_score {
        if let Some(application) = snapshot.applications.first() {
            if application.name == "WindowServer" {
                return Recommendation {
                    text: "Restart your Mac when your work is saved.".to_string(),
                    explanation: "This can help if the whole desktop starts feeling heavy, but it should wait until your work is safe.".to_string(),
                    estimated_additional_work_label: "+45 minutes".to_string(),
                };
            }

            return Recommendation {
                text: format!("Restart {} when you are finished with it.", application.name),
                explanation: format!(
                    "{} is the least disruptive care candidate right now. Wait until it is not doing important work.",
                    application.name
                ),
                estimated_additional_work_label: "+25 minutes".to_string(),
            };
        }
    }

    if memory_score <= storage_score {
        Recommendation {
            text: "Close or restart the heaviest app when you are ready.".to_string(),
            explanation: "This is the lowest-disruption way to buy more comfortable working time without restarting the whole Mac.".to_string(),
            estimated_additional_work_label: "+20 minutes".to_string(),
        }
    } else {
        Recommendation {
            text: "Review storage when you have a quiet moment.".to_string(),
            explanation: "Storage does not need to interrupt you this second, but making room later can protect smoother work.".to_string(),
            estimated_additional_work_label: "+15 minutes".to_string(),
        }
    }
}

fn flow_remaining_estimate(
    system_score: u8,
    memory_score: u8,
    storage_score: u8,
    application_score: u8,
    browser_score: u8,
    renderer_score: u8,
    window_server_score: u8,
) -> FlowEstimate {
    let weakest_signal = [
        memory_score,
        storage_score,
        application_score,
        browser_score,
        renderer_score,
        window_server_score,
    ]
    .into_iter()
    .min()
    .unwrap_or(system_score);

    let blended_score = (system_score as f64 * 0.72) + (weakest_signal as f64 * 0.28);
    let minutes = if blended_score >= 95.0 {
        410
    } else if blended_score >= 90.0 {
        360
    } else if blended_score >= 82.0 {
        240
    } else if blended_score >= 74.0 {
        110
    } else if blended_score >= 66.0 {
        52
    } else {
        18
    };

    FlowEstimate {
        label: format_flow_remaining(minutes),
        minutes,
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
    let headline = if score >= 90 {
        "RAM has room."
    } else if score >= 70 {
        "RAM is getting tighter."
    } else {
        "RAM needs care soon."
    };
    let detail = if score >= 90 {
        "Enough working memory is available for smooth app switching."
    } else if score >= 70 {
        "Heavy apps may start to affect smoothness."
    } else {
        "Your Mac may feel constrained until memory pressure eases."
    };
    let value = format!(
        "{} available ({}%), {} used of {} RAM",
        format_bytes(snapshot.memory.available_bytes),
        format_percent(snapshot.memory.available_bytes, snapshot.memory.total_bytes),
        format_bytes(snapshot.memory.used_bytes),
        format_bytes(snapshot.memory.total_bytes)
    );
    let metric_label = format!(
        "{} of {} used",
        format_bytes(snapshot.memory.used_bytes),
        format_bytes(snapshot.memory.total_bytes)
    );
    let metric_percent = format!(
        "{}%",
        format_percent(snapshot.memory.used_bytes, snapshot.memory.total_bytes)
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

fn storage_health(snapshot: &SystemSnapshot, score: u8) -> DomainHealth {
    let label = domain_label(score);
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
    let headline = if score >= 90 {
        "Disk space has room."
    } else if score >= 70 {
        "Storage is beginning to fill."
    } else {
        "Disk space needs attention soon."
    };
    let detail = if score >= 90 {
        "There is enough free disk space for normal work."
    } else if score >= 70 {
        "Free disk space is lower than ideal."
    } else {
        "Low disk space can make updates, caches, and app work less reliable."
    };
    let value = format!(
        "{} free ({}%) on {}, {} used of {} disk",
        format_bytes(snapshot.storage.available_bytes),
        format_percent(snapshot.storage.available_bytes, snapshot.storage.total_bytes),
        storage_location,
        format_bytes(snapshot.storage.used_bytes),
        format_bytes(snapshot.storage.total_bytes)
    );
    let metric_label = format!(
        "{} of {} free",
        format_bytes(snapshot.storage.available_bytes),
        format_bytes(snapshot.storage.total_bytes)
    );
    let metric_percent = format!(
        "{}%",
        format_percent(snapshot.storage.available_bytes, snapshot.storage.total_bytes)
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
        } else if score >= 70 {
            format!("{} is adding browser load.", browser.name)
        } else {
            format!("{} is likely affecting smoothness.", browser.name)
        };
        let detail = if score >= 90 {
            "Browser load is not standing out in this check-in.".to_string()
        } else if score >= 70 {
            format!(
                "{} is carrying noticeable tab and renderer work.",
                browser.name
            )
        } else {
            format!(
                "{} is one of the likely reasons the Mac may feel slower.",
                browser.name
            )
        };
        let value = format!(
            "{} RAM, {} processes, {} renderers",
            format_bytes(browser.memory_bytes),
            browser.process_count,
            browser.renderer_count
        );
        let metric_label = format!(
            "{} of {} used",
            format_bytes(browser.memory_bytes),
            format_bytes(snapshot.memory.total_bytes)
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
    } else if score >= 70 {
        format!("{top_application_name} may need care next.")
    } else {
        format!("{top_application_name} may interrupt momentum.")
    };
    let detail = if let Some(application) = top_application {
        if score >= 90 {
            format!(
                "{} is doing the most work, but it does not look disruptive.",
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
                "{top_application_name} using {} RAM",
                format_bytes(application.memory_bytes)
            )
        })
        .unwrap_or_else(|| "No standout application".to_string());
    let metric_label = top_application
        .map(|application| {
            format!(
                "{} of {} used",
                format_bytes(application.memory_bytes),
                format_bytes(snapshot.memory.total_bytes)
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
            } else if app_ratio >= 0.08 {
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
            } else if app_ratio >= 0.08 {
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
                    "PulseCore detected active work. Restarting now would interrupt your workflow, so System Pulse will stay quiet unless care becomes worthwhile later.".to_string(),
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
            } else if index == 0 && is_finder && app_ratio >= 0.08 {
                (
                    "Restart Finder at your next break".to_string(),
                    "Finder can sometimes make window movement and file browsing feel heavier. Restarting Finder is usually quick and does not restart your Mac.".to_string(),
                    "+5 minutes".to_string(),
                    "restartFinder".to_string(),
                    "Finder".to_string(),
                    "Restart Finder".to_string(),
                    true,
                    false,
                )
            } else if index == 0 && is_window_server && app_ratio >= 0.15 {
                (
                    "Review desktop responsiveness".to_string(),
                    "Desktop responsiveness is doing unusual work. Opening Activity Monitor is the safest next step before deciding whether a full Mac restart is worth it.".to_string(),
                    "+10 minutes".to_string(),
                    "openActivityMonitor".to_string(),
                    "Activity Monitor".to_string(),
                    "Open Activity Monitor".to_string(),
                    true,
                    false,
                )
            } else if index == 0 && app_ratio >= 0.15 && safe_restart_target(display_name) {
                (
                    format!("Restart {display_name} when finished"),
                    format!(
                        "{display_name} is carrying a noticeable amount of work. Restarting it after your current task may make the next work block feel smoother."
                    ),
                    "+25 minutes".to_string(),
                    "restartApp".to_string(),
                    display_name.to_string(),
                    "Restart now".to_string(),
                    true,
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
            } else if app_ratio >= 0.08 {
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
        return "Safari can hold onto tab and page work over long sessions. Quitting it at a natural break can free that work without interrupting what you are doing now.".to_string();
    }

    format!(
        "{name} gradually accumulates browser processes over long sessions. This does not usually cause problems immediately, but it can begin making your Mac feel less responsive over time."
    )
}

fn browser_recommendation_explanation(browser: &BrowserSnapshot) -> String {
    let uptime = browser
        .uptime_seconds
        .map(format_duration)
        .map(|duration| format!(" It has been running for {duration}."))
        .unwrap_or_default();
    format!(
        "{} is carrying a lot of today's browser work.{} Restarting {} at a natural break would likely make things feel smoother.",
        browser.name, uptime, browser.name
    )
}

fn domain_label(score: u8) -> &'static str {
    if score >= 90 {
        "OK"
    } else if score >= 70 {
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
