mod collectors;
mod models;
mod pulse_core;

use models::TodayPulse;
use tauri::image::Image;
use tauri::menu::{MenuBuilder, MenuItemBuilder};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{AppHandle, Emitter, LogicalSize, Manager, Size};

#[cfg(target_os = "macos")]
use std::process::Command;

#[cfg(target_os = "macos")]
use std::{thread, time::Duration};

#[tauri::command]
fn get_today_pulse() -> Result<TodayPulse, String> {
    let snapshot = collectors::collect_system_snapshot().map_err(|error| {
        format!("System Pulse could not read your local system data yet. {error}")
    })?;
    Ok(pulse_core::evaluate(snapshot))
}

#[tauri::command]
fn update_tray_score(
    app: AppHandle,
    system_score: u8,
    health_state: String,
    flow_remaining_label: String,
) -> Result<(), String> {
    let Some(tray) = app.tray_by_id("system-pulse") else {
        return Ok(());
    };

    let label = match health_state.as_str() {
        "attention" => "needs attention",
        "critical" => "needs immediate attention",
        _ => "is healthy",
    };
    tray.set_title(Some(&system_score.to_string()))
        .map_err(|error| format!("Could not update menu bar score: {error}"))?;
    tray.set_tooltip(Some(&format!(
        "System Pulse {label}: {system_score}. {flow_remaining_label} flow remaining."
    )))
        .map_err(|error| format!("Could not update menu bar tooltip: {error}"))?;
    Ok(())
}

#[tauri::command]
fn open_today_window(app: AppHandle) -> Result<(), String> {
    show_today(&app);
    Ok(())
}

#[tauri::command]
fn open_quick_checkin(app: AppHandle) -> Result<(), String> {
    show_quick_checkin(&app);
    Ok(())
}

#[tauri::command]
fn perform_care_action(action_kind: String, target: String) -> Result<String, String> {
    platform_perform_care_action(&action_kind, &target)
}

#[cfg(target_os = "macos")]
fn platform_perform_care_action(action_kind: &str, target: &str) -> Result<String, String> {
    match action_kind {
        "restartApp" => {
            ensure_allowed_app_target(target)?;
            quit_application(target)?;
            thread::sleep(Duration::from_millis(1_200));
            open_application(target)?;
            Ok(format!("{target} has been restarted."))
        }
        "quitApp" => {
            ensure_allowed_app_target(target)?;
            quit_application(target)?;
            Ok(format!("{target} has been asked to quit."))
        }
        "restartFinder" => {
            run_status("killall", &["Finder"], "Could not restart Finder")?;
            Ok("Finder has been restarted.".to_string())
        }
        "openStorageSettings" => {
            if run_status(
                "open",
                &["x-apple.systempreferences:com.apple.settings.Storage"],
                "Could not open Storage Settings",
            )
            .is_err()
            {
                run_status(
                    "open",
                    &["-b", "com.apple.systempreferences"],
                    "Could not open System Settings",
                )?;
            }
            Ok("Storage Settings has been opened.".to_string())
        }
        "openActivityMonitor" => {
            open_application("Activity Monitor")?;
            Ok("Activity Monitor has been opened.".to_string())
        }
        _ => Err("System Pulse does not know how to perform that care action yet.".to_string()),
    }
}

#[cfg(not(target_os = "macos"))]
fn platform_perform_care_action(_action_kind: &str, _target: &str) -> Result<String, String> {
    Err("Care actions are wired for macOS in this UAT build.".to_string())
}

#[cfg(target_os = "macos")]
fn ensure_allowed_app_target(target: &str) -> Result<(), String> {
    match target {
        "Google Chrome" | "Microsoft Edge" | "Firefox" | "Safari" => Ok(()),
        _ => Err("System Pulse will not control that application yet.".to_string()),
    }
}

#[cfg(target_os = "macos")]
fn quit_application(target: &str) -> Result<(), String> {
    let script = format!("tell application \"{}\" to quit", escape_applescript(target));
    run_status(
        "osascript",
        &["-e", &script],
        &format!("Could not ask {target} to quit"),
    )
}

#[cfg(target_os = "macos")]
fn open_application(target: &str) -> Result<(), String> {
    run_status(
        "open",
        &["-a", target],
        &format!("Could not open {target}"),
    )
}

#[cfg(target_os = "macos")]
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

#[cfg(target_os = "macos")]
fn escape_applescript(value: &str) -> String {
    value.replace('\\', "\\\\").replace('"', "\\\"")
}

fn show_today(app: &AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.set_min_size(Some(Size::Logical(LogicalSize::new(1040.0, 720.0))));
        let _ = window.set_size(Size::Logical(LogicalSize::new(1180.0, 820.0)));
        let _ = window.show();
        let _ = window.unminimize();
        let _ = window.set_focus();
        let _ = app.emit("system-pulse-show-today", ());
    }
}

fn show_quick_checkin(app: &AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.set_min_size(Some(Size::Logical(LogicalSize::new(360.0, 480.0))));
        let _ = window.set_size(Size::Logical(LogicalSize::new(440.0, 560.0)));
        let _ = window.show();
        let _ = window.unminimize();
        let _ = window.set_focus();
        let _ = app.emit("system-pulse-show-quick-checkin", ());
    }
}

fn main() {
    tauri::Builder::default()
        .setup(|app| {
            let open = MenuItemBuilder::with_id("open_today", "Open Today").build(app)?;
            let refresh = MenuItemBuilder::with_id("refresh_health", "Refresh Health").build(app)?;
            let quit = MenuItemBuilder::with_id("quit", "Quit System Pulse").build(app)?;
            let menu = MenuBuilder::new(app)
                .items(&[&open, &refresh])
                .separator()
                .item(&quit)
                .build()?;
            let icon = Image::from_bytes(include_bytes!("../icons/32x32.png"))?;

            TrayIconBuilder::with_id("system-pulse")
                .icon(icon)
                .title("...")
                .tooltip("System Pulse is starting")
                .menu(&menu)
                .show_menu_on_left_click(false)
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "open_today" => show_today(app),
                    "refresh_health" => {
                        show_quick_checkin(app);
                        let _ = app.emit("system-pulse-refresh", ());
                    }
                    "quit" => app.exit(0),
                    _ => {}
                })
                .on_tray_icon_event(|tray, event| {
                    if let TrayIconEvent::Click {
                        button: MouseButton::Left,
                        button_state: MouseButtonState::Up,
                        ..
                    } = event
                    {
                        show_quick_checkin(tray.app_handle());
                    }
                })
                .build(app)?;

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_today_pulse,
            update_tray_score,
            open_today_window,
            open_quick_checkin,
            perform_care_action
        ])
        .run(tauri::generate_context!())
        .expect("error while running System Pulse");
}
