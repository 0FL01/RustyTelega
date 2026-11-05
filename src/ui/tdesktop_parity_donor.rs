//! Pictures for the `ready-tdesktop-parity` demo, cut from the Telegram
//! Desktop reference screenshot itself when `QUILL_PARITY_DONOR` names it,
//! so avatars, photos and the GIF frame match the reference pixel for pixel
//! and the parity diff needs no masks. Nothing is committed: the crops go to
//! a temporary folder, which the demo's media roots allow.
//!
//! The rectangles are where the reference shows each picture (1919x1034,
//! Night, 100%). Overlays Quill draws itself (the GIF badge and the time
//! pill) are taken back out of the GIF frame by undoing their
//! `msgDateImgBg` blend.
#![deny(clippy::indexing_slicing)]

use image::{Rgba, RgbaImage};
use std::path::{Path, PathBuf};

/// `(x0, y0, x1, y1)` in the reference.
type Rect = (u32, u32, u32, u32);

const LIST_AVATARS: [Rect; 4] = [
    (82, 86, 128, 132),
    (82, 148, 128, 194),
    (82, 210, 128, 256),
    (82, 272, 128, 318),
];
const ME_AVATAR: Rect = (539, 204, 572, 237);
const LAIN_AVATAR: Rect = (539, 408, 572, 441);
/// The visible bottom of the photo under the pinned bar.
const PHOTO_VISIBLE: Rect = (581, 128, 901, 237);
const REPLY_THUMB: Rect = (599, 463, 631, 495);
const GIF_FRAME: Rect = (581, 800, 901, 980);
/// `dialogsMiniPreview` squares at `textTop + dialogsMiniPreviewTop`.
const ROW_MINIS: [Rect; 3] = [
    (140, 113, 156, 129),
    (140, 175, 156, 191),
    (140, 237, 156, 253),
];
const ALBUM_MINIS: [Rect; 3] = [
    (140, 299, 156, 315),
    (158, 299, 174, 315),
    (176, 299, 192, 315),
];

/// The first reply strip's 20 px pattern copy: `FillBackgroundEmoji`'s
/// first entry, 28 px in from the strip's right edge (985) and 4 px down
/// from its top (459), at opacity 0.32.
const PATTERN_COPY: Rect = (957, 463, 977, 483);
const PATTERN_OPACITY: f32 = 0.32;
/// The strip's fill under the copy, and the tint the copy reads as at full
/// coverage (its strongest pixel, divided back by the opacity).
const PATTERN_BG: [f32; 3] = [50., 94., 135.];
const PATTERN_TINT: [f32; 3] = [100., 185., 244.];

/// The photo's size as the fixture declares it: 320 px wide in a 430 px
/// box (`photo_current`), its unseen top filled in.
pub(super) const PHOTO_SIZE: (u32, u32) = (320, 430);

/// The overlays inside the GIF frame, frame-relative: `(x, y, w, h)`,
/// radius half the height (`gif_badge`, `media_footer_pill` with the
/// video lift).
const GIF_OVERLAYS: [(f32, f32, f32, f32); 2] = [(4., 4., 37., 22.), (243., 150., 73., 22.)];
/// `msgDateImgBg` alpha (#00000054).
const DATE_IMG_BG_ALPHA: f32 = 84. / 255.;

/// Where the crops went.
pub(super) struct DonorFiles {
    pub(super) list_avatars: [PathBuf; 4],
    pub(super) me_avatar: PathBuf,
    pub(super) lain_avatar: PathBuf,
    pub(super) photo: PathBuf,
    pub(super) reply_thumb: PathBuf,
    pub(super) gif: PathBuf,
    /// The same frame as a one-frame GIF: the clip the bubble plays.
    pub(super) gif_clip: PathBuf,
    pub(super) row_minis: [PathBuf; 3],
    /// The reply pattern's emoji: its alpha read back from the 20 px copy.
    pub(super) pattern_emoji: PathBuf,
    pub(super) album_minis: [PathBuf; 3],
}

/// The folder the crops are written to, when a reference is named.
pub(crate) fn dir() -> Option<PathBuf> {
    std::env::var_os("QUILL_PARITY_DONOR")
        .map(|_| std::env::temp_dir().join(format!("quill-parity-donor-{}", std::process::id())))
}

