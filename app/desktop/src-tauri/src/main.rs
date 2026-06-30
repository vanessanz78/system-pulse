mod collectors;
mod models;
mod pulse_core;

use models::TodayPulse;
use tauri::image::Image;
use tauri::menu::{MenuBuilder, MenuItemBuilder};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{AppHandle, Emitter, Manager};

#[tauri::command]
fn get_today_pulse() -> Result<TodayPulse, String> {
    let snapshot = collectors::collect_system_snapshot().map_err(|error| {
        format!("System Pulse could not read your local system data yet. {error}")
    })?;
    Ok(pulse_core::evaluate(snapshot))
}

#[tauri::command]
fn update_tray_score(app: AppHandle, system_score: u8, health_state: String) -> Result<(), String> {
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
    tray.set_tooltip(Some(&format!("System Pulse {label}: {system_score}")))
        .map_err(|error| format!("Could not update menu bar tooltip: {error}"))?;
    Ok(())
}

fn show_today(app: &AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.show();
        let _ = window.unminimize();
        let _ = window.set_focus();
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
                        show_today(app);
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
                        show_today(tray.app_handle());
                    }
                })
                .build(app)?;

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_today_pulse,
            update_tray_score
        ])
        .run(tauri::generate_context!())
        .expect("error while running System Pulse");
}