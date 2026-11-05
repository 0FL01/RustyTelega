//! An emoji picture inside a line of text, scaled the way Telegram Desktop
//! scales its sprite cells: `UniversalImages::draw` shrinks the 72 px cell
//! with `Qt::SmoothTransformation`, an area average over premultiplied
//! pixels, to the device size it draws at. Letting the GPU minify the 72 px
//! cell instead samples it, which reads sharper and noisier.
#![deny(clippy::indexing_slicing)]

use gpui_kit::*;
use smallvec::SmallVec;
use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::Arc;

/// Shrunk cells kept for reuse; the set is small (the emoji on screen at a
/// size or two), so a full cache is simply dropped.
const CACHE_CAP: usize = 512;

thread_local! {
    static SHRUNK: RefCell<HashMap<(ImageId, u32), Arc<RenderImage>>> =
        RefCell::new(HashMap::new());
}

/// Drop every shrunk cell (the emoji set changed), freeing their atlas
/// tiles.
pub(super) fn forget_all() {
    let images: Vec<Arc<RenderImage>> =
        SHRUNK.with(|cache| cache.borrow_mut().drain().map(|(_, image)| image).collect());
    super::image_budget::retire_all(images);
}

/// `source` drawn `edge` logical pixels square.
pub(super) fn inline_emoji(source: Arc<RenderImage>, edge: Pixels) -> InlineEmoji {
    InlineEmoji { source, edge }
}

pub(super) struct InlineEmoji {
    source: Arc<RenderImage>,
    edge: Pixels,
}

impl IntoElement for InlineEmoji {
    type Element = Self;

    fn into_element(self) -> Self::Element {
        self
    }
}

impl Element for InlineEmoji {
    type RequestLayoutState = ();
    type PrepaintState = ();

    fn id(&self) -> Option<ElementId> {
        None
    }

    fn source_location(&self) -> Option<&'static std::panic::Location<'static>> {
        None
    }

    fn request_layout(
        &mut self,
        _global_id: Option<&GlobalElementId>,
        _inspector_id: Option<&InspectorElementId>,
        window: &mut Window,
        cx: &mut App,
    ) -> (LayoutId, Self::RequestLayoutState) {
        let style = Style {
            size: size(self.edge.into(), self.edge.into()),
            flex_shrink: 0.,
            ..Style::default()
        };
        (window.request_layout(style, [], cx), ())
    }

    fn prepaint(
        &mut self,
        _global_id: Option<&GlobalElementId>,
        _inspector_id: Option<&InspectorElementId>,
        _bounds: Bounds<Pixels>,
        _request_layout: &mut Self::RequestLayoutState,
        _window: &mut Window,
        _cx: &mut App,
    ) -> Self::PrepaintState {
    }

    fn paint(
        &mut self,
        _global_id: Option<&GlobalElementId>,
        _inspector_id: Option<&InspectorElementId>,
        bounds: Bounds<Pixels>,
        _request_layout: &mut Self::RequestLayoutState,
        _prepaint: &mut Self::PrepaintState,
        window: &mut Window,
        _cx: &mut App,
    ) {
        let device = (f32::from(self.edge) * window.scale_factor())
            .round()
            .max(1.) as u32;
        let image = shrunk(&self.source, device).unwrap_or_else(|| self.source.clone());
        let _ = window.paint_image(bounds, bounds, Corners::default(), image, 0, false);
    }
}

/// `source` area-averaged to `edge` device pixels; `None` when it is not
/// larger than that or has no pixels to read.
fn shrunk(source: &Arc<RenderImage>, edge: u32) -> Option<Arc<RenderImage>> {
    let size = source.size(0);
    let (width, height) = (size.width.0 as u32, size.height.0 as u32);
    if width <= edge || height <= edge {
        return None;
    }
    let key = (source.id, edge);
    if let Some(found) = SHRUNK.with(|cache| cache.borrow().get(&key).cloned()) {
        return Some(found);
    }
    let pixels = source.as_bytes(0)?;
    let image = image::RgbaImage::from_raw(width, height, pixels.to_vec())?;
    let out = Arc::new(RenderImage::new(SmallVec::from_buf([image::Frame::new(
        area_average(&image, edge),
    )])));
    SHRUNK.with(|cache| {
        let mut cache = cache.borrow_mut();
        if cache.len() >= CACHE_CAP {
            let old: Vec<Arc<RenderImage>> = cache.drain().map(|(_, image)| image).collect();
            super::image_budget::retire_all(old);
        }
        cache.insert(key, out.clone());
    });
    Some(out)
}

/// Shrink `image` to `edge` square by averaging the premultiplied source
/// area under each target pixel (Qt's smooth scaling when shrinking).
/// Channel order does not matter; the last channel is alpha.
pub(super) fn area_average(image: &image::RgbaImage, edge: u32) -> image::RgbaImage {
    let (width, height) = image.dimensions();
    let sx = width as f32 / edge as f32;
    let sy = height as f32 / edge as f32;
    let mut out = image::RgbaImage::new(edge, edge);
    for ty in 0..edge {
        let (y0, y1) = (ty as f32 * sy, (ty + 1) as f32 * sy);
        for tx in 0..edge {
            let (x0, x1) = (tx as f32 * sx, (tx + 1) as f32 * sx);
            let mut sum = [0f32; 4];
            let mut area = 0f32;
            let mut y = y0.floor() as u32;
            while (y as f32) < y1 && y < height {
                let wy = (y1.min(y as f32 + 1.) - y0.max(y as f32)).max(0.);
                let mut x = x0.floor() as u32;
                while (x as f32) < x1 && x < width {
                    let wx = (x1.min(x as f32 + 1.) - x0.max(x as f32)).max(0.);
                    let w = wx * wy;
                    let p = image.get_pixel(x, y).0;
                    let a = f32::from(p[3]) / 255.;
                    sum[0] += f32::from(p[0]) * a * w;
                    sum[1] += f32::from(p[1]) * a * w;
                    sum[2] += f32::from(p[2]) * a * w;
                    sum[3] += a * w;
                    area += w;
                    x += 1;
                }
                y += 1;
            }
            let alpha = if area > 0. { sum[3] / area } else { 0. };
            let channel = |v: f32| {
                if sum[3] > 0. {
                    (v / sum[3]).round().clamp(0., 255.) as u8
                } else {
                    0
                }
            };
            out.put_pixel(
                tx,
                ty,
                image::Rgba([
                    channel(sum[0]),
                    channel(sum[1]),
                    channel(sum[2]),
                    (alpha * 255.).round().clamp(0., 255.) as u8,
                ]),
            );
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::area_average;

    #[test]
    fn a_four_to_one_shrink_averages_each_block() {
        let mut image = image::RgbaImage::new(4, 4);
        // Half the block opaque red, half transparent: half-alpha red.
        for y in 0..4 {
            for x in 0..2 {
                image.put_pixel(x, y, image::Rgba([255, 0, 0, 255]));
            }
        }
        let out = area_average(&image, 1);
        assert_eq!(out.get_pixel(0, 0).0, [255, 0, 0, 128]);
    }
}