/// Cut the pictures out of the reference; `None` without one or when it
/// does not read.
pub(super) fn prepare() -> Option<DonorFiles> {
    let reference = PathBuf::from(std::env::var_os("QUILL_PARITY_DONOR")?);
    let dir = dir()?;
    std::fs::create_dir_all(&dir).ok()?;
    let donor = image::open(&reference).ok()?.to_rgba8();
    let save = |name: &str, image: &RgbaImage| -> Option<PathBuf> {
        let path = dir.join(name);
        image.save(&path).ok()?;
        Some(path)
    };
    let cut = |rect: Rect| crop(&donor, rect);
    let gif_frame = without_overlays(cut(GIF_FRAME));
    let cut_all = |rects: &[Rect], prefix: &str| -> Option<Vec<PathBuf>> {
        rects
            .iter()
            .enumerate()
            .map(|(i, rect)| save(&format!("{prefix}-{i}.png"), &cut(*rect)))
            .collect()
    };
    Some(DonorFiles {
        list_avatars: cut_all(&LIST_AVATARS, "list-avatar")?.try_into().ok()?,
        me_avatar: save("me-avatar.png", &cut(ME_AVATAR))?,
        lain_avatar: save("lain-avatar.png", &cut(LAIN_AVATAR))?,
        photo: save("photo.png", &photo(&cut(PHOTO_VISIBLE)))?,
        reply_thumb: save("reply-thumb.png", &cut(REPLY_THUMB))?,
        gif: save("gif.png", &gif_frame)?,
        gif_clip: {
            let path = dir.join("gif.gif");
            let file = std::fs::File::create(&path).ok()?;
            let mut encoder = image::codecs::gif::GifEncoder::new_with_speed(file, 1);
            encoder
                .encode_frame(image::Frame::new(gif_frame.clone()))
                .ok()?;
            path
        },
        row_minis: cut_all(&ROW_MINIS, "row-mini")?.try_into().ok()?,
        album_minis: cut_all(&ALBUM_MINIS, "album-mini")?.try_into().ok()?,
        pattern_emoji: save("pattern-emoji.png", &pattern_mask(&cut(PATTERN_COPY)))?,
    })
}

/// The emoji's alpha from a tinted copy: how far each pixel moved from the
/// strip's fill towards the tint, over the copy's opacity.
fn pattern_mask(copy: &RgbaImage) -> RgbaImage {
    let axis: Vec<f32> = PATTERN_TINT
        .iter()
        .zip(PATTERN_BG)
        .map(|(t, b)| t - b)
        .collect();
    let norm: f32 = axis.iter().map(|v| v * v).sum();
    let mut out = RgbaImage::new(copy.width(), copy.height());
    for (x, y, pixel) in copy.enumerate_pixels() {
        let moved: f32 = pixel
            .0
            .iter()
            .zip(PATTERN_BG)
            .zip(&axis)
            .map(|((p, b), a)| (f32::from(*p) - b) * a)
            .sum();
        let alpha = (moved / norm / PATTERN_OPACITY).clamp(0., 1.);
        out.put_pixel(x, y, Rgba([255, 255, 255, (alpha * 255.).round() as u8]));
    }
    out
}

fn crop(image: &RgbaImage, (x0, y0, x1, y1): Rect) -> RgbaImage {
    image::imageops::crop_imm(image, x0, y0, x1 - x0, y1 - y0).to_image()
}

/// The visible bottom of the photo placed at the bottom of its declared
/// size; the hidden top repeats the first visible row.
fn photo(visible: &RgbaImage) -> RgbaImage {
    let (width, height) = PHOTO_SIZE;
    let mut out = RgbaImage::new(width, height);
    let top = height.saturating_sub(visible.height());
    for y in 0..height {
        let source_y = y
            .saturating_sub(top)
            .min(visible.height().saturating_sub(1));
        for x in 0..width.min(visible.width()) {
            out.put_pixel(x, y, *visible.get_pixel(x, source_y));
        }
    }
    out
}

/// Coverage of a rounded rectangle (radius half its height) at the centre
/// of pixel `(px, py)`.
fn pill_coverage((x, y, w, h): (f32, f32, f32, f32), px: u32, py: u32) -> f32 {
    let r = h / 2.;
    let (cx, cy) = (px as f32 + 0.5, py as f32 + 0.5);
    let nx = cx.clamp(x + r, x + w - r);
    let ny = cy.clamp(y + r, y + h - r);
    let d = ((cx - nx).powi(2) + (cy - ny).powi(2)).sqrt() - r;
    (0.5 - d).clamp(0., 1.)
}

