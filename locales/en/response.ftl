response-title = Response
response-summary = What every widget hands back — and the difference between a state and an event.

## Demo and settings. Flag names (`hovered`, `clicked`, ...) are egui's API
## and are not translated.

response-target = Poke me
response-states = states
response-events = events — true for one frame only
response-values = values
# The value of `response.rect`. Numbers arrive already rounded.
response-rect = { $width } x { $height }  at ({ $x }, { $y })
response-senses = What the widget senses
response-show-values = show values
response-try = Try: right-click, double-click, press and drag, then Tab to focus it.

## Comments in the generated code

response-code-states = States — true while the condition holds:
response-code-events = Events — true for exactly one frame:
response-code-values = Values:

## Notes

response-states-vs-events = States vs events
    .body = `hovered()` is a state: it stays true the whole time the pointer is over the widget. `clicked()` is an event: it is true for exactly one frame, then false again. At 60 fps that is 16 ms — which is why the chips above fade out instead of blinking. If you need to remember an event, you must store it yourself (a counter, a bool in your app struct); the Response is gone at the end of the frame.

response-sense = Sense decides what is even possible
    .body = A widget only reports what it senses. Switch the knob to `drag` and `clicked()` stops firing no matter how you click — the widget is not listening for clicks. `Sense` is also what makes a widget eligible for hover highlighting and focus at all.

response-disabled = Disabled widgets still return a Response
    .body = `add_enabled(false, ..)` does not remove the widget: you still get a Response back, its `rect` is still valid, but no interaction flag will ever be true. That is why you can write `if response.clicked()` unconditionally without checking whether the widget is enabled.

response-whole-api = The Response is the whole API surface
    .body = There is no widget object to query later, and no event handler to register. Everything egui will ever tell you about a widget is in the struct returned by the call — including `rect` for layout, `id` for memory, and helpers like `on_hover_text(..)` that consume and return it.
