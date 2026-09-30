eframe-title = eframe::App
eframe-summary = Where every egui program starts: a main function, a window, and one method called every frame.

## Demo and settings. Names from the API (`app_name`, `with_title`, ...) are
## not translated: they are what the reader types.

eframe-title-text = My app
eframe-hello = Hello, egui!
eframe-button = Click me
eframe-clicks = clicks:
eframe-side-text = Side panel
# Shown in the mock title bar when no inner size is set.
eframe-size-os = size: up to the OS
# Shown in the mock title bar when the window is not resizable.
eframe-fixed = fixed
eframe-preview = Preview:
eframe-preview-window = window
eframe-preview-browser = browser
eframe-web-toggle = also run in the browser
eframe-web-hint = Same App, second main. In the browser the page sets the canvas size, so the viewport settings above do nothing there.

## Comments in the generated code

eframe-code-deps = Dependencies, in Cargo.toml:

## Notes

eframe-ui-not-update = fn ui, not fn update
    .body = An App implements `fn ui(&mut self, ui: &mut egui::Ui, frame: &mut eframe::Frame)`, and that `ui` covers the whole window. This arrived in egui 0.34, which deprecated `fn update(&mut self, ctx: &egui::Context, ..)`; 0.35 removed it, together with `SidePanel` and `TopBottomPanel`. Most tutorials still show the old way. Panels are now `egui::Panel::left(..)` and friends, added to the `ui`. Work that draws nothing can go in the optional `fn logic(&mut self, ctx, frame)`, which runs before every `ui`.

eframe-central-last = Side panels first, CentralPanel last
    .body = The `ui` you are handed has no margin and no background; `egui::CentralPanel` gives it both. Each side panel takes a strip off an edge of the space that is still free, and CentralPanel fills whatever is left — so it always comes last. Tick `egui::Panel::left` and the heading moves over to make room.

eframe-inner-size = with_inner_size is where the window starts
    .body = The viewport settings describe the window as it *opens*. Once it is up, the user can resize it however they like — unless `with_resizable(false)` — and your code never hears about it: `ui` just gets a bigger or smaller `ui` next frame, and the panels reflow. Drag the corner of the window above: the size in its title bar changes, the code does not. Change `with_inner_size` and it reopens at the new size.

eframe-app-name = app_name is an id; the title is text
    .body = The first argument of `run_native` names the app to the operating system: it is the application id on Wayland, and the folder name eframe saves state under when persistence is on. It is also the window title — unless you set one with `with_title`. Untick `with_title` and watch the title bar fall back to `app_name`.

eframe-creation = The closure runs once, ui runs every frame
    .body = `Box::new(|cc| ..)` is called once, after the window and the graphics context exist. The `CreationContext` it gets is the place to set fonts or a style on `cc.egui_ctx`, or to restore saved state. After that, eframe calls `ui` every frame — and egui only paints a frame when something happened (input, an animation, a `request_repaint`), so an idle app costs nothing.

eframe-web = One App, two main functions
    .body = Tick *also run in the browser*: the App stays exactly the same and only `main` changes, picked by `#[cfg(target_arch = "wasm32")]`. In the browser eframe draws into a `<canvas>` it finds by id in your `index.html`, and the page decides its size. `trunk serve` builds and serves it; this guide is built that way.
