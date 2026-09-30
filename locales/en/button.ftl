button-title = Button
button-summary = A clickable widget — and the shortest path to understanding Response.

## Demo and settings. Names from egui's API (`frame`, `sense`, ...) are not
## translated: they are what the reader types.

button-label = Click me
button-clicks = clicks:
button-reset = reset
button-text = Text
button-sense-both = both

## Notes

button-no-callback = There is no click callback
    .body = In a retained GUI you register a handler and wait to be called back. egui has no handler: you call the widget every frame, and it returns a Response describing what happened to it *this* frame. `clicked()` is just a bool on that struct. This is the single idea the whole library is built on — every widget works this way.

button-value-not-object = The widget is a value, not an object
    .body = `egui::Button::new(..)` builds a plain struct that lives for one frame. Each `.fill(..)`-style method takes it by value and returns it back, so they chain. Nothing is stored between frames: turn a knob and the next frame simply builds a different Button. That is why the code on the left can be regenerated from state — there is no hidden widget object to sync.

button-add-vs-add-enabled = add vs add_enabled
    .body = `ui.add(widget)` shows it normally. `ui.add_enabled(false, widget)` greys it out and makes it non-interactive — the Response still comes back, but `clicked()` will never be true. Use `ui.add_enabled_ui(..)` when you want to disable a whole group at once.

button-atoms = Button::new takes Atoms, not just text
    .body = The signature is `new(impl IntoAtoms)`. An Atom is a piece of button content — text, an image, a gap — and a string is just the simplest one. That is what makes `.shortcut_text(..)` and image buttons possible. Atoms get their own lesson; a plain &str is enough for now.
