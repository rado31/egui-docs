id-title = Id
id-summary = Where a widget's state lives between frames — and what happens when two widgets share it.

## Demo and settings

# The label of every collapsing header. It is also what egui derives each
# header's Id from, so the demo depends on all headers sharing it.
id-header = Details
# In the generated code `$index` is Rust's own `{index}`, so keep it a single
# placeholder.
id-body = Body of item #{ $index }
id-header-ids = header Ids:  { $ids }
id-distinct = { $ids ->
        [one] { $ids } distinct Id
       *[other] { $ids } distinct Ids
    } for { $widgets ->
        [one] { $widgets } widget
       *[other] { $widgets } widgets
    }
id-dead = Only the last header answers the mouse — the first two are dead. Click it: all three open anyway.
id-items = Items
id-fix = Fix
id-fix-none = none — collide
id-show-ids = show resolved Ids
id-try = Open one header, then switch the fix on and off. The state does not move — the Id it is stored under does.

## Comments in the generated code. `id-code-clash` may span several lines;
## each becomes one comment line.

id-code-clash =
    Every header gets the same label, and `CollapsingHeader`
    uses the label as its id salt — so all { $count } share one Id,
    and therefore one open/closed state.
id-code-salt = replaces the label as the Id source
id-code-push = Changes the parent Id, so every Id derived inside differs.

## Notes

id-address = An Id is an address in Memory
    .body = egui keeps no widget objects, so a widget cannot remember anything by itself. Whether a header is open, where a ScrollArea is scrolled, which TextEdit has focus — all of it lives in `Context::memory`, keyed by `Id`. The Id is how this frame's call finds what last frame's call stored. Two widgets with the same Id are, as far as egui is concerned, one widget.

id-derived = You usually do not choose the Id
    .body = Most widgets derive one. `CollapsingHeader::new(label)` uses the label text as its id salt, and the Id is derived from that salt plus the Id of the Ui it is added to. That works until the label repeats — or changes, which silently resets the state, because a different label means a different Id.

id-read-it = Read the Id, do not recompute it
    .body = The Id above comes from `response.header_response.id`. Deriving it by hand with `ui.make_persistent_id("Details")` produces a different value, because a container builds its widgets inside a child Ui with its own Id — `CollapsingHeader` wraps its contents in a `ui.vertical`. Where the derivation happens is an implementation detail; the Response is not.

id-one-widget = One Id means one widget — not two that share
    .body = egui keeps a registry of widget rects keyed by Id, and a second registration under the same Id overwrites the first: `existing.rect = widget_rect.rect; // last wins` (widget_rect.rs). So three clashing headers leave *one* entry, holding the last one's rectangle. Clicks over the first two hit nothing — they are not dead in the drawing sense, they simply no longer exist to hit-testing. Accessibility sees the same thing: a screen reader is offered one "Details" header, not three. And because that same Id is also the state key, the one header that does respond toggles all three at once.

id-salt-vs-push = id_salt vs push_id
    .body = `.id_salt(x)` replaces the salt of one widget. `ui.push_id(x, ..)` replaces the parent Id for a whole block, so everything derived inside it becomes distinct at once. In a loop whose body holds several stateful widgets, push_id is the one you want: it fixes all of them, and you cannot forget one.

id-debug-warning = The warning is a debug-build feature
    .body = `Options::warn_on_id_clash` defaults to `cfg!(debug_assertions)`. In a release build — including this page, if you are reading it in a browser — egui says nothing at all about a clash. The widgets just quietly share state, and the symptom you are left with is a control that does not respond. Note also that the red overlay is a development aid, not a readable report: egui paints one label at each of the two clashing rectangles, per pair, so with three widgets the middle one receives two different texts in the same place and they overlap. The `distinct Ids` line above is this lesson's own, and is the one to read.
