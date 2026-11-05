//! Unwrapped text widths for the bubble width policy (`bubble_width`):
//! the longest line of a caption or a code block as Telegram Desktop's
//! `Ui::Text::String::maxWidth` / `countMaxMonospaceWidth` count it, with
//! each entity run measured in the face it renders in (bold, italic,
//! monospace) and a custom emoji as the em-wide placeholder it takes.

use super::*;
use quill::text::{TextRun, styled_runs};

thread_local! {
    /// Glyph advances by font, size and character, as GPUI's own line
    /// wrapper caches them; cleared when it grows past
    /// [`CHAR_WIDTH_CACHE_CAP`] entries so a long session stays bounded.
    static CHAR_WIDTHS: std::cell::RefCell<HashMap<(FontId, u32, char), Pixels>> =
        std::cell::RefCell::new(HashMap::new());
}

const CHAR_WIDTH_CACHE_CAP: usize = 16 * 1024;

thread_local! {
    /// Shaped word widths by font, size and word, bounded like
    /// [`CHAR_WIDTHS`].
    static WORD_WIDTHS: std::cell::RefCell<HashMap<(FontId, u32, String), f32>> =
        std::cell::RefCell::new(HashMap::new());
}

/// Horizontal room a `pre` block takes around its code
/// (`message_text::pre_block`: `px_2` on both sides and the copy
/// button's `pr_6`).
pub(in crate::ui) const PRE_BLOCK_CHROME: i32 = 8 + 8 + 24;

/// Horizontal room a block quote takes around its text
/// (`message_text::quote_block`: `px_2` and the 2 px accent bar).
pub(in crate::ui) const QUOTE_BLOCK_CHROME: i32 = 8 + 8 + 2;

/// The width `c` takes in `font_id` at `font_size`.
fn char_width(text_system: &TextSystem, font_id: FontId, font_size: Pixels, c: char) -> Pixels {
    let key = (font_id, f32::from(font_size).to_bits(), c);
    CHAR_WIDTHS.with(|cache| {
        if let Some(width) = cache.borrow().get(&key) {
            return *width;
        }
        let width = text_system.layout_width(font_id, font_size, c);
        let mut cache = cache.borrow_mut();
        if cache.len() >= CHAR_WIDTH_CACHE_CAP {
            cache.clear();
        }
        cache.insert(key, width);
        width
    })
}

/// The face a run renders in.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub(in crate::ui) struct Face {
    pub(in crate::ui) bold: bool,
    /// Semibold, the weight of names and file titles (`semiboldFont`).
    pub(in crate::ui) semibold: bool,
    pub(in crate::ui) italic: bool,
    pub(in crate::ui) mono: bool,
}

impl Face {
    fn of(run: &TextRun) -> Self {
        Self {
            bold: run.style.bold,
            italic: run.style.italic,
            mono: run.style.code || run.style.pre,
            ..Self::default()
        }
    }

    /// The face of plain text at `weight`.
    fn weighted(weight: FontWeight) -> Self {
        Self {
            bold: weight >= FontWeight::BOLD,
            semibold: weight >= FontWeight::SEMIBOLD && weight < FontWeight::BOLD,
            ..Self::default()
        }
    }
}

/// Which block a run belongs to; `rich_text_reserving` breaks the flow at
/// every change and gives each `pre` run a block of its own.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Block {
    Inline,
    Pre,
    Quote,
}

impl Block {
    fn of(run: &TextRun) -> Self {
        if run.style.pre {
            Self::Pre
        } else if run.style.quote {
            Self::Quote
        } else {
            Self::Inline
        }
    }

    const fn chrome(self) -> f32 {
        match self {
            Self::Inline => 0.,
            Self::Pre => PRE_BLOCK_CHROME as f32,
            Self::Quote => QUOTE_BLOCK_CHROME as f32,
        }
    }
}

/// The rendered width of every line of `runs`, each block's chrome
/// included: `advance` gives a character's width in a face, and a custom
/// emoji takes `emoji` (the placeholder glyph, whatever its text). A run
/// ending in a newline closes its line; a pre run's trailing newline is
/// dropped, as `pre_block` drops it. Returns the lines and whether the
/// text ends in a block (a pre or quote), after which the inline time
/// takes a line of its own.
pub(in crate::ui) fn line_widths(
    runs: &[TextRun],
    mut advance: impl FnMut(Face, char) -> f32,
    emoji: f32,
) -> (Vec<f32>, bool) {
    let mut lines = Vec::new();
    let mut current = 0f32;
    let mut open: Option<Block> = None;
    for run in runs {
        let block = Block::of(run);
        // A new block (and every pre run) starts on a fresh line.
        if let Some(previous) = open
            && (previous != block || block == Block::Pre)
        {
            lines.push(current + previous.chrome());
            current = 0.;
        }
        open = Some(block);
        if run.custom_emoji_id.is_some() {
            current += emoji;
            continue;
        }
        let face = Face::of(run);
        let text = if block == Block::Pre {
            run.text.trim_end_matches('\n')
        } else {
            run.text.as_str()
        };
        for c in text.chars() {
            if c == '\n' {
                lines.push(current + block.chrome());
                current = 0.;
            } else {
                current += advance(face, c);
            }
        }
    }
    if let Some(block) = open {
        lines.push(current + block.chrome());
    }
    let ends_in_block = open.is_some_and(|block| block != Block::Inline);
    (lines, ends_in_block)
}

