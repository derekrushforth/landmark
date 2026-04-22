use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use tauri::AppHandle;
use tauri_plugin_store::StoreExt;

use crate::spaces::{list_spaces, switch_to_space, SpaceInfo};

const STORE_FILE: &str = "space-names.json";
const STORE_KEY: &str = "names";

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct NameMap(pub HashMap<String, String>);

#[tauri::command]
pub fn get_spaces() -> Vec<SpaceInfo> {
    list_spaces()
}

#[tauri::command]
pub fn cmd_switch_to_space(space_id: u64, display_id: String) -> Result<(), String> {
    switch_to_space(space_id, &display_id)
}

#[tauri::command]
pub fn get_space_names(app: AppHandle) -> Result<NameMap, String> {
    let store = app
        .store(STORE_FILE)
        .map_err(|e| format!("store open failed: {e}"))?;

    let names: NameMap = store
        .get(STORE_KEY)
        .and_then(|v| serde_json::from_value(v).ok())
        .unwrap_or_default();

    Ok(names)
}

#[tauri::command]
pub fn set_space_name(
    app: AppHandle,
    space_uuid: String,
    name: String,
) -> Result<(), String> {
    let store = app
        .store(STORE_FILE)
        .map_err(|e| format!("store open failed: {e}"))?;

    let mut names: NameMap = store
        .get(STORE_KEY)
        .and_then(|v| serde_json::from_value(v).ok())
        .unwrap_or_default();

    if name.is_empty() {
        names.0.remove(&space_uuid);
    } else {
        names.0.insert(space_uuid, name);
    }

    store.set(
        STORE_KEY,
        serde_json::to_value(&names).unwrap(),
    );
    store
        .save()
        .map_err(|e| format!("store save failed: {e}"))?;

    Ok(())
}
