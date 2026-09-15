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

## Adding a lesson

Three edits, nothing else:

1. `src/lessons/<name>.rs` — implement the `Lesson` trait (`src/lesson.rs`)
2. `src/lessons/mod.rs` — one line in `all()`
3. `tests/snapshots.rs` — one `render_lesson(...)` test

`src/app.rs` does not change: it only ever knows `dyn Lesson`.

## Commands

```sh
cargo run                       # native
trunk serve --release           # web (needs the wasm32-unknown-unknown target and trunk)
cargo test                      # renders every lesson to tests/snapshots/*.png
UPDATE_SNAPSHOTS=1 cargo test   # accept the current rendering as the new baseline
```

The snapshots are how layout gets verified: read the PNG rather than asking the user for a
screenshot. A pure refactor must leave them unchanged.

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
