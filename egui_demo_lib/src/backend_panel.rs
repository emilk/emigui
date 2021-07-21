/// How often we repaint the demo app by default
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum RunMode {
    /// This is the default for the demo.
    ///
    /// If this is selected, egui is only updated if are input events
    /// (like mouse movements) or there are some animations in the GUI.
    ///
    /// Reactive mode saves CPU.
    ///
    /// The downside is that the UI can become out-of-date if something it is supposed to monitor changes.
    /// For instance, a GUI for a thermostat need to repaint each time the temperature changes.
    /// To ensure the UI is up to date you need to call `egui::Context::request_repaint()` each
    /// time such an event happens. You can also chose to call `request_repaint()` once every second
    /// or after every single frame - this is called `Continuous` mode,
    /// and for games and interactive tools that need repainting every frame anyway, this should be the default.
    Reactive,

    /// This will call `egui::Context::request_repaint()` at the end of each frame
    /// to request the backend to repaint as soon as possible.
    ///
    /// On most platforms this will mean that egui will run at the display refresh rate of e.g. 60 Hz.
    ///
    /// For this demo it is not any reason to do so except to
    /// demonstrate how quickly egui runs.
    ///
    /// For games or other interactive apps, this is probably what you want to do.
    /// It will guarantee that egui is always up-to-date.
    Continuous,
}

/// Default for demo is Reactive since
/// 1) We want to use minimal CPU
/// 2) There are no external events that could invalidate the UI
///    so there are no events to miss.
impl Default for RunMode {
    fn default() -> Self {
        RunMode::Reactive
    }
}

// ----------------------------------------------------------------------------

#[cfg_attr(feature = "persistence", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "persistence", serde(default))]
pub struct BackendPanel {
    pub open: bool,

    #[cfg_attr(feature = "persistence", serde(skip))]
    // go back to `Reactive` mode each time we start
    run_mode: RunMode,

    /// current slider value for current gui scale
    pixels_per_point: Option<f32>,

    /// maximum size of the web browser canvas
    max_size_points_ui: egui::Vec2,
    pub max_size_points_active: egui::Vec2,

    #[cfg_attr(feature = "persistence", serde(skip))]
    frame_history: crate::frame_history::FrameHistory,

    #[cfg_attr(feature = "persistence", serde(skip))]
    output_event_history: std::collections::VecDeque<egui::output::OutputEvent>,

    egui_windows: EguiWindows,
}

impl Default for BackendPanel {
    fn default() -> Self {
        Self {
            open: false,
            run_mode: Default::default(),
            pixels_per_point: Default::default(),
            max_size_points_ui: egui::Vec2::new(1024.0, 2048.0),
            max_size_points_active: egui::Vec2::new(1024.0, 2048.0),
            frame_history: Default::default(),
            output_event_history: Default::default(),
            egui_windows: Default::default(),
        }
    }
}

impl BackendPanel {
    pub fn update(&mut self, ctx: &egui::CtxRef, frame: &mut epi::Frame<'_>) {
        self.frame_history
            .on_new_frame(ctx.input().time, frame.info().cpu_usage);

        if self.run_mode == RunMode::Continuous {
            // Tell the backend to repaint as soon as possible
            ctx.request_repaint();
        }
    }

    pub fn end_of_frame(&mut self, ctx: &egui::CtxRef) {
        for event in &ctx.output().events {
            self.output_event_history.push_back(event.clone());
        }
        while self.output_event_history.len() > 10 {
            self.output_event_history.pop_front();
        }

        self.egui_windows.windows(ctx);
    }

