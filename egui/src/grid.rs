use crate::*;
use std::collections::HashMap;

#[derive(Clone, Debug, Default, PartialEq)]
#[cfg_attr(feature = "persistence", derive(serde::Deserialize, serde::Serialize))]
pub(crate) struct State {
    col_widths: Vec<f32>,
    row_heights: Vec<f32>,
}

/// Describe the dimensions of a grid state
#[derive(Clone, Copy, Debug)]
pub(crate) struct Dimensions {
    x: usize,
    y: usize,
}

impl State {
    fn set_min_col_width(&mut self, col: usize, width: f32) {
        self.col_widths
            .resize(self.col_widths.len().max(col + 1), 0.0);
        self.col_widths[col] = self.col_widths[col].max(width);
    }

    fn set_min_row_height(&mut self, row: usize, height: f32) {
        self.row_heights
            .resize(self.row_heights.len().max(row + 1), 0.0);
        self.row_heights[row] = self.row_heights[row].max(height);
    }

    fn dimensions(&self) -> Dimensions {
        Dimensions {
            x: self.row_heights.len(),
            y: self.col_widths.len(),
        }
    }

    fn col_width(&self, col: usize) -> Option<f32> {
        self.col_widths.get(col).copied()
    }

    fn row_height(&self, row: usize) -> Option<f32> {
        self.row_heights.get(row).copied()
    }

    fn full_width(&self, x_spacing: f32) -> f32 {
        self.col_widths.iter().sum::<f32>()
            + (self.col_widths.len().at_least(1) - 1) as f32 * x_spacing
    }

    fn full_height(&self, y_spacing: f32) -> f32 {
        self.row_heights.iter().sum::<f32>()
            + (self.row_heights.len().at_least(1) - 1) as f32 * y_spacing
    }
}

// ----------------------------------------------------------------------------

/// Either a single color, or a pair of colors corresponding to light/dark scheme
#[derive(Clone, Copy)]
pub enum GuiColor {
    Single(Rgba),
    Pair { light: Rgba, dark: Rgba },
}

impl From<Rgba> for GuiColor {
    fn from(color: Rgba) -> Self {
        Self::Single(color)
    }
}

impl GuiColor {
    /// Create a single color
    pub fn single(color: Rgba) -> Self {
        Self::Single(color)
    }

    /// Create a light/dark scheme pair
    pub fn pair(light: Rgba, dark: Rgba) -> Self {
        Self::Pair { light, dark }
    }
}

type SingleArgumentPredicate = fn(usize) -> bool;
type DoubleArgumentPredicate = fn(usize, usize) -> bool;

/// A predicate that takes one int argument.
pub(crate) struct SinglePredicate {
    pub color: GuiColor,
    pub predicate: SingleArgumentPredicate,
}

/// A predicate that takes two int arguments
pub(crate) struct DoublePredicate {
    pub color: GuiColor,
    pub predicate: DoubleArgumentPredicate,
}

/// Describe conditional formattings for a grid.
/// All predicates are evaluated for each row/col/cell, and colors will be painted on top of each other.
/// The evaluation order is: row, column, cell, list
#[derive(Default)]
pub(crate) struct ColorSpec {
    /// If the (row, col) exists, the corresponding color is applied
    pub list: HashMap<(usize, usize), GuiColor>,
    /// The predicate is passed the row number.
    pub by_row: Vec<SinglePredicate>,
    /// The predicate is passed the column number.
    pub by_column: Vec<SinglePredicate>,
    /// The predicate is passed the row and column numbers.
    pub by_cell: Vec<DoublePredicate>,
}

// ----------------------------------------------------------------------------

pub(crate) struct GridLayout {
    ctx: CtxRef,
    style: std::sync::Arc<Style>,
    id: Id,

    /// State previous frame (if any).
    /// This can be used to predict future sizes of cells.
    prev_state: State,
    /// State accumulated during the current frame.
    curr_state: State,

    spacing: Vec2,

    color_spec: ColorSpec,
    initial_x: f32,
    min_cell_size: Vec2,
    max_cell_size: Vec2,
    col: usize,
    row: usize,
}

impl GridLayout {
    pub(crate) fn new(ui: &Ui, id: Id) -> Self {
        let prev_state = ui.memory().id_data.get_or_default::<State>(id).clone();

        // TODO: respect current layout

        let available = ui.placer().max_rect().intersect(ui.cursor());
        let initial_x = available.min.x;
        assert!(
            initial_x.is_finite(),
            "Grid not yet available for right-to-left layouts"
        );

        Self {
            ctx: ui.ctx().clone(),
            style: ui.style().clone(),
            id,
            prev_state,
            curr_state: State::default(),
            spacing: ui.spacing().item_spacing,
            color_spec: Default::default(),
            initial_x,
            min_cell_size: ui.spacing().interact_size,
            max_cell_size: Vec2::INFINITY,
            col: 0,
            row: 0,
        }
    }
}

