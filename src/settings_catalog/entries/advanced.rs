//! Advanced page entries.

use crate::lang::keys;
use crate::settings_catalog::{AGENT_SWITCH, Entry, Page};

const P: Page = Page::Advanced;

pub(super) const ENTRIES: &[Entry] = &[
    Entry::new("advanced.experimental", P, keys::SETTINGS_EXPERIMENTAL)
        .keywords(&["labs", "unstable"]),
    Entry::new(
        "advanced.split_profile_media",
        P,
        keys::Q_SETTINGS_ADVANCED_SPLIT_PROFILE_MEDIA,
    )
    .section(keys::SETTINGS_EXPERIMENTAL)
    .keywords(&["shared media"])
    .experimental(),
    Entry::new("advanced.fast_buttons", P, keys::Q_MEDIA_FAST_BUTTONS_MODE)
        .section(keys::SETTINGS_EXPERIMENTAL)
        .keywords(&["inline keyboard", "1-9", "bots"])
        .experimental(),
    Entry::new("advanced.welcome_intro", P, keys::Q_FIRST_RUN_REPLAY).keywords(&[
        "first run",
        "onboarding",
        "tour",
    ]),
    Entry::new(
        "advanced.restore_defaults",
        P,
        keys::Q_SETTINGS_ADVANCED_RESTORE_DEFAULTS,
    )
    .section(keys::Q_SETTINGS_ADVANCED_SEC_SETTINGS_FILE)
    .keywords(&["reset settings", "factory"])
    .experimental(),
    Entry::new(
        "advanced.export_settings",
        P,
        keys::Q_SETTINGS_ADVANCED_EXPORT_SETTINGS,
    )
    .section(keys::Q_SETTINGS_ADVANCED_SEC_SETTINGS_FILE)
    .keywords(&["backup", "save"])
    .experimental(),
    Entry::new(
        "advanced.import_settings",
        P,
        keys::Q_SETTINGS_ADVANCED_IMPORT_SETTINGS,
    )
    .section(keys::Q_SETTINGS_ADVANCED_SEC_SETTINGS_FILE)
    .keywords(&["restore", "load"])
    .experimental(),
    Entry::new(AGENT_SWITCH, P, keys::Q_AGENT_ACCESS).keywords(&["ai", "mcp"]),
    Entry::new(
        "advanced.hardware_decoding",
        P,
        keys::SETTINGS_ENABLE_HWACCEL,
    )
    .keywords(&["gpu", "video performance", "acceleration"]),
];
