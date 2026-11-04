//! The palette keys Quill paints with, named as in Telegram Desktop's
//! palette (`lib_ui/ui/colors.palette`), so a `.tdesktop-theme` file sets
//! them directly. Every key here is wired to something on screen; keys a
//! theme file sets that Quill does not use are kept in the file but not
//! applied.

/// Where a key shows up, for grouping in the theme editor.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum KeyGroup {
    Window,
    Controls,
    ChatList,
    Messages,
}

impl KeyGroup {
    pub const ALL: [KeyGroup; 4] = [
        KeyGroup::Window,
        KeyGroup::Controls,
        KeyGroup::ChatList,
        KeyGroup::Messages,
    ];

    pub fn title(self) -> &'static str {
        match self {
            KeyGroup::Window => "Window",
            KeyGroup::Controls => "Buttons, menus and fields",
            KeyGroup::ChatList => "Chat list",
            KeyGroup::Messages => "Messages",
        }
    }
}

macro_rules! palette_keys {
    ($($variant:ident = $name:literal, $group:ident, $label:literal;)*) => {
        /// A palette key Quill reads from a theme.
        #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
        pub enum PaletteKey {
            $($variant),*
        }

        impl PaletteKey {
            /// Every key, in editor order.
            pub const ALL: &'static [PaletteKey] = &[$(PaletteKey::$variant),*];

            /// The key's name in a `.tdesktop-theme` file.
            pub fn name(self) -> &'static str {
                match self {
                    $(PaletteKey::$variant => $name),*
                }
            }

            /// What the key colours in Quill, for the theme editor.
            pub fn label(self) -> &'static str {
                match self {
                    $(PaletteKey::$variant => $label),*
                }
            }

            pub fn group(self) -> KeyGroup {
                match self {
                    $(PaletteKey::$variant => KeyGroup::$group),*
                }
            }

            pub fn from_name(name: &str) -> Option<Self> {
                match name {
                    $($name => Some(PaletteKey::$variant),)*
                    _ => None,
                }
            }
        }
    };
}

