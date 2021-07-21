# egui changelog

All notable changes to the egui crate will be documented in this file.

NOTE: [`eframe`](eframe/CHANGELOG.md), [`egui_web`](egui_web/CHANGELOG.md) and [`egui_glium`](egui_glium/CHANGELOG.md) have their own changelogs!


## Unreleased
* Replaced all missed opportunities to return an inner return value. (this is a breaking change!)
  * `Area::show`
  * `ComboBox::show_ui`
  * `ComboBox::combo_box_with_label`
  * `Window::show`
  * `popup::*`
  * `menu::menu`

### Added ⭐
* Plot:
  * [Line styles](https://github.com/emilk/egui/pull/482)
* [Progress bar](https://github.com/emilk/egui/pull/519)
* `Grid::num_columns`: allow the last column to take up the rest of the space of the parent `Ui`.
* Add `show_background` and `show_axes` methods to `Plot`.

### Fixed 🐛
* Fix custom font definitions getting replaced when `pixels_per_point` is changed.


## 0.13.1 - 2021-06-28 - Plot fixes

### Added ⭐
* Plot: you can now set the stroke of a `HLine/VLine`.

### Changed 🔧
* `Plot::new` now takes an `id_source: impl Hash` instead of a `name: impl ToString`. Functionally it is the same.


## 0.13.0 - 2021-06-24 - Better panels, plots and new visual style

### Added ⭐
* Plot:
  * [More plot items: Arrows, Polygons, Text, Images](https://github.com/emilk/egui/pull/471).
  * [Plot legend improvements](https://github.com/emilk/egui/pull/410).
  * [Line markers for plots](https://github.com/emilk/egui/pull/363).
* Panels:
  * Add right and bottom panels (`SidePanel::right` and `Panel::bottom`).
  * Panels can now be resized.
  * Add an option to overwrite frame of a `Panel`.
* [Improve accessibility / screen reader](https://github.com/emilk/egui/pull/412).
* Add `ScrollArea::show_rows` for efficient scrolling of huge UI:s.
* Add `ScrollArea::enable_scrolling` to allow freezing scrolling when editing TextEdit widgets within it
* Add `Ui::set_visible` as a way to hide widgets.
* Add `Style::override_text_style` to easily change the text style of everything in a `Ui` (or globally).
* You can now change `TextStyle` on checkboxes, radio buttons and `SelectableLabel`.
* Add support for [cint](https://crates.io/crates/cint) under `cint` feature.
* Add features `extra_asserts` and `extra_debug_asserts` to enable additional checks.
* `TextEdit` now supports edits on a generic buffer using `TextBuffer`.
* Add `Context::set_debug_on_hover` and `egui::trace!(ui)`

### Changed 🔧
* Minimum Rust version is now 1.51 (used to be 1.52)
* [Tweaked the default visuals style](https://github.com/emilk/egui/pull/450).
* Plot: Renamed `Curve` to `Line`.
* `TopPanel::top` is now `TopBottomPanel::top`.
* `SidePanel::left` no longet takes the default width by argument, but by a builder call.
* `SidePanel::left` is resizable by default.

### Fixed 🐛
* Fix uneven lettering on non-integral device scales ("extortion lettering").
* Fix invisible scroll bar when native window is too narrow for egui.


## 0.12.0 - 2021-05-10 - Multitouch, user memory, window pivots, and improved plots

### Added ⭐
* Add anchors to windows and areas so you can put a window in e.g. the top right corner.
* Make labels interactive with `Label::sense(Sense::click())`.
* Add `Response::request_focus` and `Response::surrender_focus`.
* Add `TextEdit::code_editor` (VERY basic).
* [Pan and zoom plots](https://github.com/emilk/egui/pull/317).
* [Add plot legends](https://github.com/emilk/egui/pull/349).
* [Users can now store custom state in `egui::Memory`](https://github.com/emilk/egui/pull/257).
* Add `Response::on_disabled_hover_text` to show tooltip for disabled widgets.
* Zoom input: ctrl-scroll and (on `egui_web`) trackpad-pinch gesture.
* Support for raw [multi touch](https://github.com/emilk/egui/pull/306) events,
  enabling zoom, rotate, and more. Works with `egui_web` on mobile devices,
  and should work with `egui_glium` for certain touch devices/screens.
* Add (optional) compatibility with [mint](https://docs.rs/mint).

### Changed 🔧
* Make `Memory::has_focus` public (again).
* `Plot` must now be given a name that is unique within its scope.
* Tab only selects labels if the `screen_reader` option is turned on.
* Rename `ui.wrap` to `ui.scope`.

### Fixed 🐛
* Fix [defocus-bug on touch screens](https://github.com/emilk/egui/issues/288).
* Fix bug with the layout of wide `DragValue`:s.

### Removed 🔥
* Moved experimental markup language to `egui_demo_lib`


## 0.11.0 - 2021-04-05 - Optimization, screen reader & new layout logic

### Added ⭐
* You can now give focus to any clickable widget with tab/shift-tab.
  * Use space or enter to click the selected widget.
  * Use arrow keys to adjust sliders and `DragValue`s.
* egui will now output events when widgets gain keyboard focus.
  * This can be hooked up to a screen reader to aid the visually impaired
* Add the option to restrict the dragging bounds of `Window` and `Area` to a specified area using `drag_bounds(rect)`.
* Add support for small and raised text.
* Add `ui.set_row_height`.
* Add `DebugOptions::show_widgets` to debug layouting by hovering widgets.
* Add `ComboBox` to more easily customize combo boxes.
* Add `Slider::new` and `DragValue::new` to replace old type-specific constructors.
* Add `TextEdit::password` to hide input characters.

### Changed 🔧
* `ui.advance_cursor` is now called `ui.add_space`.
* `kb_focus` is now just called `focus`.

### Fixed 🐛
* Fix some bugs related to centered layouts.
* Fixed secondary-click to open a menu.
* [Fix panic for zero-range sliders and zero-speed drag values](https://github.com/emilk/egui/pull/216).
* Fix false id clash error for wrapping text.
* Fix bug that would close a popup (e.g. the color picker) when clicking inside of it.

### Deprecated ☢️
* Deprectated `combo_box_with_label` in favor of new `ComboBox`.
* Deprectated type-specific constructors for `Slider` and `DragValue` (`Slider::f32`, `DragValue::usize` etc).


## 0.10.0 - 2021-02-28 - Plot and polish

<img src="media/egui-0.10-plot.gif" width="50%">

### Added ⭐
* Add `egui::plot::Plot` to plot some 2D data.
* Add `Ui::hyperlink_to(label, url)`.
* Sliders can now have a value prefix and suffix (e.g. the suffix `"°"` works like a unit).
* `Context::set_pixels_per_point` to control the scale of the UI.
* Add `Response::changed()` to query if e.g. a slider was dragged, text was entered or a checkbox was clicked.
* Add support for all integers in `DragValue` and `Slider` (except 128-bit).

### Changed 🔧
* Improve the positioning of tooltips.
* Only show tooltips if mouse is still.
* `Slider` will now show the value display by default, unless turned off with `.show_value(false)`.
* The `Slider` value is now a `DragValue` which when dragged can pick values outside of the slider range (unless `clamp_to_range` is set).


## 0.9.0 - 2021-02-07 - Light Mode and much more

<img src="media/0.9.0-disabled.gif" width="50%">

### Added ⭐
* Add support for secondary and middle mouse buttons.
* Add `Label` methods for code, strong, strikethrough, underline and italics.
* Add `ui.group(|ui| { … })` to visually group some widgets within a frame.
* Add `Ui` helpers for doing manual layout (`ui.put`, `ui.allocate_ui_at_rect` and more).
* Add `ui.set_enabled(false)` to disable all widgets in a `Ui` (grayed out and non-interactive).
* Add `TextEdit::hint_text` for showing a weak hint text when empty.
* `egui::popup::popup_below_widget`: show a popup area below another widget.
* Add `Slider::clamp_to_range(bool)`: if set, clamp the incoming and outgoing values to the slider range.
* Add: `ui.spacing()`, `ui.spacing_mut()`, `ui.visuals()`, `ui.visuals_mut()`.
* Add: `ctx.set_visuals()`.
* You can now control text wrapping with `Style::wrap`.
* Add `Grid::max_col_width`.

### Changed 🔧
* Text will now wrap at newlines, spaces, dashes, punctuation or in the middle of a words if necessary, in that order of priority.
* Widgets will now always line break at `\n` characters.
* Widgets will now more intelligently choose wether or not to wrap text.
* `mouse` has been renamed `pointer` everywhere (to make it clear it includes touches too).
* Most parts of `Response` are now methods, so `if ui.button("…").clicked {` is now `if ui.button("…").clicked() {`.
* `Response::active` is now gone. You can use `response.dragged()` or `response.clicked()` instead.
* Backend: pointer (mouse/touch) position and buttons are now passed to egui in the event stream.
* `DragValue::range` is now called `clamp_range` and also clamps incoming values.
* Renamed `Triangles` to `Mesh`.
* The tessellator now wraps the clip rectangle and mesh in `struct ClippedMesh(Rect, Mesh)`.
* `Mesh::split_to_u16` now returns a 16-bit indexed `Mesh16`.

### Fixed 🐛
* It is now possible to click widgets even when FPS is very low.
* Tessellator: handle sharp path corners better (switch to bevel instead of miter joints for > 90°).


## 0.8.0 - 2021-01-17 - Grid layout & new visual style

<img src="media/widget_gallery_0.8.0.gif" width="50%">

### Added ⭐
* Added a simple grid layout (`Grid`).
* Added `ui.allocate_at_least` and `ui.allocate_exact_size`.
* Added function `InputState::key_down`.
* Added `Window::current_pos` to position a window.

### Changed 🔧
* New simpler and sleeker look!
* Rename `PaintCmd` to `Shape`.
* Replace tuple `(Rect, Shape)` with tuple-struct `ClippedShape`.
* Rename feature `"serde"` to `"persistence"`.
* Break out the modules `math` and `paint` into separate crates `emath` and `epaint`.

### Fixed 🐛
* Fixed a bug that would sometimes trigger a "Mismatching panels" panic in debug builds.
* `Image` and `ImageButton` will no longer stretch to fill a justified layout.


## 0.7.0 - 2021-01-04

### Added ⭐
* Add `ui.scroll_to_cursor` and `response.scroll_to_me` ([#81](https://github.com/emilk/egui/pull/81) by [lucaspoffo](https://github.com/lucaspoffo)).
* Add `window.id(…)` and `area.id(…)` for overriding the default `Id`.

### Changed 🔧
* Renamed `Srgba` to `Color32`.
* All color contructions now starts with `from_`, e.g. `Color32::from_rgb`.
* Renamed `FontFamily::VariableWidth` to `FontFamily::Proportional`.
* Removed `pixels_per_point` from `FontDefinitions`.

### Fixed 🐛
* `RepaintSignal` now implements `Sync` so it can be sent to a background thread.
* `TextEdit` widgets are now slightly larger to accommodate their frames.

### Deprecated ☢️
* Deprecated `color::srgba`.


## 0.6.0 - 2020-12-26

### Added ⭐
* Turn off `Window` title bars with `window.title_bar(false)`.
* `ImageButton` - `ui.add(ImageButton::new(...))`.
* `ui.vertical_centered` and `ui.vertical_centered_justified`.
* `ui.allocate_painter` helper.
* Mouse-over explanation to duplicate ID warning.
* You can now easily constrain egui to a portion of the screen using `RawInput::screen_rect`.
* You can now control the minimum and maixumum number of decimals to show in a `Slider` or `DragValue`.
* Add `egui::math::Rot2`: rotation helper.
* `Response` now contains the `Id` of the widget it pertains to.
* `ui.allocate_response` that allocates space and checks for interactions.
* Add `response.interact(sense)`, e.g. to check for clicks on labels.

### Changed 🔧
* `ui.allocate_space` now returns an `(Id, Rect)` tuple.
* `Arc<Context>` has been replaced with `CtxRef` everywhere.
* Slight tweak of the default `Style` and font sizes.
* `SidePanel::left` and `TopPanel::top` now takes `impl Hash` as first argument.
* A `Window` may now cover an existing `CentralPanel`.
* `ui.image` now takes `impl Into<Vec2>` as a `size` argument.
* Made some more fields of `RawInput` optional.
* `Slider` and `DragValue` uses fewer decimals by default. See the full precision by hovering over the value.
* `egui::App`: added `fn name(&self)` and `fn clear_color(&self)`.
* Combo boxes has scroll bars when needed.
* Expand `Window` + `Resize` containers to be large enough for last frames content
* `ui.columns`: Columns now defaults to justified top-to-down layouts.
* Rename `Sense::nothing()` to  `Sense::hover()`.
* Replaced `parking_lot` dependency with `atomic_refcell` by default.

### Fixed 🐛
* The background for `CentralPanel` will now cover unused space too.
* `ui.columns`: Improve allocated size estimation.

### Deprecated ☢️
* `RawInput::screen_size` - use `RawInput::screen_rect` instead.
* left/centered/right column functions on `Ui`.
* `ui.interact_hover` and `ui.hovered`.


## 0.5.0 - 2020-12-13

### Added ⭐
* Emoji support: 1216 different emojis that work in any text.
  * The Demo app comes with a Font Book to explore the available glyphs.
* `ui.horizontal_wrapped(|ui| ...)`: Add widgets on a row but wrap at `max_size`.
* `ui.horizontal_wrapped_for_text`: Like `ui.horizontal_wrapped`, but with spacing made for embedding text.
* `ui.horizontal_for_text`: Like `ui.horizontal`, but with spacing made for embedding text.
* `egui::Layout` now supports justified layouts where contents is _also_ centered, right-aligned, etc.
* `ui.allocate_ui(size, |ui| ...)`: Easily create a child-`Ui` of a given size.
* `SelectableLabel` (`ui.selectable_label` and `ui.selectable_value`): A text-button that can be selected.
* `ui.small_button`: A smaller button that looks good embedded in text.
* `ui.drag_angle_tau`: For those who want to specify angles as fractions of τ (a full turn).
* Add `Resize::id_source` and `ScrollArea::id_source` to let the user avoid Id clashes.

### Changed 🔧
* New default font: [Ubuntu-Light](https://fonts.google.com/specimen/Ubuntu).
* Make it simpler to override fonts in `FontDefinitions`.
* Remove minimum button width.
* Refactor `egui::Layout` substantially, changing its interface.
* Calling `on_hover_text`/`on_hover_ui` multiple times will stack tooltips underneath the previous ones.
* Text wrapping on labels, buttons, checkboxes and radio buttons is now based on the layout.

### Removed 🔥

* Removed the `label!` macro.


## 0.4.0 - 2020-11-28

### Added ⭐
* `TextEdit` improvements:
  * Much improved text editing, with better navigation and selection.
  * Move focus between `TextEdit` widgets with tab and shift-tab.
  * Undo edtis in a `TextEdit`.
  * You can now check if a `TextEdit` lost keyboard focus with `response.lost_focus`.
  * Added `ui.text_edit_singleline` and `ui.text_edit_multiline`.
* You can now debug why your `Ui` is unexpectedly wide with `ui.style_mut().debug.show_expand_width = true;`

### Changed 🔧
* Pressing enter in a single-line `TextEdit` will now surrender keyboard focus for it.
* You must now be explicit when creating a `TextEdit` if you want it to be singeline or multiline.
* Improved automatic `Id` generation, making `Id` clashes less likely.
* egui now requires modifier key state from the integration
* Added, renamed and removed some keys in the `Key` enum.
* Fixed incorrect text wrapping width on radio buttons

### Fixed 🐛
* Fixed bug where a lost widget could still retain keyboard focus.


## 0.3.0 - 2020-11-07

### Added ⭐
* Panels: you can now create panels using `SidePanel`, `TopPanel` and `CentralPanel`.
* You can now override the default egui fonts.
* Add ability to override text color with `visuals.override_text_color`.
* The demo now includes a simple drag-and-drop example.
* The demo app now has a slider to scale all of egui.

### Changed 🔧
* `ui.horizontal(...)` etc returns `Response`.
* Refactored the interface for `egui::app::App`.
* Windows are now constrained to the screen.
* `Context::begin_frame()` no longer returns a `Ui`. Instead put your widgets into a `SidePanel`, `TopPanel`, `CentralPanel`, `Window` or `Area`.
* `Context::end_frame()` now returns shapes that need to be converted to triangles with `Context::tessellate()`.
* Anti-aliasing is now off by default in debug builds.

### Removed 🔥

* You can no longer throw windows.

### Fixed 🐛
* Fix a bug where some regions would slowly grow for non-integral scales (`pixels_per_point`).


## 0.2.0 - 2020-10-10

* Color picker
* Unicode characters in labels (limited by [what the default font supports](https://fonts.google.com/specimen/Comfortaa#glyphs))
* Simple drop-down combo box menu
* Logarithmic sliders
* Optimization: coarse culling in the tessellator
* CHANGED: switch argument order of `ui.checkbox` and `ui.radio`

## 0.1.4 - 2020-09-08

This is when I started the CHANGELOG.md, after almost two years of development. Better late than never.

* Widgets: label, text button, hyperlink, checkbox, radio button, slider, draggable value, text editing
* Layouts: horizontal, vertical, columns
* Text input: very basic, multiline, copy/paste
* Windows: move, resize, name, minimize and close. Automatically sized and positioned.
* Regions: resizing, vertical scrolling, collapsing headers (sections)
* Rendering: Anti-aliased rendering of lines, circles, text and convex polygons.
* Tooltips on hover
