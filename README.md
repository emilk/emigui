# 🖌 egui: an easy-to-use GUI in pure Rust

[![Latest version](https://img.shields.io/crates/v/egui.svg)](https://crates.io/crates/egui)
[![Documentation](https://docs.rs/egui/badge.svg)](https://docs.rs/egui)
[![unsafe forbidden](https://img.shields.io/badge/unsafe-forbidden-success.svg)](https://github.com/rust-secure-code/safety-dance/)
[![Build Status](https://github.com/emilk/egui/workflows/CI/badge.svg)](https://github.com/emilk/egui/actions?workflow=CI)
![MIT](https://img.shields.io/badge/license-MIT-blue.svg)
![Apache](https://img.shields.io/badge/license-Apache-blue.svg)


egui is a simple, fast, and highly portable immediate mode GUI library for Rust. egui runs on the web, natively, and [in your favorite game engine](#integrations) (or will soon).

egui aims to be the easiest-to-use Rust GUI library, and the simplest way to make a web app in Rust.

egui can be used anywhere you can draw textured triangles, which means you can easily integrate it into your game engine of choice.

Sections:

* [Quick start](#quick-start)
* [Demo](#demo)
* [Goals](#goals)
* [Who is egui for?](#who-is-egui-for)
* [State / features](#state)
* [How it works](#how-it-works)
* [Integrations](#integrations)
* [Why immediate mode](#why-immediate-mode)
* [FAQ](#faq)
* [Other](#other)

## Quick start

If you just want to write a GUI application in Rust (for the web or for native), go to <https://github.com/emilk/egui_template/> and follow the instructions there!

If you want to integrate egui into an existing engine, go to the [Integrations](#integrations) section.

If you have questions, use [Discussions](https://github.com/emilk/egui/discussions). If you want to contribute to egui, please read the [Contributing Guidelines](https://github.com/emilk/egui/blob/master/CONTRIBUTING.md)

## Demo

[Click to run egui web demo](https://emilk.github.io/egui/index.html) (works in any browser with WASM and WebGL support).

To test the demo app locally, run `cargo run --release -p egui_demo_app`.

The native backend is currently using [`glium`](https://github.com/glium/glium) ([though there are plans to change that](https://github.com/emilk/egui/issues/93)) and should work out-of-the-box on Mac and Windows, but on Linux you need to first run:

`sudo apt-get install libxcb-render0-dev libxcb-shape0-dev libxcb-xfixes0-dev libspeechd-dev libxkbcommon-dev`

On Fedora Rawhide you need to run `dnf install clang clang-devel clang-tools-extra speech-dispatcher-devel`

**NOTE**: This is just for the demo app - egui itself is completely platform agnostic!

### Example

``` rust
ui.heading("My egui Application");
ui.horizontal(|ui| {
    ui.label("Your name: ");
    ui.text_edit_singleline(&mut name);
});
ui.add(egui::Slider::new(&mut age, 0..=120).text("age"));
if ui.button("Click each year").clicked() {
    age += 1;
}
ui.label(format!("Hello '{}', age {}", name, age));
```

<img src="media/demo.gif">

## Goals

* The easiest to use GUI library
* Responsive: target 60 Hz in debug build
* Friendly: difficult to make mistakes, and shouldn't panic
* Portable: the same code works on the web and as a native app
* Easy to integrate into any environment
* A simple 2D graphics API for custom painting ([`epaint`](https://docs.rs/epaint)).
* No callbacks
* Pure immediate mode
* Extensible: [easy to write your own widgets for egui](https://github.com/emilk/egui/blob/master/egui_demo_lib/src/apps/demo/toggle_switch.rs)
* Modular: You should be able to use small parts of egui and combine them in new ways
* Safe: there is no `unsafe` code in egui
* Minimal dependencies: [`ahash`](https://crates.io/crates/ahash) [`atomic_refcell`](https://crates.io/crates/atomic_refcell) [`ordered-float`](https://crates.io/crates/ordered-float) [`ab_glyph`](https://crates.io/crates/ab_glyph).

egui is *not* a framework. egui is a library you call into, not an environment you program for.

**NOTE**: egui does not claim to have reached all these goals yet! egui is still work in progress.

### Non-goals

* Become the most powerful GUI library
* Native looking interface
* Advanced and flexible layouts (that's fundamentally incompatible with immediate mode)

## Who is egui for?

egui aims to be the best choice when you want a simple way to create a GUI, or you want to add a GUI to a game engine.

If you are not using Rust, egui is not for you. If you want a GUI that looks native, egui is not for you. If you want something that doesn't break when you upgrade it, egui isn't for you (yet).

But if you are writing something interactive in Rust that needs a simple GUI, egui may be for you.

### egui vs Dear ImGui

The obvious alternative to egui is [`imgui-rs`](https://github.com/Gekkio/imgui-rs), the Rust wrapper around the C++ library [Dear ImGui](https://github.com/ocornut/imgui). Dear ImGui is a great library, which a lot more features and polish compared to egui. However, egui provides some benefits for Rust users:

* egui is pure Rust
* egui is easily compiled to WASM
* egui lets you use native Rust String types (`imgui-rs` forces you to use annoying macros and wrappers for zero-terminated strings)
* [Writing your own widgets in egui is simple](https://github.com/emilk/egui/blob/master/egui_demo_lib/src/apps/demo/toggle_switch.rs)

egui also tries to improve your experience in other small ways:

* Windows are automatically sized based on their contents
* Windows are automatically positioned to not overlap with each other
* Some subtle animations make egui come alive

So in summary:

* egui: pure Rust, new, exciting, work in progress
* Dear ImGui: feature rich, well tested, cumbersome Rust integration

## State

egui is in active development. It works well for what it does, but it lacks many features and the interfaces are still in flux. New releases will have breaking changes.

### Features

* Widgets: label, text button, hyperlink, checkbox, radio button, slider, draggable value, text editing, combo box, color picker
* Layouts: horizontal, vertical, columns, automatic wrapping
* Text editing: multiline, copy/paste, undo, emoji supports
* Windows: move, resize, name, minimize and close. Automatically sized and positioned.
* Regions: resizing, vertical scrolling, collapsing headers (sections)
* Rendering: Anti-aliased rendering of lines, circles, text and convex polygons.
* Tooltips on hover
* More

<img src="media/widget_gallery.gif" width="50%">

Light Theme:

<img src="media/light_theme.png" width="50%">

## How it works

Loop:

* Gather input (mouse, touches, keyboard, screen size, etc) and give it to egui
* Run application code (Immediate Mode GUI)
* Tell egui to tessellate the frame graphics to a triangle mesh
* Render the triangle mesh with your favorite graphics API (see [OpenGL example](https://github.com/emilk/egui/blob/master/egui_glium/src/painter.rs)) or use `eframe`, the egui framework crate.

## Integrations

egui is build to be easy to integrate into any existing game engine or platform you are working on.
egui itself doesn't know or care on what OS it is running or how to render things to the screen - that is the job of the egui integration.
The integration needs to do two things:

* **IO**: Supply egui with input (mouse position, keyboard presses, ...) and handle egui output (cursor changes, copy-paste integration, ...).
* **Painting**: Render the textured triangles that egui outputs.

### Official

I maintain two official egui integrations:

* [egui_web](https://crates.io/crates/egui_web) for making a web app. Compiles to WASM, renders with WebGL. [Click to run the egui demo](https://emilk.github.io/egui/index.html).
* [egui_glium](https://crates.io/crates/egui_glium) for compiling native apps with [Glium](https://github.com/glium/glium).

The same code can be compiled to a native app or a web app.

### 3rd party

* [`bevy_egui`](https://github.com/mvlabat/bevy_egui) for [the Bevy game engine](https://bevyengine.org/).
* [`egui-miniquad`](https://github.com/not-fl3/egui-miniquad): backend for [Miniquad](https://github.com/not-fl3/miniquad).
* [`egui-macroquad`](https://github.com/optozorax/egui-macroquad): backend for [macroquad](https://github.com/not-fl3/macroquad)
* [`egui_sdl2_gl`](https://crates.io/crates/egui_sdl2_gl) for [SDL2](https://crates.io/crates/sdl2)
* [`egui_vulkano`](https://github.com/derivator/egui_vulkano): backend for [Vulkano](https://github.com/vulkano-rs/vulkano).
* [`egui_winit_vulkano`](https://github.com/hakolao/egui_winit_vulkano): backend for [Vulkano](https://github.com/vulkano-rs/vulkano).
* [`egui_winit_ash_vk_mem`](https://crates.io/crates/egui_winit_ash_vk_mem) for for [winit](https://github.com/rust-windowing/winit), [ash](https://github.com/MaikKlein/ash) and [vk_mem](https://github.com/gwihlidal/vk-mem-rs).
* [`egui_winit_platform`](https://github.com/hasenbanck/egui_winit_platform) provides bindings between [winit](https://crates.io/crates/winit) and egui. It only provides the first half of an egui integration (IO). Painting can be done with e.g. [egui_wgpu_backend](https://crates.io/crates/egui_wgpu_backend).
* For [`wgpu`](https://crates.io/crates/wgpu) (WebGPU API):
  * [`egui_wgpu_backend`](https://crates.io/crates/egui_wgpu_backend) with [example code](https://github.com/hasenbanck/egui_example)
  * Alternative: [`egui_winit_wgpu`](https://github.com/Gonkalbell/egui_winit_wgpu) (not available to crates.io)
* [`nannou_egui`](https://github.com/AlexEne/nannou_egui): backend for [nannou](https://nannou.cc).

### Writing your own egui integration

You need to collect [`egui::RawInput`](https://docs.rs/egui/latest/egui/struct.RawInput.html), paint [`egui::ClippedMesh`](https://docs.rs/epaint/):es and handle [`egui::Output`](https://docs.rs/egui/latest/egui/struct.Output.html). The basic structure is this:

``` rust
let mut egui_ctx = egui::Context::new();

// Game loop:
loop {
    let raw_input: egui::RawInput = my_integration.gather_input();
    egui_ctx.begin_frame(raw_input);
    my_app.ui(&mut egui_ctx); // add panels, windows and widgets to `egui_ctx` here
    let (output, shapes) = egui_ctx.end_frame();
    let clipped_meshes = egui_ctx.tessellate(shapes); // create triangles to paint
    my_integration.paint(clipped_meshes);
    my_integration.set_cursor_icon(output.cursor_icon);
    // Also see `egui::Output` for more
}
```

For a reference OpenGL backend, see [the `egui_glium` painter](https://github.com/emilk/egui/blob/master/egui_glium/src/painter.rs) or [the `egui_web` `WebGL` painter](https://github.com/emilk/egui/blob/master/egui_web/src/webgl1.rs).

### Debugging your integration

#### Things look jagged

* Turn off backface culling.

#### My text is blurry

* Make sure you set the proper `pixels_per_point` in the input to egui.
* Make sure the texture sampler is not off by half a pixel. Try nearest-neighbor sampler to check.

#### My windows are too transparent or too dark

* egui uses premultiplied alpha, so make sure your blending function is `(ONE, ONE_MINUS_SRC_ALPHA)`.
* Make sure your texture sampler is clamped (`GL_CLAMP_TO_EDGE`).
* Use an sRGBA-aware texture if available (e.g. `GL_SRGB8_ALPHA8`).
  * Otherwise: remember to decode gamma in the fragment shader.
* Decode the gamma of the incoming vertex colors in your vertex shader.
* Turn on sRGBA/linear framebuffer if available (`GL_FRAMEBUFFER_SRGB`).
  * Otherwise: gamma-encode the colors before you write them again.


## Why immediate mode

`egui` is an [immediate mode GUI library](https://en.wikipedia.org/wiki/Immediate_mode_GUI), as opposed to a *retained mode* GUI library. The difference between retained mode and immediate mode is best illustrated with the example of a button: In a retained GUI you create a button, add it to some UI and install some on-click handler (callback). The button is retained in the UI, and to change the text on it you need to store some sort of reference to it. By contrast, in immediate mode you show the button and interact with it immediately, and you do so every frame (e.g. 60 times per second). This means there is no need for any on-click handler, nor to store any reference to it. In `egui` this looks like this: `if ui.button("Save file").clicked() { save(file); }`.

There are advantages and disadvantages to both systems.

The short of it is this: immediate mode GUI libraries are easier to use, but less powerful.

### Advantages of immediate mode
#### Usability
The main advantage of immediate mode is that the application code becomes vastly simpler:

* You never need to have any on-click handlers and callbacks that disrupts your code flow.
* You don't have to worry about a lingering callback calling something that is gone.
* Your GUI code can easily live in a simple function (no need for an object just for the UI).
* You don't have to worry about app state and GUI state being out-of-sync (i.e. the GUI showing something outdated), because the GUI isn't storing any state - it is showing the latest state *immediately*.

In other words, a whole lot of code, complexity and bugs are gone, and you can focus your time on something more interesting than writing GUI code.

### Disadvantages of immediate mode

#### Layout
The main disadvantage of immediate mode is it makes layout more difficult. Say you want to show a small dialog window in the center of the screen. To position the window correctly the GUI library must first know the size of it. To know the size of the window the GUI library must first layout the contents of the window. In retained mode this is easy: the GUI library does the window layout, positions the window, then checks for interaction ("was the OK button clicked?").

In immediate mode you run into a paradox: to know the size of the window, we must do the layout, but the layout code also checks for interaction ("was the OK button clicked?") and so it needs to know the window position *before* showing the window contents. This means we must decide where to show the window *before* we know its size!

This is a fundamental shortcoming of immediate mode GUIs, and any attempt to resolve it comes with its own downsides.

One workaround is to store the size and use it the next frame. This produces a frame-delay for the correct layout, producing occasional flickering the first frame something shows up. `egui` does this for some things such as windows and grid layouts.

You can also call the layout code twice (once to get the size, once to do the interaction), but that is not only more expensive, it's also complex to implement, and in some cases twice is not enough. `egui` never does this.

For "atomic" widgets (e.g. a button) `egui` knows the size before showing it, so centering buttons, labels etc is possible in `egui` without any special workarounds.

#### CPU usage
Since an immediate mode GUI does a full layout each frame, the layout code needs to be quick. If you have a very complex GUI this can tax the CPU. In particular, having a very large UI in a scroll area (with very long scrollback) can be slow, as the content needs to be layed out each frame.

If you design the GUI with this in mind and refrain from huge scroll areas (or only lay out the part that is in view) then the performance hit is generally pretty small. For most cases you can expect `egui` to take up 1-2 ms per frame, but `egui` still has a lot of room for optimization (it's not something I've focused on yet). You can also set up `egui` to only repaint when there is interaction (e.g. mouse movement).

If your GUI is highly interactive, then immediate mode may actually be more performant compared to retained mode. Go to any web page and resize the browser window, and you'll notice that the browser is very slow to do the layout and eats a lot of CPU doing it. Resize a window in `egui` by contrast, and you'll get smooth 60 FPS at no extra CPU cost.


#### IDs
There are some GUI state that you want the GUI library to retain, even in an immediate mode library such as `egui`. This includes position and sizes of windows and how far the user has scrolled in some UI. In these cases you need to provide `egui` with a seed of a unique identifier (unique within the parent UI). For instance: by default `egui` uses the window titles as unique IDs to store window positions. If you want two windows with the same name (or one window with a dynamic name) you must provide some other ID source to `egui` (some unique integer or string).

`egui` also needs to track which widget is being interacted with (e.g. which slider is being dragged). `egui` uses unique id:s for this awell, but in this case the IDs are automatically generated, so there is no need for the user to worry about it. In particular, having two buttons with the same name is no problem (this is in contrast with [`Dear ImGui`](https://github.com/ocornut/imgui)).

Overall, ID handling is a rare inconvenience, and not a big disadvantage.


## FAQ

Also see [GitHub Discussions](https://github.com/emilk/egui/discussions/categories/q-a).

### Can I use `egui` with non-latin characters?
Yes! But you need to install your own font (`.ttf` or `.otf`) using `Context::set_fonts`.

### Can I customize the look of egui?
Yes! You can customize the colors, spacing and sizes of everything. By default egui comes with a dark and a light theme.

### What about accessibility, such as screen readers?
There is experimental support for a screen reader. In [the web demo](https://emilk.github.io/egui/index.html) you can enable it in the "Backend" tab.

Read more at <https://github.com/emilk/egui/issues/167>.

### What is the difference between [egui](https://docs.rs/egui) and [eframe](https://docs.rs/eframe)?

`egui` is a 2D user interface library for laying out and interacting with buttons, sliders, etc.
`egui` has no idea if it is running on the web or natively, and does not know how to collect input or show things on screen.
That is the job of *the integration* or *backend*.

It is common to use `egui` from a game engine (using e.g. [`bevy_egui`](https://docs.rs/bevy_egui)),
but you can also use `egui` stand-alone using `eframe`. `eframe` has integration for web and native, and handles input and rendering.
The _frame_ in `eframe` stands both for the frame in which your egui app resides and also for "framework" (`frame` is a framework, `egui` is a library).

### Why is `egui_web` using so much CPU in Firefox?
On Linux and Mac, Firefox will copy the WebGL render target from GPU, to CPU and then back again: https://bugzilla.mozilla.org/show_bug.cgi?id=1010527#c0

### Why does my web app not fill the full width of the screen?
To alleviate the above mentioned performance issues the default max-width of an egui web app is 1024 points. You can change this by overriding the `fn max_size_points` of `epi::App`.


## Other

### Conventions and design choices

All coordinates are in screen space coordinates, with (0, 0) in the top left corner

All coordinates are in "points" which may consist of many physical pixels.

All colors have premultiplied alpha.

egui uses the builder pattern for construction widgets. For instance: `ui.add(Label::new("Hello").text_color(RED));` I am not a big fan of the builder pattern (it is quite verbose both in implementation and in use) but until Rust has named, default arguments it is the best we can do. To alleviate some of the verbosity there are common-case helper functions, like `ui.label("Hello");`.

Instead of using matching `begin/end` style function calls (which can be error prone) egui prefers to use `FnOnce` closures passed to a wrapping function. Lambdas are a bit ugly though, so I'd like to find a nicer solution to this.

### Inspiration

The one and only [Dear ImGui](https://github.com/ocornut/imgui) is a great Immediate Mode GUI for C++ which works with many backends. That library revolutionized how I think about GUI code and turned GUI programming from something I hated to do to something I now enjoy.

### Name

The name of the library and the project is "egui" and pronounced as "e-gooey". Please don't write it as "EGUI".

The library was originally called "Emigui", but was renamed to "egui" in 2020.

### Credits / Licenses

egui author: Emil Ernerfeldt

egui is under MIT OR Apache-2.0 license.

Fonts:

* `emoji-icon-font.ttf`: [Copyright (c) 2014 John Slegers](https://github.com/jslegers/emoji-icon-font) , MIT License
* `NotoEmoji-Regular.ttf`: [google.com/get/noto](https://google.com/get/noto), [SIL Open Font License](https://scripts.sil.org/cms/scripts/page.php?site_id=nrsi&id=OFL)
* `ProggyClean.ttf`: Copyright (c) 2004, 2005 Tristan Grimmer. MIT License. <http://www.proggyfonts.net/>
* `Ubuntu-Light.ttf` by [Dalton Maag](http://www.daltonmaag.com/): [Ubuntu font licence](https://ubuntu.com/legal/font-licence)
