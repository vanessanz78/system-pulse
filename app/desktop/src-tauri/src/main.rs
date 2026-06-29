mod collectors;
mod models;
mod pulse_core;

use models::TodayPulse;

#[tauri::command]
fn get_today_pulse() -> Result<TodayPulse, String> {
    let snapshot = collectors::collect_system_snapshot()?;
    Ok(pulse_core::evaluate(snapshot))
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![get_today_pulse])
        .run(tauri::generate_context!())
        .expect("error while running System Pulse");
}
