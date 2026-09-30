### The app around the lessons. Start here when adding a language: copy this
### folder, rename it to the language's code (`de`, `pt-BR`, ...), and translate.

# This language's name, written in the language itself. Shown in the language menu.
language-name = English

app-language = Language
app-settings = Settings
app-code = Code
app-copy = Copy
app-look-up = Look up:
# At the bottom of every lesson. `$lesson` is the neighbour's number and title: `1.2 Response`.
# The arrows are ⏴ ⏵ because egui's default fonts have no ← →.
app-previous = ⏴ Previous: { $lesson }
app-next = Next: { $lesson } ⏵
app-theme-light = Switch to light mode
app-theme-dark = Switch to dark mode
app-zoom-in = Larger text  ({ $shortcut })
app-zoom-out = Smaller text  ({ $shortcut })
# `$size` is the body text size in pixels, already rounded.
app-zoom-reset =
    Text size — { $size } px body text.
    Reset: { $shortcut }

## Sections of the table of contents

section-fundamentals = Fundamentals
section-widgets = Widgets
section-layout = Layout
section-style = Style & Theme
section-text = Text & Fonts
section-interaction = Input & Interaction
section-painting = Painting
section-containers = Containers & Windows
section-state = Memory & State
