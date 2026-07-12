use std::process::Command;
use serde_json::Value;

#[derive(Debug, Default, Clone)]
pub struct NiriState {
    pub active_workspace_id: Option<u64>,
    pub total_workspaces: u64,
    pub focused_window_title: Option<String>,
}

pub fn get_niri_state() -> NiriState {
    let mut state = NiriState::default();

    // Fetch workspaces
    if let Ok(output) = Command::new("niri").args(&["msg", "-j", "workspaces"]).output() {
        if output.status.success() {
            if let Ok(json_val) = serde_json::from_slice::<Value>(&output.stdout) {
                if let Some(workspaces) = json_val.as_array() {
                    state.total_workspaces = workspaces.len() as u64;
                    for ws in workspaces {
                        if ws.get("is_active").and_then(|v| v.as_bool()).unwrap_or(false) {
                            state.active_workspace_id = ws.get("id").and_then(|v| v.as_u64());
                            break;
                        }
                    }
                }
            }
        }
    }

    // Fetch windows
    if let Ok(output) = Command::new("niri").args(&["msg", "-j", "windows"]).output() {
        if output.status.success() {
            if let Ok(json_val) = serde_json::from_slice::<Value>(&output.stdout) {
                if let Some(windows) = json_val.as_array() {
                    for win in windows {
                        if win.get("is_focused").and_then(|v| v.as_bool()).unwrap_or(false) {
                            state.focused_window_title = win.get("title")
                                .and_then(|v| v.as_str())
                                .map(|s| s.to_string());
                            break;
                        }
                    }
                }
            }
        }
    }

    state
}
