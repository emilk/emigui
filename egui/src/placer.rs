use crate::*;

pub(crate) struct Placer {
    /// If set this will take precedence over [`layout`].
    grid: Option<grid::GridLayout>,
    layout: Layout,
    pub(crate) region: Region,
}

impl Placer {
    pub(crate) fn new(max_rect: Rect, layout: Layout) -> Self {
        let region = layout.region_from_max_rect(max_rect);
        Self {
            grid: None,
            layout,
            region,
        }
    }

    #[inline(always)]
    pub(crate) fn set_grid(&mut self, grid: grid::GridLayout) {
        self.grid = Some(grid);
    }

    pub(crate) fn save_grid(&mut self) {
        if let Some(grid) = &mut self.grid {
            grid.save();
        }
    }

    #[inline(always)]
    pub(crate) fn grid(&self) -> Option<&grid::GridLayout> {
        self.grid.as_ref()
    }

    #[inline(always)]
    pub(crate) fn is_grid(&self) -> bool {
        self.grid.is_some()
    }

    #[inline(always)]
    pub(crate) fn layout(&self) -> &Layout {
        &self.layout
    }

    #[inline(always)]
    pub(crate) fn prefer_right_to_left(&self) -> bool {
        self.layout.prefer_right_to_left()
    }

    #[inline(always)]
    pub(crate) fn min_rect(&self) -> Rect {
        self.region.min_rect
    }

    #[inline(always)]
    pub(crate) fn max_rect(&self) -> Rect {
        self.region.max_rect
    }

    #[inline(always)]
    pub(crate) fn max_rect_finite(&self) -> Rect {
        self.region.max_rect_finite()
    }

    #[inline(always)]
    pub(crate) fn force_set_min_rect(&mut self, min_rect: Rect) {
        self.region.min_rect = min_rect;
    }

    #[inline(always)]
    pub(crate) fn cursor(&self) -> Rect {
        self.region.cursor
    }
}

impl Placer {
    pub(crate) fn align_size_within_rect(&self, size: Vec2, outer: Rect) -> Rect {
        if let Some(grid) = &self.grid {
            grid.align_size_within_rect(size, outer)
        } else {
            self.layout.align_size_within_rect(size, outer)
        }
    }

    pub(crate) fn available_rect_before_wrap(&self) -> Rect {
        if let Some(grid) = &self.grid {
            grid.available_rect(&self.region)
        } else {
            self.layout.available_rect_before_wrap(&self.region)
        }
    }

    pub(crate) fn available_rect_before_wrap_finite(&self) -> Rect {
        if let Some(grid) = &self.grid {
            grid.available_rect_finite(&self.region)
        } else {
            self.layout.available_rect_before_wrap_finite(&self.region)
        }
    }

    /// Amount of space available for a widget.
    /// For wrapping layouts, this is the maximum (after wrap).
    pub(crate) fn available_size(&self) -> Vec2 {
        if let Some(grid) = &self.grid {
            grid.available_rect(&self.region).size()
        } else {
            self.layout.available_size(&self.region)
        }
    }

    /// Returns where to put the next widget that is of the given size.
    /// The returned `frame_rect` will always be justified along the cross axis.
    /// This is what you then pass to `advance_after_rects`.
    /// Use `justify_and_align` to get the inner `widget_rect`.
    pub(crate) fn next_space(&self, child_size: Vec2, item_spacing: Vec2) -> Rect {
        if let Some(grid) = &self.grid {
            grid.next_cell(self.region.cursor, child_size)
        } else {
            self.layout
                .next_frame(&self.region, child_size, item_spacing)
        }
    }

    /// Where do we expect a zero-sized widget to be placed?
    pub(crate) fn next_widget_position(&self) -> Pos2 {
        if let Some(grid) = &self.grid {
            grid.next_cell(self.region.cursor, Vec2::ZERO).center()
        } else {
            self.layout.next_widget_position(&self.region)
        }
    }

    /// Apply justify or alignment after calling `next_space`.
    pub(crate) fn justify_and_align(&self, rect: Rect, child_size: Vec2) -> Rect {
        crate::egui_assert!(!rect.any_nan());
        crate::egui_assert!(!child_size.any_nan());

        if let Some(grid) = &self.grid {
            grid.justify_and_align(rect, child_size)
        } else {
            self.layout.justify_and_align(rect, child_size)
        }
    }

    /// Advance the cursor by this many points.
    /// [`Self::min_rect`] will expand to contain the cursor.
    pub(crate) fn advance_cursor(&mut self, amount: f32) {
        crate::egui_assert!(
            self.grid.is_none(),
            "You cannot advance the cursor when in a grid layout"
        );
        self.layout.advance_cursor(&mut self.region, amount);
    }

