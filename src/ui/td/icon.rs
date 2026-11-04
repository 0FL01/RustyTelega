//! tdesktop's icon masks (`Resources/icons`, `lib_ui/icons`), drawn the
//! way tdesktop draws them: the grey level of the PNG is the coverage and
//! the colour comes from the palette (`style::Icon`, `createIconMask`).
//! The 1x, 2x or 3x file is picked by the window's scale factor, as
//! tdesktop picks by its device pixel ratio.

use gpui_kit::*;
use smallvec::SmallVec;
use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::Arc;

macro_rules! td_icons {
    ($($variant:ident => $path:literal,)*) => {
        /// One tdesktop icon mask, named after its file.
        #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
        pub(crate) enum TdIcon {
            $($variant),*
        }

        impl TdIcon {
            /// The file under `assets/tdesktop-icons`, without the scale
            /// suffix and extension.
            #[allow(dead_code)]
            pub(crate) fn path(self) -> &'static str {
                match self {
                    $(TdIcon::$variant => $path),*
                }
            }

            /// The PNG for `scale` (1, 2 or 3).
            fn bytes(self, scale: u8) -> &'static [u8] {
                match (self, scale) {
                    $(
                        (TdIcon::$variant, 1) => include_bytes!(concat!(
                            env!("CARGO_MANIFEST_DIR"), "/assets/tdesktop-icons/", $path, ".png"
                        )),
                        (TdIcon::$variant, 2) => include_bytes!(concat!(
                            env!("CARGO_MANIFEST_DIR"), "/assets/tdesktop-icons/", $path, "@2x.png"
                        )),
                        (TdIcon::$variant, _) => include_bytes!(concat!(
                            env!("CARGO_MANIFEST_DIR"), "/assets/tdesktop-icons/", $path, "@3x.png"
                        )),
                    )*
                }
            }
        }
    };
}

td_icons! {
    // Frame 0 of `animations/chat/voice_to_video.tgs` at `historySend.recordSize`
    // (`SendButton::paintRecord` draws the Lottie icon).
    VoiceRecord => "derived/voice_record",
    BubbleTail => "bubble_tail",
    TitleMinimize => "title_button_minimize",
    TitleMaximize => "title_button_maximize",
    TitleRestore => "title_button_restore",
    TitleClose => "title_button_close",
    DialogsMenu => "dialogs/dialogs_menu",
    ProxyOn => "proxy_on",
    ProxyOff => "proxy_off",
    DialogsChannel => "dialogs/dialogs_channel",
    DialogsChat => "dialogs/dialogs_chat",
    DialogsBot => "dialogs/dialogs_bot",
    DialogsVerifiedStar => "dialogs/dialogs_verified_star",
    DialogsVerifiedCheck => "dialogs/dialogs_verified_check",
    TopBarSearch => "top_bar_search",
    TopBarCall => "top_bar_call",
    TopBarProfile => "top_bar_profile",
    TopBarMenu => "title_menu_dots",
    PinnedShowAll => "pinned_show_all",
    InputAttach => "chat/input_attach",
    InputSmileFace => "chat/input_smile_face",
    InputRecord => "chat/input_record",
    InputSend => "chat/input_send",
    // Stream A: folder rail masks.
    FoldersAll => "folders/folders_all",
    FoldersPrivate => "folders/folders_private",
    FoldersBots => "folders/folders_bots",
    FoldersWork => "folders/folders_work",
    FoldersChannels => "folders/folders_channels",
    FoldersStudy => "folders/folders_study",
    FoldersCustom => "folders/folders_custom",
    FoldersEdit => "folders/folders_edit",
    FoldersCat => "folders/folders_cat",
    FoldersBook => "folders/folders_book",
    FoldersMoney => "folders/folders_money",
    FoldersGame => "folders/folders_game",
    FoldersLight => "folders/folders_light",
    FoldersLike => "folders/folders_like",
    FoldersNote => "folders/folders_note",
    FoldersPalette => "folders/folders_palette",
    FoldersTravel => "folders/folders_travel",
    FoldersSport => "folders/folders_sport",
    FoldersFavorite => "folders/folders_favorite",
    FoldersAirplane => "folders/folders_airplane",
    FoldersGroups => "folders/folders_group",
    FoldersUnread => "folders/folders_unread",
    FoldersCrown => "folders/folders_crown",
    FoldersFlower => "folders/folders_flower",
    FoldersHome => "folders/folders_home",
    FoldersLove => "folders/folders_love",
    FoldersMask => "folders/folders_mask",
    FoldersParty => "folders/folders_party",
    FoldersTrade => "folders/folders_trade",
    FoldersUnmuted => "folders/folders_unmuted",
    FoldersSetup => "folders/folders_setup",
    FoldersPoo => "folders/folders_poo",
    // Stream B
    ReplyCancel => "box_button_close",
    // Stream C
    VoiceToText => "chat/voice_to_text",
    HistoryFilePlay => "history_file_play",
    HistorySent => "history_sent",
    HistoryReceived => "history_received",
}