/// `Ui::Text::String::maxWidth` for laid-out `lines`, with
/// `last_line_extra` (the time's skip block) on the last line, or on a
/// line of its own after a closing block.
pub(in crate::ui) fn longest_with_extra(lines: &[f32], ends_in_block: bool, extra: i32) -> i32 {
    let widest = |lines: &[f32]| {
        lines
            .iter()
            .map(|width| width.ceil() as i32)
            .max()
            .unwrap_or(0)
    };
    match lines.split_last() {
        None => 0,
        Some(_) if extra == 0 => widest(lines),
        Some(_) if ends_in_block => widest(lines).max(extra),
        Some((last, rest)) => widest(rest).max(last.ceil() as i32 + extra),
    }
}

/// `String::countMaxMonospaceWidth`: the widest `pre` block of `runs`,
/// its chrome included, and its language label (at 0.8 of the size, as
/// `pre_block` draws it) when that is wider; 0 without a block.
pub(in crate::ui) fn widest_pre_block(
    runs: &[TextRun],
    mut advance: impl FnMut(Face, char) -> f32,
    label_scale: f32,
) -> i32 {
    let mut widest = 0f32;
    for run in runs.iter().filter(|run| run.style.pre) {
        let face = Face::of(run);
        let code = run.text.trim_end_matches('\n');
        for line in code.split('\n') {
            let width: f32 = line.chars().map(|c| advance(face, c)).sum();
            widest = widest.max(width);
        }
        if let Some(language) = run.style.language.as_deref() {
            let label: f32 = language.chars().map(|c| advance(face, c)).sum();
            widest = widest.max(label * label_scale);
        }
        widest = widest.max(0.);
    }
    if runs.iter().any(|run| run.style.pre) {
        (widest + PRE_BLOCK_CHROME as f32).ceil() as i32
    } else {
        0
    }
}

/// Measures characters in the theme's font (or the monospace one) at
/// `font_size`, resolving each face once.
struct Measurer<'a> {
    text_system: &'a TextSystem,
    family: SharedString,
    font_size: Pixels,
    fonts: HashMap<Face, FontId>,
}

impl<'a> Measurer<'a> {
    fn new(cx: &'a App, font_size: Pixels) -> Self {
        Self {
            text_system: cx.text_system(),
            family: cx.theme().font_family.clone(),
            font_size,
            fonts: HashMap::new(),
        }
    }

    fn advance(&mut self, face: Face, c: char) -> f32 {
        let font_id = *self.fonts.entry(face).or_insert_with(|| {
            let family = if face.mono {
                SharedString::from(crate::ui::message_text::MONO_FONT)
            } else {
                self.family.clone()
            };
            let mut font = font(family);
            if face.bold {
                font.weight = FontWeight::BOLD;
            } else if face.semibold {
                font.weight = FontWeight::SEMIBOLD;
            }
            if face.italic {
                font.style = FontStyle::Italic;
            }
            self.text_system.resolve_font(&font)
        });
        f32::from(char_width(self.text_system, font_id, self.font_size, c))
    }

    /// The shaped width of one word: Telegram Desktop measures a word as
    /// one shaped block, so pairs inside it kern (`Ui::Text::String`
    /// blocks), which a sum of character advances misses.
    fn word(&mut self, face: Face, word: &str) -> f32 {
        if word.is_empty() {
            return 0.;
        }
        let _ = self.advance(face, ' ');
        let Some(&font_id) = self.fonts.get(&face) else {
            return 0.;
        };
        let key = (
            font_id,
            f32::from(self.font_size).to_bits(),
            word.to_string(),
        );
        WORD_WIDTHS.with(|cache| {
            if let Some(width) = cache.borrow().get(&key) {
                return *width;
            }
            let width = f32::from(self.text_system.shaped_width(font_id, self.font_size, word));
            let mut cache = cache.borrow_mut();
            if cache.len() >= CHAR_WIDTH_CACHE_CAP {
                cache.clear();
            }
            cache.insert(key, width);
            width
        })
    }
}

/// The unwrapped width of the longest line of `text` with its
/// `entities` at `font_size`, `last_line_extra` added to the last line
/// (the skip block the inline time takes there).
pub(in crate::ui) fn styled_longest_line(
    cx: &App,
    text: &str,
    entities: &[TextEntity],
    font_size: Pixels,
    last_line_extra: i32,
) -> i32 {
    if text.is_empty() {
        return 0;
    }
    let runs = styled_runs(text, entities);
    let mut measurer = Measurer::new(cx, font_size);
    // A custom emoji stands in as one em space (`EMOJI_PLACEHOLDER`).
    let emoji = measurer.advance(Face::default(), '\u{2003}');
    let (lines, ends_in_block) = line_widths(&runs, |face, c| measurer.advance(face, c), emoji);
    longest_with_extra(&lines, ends_in_block, last_line_extra)
}