    /// Advance cursor after a widget was added to a specific rectangle
    /// and expand the region `min_rect`.
    ///
    /// * `frame_rect`: the frame inside which a widget was e.g. centered
    /// * `widget_rect`: the actual rect used by the widget
    pub(crate) fn advance_after_rects(
        &mut self,
        frame_rect: Rect,
        widget_rect: Rect,
        item_spacing: Vec2,
    ) {
        if let Some(grid) = &mut self.grid {
            grid.advance(&mut self.region.cursor, frame_rect, widget_rect)
        } else {
            self.layout.advance_after_rects(
                &mut self.region.cursor,
                frame_rect,
                widget_rect,
                item_spacing,
            )
        }

        self.region.expand_to_include_rect(frame_rect); // e.g. for centered layouts: pretend we used whole frame
    }

    /// Move to the next row in a grid layout or wrapping layout.
    /// Otherwise does nothing.
    pub(crate) fn end_row(&mut self, item_spacing: Vec2, painter: &Painter) {
        if let Some(grid) = &mut self.grid {
            grid.end_row(&mut self.region.cursor, painter)
        } else {
            self.layout.end_row(&mut self.region, item_spacing)
        }
    }

    /// Set row height in horizontal wrapping layout.
    pub(crate) fn set_row_height(&mut self, height: f32) {
        self.layout.set_row_height(&mut self.region, height);
    }
}

impl Placer {
    /// Expand the `min_rect` and `max_rect` of this ui to include a child at the given rect.
    pub(crate) fn expand_to_include_rect(&mut self, rect: Rect) {
        self.region.expand_to_include_rect(rect);
    }

    /// Expand the `min_rect` and `max_rect` of this ui to include a child at the given x-coordinate.
    pub(crate) fn expand_to_include_x(&mut self, x: f32) {
        self.region.expand_to_include_x(x);
    }

    fn next_widget_space_ignore_wrap_justify(&self, size: Vec2) -> Rect {
        self.layout
            .next_widget_space_ignore_wrap_justify(&self.region, size)
    }

    /// Set the maximum width of the ui.
    /// You won't be able to shrink it below the current minimum size.
    pub(crate) fn set_max_width(&mut self, width: f32) {
        let rect = self.next_widget_space_ignore_wrap_justify(vec2(width, 0.0));
        let region = &mut self.region;
        region.max_rect.min.x = rect.min.x;
        region.max_rect.max.x = rect.max.x;
        region.max_rect = region.max_rect.union(region.min_rect); // make sure we didn't shrink too much
    }

    /// Set the maximum height of the ui.
    /// You won't be able to shrink it below the current minimum size.
    pub(crate) fn set_max_height(&mut self, height: f32) {
        let rect = self.next_widget_space_ignore_wrap_justify(vec2(0.0, height));
        let region = &mut self.region;
        region.max_rect.min.y = rect.min.y;
        region.max_rect.max.y = rect.max.y;
        region.max_rect = region.max_rect.union(region.min_rect); // make sure we didn't shrink too much
    }

    /// Set the minimum width of the ui.
    /// This can't shrink the ui, only make it larger.
    pub(crate) fn set_min_width(&mut self, width: f32) {
        let rect = self.next_widget_space_ignore_wrap_justify(vec2(width, 0.0));
        self.region.expand_to_include_x(rect.min.x);
        self.region.expand_to_include_x(rect.max.x);
    }

    /// Set the minimum height of the ui.
    /// This can't shrink the ui, only make it larger.
    pub(crate) fn set_min_height(&mut self, height: f32) {
        let rect = self.next_widget_space_ignore_wrap_justify(vec2(0.0, height));
        self.region.expand_to_include_y(rect.min.y);
        self.region.expand_to_include_y(rect.max.y);
    }
}

impl Placer {
    pub(crate) fn debug_paint_cursor(&self, painter: &crate::Painter, text: impl ToString) {
        let stroke = Stroke::new(1.0, Color32::DEBUG_COLOR);

        if let Some(grid) = &self.grid {
            let rect = grid.next_cell(self.cursor(), Vec2::splat(0.0));
            painter.rect_stroke(rect, 1.0, stroke);
            let align = Align2::CENTER_CENTER;
            painter.debug_text(align.pos_in_rect(&rect), align, stroke.color, text);
        } else {
            self.layout
                .paint_text_at_cursor(painter, &self.region, stroke, text)
        }
    }
}
