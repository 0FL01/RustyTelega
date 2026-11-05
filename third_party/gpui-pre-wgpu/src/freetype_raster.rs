// Added by the Quill project (2026). See third_party/gpui-pre-wgpu/QUILL-CHANGES.md.
//! Grayscale glyphs rasterised by FreeType the way Qt draws them on a
//! desktop set to slight hinting and grayscale antialiasing: the outline is
//! loaded with `FT_LOAD_TARGET_LIGHT` (vertical-only hinting), moved by the
//! glyph's sub-pixel position with `FT_Set_Transform` before loading, as
//! `QFontEngineFT::loadGlyph` does, and rendered in `FT_RENDER_MODE_LIGHT`.
//! Telegram Desktop's text goes through that path, so Quill's glyphs match
//! its pixels instead of swash's full-hinting ones.
use anyhow::{Context as _, Result, anyhow};
use collections::HashMap;
use freetype_sys as ft;
use gpui::FontId;
use std::sync::Arc;
use swash::scale::image::{Content, Image};
use swash::scale::Source;
use swash::zeno::Placement;

/// One FreeType face per font, over the font's bytes.
struct Face {
    face: ft::FT_Face,
    /// The bytes the face reads; FreeType keeps a pointer into them.
    _font: Arc<cosmic_text::Font>,
    /// The size last set, in 26.6.
    char_size: ft::FT_F26Dot6,
}

pub(crate) struct FreeTypeRaster {
    library: ft::FT_Library,
    faces: HashMap<FontId, Face>,
}

// The library and faces are only touched under the text system's lock.
unsafe impl Send for FreeTypeRaster {}
unsafe impl Sync for FreeTypeRaster {}

impl FreeTypeRaster {
    /// `None` when FreeType does not start; the caller keeps swash then.
    pub(crate) fn new() -> Option<Self> {
        let mut library = std::ptr::null_mut();
        // SAFETY: FT_Init_FreeType writes a fresh library handle.
        let error = unsafe { ft::FT_Init_FreeType(&mut library) };
        (error == 0 && !library.is_null()).then(|| Self {
            library,
            faces: HashMap::default(),
        })
    }

    fn face(&mut self, font_id: FontId, font: &Arc<cosmic_text::Font>, index: u32) -> Result<&mut Face> {
        if !self.faces.contains_key(&font_id) {
            let data = font.data();
            let mut face = std::ptr::null_mut();
            // SAFETY: the bytes outlive the face (`Face::_font` holds them).
            let error = unsafe {
                ft::FT_New_Memory_Face(
                    self.library,
                    data.as_ptr(),
                    data.len() as ft::FT_Long,
                    index as ft::FT_Long,
                    &mut face,
                )
            };
            if error != 0 || face.is_null() {
                return Err(anyhow!("FT_New_Memory_Face failed: {error}"));
            }
            self.faces.insert(
                font_id,
                Face {
                    face,
                    _font: font.clone(),
                    char_size: 0,
                },
            );
        }
        self.faces.get_mut(&font_id).context("face just inserted")
    }

    /// The glyph at `pixel_size` device pixels, its origin moved right by
    /// `offset_x` and down by `offset_y` device pixels.
    pub(crate) fn render(
        &mut self,
        font_id: FontId,
        font: &Arc<cosmic_text::Font>,
        index: u32,
        glyph_id: u32,
        pixel_size: f32,
        offset_x: f32,
        offset_y: f32,
    ) -> Result<Image> {
        let face = self.face(font_id, font, index)?;
        let char_size = (pixel_size * 64.).round() as ft::FT_F26Dot6;
        // SAFETY: `face.face` is a live face; FreeType copies the vector.
        unsafe {
            if face.char_size != char_size {
                let error = ft::FT_Set_Char_Size(face.face, 0, char_size, 72, 72);
                if error != 0 {
                    return Err(anyhow!("FT_Set_Char_Size failed: {error}"));
                }
                face.char_size = char_size;
            }
            let mut delta = ft::FT_Vector {
                x: (offset_x * 64.).round() as ft::FT_Pos,
                y: -(offset_y * 64.).round() as ft::FT_Pos,
            };
            ft::FT_Set_Transform(face.face, std::ptr::null_mut(), &mut delta);
            let error = ft::FT_Load_Glyph(
                face.face,
                glyph_id,
                ft::FT_LOAD_TARGET_LIGHT | ft::FT_LOAD_NO_BITMAP,
            );
            if error != 0 {
                return Err(anyhow!("FT_Load_Glyph failed: {error}"));
            }
            let slot = (*face.face).glyph;
            let error = ft::FT_Render_Glyph(slot, ft::FT_RENDER_MODE_LIGHT);
            if error != 0 {
                return Err(anyhow!("FT_Render_Glyph failed: {error}"));
            }
            let bitmap = &(*slot).bitmap;
            let width = bitmap.width as usize;
            let rows = bitmap.rows as usize;
            let pitch = bitmap.pitch as isize;
            let mut data = Vec::with_capacity(width * rows);
            for row in 0..rows {
                let start = bitmap.buffer.offset(row as isize * pitch);
                data.extend_from_slice(std::slice::from_raw_parts(start, width));
            }
            Ok(Image {
                source: Source::Outline,
                content: Content::Mask,
                placement: Placement {
                    left: (*slot).bitmap_left,
                    top: (*slot).bitmap_top,
                    width: width as u32,
                    height: rows as u32,
                },
                data,
            })
        }
    }
}

impl Drop for FreeTypeRaster {
    fn drop(&mut self) {
        // SAFETY: each face and then the library are released once.
        unsafe {
            for (_, face) in self.faces.drain() {
                ft::FT_Done_Face(face.face);
            }
            ft::FT_Done_FreeType(self.library);
        }
    }
}