    pub fn context_menu(ui: &mut egui::Ui, menu_state: &mut egui::context_menu::MenuState) {
        if ui.button("Open...").clicked() {
            menu_state.close();
        }
        menu_state.submenu("SubMenu").show(ui, |ui, menu_state| {
            menu_state.submenu("SubMenu").show(ui, |ui, menu_state| {
                if ui.button("Open...").clicked() {
                    menu_state.close();
                }
                let _ = ui.button("Item");
            });
            menu_state.submenu("SubMenu").show(ui, |ui, menu_state| {
                if ui.button("Open...").clicked() {
                    menu_state.close();
                }
                let _ = ui.button("Item");
            });
            let _ = ui.button("Item");
            if ui.button("Open...").clicked() {
                menu_state.close();
            }
        });
        menu_state.submenu("SubMenu").show(ui, |ui, _menu_state| {
            let _ = ui.button("Item1");
            let _ = ui.button("Item2");
            let _ = ui.button("Item3");
            let _ = ui.button("Item4");
        });
        let _ = ui.button("Very long text for this item");
    }

    pub fn ui(&mut self, ui: &mut egui::Ui, frame: &mut epi::Frame<'_>) {
        egui::trace!(ui);
        ui.vertical_centered(|ui| {
            ui.heading("💻 Backend");
        });
        ui.separator();

        self.run_mode_ui(ui);

        if ui
            .button("Clear egui memory")
            .on_hover_text("Forget scroll, positions, sizes etc")
            .clicked()
        {
            *ui.ctx().memory() = Default::default();
        }

        ui.separator();

        self.frame_history.ui(ui);

        // For instance: `egui_web` sets `pixels_per_point` every frame to force
        // egui to use the same scale as the web zoom factor.
        let integration_controls_pixels_per_point = ui.input().raw.pixels_per_point.is_some();
        if !integration_controls_pixels_per_point {
            ui.separator();
            if let Some(new_pixels_per_point) = self.pixels_per_point_ui(ui, frame.info()) {
                ui.ctx().set_pixels_per_point(new_pixels_per_point);
            }
        }

        if !frame.is_web()
            && ui
                .button("📱 Phone Size")
                .on_hover_text("Resize the window to be small like a phone.")
                .clicked()
        {
            frame.set_window_size(egui::Vec2::new(375.0, 812.0)); // iPhone 12 mini
        }

        ui.separator();

        ui.label("egui windows");
        self.egui_windows.checkboxes(ui);

        ui.separator();

        if frame.is_web() {
            ui.label("egui is an immediate mode GUI written in Rust, compiled to WebAssembly, rendered with WebGL.");
            ui.label(
                "Everything you see is rendered as textured triangles. There is no DOM. There are no HTML elements. \
                This is not JavaScript. This is Rust, running at 60 FPS. This is the web page, reinvented with game tech.");
            ui.label("This is also work in progress, and not ready for production... yet :)");
            ui.horizontal_wrapped(|ui| {
                ui.label("Project home page:");
                ui.hyperlink("https://github.com/emilk/egui");
            });

            ui.separator();

            ui.add(
                egui::Slider::new(&mut self.max_size_points_ui.x, 512.0..=f32::INFINITY)
                    .logarithmic(true)
                    .largest_finite(8192.0)
                    .text("Max width"),
            )
            .on_hover_text("Maximum width of the egui region of the web page.");
            if !ui.ctx().is_using_pointer() {
                self.max_size_points_active = self.max_size_points_ui;
            }
        }

        {
            let mut debug_on_hover = ui.ctx().debug_on_hover();
            ui.checkbox(&mut debug_on_hover, "🐛 Debug on hover")
                .on_hover_text("Show structure of the ui when you hover with the mouse");
            ui.ctx().set_debug_on_hover(debug_on_hover);
        }

        ui.separator();

        {
            let mut screen_reader = ui.ctx().memory().options.screen_reader;
            ui.checkbox(&mut screen_reader, "🔈 Screen reader").on_hover_text("Experimental feature: checking this will turn on the screen reader on supported platforms");
            ui.ctx().memory().options.screen_reader = screen_reader;
        }

        ui.collapsing("Output events", |ui| {
            ui.set_max_width(450.0);
            ui.label(
                "Recent output events from egui. \
            These are emitted when you switch selected widget with tab, \
            and can be hooked up to a screen reader on supported platforms.",
            );
            ui.add_space(8.0);
            for event in &self.output_event_history {
                ui.label(format!("{:?}", event));
            }
        });

        if !frame.is_web() {
            ui.separator();
            if ui.button("Quit").clicked() {
                frame.quit();
            }
        }
    }