impl GridLayout {
    fn prev_col_width(&self, col: usize) -> f32 {
        self.prev_state
            .col_width(col)
            .unwrap_or(self.min_cell_size.x)
    }
    fn prev_row_height(&self, row: usize) -> f32 {
        self.prev_state
            .row_height(row)
            .unwrap_or(self.min_cell_size.y)
    }

    pub(crate) fn wrap_text(&self) -> bool {
        self.max_cell_size.x.is_finite()
    }

    pub(crate) fn available_rect(&self, region: &Region) -> Rect {
        // required for putting CollapsingHeader in anything but the last column:
        self.available_rect_finite(region)
    }

    pub(crate) fn available_rect_finite(&self, region: &Region) -> Rect {
        let width = if self.max_cell_size.x.is_finite() {
            // TODO: should probably heed `prev_state` here too
            self.max_cell_size.x
        } else {
            // If we want to allow width-filling widgets like `Separator` in one of the first cells
            // then we need to make sure they don't spill out of the first cell:
            self.prev_state
                .col_width(self.col)
                .or_else(|| self.curr_state.col_width(self.col))
                .unwrap_or(self.min_cell_size.x)
        };

        let available = region.max_rect.intersect(region.cursor);

        let height = region.max_rect_finite().max.y - available.top();
        let height = height
            .at_least(self.min_cell_size.y)
            .at_most(self.max_cell_size.y);

        Rect::from_min_size(available.min, vec2(width, height))
    }

    pub(crate) fn next_cell(&self, cursor: Rect, child_size: Vec2) -> Rect {
        let width = self.prev_state.col_width(self.col).unwrap_or(0.0);
        let height = self.prev_row_height(self.row);
        let size = child_size.max(vec2(width, height));
        Rect::from_min_size(cursor.min, size)
    }

    #[allow(clippy::unused_self)]
    pub(crate) fn align_size_within_rect(&self, size: Vec2, frame: Rect) -> Rect {
        // TODO: allow this alignment to be customized
        Align2::LEFT_CENTER.align_size_within_rect(size, frame)
    }

    pub(crate) fn justify_and_align(&self, frame: Rect, size: Vec2) -> Rect {
        self.align_size_within_rect(size, frame)
    }

    pub(crate) fn advance(&mut self, cursor: &mut Rect, frame_rect: Rect, widget_rect: Rect) {
        let debug_expand_width = self.style.debug.show_expand_width;
        let debug_expand_height = self.style.debug.show_expand_height;
        if debug_expand_width || debug_expand_height {
            let rect = widget_rect;
            let too_wide = rect.width() > self.prev_col_width(self.col);
            let too_high = rect.height() > self.prev_row_height(self.row);

            if (debug_expand_width && too_wide) || (debug_expand_height && too_high) {
                let painter = self.ctx.debug_painter();
                painter.rect_stroke(rect, 0.0, (1.0, Color32::LIGHT_BLUE));

                let stroke = Stroke::new(2.5, Color32::from_rgb(200, 0, 0));
                let paint_line_seg = |a, b| painter.line_segment([a, b], stroke);

                if debug_expand_width && too_wide {
                    paint_line_seg(rect.left_top(), rect.left_bottom());
                    paint_line_seg(rect.left_center(), rect.right_center());
                    paint_line_seg(rect.right_top(), rect.right_bottom());
                }
            }
        }

        self.curr_state
            .set_min_col_width(self.col, widget_rect.width().at_least(self.min_cell_size.x));
        self.curr_state.set_min_row_height(
            self.row,
            widget_rect.height().at_least(self.min_cell_size.y),
        );

        self.col += 1;
        cursor.min.x += frame_rect.width() + self.spacing.x;
    }

    /// Paint a row.
    pub(crate) fn paint_row(&self, min: Pos2, color: GuiColor, painter: &Painter) {
        let color = match color {
            GuiColor::Single(color) => color,
            GuiColor::Pair { light, dark } => match self.style.visuals.dark_mode {
                true => dark,
                false => light,
            },
        };
        if let Some(height) = self.prev_state.row_height(self.row) {
            // Paint background for coming row:
            let size = Vec2::new(self.prev_state.full_width(self.spacing.x), height);
            let rect = Rect::from_min_size(min, size);
            let mut rect = rect.expand2(Vec2::new(2.0, 0.0));
            rect.max += Vec2::new(1.0, 0.5 * self.spacing.y + 1.5);

            painter.rect_filled(rect, 2.0, color);
        }
    }

