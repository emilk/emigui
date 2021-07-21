//! [`egui`] bindings for [`glium`](https://github.com/glium/glium).
//!
//! The main type you want to use is [`EguiGlium`].
//!
//! This library is an [`epi`] backend.
//! If you are writing an app, you may want to look at [`eframe`](https://docs.rs/eframe) instead.

// Forbid warnings in release builds:
#![cfg_attr(not(debug_assertions), deny(warnings))]
// Disabled so we can support rust 1.51:
// #![deny(
//     rustdoc::broken_intra_doc_links,
//     rustdoc::invalid_codeblock_attributes,
//     rustdoc::missing_crate_level_docs,
//     rustdoc::private_intra_doc_links
// )]
#![forbid(unsafe_code)]
#![warn(clippy::all, rust_2018_idioms)]
#![allow(clippy::manual_range_contains, clippy::single_match)]

mod backend;
#[cfg(feature = "http")]
pub mod http;
mod painter;
#[cfg(feature = "persistence")]
pub mod persistence;
pub mod screen_reader;
pub mod window_settings;

pub use backend::*;
pub use painter::Painter;

pub use epi::NativeOptions;

use {
    copypasta::ClipboardProvider,
    egui::*,
    glium::glutin::{
        self,
        event::{Force, VirtualKeyCode},
    },
    std::hash::{Hash, Hasher},
};

pub use copypasta::ClipboardContext; // TODO: remove

pub struct GliumInputState {
    pub pointer_pos_in_points: Option<Pos2>,
    pub raw: egui::RawInput,
}

impl GliumInputState {
    pub fn from_pixels_per_point(pixels_per_point: f32) -> Self {
        Self {
            pointer_pos_in_points: Default::default(),
            raw: egui::RawInput {
                pixels_per_point: Some(pixels_per_point),
                ..Default::default()
            },
        }
    }
}

/// Helper: checks for Alt-F4 (windows/linux) or Cmd-Q (Mac)
pub fn is_quit_shortcut(
    input_state: &GliumInputState,
    input: &glium::glutin::event::KeyboardInput,
) -> bool {
    if cfg!(target_os = "macos") {
        input.state == glutin::event::ElementState::Pressed
            && input_state.raw.modifiers.mac_cmd
            && input.virtual_keycode == Some(VirtualKeyCode::Q)
    } else {
        input.state == glutin::event::ElementState::Pressed
            && input_state.raw.modifiers.alt
            && input.virtual_keycode == Some(VirtualKeyCode::F4)
    }
}

/// Is this a close event or a Cmd-Q/Alt-F4 keyboard command?
pub fn is_quit_event(
    input_state: &GliumInputState,
    event: &glutin::event::WindowEvent<'_>,
) -> bool {
    use glutin::event::WindowEvent;
    match event {
        WindowEvent::CloseRequested | WindowEvent::Destroyed => true,
        WindowEvent::KeyboardInput { input, .. } => is_quit_shortcut(input_state, input),
        _ => false,
    }
}

