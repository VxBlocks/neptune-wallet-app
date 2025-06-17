use wallet_macros::tauri_command;

use super::{persist::PersisStore, Memstore};

#[tauri_command]
pub async fn session_store_get(key: String) -> Option<String> {
    let store = crate::service::get_state::<Memstore>();
    let value = store.get(&key).await;
    value
}

#[tauri_command]
pub async fn session_store_set(key: String, value: String) {
    let store = crate::service::get_state::<Memstore>();
    store.set(&key, &value).await;
}

#[tauri_command]
pub async fn session_store_del(key: String) -> Option<String> {
    let store = crate::service::get_state::<Memstore>();
    let value = store.del(&key).await;
    value
}

#[tauri_command]
pub async fn persist_store_execute(sql: String) -> Result<Vec<serde_json::Value>, String> {
    let store = crate::service::get_state::<PersisStore>();
    store.execute(&sql).await.map_err(|e| e.to_string())
}