    /// Paint a column
    pub(crate) fn paint_column(&self, col: usize, min: Pos2, color: GuiColor, painter: &Painter) {
        let col_f = col as f32;

        let color = match color {
            GuiColor::Single(color) => color,
            GuiColor::Pair { light, dark } => match self.style.visuals.dark_mode {
                true => dark,
                false => light,
            },
        };

        // Paint a column:
        // Offset from the cursor to paint the col at the right spot.
        let min_offset = min
            + Vec2::new(
                // Sum up all the previous widths and add the padding
                self.prev_state.col_widths.iter().take(col).sum::<f32>()
                    + (self.spacing.x + 1.0) * (col_f - 1.0),
                -1.0,
            );
        let size = Vec2::new(
            self.prev_col_width(col),
            self.prev_state.full_height(self.spacing.y),
        );
        let size = size
            + Vec2::new(
                // Add padding
                0.5 * self.spacing.x + 5.0,
                3.0,
            );

        let rect = Rect::from_min_size(min_offset, size);

        painter.rect_filled(rect, 2.0, color);
    }

    /// Paint a cell. Note that we can only paint for the current row.
    pub(crate) fn paint_cell(&self, col: usize, min: Pos2, color: GuiColor, painter: &Painter) {
        let col_f = col as f32;

        let color = match color {
            GuiColor::Single(color) => color,
            GuiColor::Pair { light, dark } => match self.style.visuals.dark_mode {
                true => dark,
                false => light,
            },
        };

        // Paint a column:
        // Offset from the cursor to paint the col at the right spot.
        let min_offset = min
            + Vec2::new(
                // Sum up all the previous widths and add the padding
                self.prev_state.col_widths.iter().take(col).sum::<f32>()
                    + (self.spacing.x + 1.0) * (col_f - 1.0),
                0.0,
            );
        let size = Vec2::new(self.prev_col_width(col), self.prev_row_height(self.row));
        let size = size
            + Vec2::new(
                // Add padding
                0.5 * self.spacing.x + 5.0,
                3.0,
            );

        let rect = Rect::from_min_size(min_offset, size);

        painter.rect_filled(rect, 2.0, color);
    }

    /// Paint the background for the next row & columns.
    pub(crate) fn do_paint(&mut self, min: Pos2, painter: &Painter) {
        for p in &self.color_spec.by_row {
            if (p.predicate)(self.row) {
                self.paint_row(min, p.color, painter);
            }
        }
        // Only paint columns when we're on the first row.
        if self.row == 0 {
            for p in &self.color_spec.by_column {
                // Iterate over columns
                for col in 0..self.prev_state.dimensions().y {
                    if (p.predicate)(col) {
                        self.paint_column(col, min, p.color, painter);
                    }
                }
            }
        }

        // Finally, do cells
        for col in 0..self.prev_state.dimensions().y {
            for p in &self.color_spec.by_cell {
                if (p.predicate)(self.row, col) {
                    self.paint_cell(col, min, p.color, painter);
                }
            }
            if let Some(color) = self.color_spec.list.get(&(self.row, col)) {
                self.paint_cell(col, min, *color, painter);
            }
        }
    }

    pub(crate) fn end_row(&mut self, cursor: &mut Rect, painter: &Painter) {
        let row_height = self.prev_row_height(self.row);

        cursor.min.x = self.initial_x;
        cursor.min.y += row_height + self.spacing.y;
        self.col = 0;
        self.row += 1;

        self.do_paint(cursor.min, painter);
    }

    pub(crate) fn save(&self) {
        if self.curr_state != self.prev_state {
            self.ctx
                .memory()
                .id_data
                .insert(self.id, self.curr_state.clone());
            self.ctx.request_repaint();
        }
    }
}

// ----------------------------------------------------------------------------

/// A simple grid layout.
///
/// The cells are always layed out left to right, top-down.
/// The contents of each cell will be aligned to the left and center.
///
/// If you want to add multiple widgets to a cell you need to group them with
/// [`Ui::horizontal`], [`Ui::vertical`] etc.
///
/// ```
/// # let ui = &mut egui::Ui::__test();
/// egui::Grid::new("some_unique_id").show(ui, |ui| {
///     ui.label("First row, first column");
///     ui.label("First row, second column");
///     ui.end_row();
///
///     ui.label("Second row, first column");
///     ui.label("Second row, second column");
///     ui.label("Second row, third column");
///     ui.end_row();
///
///     ui.horizontal(|ui| { ui.label("Same"); ui.label("cell"); });
///     ui.label("Third row, second column");
///     ui.end_row();
/// });
/// ```
pub struct Grid {
    id_source: Id,
    striped: bool,
    header_row: bool,
    min_col_width: Option<f32>,
    min_row_height: Option<f32>,
    max_cell_size: Vec2,
    spacing: Option<Vec2>,
    color_spec: ColorSpec,
}