pub fn input_to_egui(
    pixels_per_point: f32,
    event: &glutin::event::WindowEvent<'_>,
    clipboard: Option<&mut ClipboardContext>,
    input_state: &mut GliumInputState,
) {
    use glutin::event::WindowEvent;
    match event {
        WindowEvent::ScaleFactorChanged { scale_factor, .. } => {
            input_state.raw.pixels_per_point = Some(*scale_factor as f32);
        }
        WindowEvent::MouseInput { state, button, .. } => {
            if let Some(pos_in_points) = input_state.pointer_pos_in_points {
                if let Some(button) = translate_mouse_button(*button) {
                    input_state.raw.events.push(egui::Event::PointerButton {
                        pos: pos_in_points,
                        button,
                        pressed: *state == glutin::event::ElementState::Pressed,
                        modifiers: input_state.raw.modifiers,
                    });
                }
            }
        }
        WindowEvent::CursorMoved {
            position: pos_in_pixels,
            ..
        } => {
            let pos_in_points = pos2(
                pos_in_pixels.x as f32 / pixels_per_point,
                pos_in_pixels.y as f32 / pixels_per_point,
            );
            input_state.pointer_pos_in_points = Some(pos_in_points);
            input_state
                .raw
                .events
                .push(egui::Event::PointerMoved(pos_in_points));
        }
        WindowEvent::CursorLeft { .. } => {
            input_state.pointer_pos_in_points = None;
            input_state.raw.events.push(egui::Event::PointerGone);
        }
        WindowEvent::ReceivedCharacter(ch) => {
            if is_printable_char(*ch)
                && !input_state.raw.modifiers.ctrl
                && !input_state.raw.modifiers.mac_cmd
            {
                input_state.raw.events.push(Event::Text(ch.to_string()));
            }
        }
        WindowEvent::KeyboardInput { input, .. } => {
            if let Some(keycode) = input.virtual_keycode {
                let pressed = input.state == glutin::event::ElementState::Pressed;

                // We could also use `WindowEvent::ModifiersChanged` instead, I guess.
                if matches!(keycode, VirtualKeyCode::LAlt | VirtualKeyCode::RAlt) {
                    input_state.raw.modifiers.alt = pressed;
                }
                if matches!(keycode, VirtualKeyCode::LControl | VirtualKeyCode::RControl) {
                    input_state.raw.modifiers.ctrl = pressed;
                    if !cfg!(target_os = "macos") {
                        input_state.raw.modifiers.command = pressed;
                    }
                }
                if matches!(keycode, VirtualKeyCode::LShift | VirtualKeyCode::RShift) {
                    input_state.raw.modifiers.shift = pressed;
                }
                if cfg!(target_os = "macos")
                    && matches!(keycode, VirtualKeyCode::LWin | VirtualKeyCode::RWin)
                {
                    input_state.raw.modifiers.mac_cmd = pressed;
                    input_state.raw.modifiers.command = pressed;
                }

                if pressed {
                    // VirtualKeyCode::Paste etc in winit are broken/untrustworthy,
                    // so we detect these things manually:
                    if is_cut_command(input_state.raw.modifiers, keycode) {
                        input_state.raw.events.push(Event::Cut);
                    } else if is_copy_command(input_state.raw.modifiers, keycode) {
                        input_state.raw.events.push(Event::Copy);
                    } else if is_paste_command(input_state.raw.modifiers, keycode) {
                        if let Some(clipboard) = clipboard {
                            match clipboard.get_contents() {
                                Ok(contents) => {
                                    input_state.raw.events.push(Event::Text(contents));
                                }
                                Err(err) => {
                                    eprintln!("Paste error: {}", err);
                                }
                            }
                        }
                    }
                }

                if let Some(key) = translate_virtual_key_code(keycode) {
                    input_state.raw.events.push(Event::Key {
                        key,
                        pressed,
                        modifiers: input_state.raw.modifiers,
                    });
                }
            }
        }
        WindowEvent::Focused(_) => {
            // We will not be given a KeyboardInput event when the modifiers are released while
            // the window does not have focus. Unset all modifier state to be safe.
            input_state.raw.modifiers = Modifiers::default();
        }
        WindowEvent::MouseWheel { delta, .. } => {
            let mut delta = match *delta {
                glutin::event::MouseScrollDelta::LineDelta(x, y) => {
                    let line_height = 8.0; // magic value!
                    vec2(x, y) * line_height
                }
                glutin::event::MouseScrollDelta::PixelDelta(delta) => {
                    vec2(delta.x as f32, delta.y as f32) / pixels_per_point
                }
            };

            // Various hacks for https://github.com/emilk/egui/issues/356
            if cfg!(target_os = "macos") {
                // This is still buggy in winit despite
                // https://github.com/rust-windowing/winit/issues/1695 being closed
                delta.x *= -1.0;
            }
            if cfg!(target_os = "linux") {
                delta.y *= -1.0;
            }

            if input_state.raw.modifiers.ctrl || input_state.raw.modifiers.command {
                // Treat as zoom instead:
                input_state.raw.zoom_delta *= (delta.y / 200.0).exp();
            } else {
                input_state.raw.scroll_delta += delta;
            }
        }
        WindowEvent::TouchpadPressure {
            // device_id,
            // pressure,
            // stage,
            ..
        } => {
            // TODO
        }
        WindowEvent::Touch(touch) => {
            let pixels_per_point_recip = 1. / pixels_per_point;
            let mut hasher = std::collections::hash_map::DefaultHasher::new();
            touch.device_id.hash(&mut hasher);
            input_state.raw.events.push(Event::Touch {
                device_id: TouchDeviceId(hasher.finish()),
                id: TouchId::from(touch.id),
                phase: match touch.phase {
                    glutin::event::TouchPhase::Started => egui::TouchPhase::Start,
                    glutin::event::TouchPhase::Moved => egui::TouchPhase::Move,
                    glutin::event::TouchPhase::Ended => egui::TouchPhase::End,
                    glutin::event::TouchPhase::Cancelled => egui::TouchPhase::Cancel,
                },
                pos: pos2(touch.location.x as f32 * pixels_per_point_recip,
                    touch.location.y as f32 * pixels_per_point_recip),
                force: match touch.force {
                    Some(Force::Normalized(force)) => force as f32,
                    Some(Force::Calibrated {
                        force,
                        max_possible_force,
                        ..
                    }) => (force / max_possible_force) as f32,
                    None => 0_f32,
                },
            });
        }
        _ => {
            // dbg!(event);
        }
    }
}

