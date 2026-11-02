//! The rich editor's blocks and preview, kept across revisions of the text.
//! An edit touches one block, and the blocks around it read the same as
//! before, so only the text around the edit is parsed again and only the
//! blocks that changed are prepared for the preview.
use super::*;
use std::ops::Range;
use std::rc::Rc;
use std::sync::Arc;

/// Text up to and including a blank line outside a code fence, and its
/// blocks. No block crosses such a line, so each chunk parses alone. An
/// unchanged chunk keeps its `Arc`s, which also makes comparing it cheap.
struct Chunk {
    len: usize,
    blocks: Vec<Arc<RichBlock>>,
}

/// [`markup_to_blocks`] of the latest text, parsing only what an edit touched.
#[derive(Default)]
pub struct MarkupBlocks {
    text: String,
    chunks: Vec<Chunk>,
}

fn chunks_of(text: &str) -> Vec<Chunk> {
    let mut chunks = Vec::new();
    let (mut start, mut at) = (0, 0);
    let fences = super::markup::code_fences(text.split_inclusive('\n'));
    for (index, line) in text.split_inclusive('\n').enumerate() {
        at += line.len();
        let in_code = fences.iter().any(|code| code.contains(&index));
        if (line.trim().is_empty() && !in_code) || at == text.len() {
            let blocks = markup_to_blocks(text.get(start..at).unwrap_or_default());
            chunks.push(Chunk {
                len: at - start,
                blocks: blocks.into_iter().map(Arc::new).collect(),
            });
            start = at;
        }
    }
    chunks
}

/// Bytes compared at once before looking for the one that differs: a draft
/// can be megabytes, and it is walked on every keystroke.
const COMPARE_STEP: usize = 64;
const FENCE_BYTES: &[u8] = b"```";

/// Whether the whole lines around `range` hold a fence.
fn lines_hold_fence(text: &[u8], range: Range<usize>) -> bool {
    let start = text.get(..range.start).unwrap_or_default();
    let start = start.iter().rposition(|byte| *byte == b'\n').unwrap_or(0);
    let end = text.get(range.end..).unwrap_or_default();
    let end = end
        .iter()
        .position(|byte| *byte == b'\n')
        .map_or(text.len(), |at| range.end + at);
    let lines = text.get(start..end).unwrap_or_default();
    lines
        .windows(FENCE_BYTES.len())
        .any(|bytes| bytes == FENCE_BYTES)
}

fn shared_start(a: &[u8], b: &[u8]) -> usize {
    let same_steps = a.chunks(COMPARE_STEP).zip(b.chunks(COMPARE_STEP));
    let whole = same_steps.take_while(|(a, b)| a == b).count() * COMPARE_STEP;
    let whole = whole.min(a.len()).min(b.len());
    let rest = a.get(whole..).unwrap_or_default().iter();
    let rest = rest.zip(b.get(whole..).unwrap_or_default());
    whole + rest.take_while(|(a, b)| a == b).count()
}

fn shared_end(a: &[u8], b: &[u8]) -> usize {
    let same_steps = a.rchunks(COMPARE_STEP).zip(b.rchunks(COMPARE_STEP));
    let whole = same_steps.take_while(|(a, b)| a == b).count() * COMPARE_STEP;
    let whole = whole.min(a.len()).min(b.len());
    let (a, b) = (
        a.get(..a.len() - whole).unwrap_or_default(),
        b.get(..b.len() - whole).unwrap_or_default(),
    );
    let rest = a.iter().rev().zip(b.iter().rev());
    whole + rest.take_while(|(a, b)| a == b).count()
}

impl MarkupBlocks {
    /// The blocks of `text`, the same as [`markup_to_blocks`] gives.
    pub fn parse(&mut self, text: &str) -> Vec<Arc<RichBlock>> {
        if self.text != text {
            self.reparse(text);
        }
        let blocks = self.chunks.iter().flat_map(|chunk| &chunk.blocks);
        blocks.cloned().collect()
    }

    fn reparse(&mut self, text: &str) {
        let (old, new) = (self.text.as_bytes(), text.as_bytes());
        let start = shared_start(old, new);
        let shared_end = shared_end(
            old.get(start..).unwrap_or_default(),
            new.get(start..).unwrap_or_default(),
        );
        let touched_end = old.len() - shared_end;
        // Fences pair up across the whole text, so one that moved can change
        // every chunk after it.
        if lines_hold_fence(old, start..touched_end)
            || lines_hold_fence(new, start..new.len() - shared_end)
        {
            self.chunks = chunks_of(text);
            text.clone_into(&mut self.text);
            return;
        }

        // The chunk the edit starts in and the one it ends in, and one more
        // on each side: an edit at a chunk's edge can join it to its
        // neighbour, and past that neighbour lies a blank line it left alone.
        let (mut first, mut last, mut at) = (self.chunks.len(), 0, 0);
        let mut starts = Vec::with_capacity(self.chunks.len());
        for (index, chunk) in self.chunks.iter().enumerate() {
            starts.push(at);
            let end = at + chunk.len;
            if first == self.chunks.len() && start <= end {
                first = index;
            }
            if at <= touched_end {
                last = index;
            }
            at = end;
        }
        let first = first.saturating_sub(1);
        let last = (last + 2).min(self.chunks.len());
        let window_start = starts.get(first).copied().unwrap_or(0);
        let window_end = match starts.get(last) {
            Some(end) => new.len() - (old.len() - end),
            None => new.len(),
        };
        let parsed = chunks_of(text.get(window_start..window_end).unwrap_or_default());
        self.chunks.splice(first..last, parsed);
        text.clone_into(&mut self.text);
    }
}