impl Grid {
    /// Create a new [`Grid`] with a locally unique identifier.
    pub fn new(id_source: impl std::hash::Hash) -> Self {
        Self {
            id_source: Id::new(id_source),
            striped: false,
            header_row: false,
            min_col_width: None,
            min_row_height: None,
            max_cell_size: Vec2::INFINITY,
            spacing: None,
            color_spec: Default::default(),
        }
    }

    /// Add a new colorspec for a column
    pub fn with_column_spec<T: Into<GuiColor>>(
        mut self,
        color: T,
        predicate: SingleArgumentPredicate,
    ) -> Self {
        self.color_spec.by_column.push(SinglePredicate {
            color: color.into(),
            predicate,
        });
        self
    }

    pub fn with_row_spec<T: Into<GuiColor>>(
        mut self,
        color: T,
        predicate: SingleArgumentPredicate,
    ) -> Self {
        self.color_spec.by_row.push(SinglePredicate {
            color: color.into(),
            predicate,
        });
        self
    }

    pub fn with_cell_spec<T: Into<GuiColor>>(
        mut self,
        color: T,
        predicate: DoubleArgumentPredicate,
    ) -> Self {
        self.color_spec.by_cell.push(DoublePredicate {
            color: color.into(),
            predicate,
        });
        self
    }

    /// If `true`, add a subtle background color to every other row.
    ///
    /// This can make a table easier to read.
    /// Default: `false`.
    pub fn striped(mut self, striped: bool) -> Self {
        self.striped = striped;
        self
    }

    pub fn header_row(mut self, header_row: bool) -> Self {
        self.header_row = header_row;
        self
    }

    /// Set minimum width of each column.
    /// Default: [`crate::style::Spacing::interact_size`]`.x`.
    pub fn min_col_width(mut self, min_col_width: f32) -> Self {
        self.min_col_width = Some(min_col_width);
        self
    }

    /// Set minimum height of each row.
    /// Default: [`crate::style::Spacing::interact_size`]`.y`.
    pub fn min_row_height(mut self, min_row_height: f32) -> Self {
        self.min_row_height = Some(min_row_height);
        self
    }

    /// Set soft maximum width (wrapping width) of each column.
    pub fn max_col_width(mut self, max_col_width: f32) -> Self {
        self.max_cell_size.x = max_col_width;
        self
    }

    /// Set spacing between columns/rows.
    /// Default: [`crate::style::Spacing::item_spacing`].
    pub fn spacing(mut self, spacing: impl Into<Vec2>) -> Self {
        self.spacing = Some(spacing.into());
        self
    }
}

impl Grid {
    pub fn show<R>(self, ui: &mut Ui, add_contents: impl FnOnce(&mut Ui) -> R) -> InnerResponse<R> {
        let Self {
            id_source,
            striped,
            header_row,
            min_col_width,
            min_row_height,
            max_cell_size,
            spacing,
            color_spec,
        } = self;
        let min_col_width = min_col_width.unwrap_or_else(|| ui.spacing().interact_size.x);
        let min_row_height = min_row_height.unwrap_or_else(|| ui.spacing().interact_size.y);
        let spacing = spacing.unwrap_or_else(|| ui.spacing().item_spacing);

        // Each grid cell is aligned LEFT_CENTER.
        // If somebody wants to wrap more things inside a cell,
        // then we should pick a default layout that matches that alignment,
        // which we do here:
        ui.horizontal(|ui| {
            let id = ui.make_persistent_id(id_source);
            let mut grid = GridLayout {
                spacing,
                min_cell_size: vec2(min_col_width, min_row_height),
                max_cell_size,
                color_spec,
                ..GridLayout::new(ui, id)
            };

            // Set convenience color specs
            if striped {
                grid.color_spec.by_row.push(SinglePredicate {
                    color: GuiColor::pair(
                        Rgba::from_black_alpha(0.075),
                        Rgba::from_white_alpha(0.0075),
                    ),
                    predicate: |row| row % 2 == 0,
                });
            }

            if header_row {
                grid.color_spec.by_row.push(SinglePredicate {
                    color: GuiColor::pair(
                        Rgba::from_black_alpha(0.75),
                        Rgba::from_white_alpha(0.075),
                    ),
                    predicate: |row| row == 0,
                });
            }

            // ---

            ui.set_grid(grid);
            ui.start_row();
            let r = add_contents(ui);
            ui.save_grid();
            r
        })
    }
}