/// Take the badge and the time pill back out of the GIF frame: undo the
/// translucent black where only the pill covers the frame, and fill the
/// label's text pixels from their neighbours (Quill draws the text again).
fn without_overlays(mut frame: RgbaImage) -> RgbaImage {
    let (width, height) = frame.dimensions();
    let mut text = vec![false; (width * height) as usize];
    for &pill in &GIF_OVERLAYS {
        for y in 0..height {
            for x in 0..width {
                let coverage = pill_coverage(pill, x, y);
                if coverage <= 0. {
                    continue;
                }
                let keep = 1. - DATE_IMG_BG_ALPHA * coverage;
                let Rgba([r, g, b, a]) = *frame.get_pixel(x, y);
                let undo = |v: u8| f32::from(v) / keep;
                let restored = [undo(r), undo(g), undo(b)];
                // Light pixels are the label's text (and its antialiased
                // edge), not the darkened frame, which is dark there.
                if restored.iter().any(|v| *v > 70.) {
                    if let Some(slot) = text.get_mut((y * width + x) as usize) {
                        *slot = true;
                    }
                    continue;
                }
                frame.put_pixel(
                    x,
                    y,
                    Rgba([
                        restored[0].round() as u8,
                        restored[1].round() as u8,
                        restored[2].round() as u8,
                        a,
                    ]),
                );
            }
        }
    }
    // The text's faint edge pixels sit next to the marked ones.
    let marked = text.clone();
    for y in 0..height {
        for x in 0..width {
            let near = [(-1i32, 0i32), (1, 0), (0, -1), (0, 1)]
                .iter()
                .any(|(dx, dy)| {
                    let (nx, ny) = (x as i32 + dx, y as i32 + dy);
                    nx >= 0
                        && ny >= 0
                        && (nx as u32) < width
                        && (ny as u32) < height
                        && marked
                            .get((ny as u32 * width + nx as u32) as usize)
                            .copied()
                            .unwrap_or(false)
                });
            if near && let Some(slot) = text.get_mut((y * width + x) as usize) {
                *slot = true;
            }
        }
    }
    fill_from_neighbours(&mut frame, &mut text);
    frame
}

/// Replace each marked pixel with the mean of its unmarked neighbours,
/// growing inwards until none is left.
fn fill_from_neighbours(image: &mut RgbaImage, marked: &mut [bool]) {
    let (width, height) = image.dimensions();
    let at = |x: u32, y: u32| (y * width + x) as usize;
    for _ in 0..16 {
        let mut changed = Vec::new();
        for y in 0..height {
            for x in 0..width {
                if !marked.get(at(x, y)).copied().unwrap_or(false) {
                    continue;
                }
                let mut sum = [0u32; 4];
                let mut n = 0;
                for (dx, dy) in [(-1i32, 0i32), (1, 0), (0, -1), (0, 1)] {
                    let (nx, ny) = (x as i32 + dx, y as i32 + dy);
                    if nx < 0 || ny < 0 || nx >= width as i32 || ny >= height as i32 {
                        continue;
                    }
                    let (nx, ny) = (nx as u32, ny as u32);
                    if marked.get(at(nx, ny)).copied().unwrap_or(true) {
                        continue;
                    }
                    let p = image.get_pixel(nx, ny).0;
                    for (s, v) in sum.iter_mut().zip(p) {
                        *s += u32::from(v);
                    }
                    n += 1;
                }
                if n > 0 {
                    let mean = sum.map(|s| (s / n) as u8);
                    changed.push((x, y, mean));
                }
            }
        }
        if changed.is_empty() {
            break;
        }
        for (x, y, mean) in changed {
            image.put_pixel(x, y, Rgba(mean));
            if let Some(slot) = marked.get_mut(at(x, y)) {
                *slot = false;
            }
        }
    }
}

/// A minithumbnail JSON for the picture at `path` (exact pixels: JPEG at
/// full quality).
pub(super) fn minithumbnail(path: &Path) -> Option<serde_json::Value> {
    use base64::Engine as _;
    let image = image::open(path).ok()?.to_rgb8();
    let mut bytes = Vec::new();
    image::codecs::jpeg::JpegEncoder::new_with_quality(&mut bytes, 100)
        .encode_image(&image)
        .ok()?;
    Some(
        serde_json::json!({"@type":"minithumbnail","width":image.width(),"height":image.height(),
        "data":base64::engine::general_purpose::STANDARD.encode(bytes)}),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_pill_blend_comes_back_out() {
        let mut frame = RgbaImage::from_pixel(320, 180, Rgba([60, 40, 20, 255]));
        let keep = 1. - DATE_IMG_BG_ALPHA;
        for y in 160..166 {
            for x in 280..300 {
                let v = |c: f32| (c * keep).round() as u8;
                frame.put_pixel(x, y, Rgba([v(60.), v(40.), v(20.), 255]));
            }
        }
        let out = without_overlays(frame);
        let Rgba([r, g, b, _]) = *out.get_pixel(290, 162);
        assert!((i32::from(r) - 60).abs() <= 1 && (i32::from(g) - 40).abs() <= 1);
        assert!((i32::from(b) - 20).abs() <= 1);
    }
}