/// `shown` is `source` through [`preview_block`], index for index.
#[derive(Default)]
pub struct PreviewBlocks {
    source: Vec<Arc<RichBlock>>,
    shown: Vec<Rc<RichBlock>>,
}

impl PreviewBlocks {
    pub fn shown(&self) -> &[Rc<RichBlock>] {
        &self.shown
    }

    /// Move to the blocks of a new text. Returns the range of old blocks that
    /// were replaced and how many new ones took their place.
    pub fn update(&mut self, source: Vec<Arc<RichBlock>>) -> (Range<usize>, usize) {
        let start = self
            .source
            .iter()
            .zip(&source)
            .take_while(|(old, new)| old == new)
            .count();
        let old_rest = self.source.get(start..).unwrap_or_default();
        let new_rest = source.get(start..).unwrap_or_default();
        let shared_end = old_rest
            .iter()
            .rev()
            .zip(new_rest.iter().rev())
            .take_while(|(old, new)| old == new)
            .count();
        let replaced = start..start + old_rest.len() - shared_end;
        let added = new_rest.len() - shared_end;
        let parsed: Vec<Rc<RichBlock>> = new_rest
            .iter()
            .take(added)
            .map(|block| Rc::new(preview_block(RichBlock::clone(block))))
            .collect();
        self.shown.splice(replaced.clone(), parsed);
        self.source = source;
        (replaced, added)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const LINES: [&str; 8] = [
        "# **Title**",
        "- first ~~point~~",
        "1. **second**",
        "plain _text_",
        "",
        "---",
        ">> More",
        "- [x] done",
    ];

    fn update(preview: &mut PreviewBlocks, text: &str) -> (Range<usize>, usize) {
        preview.update(MarkupBlocks::default().parse(text))
    }

    #[test]
    fn every_edit_shows_what_a_fresh_parse_shows() {
        let mut preview = PreviewBlocks::default();
        let mut parsed = MarkupBlocks::default();
        let mut lines: Vec<&str> = Vec::new();
        let mut seed = 7usize;
        for step in 0..4_000 {
            seed = seed.wrapping_mul(31).wrapping_add(step) % 1_000_003;
            let at = seed % (lines.len() + 1);
            let line = LINES.get(seed % LINES.len()).copied().unwrap_or_default();
            match seed % 3 {
                0 if at < lines.len() => {
                    lines.remove(at);
                }
                1 if at < lines.len() => {
                    if let Some(slot) = lines.get_mut(at) {
                        *slot = line;
                    }
                }
                _ => lines.insert(at, line),
            }
            let text = lines.join("\n");
            let before = preview.shown().len();
            let (replaced, added) = preview.update(parsed.parse(&text));
            let shown: Vec<RichBlock> = preview.shown().iter().map(|b| (**b).clone()).collect();
            assert_eq!(shown, preview_blocks(&text), "{text:?}");
            assert_eq!(before - replaced.len() + added, shown.len());
        }
    }

    #[test]
    fn an_edit_parses_only_the_block_it_touches() {
        let mut preview = PreviewBlocks::default();
        update(&mut preview, "# a\n\n- b\n\nc\n\n---");
        let before = preview.shown().to_vec();
        let (replaced, added) = update(&mut preview, "# a\n\n- bb\n\nc\n\n---");
        assert_eq!((replaced, added), (1..2, 1));
        for (index, block) in before.iter().enumerate() {
            let kept = preview
                .shown()
                .get(index)
                .is_some_and(|b| Rc::ptr_eq(b, block));
            assert_eq!(kept, index != 1, "block {index}");
        }
    }

    fn plain(blocks: &[Arc<RichBlock>]) -> Vec<RichBlock> {
        blocks.iter().map(|block| (**block).clone()).collect()
    }

    #[test]
    fn any_edit_parses_as_the_whole_text_does() {
        const PIECES: [&str; 12] = [
            "\n", "\n\n", " \n", "# a", "- b", "1. c", ">> d", "---", "text", "**x**", "é", "\r\n",
        ];
        let mut parsed = MarkupBlocks::default();
        let mut text = String::new();
        let mut seed = 11usize;
        for step in 0..20_000 {
            seed = seed.wrapping_mul(1_000_003).wrapping_add(step);
            let boundaries: Vec<usize> = text.char_indices().map(|(at, _)| at).collect();
            let pick = |n: usize| {
                boundaries
                    .get(seed.wrapping_add(n) % (boundaries.len() + 1))
                    .copied()
                    .unwrap_or(text.len())
            };
            let (a, b) = (pick(0), pick(seed >> 7));
            let (from, to) = (a.min(b), a.max(b).min(a.min(b) + 12));
            let to = (to..=text.len())
                .find(|at| text.is_char_boundary(*at))
                .unwrap_or(text.len());
            let piece = PIECES
                .get((seed >> 3) % PIECES.len())
                .copied()
                .unwrap_or("");
            match seed % 4 {
                0 => text.replace_range(from..to, ""),
                1 => text.replace_range(from..to, piece),
                _ => text.insert_str(from, piece),
            }
            if text.len() > 300 {
                text.clear();
            }
            assert_eq!(
                plain(&parsed.parse(&text)),
                markup_to_blocks(&text),
                "{text:?}"
            );
        }
    }

    #[test]
    fn an_edit_in_a_long_text_leaves_far_blocks_alone() {
        let text: String = (0..100).map(|i| format!("# h{i}\n\n")).collect();
        let mut parsed = MarkupBlocks::default();
        let before = parsed.parse(&text);
        let edited = text.replacen("# h50", "# h50x", 1);
        let after = parsed.parse(&edited);
        assert_eq!(before.len(), after.len());
        let kept = before.iter().zip(&after);
        let same = kept.filter(|(a, b)| Arc::ptr_eq(a, b)).count();
        assert!(same >= 96, "{same} of 100 blocks kept");
    }
}
