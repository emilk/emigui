use super::{
    Color32, Id, Rect, Ui,
    Frame, Area,
    Response, CtxRef,
    Pos2, Order,
    Align, Layout,
    PointerButton,
};

#[derive(Default, Clone)]
pub struct ContextMenuSystem {
    context_menu: Option<ContextMenuRoot>,
}
impl ContextMenuSystem {
    fn sense_click(&mut self, response: &Response) -> MenuResponse {
        let Response {
            id,
            ctx,
            ..
        } = response;
        let pointer = &ctx.input().pointer;
        if pointer.any_pressed() {
            if let Some(pos) = pointer.interact_pos() {
                let mut destroy = false;
                let mut in_old_menu = false;
                if let Some(context_menu) = &mut self.context_menu {
                    in_old_menu = context_menu.area_contains(pos);
                    destroy = context_menu.ui_id == *id;
                }
                if !in_old_menu {
                    if response.hovered() {
                        if pointer.button_down(PointerButton::Secondary) {
                            // todo: adapt to context
                            return MenuResponse::Create(pos);
                        } else {
                            return MenuResponse::Close;
                        }
                    } else if destroy {
                        return MenuResponse::Close;
                    }
                }
            }
        }
        MenuResponse::Stay
    }
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
    pub fn ui_context_menu(&mut self, response: &Response, add_contents: impl FnOnce(&mut Ui, &mut MenuState)) {
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
#[cfg_attr(feature = "persistence", derive(serde::Deserialize, serde::Serialize))]
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
#[derive(Clone)]
struct ContextMenuRoot {
    context_menu: ContextMenu,
    ui_id: Id,
}
impl ContextMenuRoot {
    pub fn new(position: Pos2, ui_id: Id) -> Self {
        Self {
            context_menu: ContextMenu::root(position),
            ui_id,
        }
    }
    pub(crate) fn show(&mut self, ctx: &CtxRef, add_contents: impl FnOnce(&mut Ui, &mut MenuState)) -> Response {
        self.context_menu.show_root(ctx, add_contents)
    }
}
impl std::ops::Deref for ContextMenuRoot {
    type Target = MenuState;
    fn deref(&self) -> &Self::Target {
        &self.context_menu.state
    }
}
impl std::ops::DerefMut for ContextMenuRoot {
    fn deref_mut(&mut self) -> &mut <Self as std::ops::Deref>::Target {
        &mut self.context_menu.state
    }
}
#[derive(Default, Clone)]
struct ContextMenu {
    state: MenuState,
    position: Pos2,
}
impl ContextMenu {
    pub fn root(position: Pos2) -> Self {
        Self {
            state: MenuState::default(),
            position,
        }
    }
    pub fn sub_menu(position: Pos2, state: MenuState) -> Self {
        Self {
            state,
            position,
        }
    }
    fn show_impl(&mut self, ctx: &CtxRef, add_contents: impl FnOnce(&mut Ui)) -> Response {
        Area::new(format!("context_menu_{:#?}", self.position))
            .order(Order::Foreground)
            .fixed_pos(self.position)
            .interactable(true)
            .show(ctx, |ui| {
                Frame::none()
                    .fill(Color32::BLACK)
                    .corner_radius(3.0)
                    .margin((0.0, 3.0))
                    .show(ui, |ui|
                        ui.with_layout(
                            Layout::top_down_justified(Align::LEFT),
                            add_contents,
                        )
                    );
            })
    }
    pub(crate) fn show_root(&mut self, ctx: &CtxRef, add_contents: impl FnOnce(&mut Ui, &mut MenuState)) -> Response {
        let mut state = self.state.clone();
        let response = self.show_impl(ctx, |ui| add_contents(ui, &mut state));
        self.state = state;
        response
    }
    pub fn show(&mut self, ctx: &CtxRef, add_contents: impl FnOnce(&mut Ui)) -> Response {
        self.show_impl(ctx, add_contents)
    }
}
impl std::ops::Deref for ContextMenu {
    type Target = MenuState;
    fn deref(&self) -> &Self::Target {
        &self.state
    }
}
impl std::ops::DerefMut for ContextMenu {
    fn deref_mut(&mut self) -> &mut <Self as std::ops::Deref>::Target {
        &mut self.state
    }
}
#[derive(Clone)]
pub struct SubMenu {
    text: String,
}
impl SubMenu {
    pub fn new(text: impl ToString) -> Self {
        Self {
            text: text.to_string(),
        }
    }
    pub fn show(self, ui: &mut Ui, parent_state: &mut MenuState, add_contents: impl FnOnce(&mut Ui, &mut MenuState)) -> Response {
        let button = ui.button(self.text);
        let mut sub_hovered = false;
        if let Some(sub_menu) = parent_state.get_submenu(button.id) {
            if let Some(pos) = ui.input().pointer.hover_pos() {
                sub_hovered = sub_menu.area_contains(pos);
            }
        }
        if !sub_hovered {
            if button.hovered() {
                parent_state.open_submenu(button.id);
            } else {
                parent_state.close_submenu(button.id);
            }
        }
        let responses = parent_state.get_submenu(button.id).map(|menu_state| {
            let response = ContextMenu::sub_menu(button.rect.right_top(), menu_state.clone())
                .show(ui.ctx(), |ui| add_contents(ui, menu_state));
            // set submenu bounding box
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
#[cfg_attr(feature = "persistence", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "persistence", serde(default))]
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
#[allow(unused)]
impl MenuState {
    pub(crate) fn area_contains(&self, pos: Pos2) -> bool{
        self.rect.contains(pos) ||
            self.sub_menu.as_ref()
                .map(|(_, sub)| sub.area_contains(pos))
                .unwrap_or(false)
    }
    pub fn close(&mut self) {
        self.response = MenuResponse::Close;
    }
    fn cascade_response(&mut self, response: MenuResponse) {
        if response.is_close() {
            self.response = response;
        }
    }
    pub fn get_submenu(&mut self, id: Id) -> Option<&mut MenuState> {
        self.sub_menu.as_mut().and_then(|(k, sub)| if id == *k {
            Some(sub.as_mut())
        } else {
            None
        })
    }
    fn open_submenu(&mut self, id: Id) {
        if let Some((k, _)) = self.sub_menu {
            if k == id {
                return;
            }
        }
        self.sub_menu = Some((id, Box::new(MenuState::default())));
    }
    fn close_submenu(&mut self, id: Id) {
        if let Some((k, _)) = self.sub_menu {
            if k == id {
                self.sub_menu = None;
            }
        }
    }
    pub fn toggle_submenu(&mut self, id: Id) {
        if let Some((k, _)) = self.sub_menu.take() {
            if k == id {
                self.sub_menu = None;
                return;
            }
        }
        self.sub_menu = Some((id, Box::new(MenuState::default())));
    }
}
