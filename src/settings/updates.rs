use super::*;

/// Device-wide updater policy. Old installations keep automatic stable updates.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct UpdatePreferences {
    pub automatic: bool,
    pub beta: bool,
}

impl Default for UpdatePreferences {
    fn default() -> Self {
        Self {
            automatic: true,
            beta: false,
        }
    }
}

pub fn load_update_preferences() -> UpdatePreferences {
    let Some(root) = safe_app_root() else {
        return UpdatePreferences::default();
    };
    load_json_prefs(
        &AccountPaths::for_root(&root, &AccountKey::primary()),
        "update_preferences.json",
    )
}

pub fn save_update_preferences(prefs: &UpdatePreferences) {
    if let Some(root) = safe_app_root() {
        save_json_prefs(
            &AccountPaths::for_root(&root, &AccountKey::primary()),
            "update_preferences.json",
            prefs,
        );
        flush_deferred_writes();
    }
}