/// [`widest_pre_block`] for `text` with its `entities` at `font_size`.
pub(in crate::ui) fn monospace_width(
    cx: &App,
    text: &str,
    entities: &[TextEntity],
    font_size: Pixels,
) -> i32 {
    use quill::text::TextEntityKind;
    if !entities.iter().any(|entity| {
        matches!(
            entity.kind,
            TextEntityKind::Pre | TextEntityKind::PreCode { .. }
        )
    }) {
        return 0;
    }
    let runs = styled_runs(text, entities);
    let mut measurer = Measurer::new(cx, font_size);
    widest_pre_block(&runs, |face, c| measurer.advance(face, c), 0.8)
}

/// The unwrapped width of the longest line of plain `text` in the
/// theme's font at `font_size` and `weight`, `last_line_extra` added to
/// the last line.
pub(in crate::ui) fn longest_line_width(
    cx: &App,
    text: &str,
    font_size: Pixels,
    weight: FontWeight,
    last_line_extra: i32,
) -> i32 {
    if text.is_empty() {
        return 0;
    }
    let mut measurer = Measurer::new(cx, font_size);
    let face = Face::weighted(weight);
    let lines: Vec<f32> = text
        .split('\n')
        .map(|line| measurer.word(face, line))
        .collect();
    longest_with_extra(&lines, false, last_line_extra)
}

/// tdesktop `textRealWidth`: the widest line of `text` once it wraps at
/// words into `available`, with `last_extra` (the time's skip block) after
/// its last line when it still fits there. A bubble of text is as wide as
/// this plus its padding (`Message::countGeometry`), not as wide as it may
/// get.
pub(in crate::ui) fn wrapped_width(
    cx: &App,
    text: &str,
    font_size: Pixels,
    available: f32,
    last_extra: f32,
) -> f32 {
    let mut measurer = Measurer::new(cx, font_size);
    let face = Face::default();
    let mut widest = 0f32;
    let mut last = 0f32;
    for paragraph in text.split('\n') {
        let mut line = 0f32;
        let mut pending_space = 0f32;
        for word in paragraph.split(' ') {
            let space = measurer.advance(face, ' ');
            let width = measurer.word(face, word);
            if line > 0. && line + pending_space + width > available {
                widest = widest.max(line);
                line = 0.;
                pending_space = 0.;
            }
            if line == 0. && width > available {
                // A word longer than the line breaks anywhere.
                let mut part = 0f32;
                for c in word.chars() {
                    let advance = measurer.advance(face, c);
                    if part + advance > available && part > 0. {
                        widest = widest.max(part);
                        part = 0.;
                    }
                    part += advance;
                }
                line = part;
            } else {
                line += pending_space + width;
            }
            pending_space = space;
        }
        widest = widest.max(line);
        last = line;
    }
    if last_extra > 0. && last + last_extra <= available {
        widest = widest.max(last + last_extra);
    }
    widest.min(available)
}

/// tdesktop's middle elision (`elisionMiddle` in `text_renderer.cpp`): the
/// longest start and end of `text` that each fit half of `width` less the
/// "..." between them. `None` when the whole text fits.
pub(in crate::ui) fn middle_elision(
    cx: &App,
    text: &str,
    font_size: Pixels,
    weight: FontWeight,
    width: f32,
) -> Option<MiddleElided> {
    let mut measurer = Measurer::new(cx, font_size);
    let face = Face::weighted(weight);
    if measurer.word(face, text) <= width {
        return None;
    }
    // `elidew` is an integer font width; each side takes shaped letters
    // while they fit half of what the ellipsis leaves (`Renderer::drawLine`).
    let ellipsis = measurer.word(face, "...").ceil();
    let half = (width - ellipsis) / 2.;
    let chars: Vec<char> = text.chars().collect();
    let space_width = measurer.advance(face, ' ');
    let mut fit = |from_end: bool, budget: f32| {
        let (mut count, mut used) = (0, 0.);
        for n in 1..=chars.len() {
            let part: String = if from_end {
                chars.iter().skip(chars.len() - n).collect()
            } else {
                chars.iter().take(n).collect()
            };
            let width = measurer.word(face, &part);
            if width > budget {
                break;
            }
            (count, used) = (n, width);
        }
        (count, used)
    };
    let (left, left_width) = fit(false, half);
    // A cut at a space gives the right side the space's width.
    let space = if chars.get(left) == Some(&' ') {
        space_width
    } else {
        0.
    };
    let (right, right_width) = fit(true, half + space);
    Some(MiddleElided {
        start: chars.iter().take(left).collect(),
        end: chars
            .iter()
            .skip(chars.len().saturating_sub(right))
            .collect(),
        start_width: left_width,
        end_width: right_width,
        ellipsis_width: ellipsis,
    })
}

/// A text cut in the middle by [`middle_elision`].
pub(in crate::ui) struct MiddleElided {
    pub start: String,
    pub end: String,
    pub start_width: f32,
    pub end_width: f32,
    pub ellipsis_width: f32,
}

#[cfg(test)]
mod tests;