/// Glium sends special keys (backspace, delete, F1, ...) as characters.
/// Ignore those.
/// We also ignore '\r', '\n', '\t'.
/// Newlines are handled by the `Key::Enter` event.
fn is_printable_char(chr: char) -> bool {
    let is_in_private_use_area = '\u{e000}' <= chr && chr <= '\u{f8ff}'
        || '\u{f0000}' <= chr && chr <= '\u{ffffd}'
        || '\u{100000}' <= chr && chr <= '\u{10fffd}';

    !is_in_private_use_area && !chr.is_ascii_control()
}

fn is_cut_command(modifiers: egui::Modifiers, keycode: VirtualKeyCode) -> bool {
    (modifiers.command && keycode == VirtualKeyCode::X)
        || (cfg!(target_os = "windows") && modifiers.shift && keycode == VirtualKeyCode::Delete)
}

fn is_copy_command(modifiers: egui::Modifiers, keycode: VirtualKeyCode) -> bool {
    (modifiers.command && keycode == VirtualKeyCode::C)
        || (cfg!(target_os = "windows") && modifiers.ctrl && keycode == VirtualKeyCode::Insert)
}

fn is_paste_command(modifiers: egui::Modifiers, keycode: VirtualKeyCode) -> bool {
    (modifiers.command && keycode == VirtualKeyCode::V)
        || (cfg!(target_os = "windows") && modifiers.shift && keycode == VirtualKeyCode::Insert)
}

pub fn translate_mouse_button(button: glutin::event::MouseButton) -> Option<egui::PointerButton> {
    match button {
        glutin::event::MouseButton::Left => Some(egui::PointerButton::Primary),
        glutin::event::MouseButton::Right => Some(egui::PointerButton::Secondary),
        glutin::event::MouseButton::Middle => Some(egui::PointerButton::Middle),
        _ => None,
    }
}

pub fn translate_virtual_key_code(key: VirtualKeyCode) -> Option<egui::Key> {
    use VirtualKeyCode::*;

    Some(match key {
        Down => Key::ArrowDown,
        Left => Key::ArrowLeft,
        Right => Key::ArrowRight,
        Up => Key::ArrowUp,

        Escape => Key::Escape,
        Tab => Key::Tab,
        Back => Key::Backspace,
        Return => Key::Enter,
        Space => Key::Space,

        Insert => Key::Insert,
        Delete => Key::Delete,
        Home => Key::Home,
        End => Key::End,
        PageUp => Key::PageUp,
        PageDown => Key::PageDown,

        Key0 | Numpad0 => Key::Num0,
        Key1 | Numpad1 => Key::Num1,
        Key2 | Numpad2 => Key::Num2,
        Key3 | Numpad3 => Key::Num3,
        Key4 | Numpad4 => Key::Num4,
        Key5 | Numpad5 => Key::Num5,
        Key6 | Numpad6 => Key::Num6,
        Key7 | Numpad7 => Key::Num7,
        Key8 | Numpad8 => Key::Num8,
        Key9 | Numpad9 => Key::Num9,

        A => Key::A,
        B => Key::B,
        C => Key::C,
        D => Key::D,
        E => Key::E,
        F => Key::F,
        G => Key::G,
        H => Key::H,
        I => Key::I,
        J => Key::J,
        K => Key::K,
        L => Key::L,
        M => Key::M,
        N => Key::N,
        O => Key::O,
        P => Key::P,
        Q => Key::Q,
        R => Key::R,
        S => Key::S,
        T => Key::T,
        U => Key::U,
        V => Key::V,
        W => Key::W,
        X => Key::X,
        Y => Key::Y,
        Z => Key::Z,

        _ => {
            return None;
        }
    })
}

