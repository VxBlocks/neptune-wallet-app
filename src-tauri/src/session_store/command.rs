use super::{persist::PersisStore, Memstore};

#[cfg_attr(feature = "gui", tauri::command)]
pub async fn session_store_get(key: String) -> Option<String> {
    let store = crate::service::get_state::<Memstore>();
    let value = store.get(&key).await;
    value
}

#[cfg_attr(feature = "gui", tauri::command)]
pub async fn session_store_set(key: String, value: String) {
    let store = crate::service::get_state::<Memstore>();
    store.set(&key, &value).await;
}

#[cfg_attr(feature = "gui", tauri::command)]
pub async fn session_store_del(key: String) -> Option<String> {
    let store = crate::service::get_state::<Memstore>();
    let value = store.del(&key).await;
    value
}

#[cfg_attr(feature = "gui", tauri::command)]
pub async fn persist_store_execute(sql: String) -> Result<Vec<serde_json::Value>, String> {
    let store = crate::service::get_state::<PersisStore>();
    store.execute(&sql).await.map_err(|e| e.to_string())
}