    fn pixels_per_point_ui(
        &mut self,
        ui: &mut egui::Ui,
        info: &epi::IntegrationInfo,
    ) -> Option<f32> {
        self.pixels_per_point = self
            .pixels_per_point
            .or(info.native_pixels_per_point)
            .or_else(|| Some(ui.ctx().pixels_per_point()));

        let pixels_per_point = self.pixels_per_point.as_mut()?;

        ui.horizontal(|ui| {
            ui.spacing_mut().slider_width = 90.0;
            ui.add(
                egui::Slider::new(pixels_per_point, 0.5..=5.0)
                    .logarithmic(true)
                    .clamp_to_range(true)
                    .text("Scale"),
            )
            .on_hover_text("Physical pixels per point.");
            if let Some(native_pixels_per_point) = info.native_pixels_per_point {
                let button = egui::Button::new("Reset")
                    .enabled(*pixels_per_point != native_pixels_per_point);
                if ui
                    .add(button)
                    .on_hover_text(format!(
                        "Reset scale to native value ({:.1})",
                        native_pixels_per_point
                    ))
                    .clicked()
                {
                    *pixels_per_point = native_pixels_per_point;
                }
            }
        });

        // We wait until mouse release to activate:
        if ui.ctx().is_using_pointer() {
            None
        } else {
            Some(*pixels_per_point)
        }
    }

    fn run_mode_ui(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            let run_mode = &mut self.run_mode;
            ui.label("Mode:");
            ui.radio_value(run_mode, RunMode::Continuous, "Continuous")
                .on_hover_text("Repaint everything each frame");
            ui.radio_value(run_mode, RunMode::Reactive, "Reactive")
                .on_hover_text("Repaint when there are animations or input (e.g. mouse movement)");
        });

        if self.run_mode == RunMode::Continuous {
            ui.label(format!(
                "Repainting the UI each frame. FPS: {:.1}",
                self.frame_history.fps()
            ));
        } else {
            ui.label("Only running UI code when there are animations or input");
        }
    }
}

// ----------------------------------------------------------------------------

#[cfg_attr(feature = "persistence", derive(serde::Deserialize, serde::Serialize))]
struct EguiWindows {
    // egui stuff:
    settings: bool,
    inspection: bool,
    memory: bool,
}

impl Default for EguiWindows {
    fn default() -> Self {
        EguiWindows::none()
    }
}

impl EguiWindows {
    fn none() -> Self {
        Self {
            settings: false,
            inspection: false,
            memory: false,
        }
    }

    fn checkboxes(&mut self, ui: &mut egui::Ui) {
        let Self {
            settings,
            inspection,
            memory,
        } = self;

        ui.checkbox(settings, "🔧 Settings");
        ui.checkbox(inspection, "🔍 Inspection");
        ui.checkbox(memory, "📝 Memory");
    }

    fn windows(&mut self, ctx: &egui::CtxRef) {
        let Self {
            settings,
            inspection,
            memory,
        } = self;

        egui::Window::new("🔧 Settings")
            .open(settings)
            .scroll(true)
            .show(ctx, |ui| {
                ctx.settings_ui(ui);
            });

        egui::Window::new("🔍 Inspection")
            .open(inspection)
            .scroll(true)
            .show(ctx, |ui| {
                ctx.inspection_ui(ui);
            });

        egui::Window::new("📝 Memory")
            .open(memory)
            .resizable(false)
            .show(ctx, |ui| {
                ctx.memory_ui(ui);
            });
    }
}
