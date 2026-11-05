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

/// `text` centred in `room` px the way tdesktop centres a label
/// (`al_top`: `x += (_wLeft / 2).toInt()`, the free width halved and
/// rounded, as `QFixed::toInt` rounds), or `wrapped` when it does not fit on
/// one line.
pub(crate) fn centered_label(
    text: impl Into<SharedString>,
    room: f32,
    wrapped: impl IntoElement,
) -> CenteredLabel {
    CenteredLabel {
        text: text.into(),
        room,
        wrapped: Some(wrapped.into_any_element()),
        line: None,
    }
}

pub(crate) struct CenteredLabel {
    text: SharedString,
    room: f32,
    /// The multi-line layout, used when the text is wider than `room`.
    wrapped: Option<AnyElement>,
    /// The shaped line, its line height and its left offset, when it fits.
    line: Option<(ShapedLine, Pixels, Pixels)>,
}

impl IntoElement for CenteredLabel {
    type Element = Self;

    fn into_element(self) -> Self::Element {
        self
    }
}

impl Element for CenteredLabel {
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
        let width = f32::from(line.width);
        if width > self.room {
            self.line = None;
            if let Some(wrapped) = self.wrapped.as_mut() {
                return (wrapped.request_layout(window, cx), ());
            }
        }
        let left = px(((self.room - width) / 2.).round());
        self.line = Some((line, line_height, left));
        let layout = Style {
            size: size(px(self.room).into(), line_height.into()),
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
        window: &mut Window,
        cx: &mut App,
    ) -> Self::PrepaintState {
        if self.line.is_none()
            && let Some(wrapped) = self.wrapped.as_mut()
        {
            wrapped.prepaint(window, cx);
        }
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
        match &self.line {
            Some((line, line_height, left)) => {
                let origin = point(bounds.origin.x + *left, bounds.origin.y);
                let _ = line.paint(origin, *line_height, TextAlign::Left, None, window, cx);
            }
            None => {
                if let Some(wrapped) = self.wrapped.as_mut() {
                    wrapped.paint(window, cx);
                }
            }
        }
    }
}