fn translate_cursor(cursor_icon: egui::CursorIcon) -> Option<glutin::window::CursorIcon> {
    match cursor_icon {
        CursorIcon::None => None,

        CursorIcon::Alias => Some(glutin::window::CursorIcon::Alias),
        CursorIcon::AllScroll => Some(glutin::window::CursorIcon::AllScroll),
        CursorIcon::Cell => Some(glutin::window::CursorIcon::Cell),
        CursorIcon::ContextMenu => Some(glutin::window::CursorIcon::ContextMenu),
        CursorIcon::Copy => Some(glutin::window::CursorIcon::Copy),
        CursorIcon::Crosshair => Some(glutin::window::CursorIcon::Crosshair),
        CursorIcon::Default => Some(glutin::window::CursorIcon::Default),
        CursorIcon::Grab => Some(glutin::window::CursorIcon::Grab),
        CursorIcon::Grabbing => Some(glutin::window::CursorIcon::Grabbing),
        CursorIcon::Help => Some(glutin::window::CursorIcon::Help),
        CursorIcon::Move => Some(glutin::window::CursorIcon::Move),
        CursorIcon::NoDrop => Some(glutin::window::CursorIcon::NoDrop),
        CursorIcon::NotAllowed => Some(glutin::window::CursorIcon::NotAllowed),
        CursorIcon::PointingHand => Some(glutin::window::CursorIcon::Hand),
        CursorIcon::Progress => Some(glutin::window::CursorIcon::Progress),
        CursorIcon::ResizeHorizontal => Some(glutin::window::CursorIcon::EwResize),
        CursorIcon::ResizeNeSw => Some(glutin::window::CursorIcon::NeswResize),
        CursorIcon::ResizeNwSe => Some(glutin::window::CursorIcon::NwseResize),
        CursorIcon::ResizeVertical => Some(glutin::window::CursorIcon::NsResize),
        CursorIcon::Text => Some(glutin::window::CursorIcon::Text),
        CursorIcon::VerticalText => Some(glutin::window::CursorIcon::VerticalText),
        CursorIcon::Wait => Some(glutin::window::CursorIcon::Wait),
        CursorIcon::ZoomIn => Some(glutin::window::CursorIcon::ZoomIn),
        CursorIcon::ZoomOut => Some(glutin::window::CursorIcon::ZoomOut),
    }
}

fn set_cursor_icon(display: &glium::backend::glutin::Display, cursor_icon: egui::CursorIcon) {
    if let Some(cursor_icon) = translate_cursor(cursor_icon) {
        display.gl_window().window().set_cursor_visible(true);
        display.gl_window().window().set_cursor_icon(cursor_icon);
    } else {
        display.gl_window().window().set_cursor_visible(false);
    }
}

pub fn handle_output(
    output: egui::Output,
    clipboard: Option<&mut ClipboardContext>,
    display: &glium::Display,
) {
    if let Some(open) = output.open_url {
        if let Err(err) = webbrowser::open(&open.url) {
            eprintln!("Failed to open url: {}", err);
        }
    }

    if !output.copied_text.is_empty() {
        if let Some(clipboard) = clipboard {
            if let Err(err) = clipboard.set_contents(output.copied_text) {
                eprintln!("Copy/Cut error: {}", err);
            }
        }
    }

    if let Some(egui::Pos2 { x, y }) = output.text_cursor_pos {
        display
            .gl_window()
            .window()
            .set_ime_position(glium::glutin::dpi::LogicalPosition { x, y })
    }
}

pub fn init_clipboard() -> Option<ClipboardContext> {
    match ClipboardContext::new() {
        Ok(clipboard) => Some(clipboard),
        Err(err) => {
            eprintln!("Failed to initialize clipboard: {}", err);
            None
        }
    }
}

// ----------------------------------------------------------------------------

/// Time of day as seconds since midnight. Used for clock in demo app.
pub fn seconds_since_midnight() -> Option<f64> {
    #[cfg(feature = "time")]
    {
        use chrono::Timelike;
        let time = chrono::Local::now().time();
        let seconds_since_midnight =
            time.num_seconds_from_midnight() as f64 + 1e-9 * (time.nanosecond() as f64);
        Some(seconds_since_midnight)
    }
    #[cfg(not(feature = "time"))]
    None
}

