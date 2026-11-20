# App UI: upgrading an existing app's interface

Verified on a real upgrade: the model and account pages of a Tauri + React desktop app (cards in the middle, directories on the right), designed with the user in a preview page over about fifteen rounds, then ported into the app's own components. On a copy of the app, its type check, lint, formatting check and full test suite (202 files, 3,249 tests) passed; the change shipped as a patch with a handoff note. The [preview kit](../examples/app-ui-preview/index.html) and [`brand_colour.py`](../scripts/brand_colour.py) come from that work.

Use this page when the user wants an existing app's screens to look better: cards, lists, side panels, empty states, buttons. The design method is Playmand's (look first, compare, check the real result); the code follows the app's own stack. React, Vue, Svelte or plain CSS all stay what they are, and nothing moves to Rust for this.

## Read the app first

Before drawing anything, find:

- **The components** behind the screen, their props, and every state they show: loading, empty, busy, failed, disabled, not installed, hidden by a privacy switch. A redesign that only covers the happy state breaks in use.
- **The theme**: the colour variables (often `--bg-surface-rgb` style tokens), radii, fonts, light and dark. Design with these, not with new colours.
- **The app's own rules**, usually in an agent or contributor guide: for example "no custom tooltips" or "keep the system cursor". Follow them in the preview too.
- **The assets** the screen uses: icons, logos. They are the art; do not redraw them.
- **The tests** that touch these components, and what they look up (labels, roles, component types, class names). Run them once before changing anything, so later failures are yours.

## Look first, in a preview page

Start from a picture of the target: the user's screenshot of something they like is the best brief. Then build a **standalone HTML preview page** before touching the app:

- Copy the app's theme variables into it, use the app's real icons and real names (a directory file is often bundled in the repo), and show dark and light side by side.
- Put the current design next to the new one, so every round is a comparison.
- When the direction is open, lay out **three or four versions** next to each other and let the user pick; then refine the one they chose. Users choose quickly between pictures and slowly between descriptions.
- Change one or two things per round and say what changed. Expect many small rounds (heights, overlap, how far a hover moves, which labels stay); each costs seconds in a preview and minutes in the app.
- Keep every round reloadable (bust the module cache while iterating) and screenshot it yourself before telling the user it is ready.

The [preview kit](../examples/app-ui-preview/index.html) is a starting point: theme variables at the top, sample rows at the bottom, a card grid and a side deck. Serve its folder (`python3 -m http.server`) so the icons can be read for their colour.

## Patterns that worked

**Brand colour from the icon.** Give each provider's card its own colour, read from its icon: the most common saturated hue (white, black and grey pixels do not vote), with lightness and saturation settled into a band where white text reads on every brand, then darkened until white text reaches 3:1 contrast (WCAG large text). Black-and-white logos get a neutral slate card. No palette to maintain, and new entries from a remote list colour themselves. In the app this runs in the browser on the icon's pixels ([`brand.js`](../examples/app-ui-preview/brand.js)); `scripts/brand_colour.py icon.png icon.svg` gives the same colours from the command line, for a design note or a static table. For SVG it counts declared colours rather than covered area, so a multi-colour SVG logo can differ from the browser's result.

**Readability on coloured cards.** Once cards carry brand colours, rules that worked on grey break:

- Numbers stay in the text colour; the progress bar carries the meaning (green, yellow, red). A green number on a green card disappears.
- A danger value becomes a small white-on-red badge, readable on any brand colour, red and orange included.
- A one-colour logo on transparency is drawn white on its own colour, or it vanishes (red mark on red card). Multi-colour logos and logos on their own filled square keep their colours.
- Black-and-white logos follow the theme: white in dark, black in light.
- Check both themes every round; a tint that is rich in dark can be muddy in light, so give light a lighter tint.

**A stacked deck for directories.** For a side list of providers to add: full-width cards in brand colour, each tucking its bottom edge (7 px) under the next; the hovered card lifts a little (3 px; the hidden part has no content, so a big lift reveals nothing). The last card has no hidden edge. The "add" entry is the first card of the deck in a light colour, so it stands out without leaving the deck. The whole card is the main action; secondary actions (open the website, cancel, a region switch) are separate buttons on top of it, never buttons inside a button.

**States on a deck card.** Busy: a light sweeps across, the countdown shows, a small cancel button appears. Not installed: the card turns grey and says so. A choice (region, provider): a small chip on the card that keeps the app's existing toggle button.

**Text the user asked to keep.** Users notice lost information: when a redesign drops field labels or puts two fields on one line, expect to be asked to put them back. Offer the cleaner version, but keep labels one step away.

## Port into the app

- Work in a **copy** of the app (or a branch the user approves). Commit a baseline in the copy so the patch contains only this work, including any uncommitted changes the user already had.
- Change appearance, not behaviour: same props, same handlers, same accessible names, same disabled and busy rules. Put the look in a stylesheet or the app's utility classes, with the theme variables; a small shared component (a deck card) is better than copying markup into two pages.
- Keep test-visible structure where tests depend on it (a labelled `section` per row, a class a test reads), and update a test only where the change is intended, saying why.
- Follow the repository's formatting **with its own config**: run its formatter only after its config is present, keep its line endings (CRLF repositories exist), and check the diff for reformatting noise before delivering.
- Run the app's own checks: type check, lint, format check, the full test suite.
- Render the real components once (a small viewing page that mounts them with sample data and both themes) and compare with the approved preview.

## Deliver

When the app lives in another repository, deliver a **patch and a handoff note** instead of committing there: what changed per area, deliberate behaviour differences, the apply commands, test results, screenshots of the real components, and what could not be checked (the real desktop window on each platform, older web views without `color-mix()`). Dry-run the patch (`git apply --check`) against the repository's current state; if someone applied an earlier version meanwhile, add a follow-up patch rather than a new full one.

## Pitfalls met

- A colour written as `hsl(220 9% 30%)` put a "%" into the markup, and a test that checks a card shows no fake "0%" failed. Write colours as `rgb()`.
- A pale yellow brand gave a card too light for white text until the contrast step was added.
- Running the formatter before copying the repository's config wrapped lines the project keeps on one line; running it again with the right config did not undo it.
- A script that rewrote files with LF turned every line of a CRLF file into a change.
- The deck card's link button called `event.stopPropagation()`, and tests that call `onClick()` without an event failed; a sibling button does not need it.