/// The scale a window draws at, as the mask file to use.
fn mask_scale(window: &Window) -> u8 {
    let factor = window.scale_factor();
    if factor <= 1. {
        1
    } else if factor <= 2. {
        2
    } else {
        3
    }
}

type Key = (TdIcon, u32, u8, bool);

thread_local! {
    /// Tinted masks: a handful of icons in a handful of colours, kept for
    /// the run (their sprites stay in the atlas like any other image).
    static TINTED: RefCell<HashMap<Key, Arc<RenderImage>>> = RefCell::new(HashMap::new());
    /// 1x sizes, decoded once per icon.
    static SIZES: RefCell<HashMap<TdIcon, Size<Pixels>>> = RefCell::new(HashMap::new());
}

/// The icon's size at 1x, in logical pixels (tdesktop `icon.size()`).
pub(crate) fn icon_size(icon: TdIcon) -> Size<Pixels> {
    SIZES.with(|sizes| {
        *sizes.borrow_mut().entry(icon).or_insert_with(|| {
            let (w, h) = image::load_from_memory(icon.bytes(1))
                .map(|image| (image.width(), image.height()))
                .unwrap_or((0, 0));
            size(px(w as f32), px(h as f32))
        })
    })
}

fn pack(color: Rgba) -> u32 {
    let byte = |v: f32| (v.clamp(0., 1.) * 255.).round() as u32;
    (byte(color.r) << 24) | (byte(color.g) << 16) | (byte(color.b) << 8) | byte(color.a)
}

/// The mask for `scale` in `color`, as straight-alpha BGRA (GPUI sprites),
/// mirrored left to right when `mirror` (tdesktop's `-flip_horizontal`).
fn tint(icon: TdIcon, color: u32, scale: u8, mirror: bool) -> Option<Arc<RenderImage>> {
    let mut image = image::load_from_memory(icon.bytes(scale))
        .ok()?
        .into_rgba8();
    if mirror {
        image::imageops::flip_horizontal_in_place(&mut image);
    }
    let [r, g, b, a] = color.to_be_bytes();
    for pixel in image.pixels_mut() {
        // tdesktop masks are grey: the red channel is the coverage.
        let coverage = u16::from(pixel.0[0]);
        let alpha = (coverage * u16::from(a) + 127) / 255;
        pixel.0 = [b, g, r, alpha as u8];
    }
    Some(Arc::new(RenderImage::new(SmallVec::from_buf([
        image::Frame::new(image),
    ]))))
}

/// `icon` in `color`, sized as tdesktop sizes it.
pub(crate) fn td_icon(icon: TdIcon, color: impl Into<Rgba>) -> Img {
    td_icon_mirrored(icon, color, false)
}

/// [`td_icon`], mirrored left to right when `mirror`.
pub(crate) fn td_icon_mirrored(icon: TdIcon, color: impl Into<Rgba>, mirror: bool) -> Img {
    let color = pack(color.into());
    let size = icon_size(icon);
    img(ImageSource::Custom(Arc::new(move |window, _cx| {
        let key = (icon, color, mask_scale(window), mirror);
        TINTED.with(|cache| {
            let mut cache = cache.borrow_mut();
            if let Some(image) = cache.get(&key) {
                return Some(Ok(image.clone()));
            }
            let image = tint(icon, color, key.2, mirror)?;
            cache.insert(key, image.clone());
            Some(Ok(image))
        })
    })))
    .w(size.width)
    .h(size.height)
    .flex_none()
}
