pub mod commands;

pub type Result<T> = std::result::Result<T, String>;

pub trait TauriCommandResultExt {
    type Output;

    /// Converts any error into a string automatically for Tauri commands
    fn into_tauri_result(self) -> std::result::Result<Self::Output, String>;
}

impl<T> TauriCommandResultExt for std::result::Result<T, anyhow::Error> {
    type Output = T;

    fn into_tauri_result(self) -> std::result::Result<T, String> {
        self.map_err(|e| format!("{:#?}", e))
    }
}