palette_keys! {
    WindowBg = "windowBg", Window, "Background";
    WindowFg = "windowFg", Window, "Text";
    WindowBoldFg = "windowBoldFg", Window, "Titles and names";
    WindowSubTextFg = "windowSubTextFg", Window, "Secondary text";
    WindowBgOver = "windowBgOver", Window, "Hovered row";
    WindowBgRipple = "windowBgRipple", Window, "Pressed row";
    WindowBgActive = "windowBgActive", Window, "Accent";
    WindowFgActive = "windowFgActive", Window, "Text on the accent";
    WindowActiveTextFg = "windowActiveTextFg", Window, "Links and active text";
    ActiveButtonBg = "activeButtonBg", Controls, "Primary button";
    ActiveButtonBgOver = "activeButtonBgOver", Controls, "Primary button, hovered";
    ActiveButtonFg = "activeButtonFg", Controls, "Primary button text";
    AttentionButtonFg = "attentionButtonFg", Controls, "Destructive actions";
    BoxTextFgError = "boxTextFgError", Controls, "Errors";
    MenuBg = "menuBg", Controls, "Menus and popovers";
    MenuSeparatorFg = "menuSeparatorFg", Controls, "Separators";
    InputBorderFg = "inputBorderFg", Controls, "Borders";
    PlaceholderFg = "placeholderFg", Controls, "Placeholders";
    FilterInputInactiveBg = "filterInputInactiveBg", Controls, "Panels and code blocks";
    CheckboxFg = "checkboxFg", Controls, "Switches, off";
    ScrollBarBg = "scrollBarBg", Controls, "Scroll bars";
    TitleBgActive = "titleBgActive", Controls, "Title bar";
    DialogsBg = "dialogsBg", ChatList, "Chat list background";
    DialogsUnreadBgMuted = "dialogsUnreadBgMuted", ChatList, "Muted unread badge";
    MsgInBg = "msgInBg", Messages, "Incoming bubble";
    MsgOutBg = "msgOutBg", Messages, "Outgoing bubble";
    MsgOutServiceFg = "msgOutServiceFg", Messages, "Links and names in outgoing bubbles";
    TitleBg = "titleBg", Window, "Title bar, inactive window";
    TitleButtonFg = "titleButtonFg", Window, "Window buttons";
    TitleButtonFgOver = "titleButtonFgOver", Window, "Window buttons, hovered";
    TitleButtonBgActiveOver = "titleButtonBgActiveOver", Window, "Window button background, hovered";
    TitleButtonCloseBgOver = "titleButtonCloseBgOver", Window, "Close button, hovered";
    TitleButtonCloseFgOver = "titleButtonCloseFgOver", Window, "Close button icon, hovered";
    ShadowFg = "shadowFg", Window, "Dividers and shadows";
    SideBarBg = "sideBarBg", ChatList, "Folder sidebar";
    SideBarBgActive = "sideBarBgActive", ChatList, "Folder sidebar, selected folder";
    SideBarBgRipple = "sideBarBgRipple", ChatList, "Folder sidebar, pressed";
    SideBarTextFg = "sideBarTextFg", ChatList, "Folder names";
    SideBarTextFgActive = "sideBarTextFgActive", ChatList, "Selected folder name";
    SideBarIconFg = "sideBarIconFg", ChatList, "Folder icons";
    SideBarIconFgActive = "sideBarIconFgActive", ChatList, "Selected folder icon";
    SideBarBadgeBg = "sideBarBadgeBg", ChatList, "Folder unread badge";
    SideBarBadgeBgMuted = "sideBarBadgeBgMuted", ChatList, "Folder unread badge, muted";
    SideBarBadgeFg = "sideBarBadgeFg", ChatList, "Folder unread badge text";
    DialogsBgOver = "dialogsBgOver", ChatList, "Hovered chat";
    DialogsBgActive = "dialogsBgActive", ChatList, "Selected chat";
    DialogsNameFg = "dialogsNameFg", ChatList, "Chat names";
    DialogsNameFgActive = "dialogsNameFgActive", ChatList, "Selected chat name";
    DialogsChatIconFg = "dialogsChatIconFg", ChatList, "Chat type icons";
    DialogsChatIconFgActive = "dialogsChatIconFgActive", ChatList, "Chat type icons, selected chat";
    DialogsDateFg = "dialogsDateFg", ChatList, "Chat dates";
    DialogsDateFgActive = "dialogsDateFgActive", ChatList, "Selected chat date";
    DialogsTextFg = "dialogsTextFg", ChatList, "Message previews";
    DialogsTextFgActive = "dialogsTextFgActive", ChatList, "Selected chat preview";
    DialogsTextFgService = "dialogsTextFgService", ChatList, "Senders and drafts in previews";
    DialogsTextFgServiceActive = "dialogsTextFgServiceActive", ChatList, "Senders in the selected chat preview";
    DialogsUnreadBg = "dialogsUnreadBg", ChatList, "Unread badge";
    DialogsUnreadFg = "dialogsUnreadFg", ChatList, "Unread badge text";
    DialogsVerifiedIconBg = "dialogsVerifiedIconBg", ChatList, "Verified badge";
    DialogsVerifiedIconFg = "dialogsVerifiedIconFg", ChatList, "Verified badge check";
    DialogsMenuIconFg = "dialogsMenuIconFg", ChatList, "Main menu button";
    DialogsMenuIconFgOver = "dialogsMenuIconFgOver", ChatList, "Main menu button, hovered";
    FilterInputActiveBg = "filterInputActiveBg", Controls, "Search field, focused";
    FilterInputBorderFg = "filterInputBorderFg", Controls, "Search field border, focused";
    PlaceholderFgActive = "placeholderFgActive", Controls, "Placeholders, focused";
    TopBarBg = "topBarBg", Messages, "Chat header";
    MenuIconFg = "menuIconFg", Messages, "Header buttons";
    MenuIconFgOver = "menuIconFgOver", Messages, "Header buttons, hovered";
    HistoryPinnedBg = "historyPinnedBg", Messages, "Pinned message bar";
    HistoryComposeAreaBg = "historyComposeAreaBg", Messages, "Message field background";
    HistoryComposeAreaFg = "historyComposeAreaFg", Messages, "Message field text";
    HistoryComposeIconFg = "historyComposeIconFg", Messages, "Message field buttons";
    HistoryComposeIconFgOver = "historyComposeIconFgOver", Messages, "Message field buttons, hovered";
    HistorySendIconFg = "historySendIconFg", Messages, "Send button";
    HistoryTextInFg = "historyTextInFg", Messages, "Incoming message text";
    HistoryTextOutFg = "historyTextOutFg", Messages, "Outgoing message text";
    MsgInDateFg = "msgInDateFg", Messages, "Incoming message time";
    MsgOutDateFg = "msgOutDateFg", Messages, "Outgoing message time";
    HistoryOutIconFg = "historyOutIconFg", Messages, "Read check marks";
    MsgInReplyBarColor = "msgInReplyBarColor", Messages, "Reply bar, incoming";
    MsgOutReplyBarColor = "msgOutReplyBarColor", Messages, "Reply bar, outgoing";
    MsgFileInBg = "msgFileInBg", Messages, "File and play buttons, incoming";
    MsgFileOutBg = "msgFileOutBg", Messages, "File and play buttons, outgoing";
    HistoryFileInIconFg = "historyFileInIconFg", Messages, "Play icon, incoming";
    HistoryFileOutIconFg = "historyFileOutIconFg", Messages, "Play icon, outgoing";
    MsgWaveformInActive = "msgWaveformInActive", Messages, "Voice waveform played, incoming";
    MsgWaveformInInactive = "msgWaveformInInactive", Messages, "Voice waveform, incoming";
    MsgWaveformOutActive = "msgWaveformOutActive", Messages, "Voice waveform played, outgoing";
    MsgWaveformOutInactive = "msgWaveformOutInactive", Messages, "Voice waveform, outgoing";
    MsgInServiceFg = "msgInServiceFg", Messages, "Links and names in incoming bubbles";
    MsgServiceBg = "msgServiceBg", Messages, "Service messages and date bubbles";
    MsgServiceFg = "msgServiceFg", Messages, "Service message text";
    MsgDateImgBg = "msgDateImgBg", Messages, "Time over media";
    MsgDateImgFg = "msgDateImgFg", Messages, "Time over media text";
    HistoryLinkInFg = "historyLinkInFg", Messages, "Links, incoming";
    HistoryLinkOutFg = "historyLinkOutFg", Messages, "Links, outgoing";
    HistoryPeer1NameFg = "historyPeer1NameFg", Messages, "Sender name colour 1";
    HistoryPeer2NameFg = "historyPeer2NameFg", Messages, "Sender name colour 2";
    HistoryPeer3NameFg = "historyPeer3NameFg", Messages, "Sender name colour 3";
    HistoryPeer4NameFg = "historyPeer4NameFg", Messages, "Sender name colour 4";
    HistoryPeer5NameFg = "historyPeer5NameFg", Messages, "Sender name colour 5";
    HistoryPeer6NameFg = "historyPeer6NameFg", Messages, "Sender name colour 6";
    HistoryPeer7NameFg = "historyPeer7NameFg", Messages, "Sender name colour 7";
    HistoryPeer8NameFg = "historyPeer8NameFg", Messages, "Sender name colour 8";
}

