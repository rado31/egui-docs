atom-title = Atom
atom-summary = What a Button is made of: a row of atoms that grow, shrink, or leave room for you.

## Demo and settings. Method names (`atom_shrink(true)`, `Atom::grow()`, ...)
## and wrap modes (`truncate`, ...) are egui's API and are not translated.

# The file name on the demo button. The reader can edit it.
atom-label = Quarterly report – final.pdf
atom-measurements = Atoms, left to right — each name in the colour of its outline
# What each atom is called in the table and the settings.
atom-part-icon = icon
atom-part-label = label
atom-part-spacer = spacer
atom-part-shortcut = shortcut
atom-part-custom = custom
atom-part-button = button
# Numbers arrive already rounded.
atom-px-wide = { $width } px wide
atom-nothing-after = — nothing after it to push
atom-offered = offered { $width } px
# Appended to the button's own `atom-px-wide`.
atom-would-like = , would like { $width }
atom-width-offered = Width offered to the button
atom-atoms = Atoms
atom-wrap-mode = Wrap mode
# The wrap mode used when the button sets none: whatever its Ui says.
atom-wrap-default = Ui's
atom-outlines = outline atoms
atom-try = Narrow the width to 200 px, then turn off atom_shrink on the label: egui picks the first text atom to shrink instead — the icon, which cannot.

## Comments in the generated code. `atom-code-justified` may span several
## lines; each becomes one comment line.

atom-code-ext = the .atom_*() methods
atom-code-atom-ui = `atom_ui` instead of `ui.add`: it hands back each custom atom's rect.
atom-code-justified =
    Justified, so the button fills the width — which gives grow
    something to fill and shrink something to fit into.

## Notes

atom-everything-atom = Everything inside a Button is an Atom
    .body = `Button::new` takes `impl IntoAtoms`. A `&str`, a `RichText`, an `Image` or an `Atom` is one atom; a tuple of up to six of them is a row. Button, Checkbox, RadioButton and menu buttons are all built on the same `AtomLayout`, so what you learn here applies to all of them. Even `.shortcut_text(..)` is nothing special: it is `push_right(Atom::grow())` followed by the text, made weak.

atom-grow = grow takes the slack — if there is any
    .body = Space left over after every atom is sized is split equally between the atoms marked `grow`. But a Button is normally only as wide as its content, so there is no slack and `grow` does nothing. It needs a width from outside: a justified layout, as here, or `.min_size(..)` on the button. Without one, `Atom::grow()` is a silent no-op.

atom-shrink = Exactly one atom shrinks — egui picks one if you do not
    .body = When the atoms do not fit, one atom gives up space: the one marked `atom_shrink(true)`. It is sized last, with whatever width the others left. Mark none and egui marks the *first text atom* for you. In this button that is the icon — a single glyph, which cannot get any narrower. So nothing gives, and the button runs past the dashed line as if it could not shrink at all. Marking two is a bug: a debug assertion, and in release only the first counts.

atom-wrap-mode-note = The wrap mode decides what shrinking means
    .body = `.truncate()` cuts the shrinking atom with an ellipsis, `.wrap()` breaks it onto more lines and makes the button taller, and `Extend` never shrinks at all — the button runs past the dashed line. With no call the `Ui` decides: `Wrap` in a vertical layout, but `Extend` in `ui.horizontal`. The same button behaves differently depending on where you put it. `atom_max_width` on an atom switches that atom to truncating regardless.

atom-custom = Atom::custom leaves a hole for you to paint in
    .body = `Atom::custom(id, size)` is an empty atom that takes part in the layout like any other. Its rect is only known after layout, so show the button with `.atom_ui(ui)` instead of `ui.add(..)`, and ask the response: `response.rect(id)`. It returns an Option — `None` when the button was not visible and so never painted. Never unwrap it. The outlines on this page are drawn the same way: every atom gets an `.atom_id(..)`, which asks egui to report its rect and changes nothing else.

atom-screen-reader = A screen reader hears every text atom
    .body = A Button's accessible name is all of its text atoms joined with spaces, so this one is announced as "🗀 Quarterly report – final.pdf Ctrl+O" — the icon glyph included. An emoji or icon-font glyph is text as far as egui is concerned. An `Image` atom is not: an icon drawn as an image stays silent. Its alt text is only used when the button has no text at all.