pub fn screen_size_in_pixels(display: &glium::Display) -> Vec2 {
    let (width_in_pixels, height_in_pixels) = display.get_framebuffer_dimensions();
    vec2(width_in_pixels as f32, height_in_pixels as f32)
}

pub fn native_pixels_per_point(display: &glium::Display) -> f32 {
    display.gl_window().window().scale_factor() as f32
}

// ----------------------------------------------------------------------------

/// Use [`egui`] from a [`glium`] app.
pub struct EguiGlium {
    egui_ctx: egui::CtxRef,
    start_time: std::time::Instant,
    clipboard: Option<crate::ClipboardContext>,
    input_state: crate::GliumInputState,
    painter: crate::Painter,
    current_cursor_icon: egui::CursorIcon,
    screen_reader: crate::screen_reader::ScreenReader,
}

impl EguiGlium {
    pub fn new(display: &glium::Display) -> Self {
        Self {
            egui_ctx: Default::default(),
            start_time: std::time::Instant::now(),
            clipboard: crate::init_clipboard(),
            input_state: crate::GliumInputState::from_pixels_per_point(
                crate::native_pixels_per_point(display),
            ),
            painter: crate::Painter::new(display),
            current_cursor_icon: egui::CursorIcon::Default,
            screen_reader: crate::screen_reader::ScreenReader::default(),
        }
    }

    pub fn ctx(&self) -> &egui::CtxRef {
        &self.egui_ctx
    }

    pub fn ctx_and_painter_mut(&mut self) -> (&egui::CtxRef, &mut crate::Painter) {
        (&self.egui_ctx, &mut self.painter)
    }

    pub fn on_event(&mut self, event: &glium::glutin::event::WindowEvent<'_>) {
        crate::input_to_egui(
            self.egui_ctx.pixels_per_point(),
            &event,
            self.clipboard.as_mut(),
            &mut self.input_state,
        );
    }

    /// Is this a close event or a Cmd-Q/Alt-F4 keyboard command?
    pub fn is_quit_event(&self, event: &glutin::event::WindowEvent<'_>) -> bool {
        crate::is_quit_event(&self.input_state, event)
    }

    pub fn begin_frame(&mut self, display: &glium::Display) {
        let pixels_per_point = self
            .input_state
            .raw
            .pixels_per_point
            .unwrap_or_else(|| self.egui_ctx.pixels_per_point());

        self.input_state.raw.time = Some(self.start_time.elapsed().as_nanos() as f64 * 1e-9);

        // On Windows, a minimized window will have 0 width and height.
        // See: https://github.com/rust-windowing/winit/issues/208
        // This solves an issue where egui window positions would be changed when minimizing on Windows.
        let screen_size = screen_size_in_pixels(display);
        self.input_state.raw.screen_rect = if screen_size.x > 0.0 && screen_size.y > 0.0 {
            Some(Rect::from_min_size(
                Default::default(),
                screen_size / pixels_per_point,
            ))
        } else {
            None
        };

        self.egui_ctx.begin_frame(self.input_state.raw.take());
    }

    /// Returns `needs_repaint` and shapes to draw.
    pub fn end_frame(
        &mut self,
        display: &glium::Display,
    ) -> (bool, Vec<egui::epaint::ClippedShape>) {
        let (egui_output, shapes) = self.egui_ctx.end_frame();

        if self.egui_ctx.memory().options.screen_reader {
            self.screen_reader.speak(&egui_output.events_description());
        }
        if self.current_cursor_icon != egui_output.cursor_icon {
            // call only when changed to prevent flickering near frame boundary
            // when Windows OS tries to control cursor icon for window resizing
            set_cursor_icon(display, egui_output.cursor_icon);
            self.current_cursor_icon = egui_output.cursor_icon;
        }

        let needs_repaint = egui_output.needs_repaint;

        handle_output(egui_output, self.clipboard.as_mut(), display);

        (needs_repaint, shapes)
    }

    pub fn paint(
        &mut self,
        display: &glium::Display,
        target: &mut glium::Frame,
        shapes: Vec<egui::epaint::ClippedShape>,
    ) {
        let clipped_meshes = self.egui_ctx.tessellate(shapes);
        self.painter.paint_meshes(
            display,
            target,
            self.egui_ctx.pixels_per_point(),
            clipped_meshes,
            &self.egui_ctx.texture(),
        );
    }
}
