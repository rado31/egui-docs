# egui-docs

An interactive, explained guide to `egui`: a live widget, the settings that drive it, and
the code that produces it — side by side. Runs natively and in the browser.

## egui 0.36 is not the egui you remember

Pinned to `=0.36.2` on purpose. The 0.36 API differs from nearly every tutorial online,
and from what a model is likely to recall:

- `eframe::App` has `fn ui(&mut self, ui: &mut egui::Ui, frame: &mut Frame)` —
  **not** `fn update(&mut self, ctx: &Context, frame)`.
- Panels are added to a `&mut Ui`, not to a `&Context`.
- `SidePanel` / `TopBottomPanel` are **gone**. Use `egui::Panel::left/right/top/bottom(id)`,
  and `.default_size()` in place of `.default_width()`.

Before writing any egui call from memory, check it against the vendored source at
`~/.cargo/registry/src/*/egui-0.36.2/src/`. When `cargo check` disagrees with recollection,
the source wins.

On an egui release: bump all three pins, `cargo check`, fix what broke, update the lessons.

## The one invariant

A lesson's code snippet is **generated from the same state its demo reads** —
`Lesson::code()` and `Lesson::demo()` both derive from the lesson's own fields.

Never hand-write a snippet next to the widget it documents. It silently rots, which is
exactly the failure this project exists to avoid (egui's own `code_example.rs` has it).
`src/lessons/button.rs` is the reference implementation.

Text follows the same rule: the demo and `code()` read the same message through the `Tr`
they are handed. A button labelled from `tr.get("x")` gets `tr.get("x")` in its snippet too.

## Translations

English (`en`) and Russian (`ru`) are maintained. **Every change to user-visible text is made
in both** — `cargo test` fails if a language is missing a message English has.

- No reader-visible string literals in Rust. Text goes in `locales/<lang>/<file>.ftl`
  (Fluent), read with `tr.get("id")`, or `tr.fmt("id", &[("n", n.into())])` for variables.
  `build.rs` embeds every `locales/*/` folder; `src/i18n.rs` loads them.
- egui API names stay English in every language: flag names, method names, layout names.
  Lesson titles are type names (`Response`, `Ui`), so they stay English too.
- Pass counts as numbers, not preformatted strings, so Fluent can pick the plural form
  (Russian has three: `one`/`few`/`many`). Measurements are preformatted strings.
- Keep each note `.body` on one line. Fluent keeps line breaks in multi-line values, and
  `prose()` would render them. A blank line inside one is a paragraph break (see `ui-align`).
- Text the reader can edit (a button label) is state: set it in the lesson's `new(tr)`, and
  swap it in `retranslate()` only while it is unedited (`retranslate_text`).
- `DocsApp::default()` is always English (the tests depend on it); `DocsApp::new` follows
  the system language via `sys-locale`. Nothing about the language is persisted.

## Adding a lesson

Four edits, nothing else:

1. `src/lessons/<name>.rs` — implement the `Lesson` trait (`src/lesson.rs`)
2. `src/lessons/mod.rs` — one line in `all()`
3. `locales/en/<id>.ftl` and `locales/ru/<id>.ftl` — its text
4. `tests/snapshots.rs` — one `render_lesson("<id>")` test

`src/app.rs` does not change: it only ever knows `dyn Lesson`.

## Commands

```sh
cargo run                       # native
trunk serve --release           # web (needs the wasm32-unknown-unknown target and trunk)
cargo test                      # renders every lesson to tests/snapshots/*.png
UPDATE_SNAPSHOTS=1 cargo test   # accept the current rendering as the new baseline
```

Snapshots are per language: `lesson_<id>.png` (English) and `lesson_<id>_ru.png`. Read both
after a text change — Russian text usually runs longer than English.

The snapshots are how layout gets verified: read the PNG rather than asking the user for a
screenshot. A pure refactor must leave them unchanged.

## Checking a lesson live

Snapshots verify layout. For interaction and accessibility, drive the real app through the
`egui` MCP server ([egui_mcp](https://github.com/rerun-io/kittest_inspector)):

```sh
EGUI_INSPECTION=1 cargo run --features eframe/inspection   # listens on 127.0.0.1:5719; then `attach`
```

The window must stay visible for screenshots (macOS). The accessible names in `widget_tree`
are what a screen reader announces — snapshots cannot show them. Without the MCP server
registered, snapshots are the whole check.

## Deliberate non-features

- **No persistence.** eframe's `persistence` feature is off: theme, text size, open lesson,
  panel widths and window geometry all reset on every launch. This was removed on purpose —
  do not reintroduce it.
- `dist/` stays out of git. GitHub Pages build: `trunk build --release --public-url /egui-docs/`.

## Knowledge graph

`graphify-out/` holds a graph of this repo — `graphify explain "<symbol>"` and
`graphify query "<question>"` to orient before grepping.

One caveat: method calls through `dyn Lesson` are invisible to it (virtual dispatch cannot be
resolved statically), so it under-reports who consumes a lesson. Trait *implementations* it
sees correctly.
