use super::{
    Id, Rect, Ui,
    Response, CtxRef,
    Pos2, Sense, Vec2,
    PointerState,
    Style, TextStyle,
    style::Spacing,
    Frame, Label,
    Layout,
};

#[derive(Default)]
pub struct ContextMenuSystem {
    context_menu: Option<ContextMenuRoot>,
}
impl ContextMenuSystem {
    /// sense if a context menu needs to be (re-)created or destroyed
    fn sense_click(&mut self, response: &Response) -> MenuResponse {
        let response = response.interact(Sense::click());
        let pointer = &response.ctx.input().pointer;
        if pointer.any_click() {
            if let Some(pos) = pointer.interact_pos() {
                let mut destroy = false;
                let mut in_old_menu = false;
                if let Some(context_menu) = &mut self.context_menu {
                    in_old_menu = context_menu.area_contains(pos);
                    destroy = context_menu.ui_id == response.id;
                }
                if !in_old_menu {
                    if response.secondary_clicked() {
                        return MenuResponse::Create(pos);
                    } else if response.clicked() || destroy {
                        return MenuResponse::Close;
                    }
                }
            }
        }
        MenuResponse::Stay
    }
    /// show the current context menu
    fn show(&mut self, response: &Response, add_contents: impl FnOnce(&mut Ui, &mut MenuState)) -> MenuResponse {
        if let Some(context_menu) = &mut self.context_menu {
            if context_menu.ui_id == response.id {
                let response = context_menu.show(&response.ctx, response.id, add_contents);
                context_menu.rect = response.rect;

                if context_menu.response.is_close() {
                    return MenuResponse::Close;
                }
            }
        }
        MenuResponse::Stay
    }
    /// should be called from Context on a Response
    pub fn context_menu(&mut self, response: &Response, add_contents: impl FnOnce(&mut Ui, &mut MenuState)) {
        match self.sense_click(response) {
            MenuResponse::Create(pos) => {
                self.context_menu = Some(ContextMenuRoot::new(pos, response.id));
            },
            MenuResponse::Close => {
                self.context_menu = None
            },
            MenuResponse::Stay => {}
        };
        if let MenuResponse::Close = self.show(response, add_contents) {
            self.context_menu = None
        }
    }
}
#[derive(Clone, PartialEq)]
enum MenuResponse {
    Close,
    Stay,
    Create(Pos2),
}
impl MenuResponse {
    pub fn is_close(&self) -> bool {
        *self == Self::Close
    }
}
/// Context menu root associated with an Id from a Response
struct ContextMenuRoot {
    context_menu: MenuState,
    ui_id: Id,
}
impl ContextMenuRoot {
    pub fn new(position: Pos2, ui_id: Id) -> Self {
        Self {
            context_menu: MenuState::new(position),
            ui_id,
        }
    }
}
impl std::ops::Deref for ContextMenuRoot {
    type Target = MenuState;
    fn deref(&self) -> &Self::Target {
        &self.context_menu
    }
}
impl std::ops::DerefMut for ContextMenuRoot {
    fn deref_mut(&mut self) -> &mut <Self as std::ops::Deref>::Target {
        &mut self.context_menu
    }
}

pub struct SubMenu<'a> {
    text: String,
    parent_state: &'a mut MenuState,
}
impl<'a> SubMenu<'a> {
    #[allow(clippy::needless_pass_by_value)]
    fn new(text: impl ToString, parent_state: &'a mut MenuState) -> Self {
        Self {
            text: text.to_string(),
            parent_state,
        }
    }
    pub fn show(self, ui: &mut Ui, add_contents: impl FnOnce(&mut Ui, &mut MenuState)) -> Response {
        let sub_id = ui.id().with(format!("{:?}", ui.placer.cursor().min));
        let mut label = Label::new(self.text)
            .text_style(TextStyle::Button)
            .text_color(ui.visuals().widgets.inactive.fg_stroke.color);
        let mut icon = Label::new("⏵")
            .text_style(TextStyle::Button)
            .text_color(ui.visuals().widgets.inactive.fg_stroke.color);
        let mut frame = Frame::none();
        let pointer = &ui.input().pointer.clone();
        let open = self.parent_state.is_open(sub_id);
        if open {
            icon = icon.text_color(ui.visuals().widgets.hovered.fg_stroke.color);
            label = label.text_color(ui.visuals().widgets.hovered.fg_stroke.color);
            frame = frame.fill(ui.visuals().widgets.hovered.bg_fill);
        }
        let padding = ui.spacing().button_padding.x; 
        let button = frame.show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.with_layout(Layout::left_to_right(), |ui| {
                    ui.add_space(padding);
                    ui.label(label);
                });
                ui.with_layout(Layout::right_to_left(), |ui| {
                    ui.add(icon);
                    ui.add_space(padding);
                });
            });
        })
        .response;
        if self.parent_state.moving_towards_current_submenu(pointer) {
            // ensure to repaint once even when pointer is not moving
            ui.ctx().request_repaint();
        } else if !open && button.hovered() {
            let pos = button.rect.right_top();
            self.parent_state.open_submenu(sub_id, pos);
        } else if open && !button.hovered() && !self.parent_state.hovering_current_submenu(pointer) {
            self.parent_state.close_submenu();
        }
        let responses = self.parent_state.get_submenu(sub_id)
            .map(|menu_state| {
                let response = menu_state.show(ui.ctx(), sub_id, add_contents);
                menu_state.rect = response.rect;
                (menu_state.response.clone(), response)
            });
        if let Some((menu_response, response)) = responses {
            self.parent_state.cascade_response(menu_response);
            response
        } else {
            button
        }
    }
}

