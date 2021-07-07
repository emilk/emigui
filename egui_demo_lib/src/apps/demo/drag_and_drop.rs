use egui::*;

pub fn drag_source(ui: &mut Ui, id: Id, body: impl FnOnce(&mut Ui)) {
    let is_being_dragged = ui.memory().is_being_dragged(id);

    if !is_being_dragged {
        let response = ui.scope(body).response;

        // Check for drags:
        let response = ui.interact(response.rect, id, Sense::drag());
        if response.hovered() {
            ui.output().cursor_icon = CursorIcon::Grab;
        }
    } else {
        ui.output().cursor_icon = CursorIcon::Grabbing;

        // Paint the body to a new layer:
        let layer_id = LayerId::new(Order::Tooltip, id);
        let response = ui.with_layer_id(layer_id, body).response;

        // Now we move the visuals of the body to where the mouse is.
        // Normally you need to decide a location for a widget first,
        // because otherwise that widget cannot interact with the mouse.
        // However, a dragged component cannot be interacted with anyway
        // (anything with `Order::Tooltip` always gets an empty `Response`)
        // So this is fine!

        if let Some(pointer_pos) = ui.input().pointer.interact_pos() {
            let delta = pointer_pos - response.rect.center();
            ui.ctx().translate_layer(layer_id, delta);
        }
    }
}

pub fn drop_target<R>(
    ui: &mut Ui,
    can_accept_what_is_being_dragged: bool,
    body: impl FnOnce(&mut Ui) -> R,
) -> InnerResponse<R> {
    let is_being_dragged = ui.memory().is_anything_being_dragged();

    let margin = Vec2::splat(4.0);

    let outer_rect_bounds = ui.available_rect_before_wrap();
    let inner_rect = outer_rect_bounds.shrink2(margin);
    let where_to_put_background = ui.painter().add(Shape::Noop);
    let mut content_ui = ui.child_ui(inner_rect, *ui.layout());
    let ret = body(&mut content_ui);
    let outer_rect = Rect::from_min_max(outer_rect_bounds.min, content_ui.min_rect().max + margin);
    let (rect, response) = ui.allocate_at_least(outer_rect.size(), Sense::hover());

    let style = if is_being_dragged && can_accept_what_is_being_dragged && response.hovered() {
        ui.visuals().widgets.active
    } else {
        ui.visuals().widgets.inactive
    };

    let mut fill = style.bg_fill;
    let mut stroke = style.bg_stroke;
    if is_being_dragged && !can_accept_what_is_being_dragged {
        // gray out:
        fill = color::tint_color_towards(fill, ui.visuals().window_fill());
        stroke.color = color::tint_color_towards(stroke.color, ui.visuals().window_fill());
    }

    ui.painter().set(
        where_to_put_background,
        Shape::Rect {
            corner_radius: style.corner_radius,
            fill,
            stroke,
            rect,
        },
    );

    InnerResponse::new(ret, response)
}

pub struct DragAndDropDemo {
    /// columns with items
    columns: Vec<Vec<String>>,
}

impl Default for DragAndDropDemo {
    fn default() -> Self {
        Self {
            columns: vec![
                vec!["Item A", "Item B", "Item C"],
                vec!["Item D", "Item E"],
                vec!["Item F", "Item G", "Item H"],
            ].into_iter()
            .map(|v| v.into_iter().map(ToString::to_string).collect())
            .collect(),
        }
    }
}

impl super::Demo for DragAndDropDemo {
    fn name(&self) -> &'static str {
        "✋ Drag and Drop"
    }

    fn show(&mut self, ctx: &CtxRef, open: &mut bool) {
        use super::View;
        Window::new(self.name())
            .open(open)
            .default_size(vec2(256.0, 256.0))
            .scroll(false)
            .resizable(false)
            .show(ctx, |ui| self.ui(ui));
    }
}

impl super::View for DragAndDropDemo {
    fn ui(&mut self, ui: &mut Ui) {
        ui.label("This is a proof-of-concept of drag-and-drop in egui");
        ui.label("Drag items between columns.");

        let mut source_col_row = None;
        let mut drop_col = None;
        ui.columns(self.columns.len(), |uis| {
            for (col_idx, column) in self.columns.clone().into_iter().enumerate() {
                let ui = &mut uis[col_idx];
                let can_accept_what_is_being_dragged = true; // We accept anything being dragged (for now) ¯\_(ツ)_/¯
                let response = drop_target(ui, can_accept_what_is_being_dragged, |ui| {
                    ui.set_min_size(vec2(64.0, 100.0));
                    for (row_idx, item) in column.iter().enumerate() {
                        let item_id = Id::new("item").with(col_idx).with(row_idx);
                        drag_source(ui, item_id, |ui| {
                            ui.add(Label::new(item).sense(Sense::click()))
                                .context_menu(|ui, menu_state| {
                                    if ui.button("Remove...").clicked() {
                                        self.columns[col_idx].remove(row_idx);
                                        menu_state.close();
                                    }
                                });
                        });

                        if ui.memory().is_being_dragged(item_id) {
                            source_col_row = Some((col_idx, row_idx));
                        }
                    }
                })
                .response;

                if col_idx == 0 {
                    response.context_menu(|ui, menu_state| {
                        if ui.button("New Item...").clicked() {
                            self.columns[0].push("New Item".to_string());
                            menu_state.close();
                        }
                    });
                }

                let is_being_dragged = ui.memory().is_anything_being_dragged();
                if is_being_dragged && can_accept_what_is_being_dragged && response.hovered() {
                    drop_col = Some(col_idx);
                }
            }
        });

        if let Some((source_col, source_row)) = source_col_row {
            if let Some(drop_col) = drop_col {
                if ui.input().pointer.any_released() {
                    // do the drop:
                    let item = self.columns[source_col].remove(source_row);
                    self.columns[drop_col].push(item);
                }
            }
        }

        ui.vertical_centered(|ui| {
            ui.add(crate::__egui_github_link_file!());
        });
    }
}
