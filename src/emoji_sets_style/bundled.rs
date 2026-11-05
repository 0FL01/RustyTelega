//! Telegram Desktop's emoji sheets, shipped inside Quill (tdesktop
//! `Telegram/Resources/emoji/emoji_1..8.webp`, indexed as its
//! `codegen_emoji` indexes `lib_ui/emoji.txt`). They are written out once
//! as an ordinary pack under `<app data>/emoji-sets/tdesktop/`, so drawing
//! goes through the same loader as a downloaded set.
#![deny(clippy::indexing_slicing)]

use super::catalog::{EmojiSetId, set_dir};
use std::path::Path;

/// The pack's files, written in this order (`config.json` last, so a pack
/// cut short by a crash is not ready).
const FILES: [(&str, &[u8]); 9] = [
    (
        "emoji_0.webp",
        include_bytes!("../../assets/emoji/tdesktop/emoji_0.webp"),
    ),
    (
        "emoji_1.webp",
        include_bytes!("../../assets/emoji/tdesktop/emoji_1.webp"),
    ),
    (
        "emoji_2.webp",
        include_bytes!("../../assets/emoji/tdesktop/emoji_2.webp"),
    ),
    (
        "emoji_3.webp",
        include_bytes!("../../assets/emoji/tdesktop/emoji_3.webp"),
    ),
    (
        "emoji_4.webp",
        include_bytes!("../../assets/emoji/tdesktop/emoji_4.webp"),
    ),
    (
        "emoji_5.webp",
        include_bytes!("../../assets/emoji/tdesktop/emoji_5.webp"),
    ),
    (
        "emoji_6.webp",
        include_bytes!("../../assets/emoji/tdesktop/emoji_6.webp"),
    ),
    (
        "emoji_7.webp",
        include_bytes!("../../assets/emoji/tdesktop/emoji_7.webp"),
    ),
    (
        "config.json",
        include_bytes!("../../assets/emoji/tdesktop/config.json"),
    ),
];

/// The four faces the "Choose emoji set" box shows for the set.
pub const PREVIEW: &[u8] = include_bytes!("../../assets/emoji/tdesktop/preview.webp");

/// Write the bundled pack under `app_root` unless an identical one is there.
/// Blocking; call off the UI thread.
pub fn ensure_installed(app_root: &Path) -> std::io::Result<()> {
    let dir = set_dir(app_root, EmojiSetId::Tdesktop);
    let current = FILES.iter().all(|(name, bytes)| {
        std::fs::metadata(dir.join(name)).is_ok_and(|meta| meta.len() == bytes.len() as u64)
    }) && std::fs::read(dir.join("config.json"))
        .is_ok_and(|config| FILES.last().is_some_and(|(_, bytes)| config == *bytes));
    if current {
        return Ok(());
    }
    std::fs::create_dir_all(&dir)?;
    let _ = std::fs::remove_file(dir.join("config.json"));
    for (name, bytes) in FILES {
        let part = dir.join(format!(".{name}.part"));
        std::fs::write(&part, bytes)?;
        crate::fs_retry::rename_retrying(&part, &dir.join(name), crate::fs_retry::LONG)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::emoji_sets_style::pack::SpritePack;

    #[test]
    fn the_bundled_pack_installs_and_finds_the_robot() {
        let root = std::env::temp_dir().join(format!("quill-emoji-bundled-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        ensure_installed(&root).expect("installed");
        let pack = SpritePack::load(&set_dir(&root, EmojiSetId::Tdesktop)).expect("ready");
        assert_eq!(pack.cell, 72);
        // 🤖 is entry 118 of emoji.txt's first section: sheet 0, cell 118.
        let robot = pack.lookup("\u{1f916}").expect("robot");
        assert_eq!((robot.sheet, robot.index), (0, 118));
        assert_eq!(pack.cell_origin(robot.index), (22 * 72, 3 * 72));
        // A second call leaves the files alone.
        ensure_installed(&root).expect("still installed");
        let _ = std::fs::remove_dir_all(&root);
    }
}