pub struct MenuState {
    sub_menu: Option<(Id, Box<MenuState>)>,
    rect: Rect,
    response: MenuResponse,
}
impl MenuState {
    /// close menu hierarchy
    pub fn close(&mut self) {
        self.response = MenuResponse::Close;
    }
    /// show a sub-menu
    pub fn submenu(&'_ mut self, text: impl ToString) -> SubMenu<'_> {
        SubMenu::new(text, self)
    }
    fn new(position: Pos2) -> Self {
        Self {
            rect: Rect::from_min_size(position, Vec2::ZERO),
            sub_menu: None,
            response: MenuResponse::Stay
        }
    }
    fn show(&mut self, ctx: &CtxRef, id: Id, add_contents: impl FnOnce(&mut Ui, &mut MenuState)) -> Response {
        crate::menu::menu_ui(
            ctx,
            id,
            self.rect.min,
            Style {
                spacing: Spacing {
                    item_spacing: Vec2::ZERO,
                    ..Default::default()
                },
                ..Default::default()
            },
            |ui| {
                ui.set_width(100.0);
                add_contents(ui, self)
            }
        )
        .response
    }
    /// check if position is in the menu hierarchy's area
    fn area_contains(&self, pos: Pos2) -> bool{
        self.rect.contains(pos) ||
            self.sub_menu.as_ref()
                .map(|(_, sub)| sub.area_contains(pos))
                .unwrap_or(false)
    }
    /// check if dir points from pos towards left side of rect
    fn points_at_left_of_rect(pos: Pos2, dir: Vec2, rect: Rect) -> bool {
        let vel_a = dir.angle();
        let top_a = (rect.left_top() - pos).angle();
        let bottom_a = (rect.left_bottom() - pos).angle();
        bottom_a - vel_a >= 0.0 && top_a - vel_a <= 0.0
    }
    /// check if pointer is moving towards current submenu
    fn moving_towards_current_submenu(&self, pointer: &PointerState) -> bool{
        if pointer.is_still() { return false; }
        if let Some(sub_menu) = self.get_current_submenu() {
            if let Some(pos) = pointer.hover_pos() {
                return Self::points_at_left_of_rect(pos, pointer.velocity(), sub_menu.rect);
            }
        }
        false
    }
    /// check if pointer is hovering current submenu
    fn hovering_current_submenu(&self, pointer: &PointerState) -> bool{
        if let Some(sub_menu) = self.get_current_submenu() {
            if let Some(pos) = pointer.hover_pos() {
                return sub_menu.area_contains(pos);
            }
        }
        false
    }
    fn is_open(&self, id: Id) -> bool {
        self.get_sub_id() == Some(id)
    }
    /// cascade close response to menu root
    fn cascade_response(&mut self, response: MenuResponse) {
        if response.is_close() {
            self.response = response;
        }
    }
    fn get_sub_id(&self) -> Option<Id> {
        self.sub_menu.as_ref().map(|(id, _)| *id)
    }
    fn get_current_submenu(&self) -> Option<&MenuState> {
        self.sub_menu.as_ref().map(|(_, sub)| sub.as_ref())
    }
    fn get_submenu(&mut self, id: Id) -> Option<&mut MenuState> {
        self.sub_menu.as_mut().and_then(|(k, sub)| if id == *k {
            Some(sub.as_mut())
        } else {
            None
        })
    }
    /// open submenu at position, if not already open
    fn open_submenu(&mut self, id: Id, pos: Pos2) {
        if !self.is_open(id) {
            self.sub_menu = Some((id, Box::new(MenuState::new(pos))));
        }
    }
    fn close_submenu(&mut self) {
        self.sub_menu = None;
    }
}
