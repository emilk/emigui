/// All the different demo apps.
#[derive(Default)]
#[cfg_attr(feature = "persistence", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "persistence", serde(default))]
pub struct Apps {
    demo: crate::apps::DemoApp,
    easy_mark_editor: crate::easy_mark::EasyMarkEditor,
    #[cfg(feature = "http")]
    http: crate::apps::HttpApp,
    clock: crate::apps::FractalClock,
    color_test: crate::apps::ColorTest,
}

impl Apps {
    fn iter_mut(&mut self) -> impl Iterator<Item = (&str, &mut dyn epi::App)> {
        vec![
            ("demo", &mut self.demo as &mut dyn epi::App),
            ("easymark", &mut self.easy_mark_editor as &mut dyn epi::App),
            #[cfg(feature = "http")]
            ("http", &mut self.http as &mut dyn epi::App),
            ("clock", &mut self.clock as &mut dyn epi::App),
            ("colors", &mut self.color_test as &mut dyn epi::App),
        ]
        .into_iter()
    }
}

/// Wraps many demo/test apps into one.
#[derive(Default)]
#[cfg_attr(feature = "persistence", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "persistence", serde(default))]
pub struct WrapApp {
    selected_anchor: String,
    apps: Apps,
    backend_panel: super::backend_panel::BackendPanel,
}

impl epi::App for WrapApp {
    fn name(&self) -> &str {
        "egui demo apps"
    }

    fn setup(
        &mut self,
        _ctx: &egui::CtxRef,
        _frame: &mut epi::Frame<'_>,
        storage: Option<&dyn epi::Storage>,
    ) {
        #[cfg(feature = "persistence")]
        if let Some(storage) = storage {
            *self = epi::get_value(storage, epi::APP_KEY).unwrap_or_default()
        }
    }

    #[cfg(feature = "persistence")]
    fn save(&mut self, storage: &mut dyn epi::Storage) {
        epi::set_value(storage, epi::APP_KEY, self);
    }

    fn max_size_points(&self) -> egui::Vec2 {
        self.backend_panel.max_size_points_active
    }

    fn clear_color(&self) -> egui::Rgba {
        egui::Rgba::TRANSPARENT // we set a `CentralPanel` fill color in `demo_windows.rs`
    }

    fn warm_up_enabled(&self) -> bool {
        // The example windows use a lot of emojis. Pre-cache them by running one frame where everything is open
        cfg!(not(debug_assertions))
    }

    fn update(&mut self, ctx: &egui::CtxRef, frame: &mut epi::Frame<'_>) {

        if let Some(web_info) = frame.info().web_info.as_ref() {
            if let Some(anchor) = web_info.web_location_hash.strip_prefix('#') {
                self.selected_anchor = anchor.to_owned();
            }
        }

        if self.selected_anchor.is_empty() {
            self.selected_anchor = self.apps.iter_mut().next().unwrap().0.to_owned();
        }

        egui::TopBottomPanel::top("wrap_app_top_bar").show(ctx, |ui| {
            egui::trace!(ui);
            self.bar_contents(ui, frame);
        });

        self.backend_panel.update(ctx, frame);

        if self.backend_panel.open || ctx.memory().everything_is_visible() {
            egui::SidePanel::left("backend_panel").show(ctx, |ui| {
                self.backend_panel.ui(ui, frame);
            });
        }

        for (anchor, app) in self.apps.iter_mut() {
            if anchor == self.selected_anchor || ctx.memory().everything_is_visible() {
                app.update(ctx, frame);
            }
        }

        self.backend_panel.end_of_frame(ctx);
    }
}

impl WrapApp {
    fn bar_contents(&mut self, ui: &mut egui::Ui, frame: &mut epi::Frame<'_>) {
        // A menu-bar is a horizontal layout with some special styles applied.
        // egui::menu::bar(ui, |ui| {
        ui.horizontal_wrapped(|ui| {
            ui.context_menu(|ui, _menu_state| {
                if ui.button("Print something").clicked() {
                    println!("something");
                }
            });
            dark_light_mode_switch(ui);

            ui.checkbox(&mut self.backend_panel.open, "💻 Backend");
            ui.separator();

            for (anchor, app) in self.apps.iter_mut() {
                if ui
                    .selectable_label(self.selected_anchor == anchor, app.name())
                    .clicked()
                {
                    self.selected_anchor = anchor.to_owned();
                    if frame.is_web() {
                        ui.output().open_url(format!("#{}", anchor));
                    }
                }
            }

            ui.with_layout(egui::Layout::right_to_left(), |ui| {
                if false {
                    // TODO: fix the overlap on small screens
                    if let Some(seconds_since_midnight) = frame.info().seconds_since_midnight {
                        if clock_button(ui, seconds_since_midnight).clicked() {
                            self.selected_anchor = "clock".to_owned();
                            if frame.is_web() {
                                ui.output().open_url("#clock");
                            }
                        }
                    }
                }

                egui::warn_if_debug_build(ui);
            });
        });
    }
}

fn clock_button(ui: &mut egui::Ui, seconds_since_midnight: f64) -> egui::Response {
    let time = seconds_since_midnight;
    let time = format!(
        "{:02}:{:02}:{:02}.{:02}",
        (time % (24.0 * 60.0 * 60.0) / 3600.0).floor(),
        (time % (60.0 * 60.0) / 60.0).floor(),
        (time % 60.0).floor(),
        (time % 1.0 * 100.0).floor()
    );

    ui.add(egui::Button::new(time).text_style(egui::TextStyle::Monospace))
}

/// Show a button to switch to/from dark/light mode (globally).
fn dark_light_mode_switch(ui: &mut egui::Ui) {
    let style: egui::Style = (*ui.ctx().style()).clone();
    let new_visuals = style.visuals.light_dark_small_toggle_button(ui);
    if let Some(visuals) = new_visuals {
        ui.ctx().set_visuals(visuals);
    }
}
