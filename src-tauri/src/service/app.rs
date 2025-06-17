use serde::{Deserialize, Serialize};
#[cfg(feature = "gui")]
use tauri::Emitter;
#[cfg(all(feature = "gui", desktop))]
use tauri_plugin_dialog::DialogExt;

#[cfg(feature = "gui")]
#[derive(Debug, Serialize)]
pub struct BuildInfo {
    pub time: String,
    pub commit: String,
}

#[cfg_attr(feature = "gui", tauri::command)]
#[cfg(feature = "gui")]
pub fn get_build_info() -> BuildInfo {
    let commit = env!("git_commit").to_string();
    let time = env!("build_time").to_string();

    BuildInfo { time, commit }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateInfo {
    pub version: String,
    pub url: String,
}

#[cfg_attr(feature = "gui", tauri::command)]
#[cfg(feature = "gui")]
pub async fn update_info() -> Result<UpdateInfo, String> {
    let resp = reqwest::get(
        "https://raw.githubusercontent.com/VxBlocks/vxb_neptune_wallet/refs/heads/main/update.json",
    )
    .await
    .map_err(|e| e.to_string())?
    .json::<UpdateInfo>()
    .await
    .map_err(|e| e.to_string())?;

    Ok(resp)
}

#[cfg(all(feature = "gui", desktop))]
pub fn error_dialog(app: &tauri::AppHandle, message: &str) {
    use tauri_plugin_dialog::MessageDialogButtons;

    app.dialog()
        .message(message)
        .title("error")
        .buttons(MessageDialogButtons::Ok)
        .blocking_show();
    std::process::exit(1)
}

#[cfg(feature = "gui")]
pub fn emit_event_to<I, S>(target: I, event: &str, payload: S) -> anyhow::Result<()>
where
    I: Into<tauri::EventTarget>,
    S: Serialize + Clone,
{
    let app = crate::service::get_state::<tauri::AppHandle>();
    let _ = app.emit_to(target, event, payload);
    Ok(())
}

#[cfg(not(feature = "gui"))]
pub fn emit_event_to<I, S>(_target: I, _event: &str, _payload: S) -> anyhow::Result<()> {
    Ok(())
}