impl PaletteKey {
    pub const COUNT: usize = Self::ALL.len();

    /// Position in [`PaletteKey::ALL`] (and in a palette's colour array).
    pub fn index(self) -> usize {
        self as usize
    }

    /// tdesktop's `kColorizeIgnoredKeys`: semantic colours an accent never
    /// moves.
    pub fn colorize_ignored(self) -> bool {
        matches!(self, PaletteKey::BoxTextFgError)
    }

    /// The key tdesktop's `colors.palette` defines this one by, for a
    /// theme that does not set it (Omarchy's palette, an old custom theme).
    pub fn fallback(self) -> Option<PaletteKey> {
        match self {
            PaletteKey::TitleBg => Some(PaletteKey::WindowBgOver),
            PaletteKey::TitleButtonCloseFgOver => Some(PaletteKey::WindowFgActive),
            PaletteKey::DialogsBgOver => Some(PaletteKey::WindowBgOver),
            PaletteKey::DialogsNameFg => Some(PaletteKey::WindowBoldFg),
            PaletteKey::DialogsNameFgActive => Some(PaletteKey::WindowFgActive),
            PaletteKey::DialogsChatIconFg => Some(PaletteKey::DialogsNameFg),
            PaletteKey::DialogsChatIconFgActive => Some(PaletteKey::DialogsNameFgActive),
            PaletteKey::DialogsDateFg => Some(PaletteKey::WindowSubTextFg),
            PaletteKey::DialogsDateFgActive => Some(PaletteKey::WindowFgActive),
            PaletteKey::DialogsTextFg => Some(PaletteKey::WindowSubTextFg),
            PaletteKey::DialogsTextFgActive => Some(PaletteKey::WindowFgActive),
            PaletteKey::DialogsTextFgService => Some(PaletteKey::WindowActiveTextFg),
            PaletteKey::DialogsTextFgServiceActive => Some(PaletteKey::DialogsTextFgActive),
            PaletteKey::DialogsUnreadBg => Some(PaletteKey::WindowBgActive),
            PaletteKey::DialogsUnreadFg => Some(PaletteKey::WindowFgActive),
            PaletteKey::DialogsVerifiedIconBg => Some(PaletteKey::WindowBgActive),
            PaletteKey::DialogsVerifiedIconFg => Some(PaletteKey::WindowFgActive),
            PaletteKey::DialogsMenuIconFg => Some(PaletteKey::MenuIconFg),
            PaletteKey::DialogsMenuIconFgOver => Some(PaletteKey::MenuIconFgOver),
            PaletteKey::FilterInputActiveBg => Some(PaletteKey::WindowBg),
            PaletteKey::TopBarBg => Some(PaletteKey::WindowBg),
            PaletteKey::HistoryPinnedBg => Some(PaletteKey::HistoryComposeAreaBg),
            PaletteKey::HistoryComposeAreaBg => Some(PaletteKey::MsgInBg),
            PaletteKey::HistoryComposeAreaFg => Some(PaletteKey::HistoryTextInFg),
            PaletteKey::HistoryComposeIconFg => Some(PaletteKey::MenuIconFg),
            PaletteKey::HistoryComposeIconFgOver => Some(PaletteKey::MenuIconFgOver),
            PaletteKey::HistorySendIconFg => Some(PaletteKey::WindowBgActive),
            PaletteKey::HistoryTextInFg => Some(PaletteKey::WindowFg),
            PaletteKey::HistoryTextOutFg => Some(PaletteKey::WindowFg),
            PaletteKey::MsgFileInBg => Some(PaletteKey::WindowBgActive),
            PaletteKey::HistoryFileInIconFg => Some(PaletteKey::MsgInBg),
            PaletteKey::HistoryFileOutIconFg => Some(PaletteKey::MsgOutBg),
            PaletteKey::MsgWaveformInActive => Some(PaletteKey::WindowBgActive),
            PaletteKey::MsgInServiceFg => Some(PaletteKey::WindowActiveTextFg),
            PaletteKey::MsgServiceFg => Some(PaletteKey::WindowFgActive),
            PaletteKey::MsgDateImgFg => Some(PaletteKey::MsgServiceFg),
            PaletteKey::HistoryLinkInFg => Some(PaletteKey::WindowActiveTextFg),
            PaletteKey::HistoryLinkOutFg => Some(PaletteKey::WindowActiveTextFg),
            PaletteKey::HistoryPeer4NameFg => Some(PaletteKey::WindowActiveTextFg),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::PaletteKey;

    #[test]
    fn names_round_trip_and_indexes_match() {
        for (i, key) in PaletteKey::ALL.iter().enumerate() {
            assert_eq!(key.index(), i);
            assert_eq!(PaletteKey::from_name(key.name()), Some(*key));
            assert!(!key.label().is_empty());
        }
        assert_eq!(PaletteKey::from_name("nope"), None);
    }
}
