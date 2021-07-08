use super::{
    Id, Rect, Ui,
    Response, CtxRef,
    Pos2, Sense, Vec2,
    PointerState,
    Style,
};

#[derive(Default, Clone)]
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
                let response = context_menu.show(&response.ctx, add_contents);
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
#[derive(Clone)]
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
#[derive(Clone)]
pub struct SubMenu {
    text: String,
}
impl SubMenu {
    #[allow(clippy::needless_pass_by_value)]
    pub fn new(text: impl ToString) -> Self {
        Self {
            text: text.to_string(),
        }
    }
    pub fn show(self, ui: &mut Ui, parent_state: &mut MenuState, add_contents: impl FnOnce(&mut Ui, &mut MenuState)) -> Response {
        let button = ui.button(self.text);
        let pointer = &ui.input().pointer;
        if !parent_state.moving_towards_current_submenu(pointer) {
            if button.hovered() {
                parent_state.open_submenu(button.id, button.rect.right_top());
            } else if !parent_state.hovering_current_submenu(pointer) {
                parent_state.close_submenu();
            }
        } else {
            // ensure to repaint even when pointer is not moving
            ui.ctx().request_repaint();
        }
        let responses = parent_state.get_submenu(button.id).map(|menu_state| {
            let response = menu_state.show(ui.ctx(), add_contents);
            menu_state.rect = response.rect;
            (menu_state.response.clone(), response)
        });
        if let Some((menu_response, response)) = responses {
            parent_state.cascade_response(menu_response);
            response
        } else {
            button
        }
    }
}

#[derive(Clone)]
pub struct MenuState {
    sub_menu: Option<(Id, Box<MenuState>)>,
    pub rect: Rect,
    response: MenuResponse,
}
impl Default for MenuState {
    fn default() -> Self {
        Self {
            rect: Rect::NOTHING,
            sub_menu: None,
            response: MenuResponse::Stay
        }
    }
}
impl MenuState {
    pub fn new(position: Pos2) -> Self {
        Self {
            rect: Rect::from_min_size(position, Vec2::ZERO),
            ..Default::default()
        }
    }
    pub(crate) fn show(&mut self, ctx: &CtxRef, add_contents: impl FnOnce(&mut Ui, &mut MenuState)) -> Response {
        crate::menu::menu_ui(
            ctx,
            Id::new(format!("context_menu_{:#?}", self.rect.min)),
            self.rect.min,
            Style::default(),
            |ui| add_contents(ui, self)
        )
        .response
    }
    /// check if position is in the menu hierarchy's area
    pub(crate) fn area_contains(&self, pos: Pos2) -> bool{
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
    pub(crate) fn moving_towards_current_submenu(&self, pointer: &PointerState) -> bool{
        if pointer.is_still() { return false; }
        if let Some(sub_menu) = self.get_current_submenu() {
            if let Some(pos) = pointer.hover_pos() {
                return Self::points_at_left_of_rect(pos, pointer.velocity(), sub_menu.rect);
            }
        }
        false
    }
    /// check if pointer is hovering current submenu
    pub(crate) fn hovering_current_submenu(&self, pointer: &PointerState) -> bool{
        if let Some(sub_menu) = self.get_current_submenu() {
            if let Some(pos) = pointer.hover_pos() {
                return sub_menu.area_contains(pos);
            }
        }
        false
    }
    /// set close response
    pub fn close(&mut self) {
        self.response = MenuResponse::Close;
    }
    /// cascade close response to menu root
    fn cascade_response(&mut self, response: MenuResponse) {
        if response.is_close() {
            self.response = response;
        }
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
        if let Some((k, _)) = self.sub_menu {
            if k == id {
                return;
            }
        }
        self.sub_menu = Some((id, Box::new(MenuState::new(pos))));
    }
    fn close_submenu(&mut self) {
        self.sub_menu = None;
    }
}
