//! One line of plain text as wide as Telegram Desktop counts it.
//!
//! `style::font::width` ceils `QFontMetricsF::horizontalAdvance`
//! (`lib_ui/ui/style/style_core_font.h`), and tdesktop places right-aligned
//! labels (chat-list dates, message times) at `right - width`. A GPUI text
//! node is as wide as its fractional advance, so the same label sits up to a
//! pixel further right. This element takes the ceiled width and draws the
//! line from its left edge, as tdesktop does.
#![deny(clippy::indexing_slicing)]

use gpui_kit::*;

/// `text` on one line, `ceil(advance)` wide, in the inherited text style.
pub(crate) fn label(text: impl Into<SharedString>) -> TdLabel {
    TdLabel {
        text: text.into(),
        line: None,
    }
}

pub(crate) struct TdLabel {
    text: SharedString,
    line: Option<(ShapedLine, Pixels)>,
}

impl IntoElement for TdLabel {
    type Element = Self;

    fn into_element(self) -> Self::Element {
        self
    }
}

impl Element for TdLabel {
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
        let style = window.text_style();
        let font_size = style.font_size.to_pixels(window.rem_size());
        let line_height = window.pixel_snap(
            style
                .line_height
                .to_pixels(font_size.into(), window.rem_size()),
        );
        let run = style.to_run(self.text.len());
        let line = window
            .text_system()
            .shape_line(self.text.clone(), font_size, &[run], None);
        let width = px(f32::from(line.width).ceil());
        self.line = Some((line, line_height));
        let layout = Style {
            size: size(width.into(), line_height.into()),
            flex_shrink: 0.,
            ..Style::default()
        };
        (window.request_layout(layout, [], cx), ())
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
        cx: &mut App,
    ) {
        if let Some((line, line_height)) = &self.line {
            let _ = line.paint(
                bounds.origin,
                *line_height,
                TextAlign::Left,
                None,
                window,
                cx,
            );
        }
    }
}
