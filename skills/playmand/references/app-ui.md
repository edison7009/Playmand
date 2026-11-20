# App UI: upgrading an existing app's interface

Verified on two real upgrades of the same Tauri + React desktop app:

- **First upgrade.** The model and account pages (cards in the middle, directories on the right). They were designed with the user in a preview page over about fifteen rounds, ported in a copy of the app, and delivered as a patch with a handoff note. The app's type check, lint, formatting check and full test suite (202 files, 3,249 tests) passed.
- **Second upgrade.** The whole app, taken over directly. Every page was redesigned in its own repository while another agent was committing there too. Every question went through a comparison page. Every change was checked on the real components, running in a browser with the backend mocked, before it was committed to the app's main branch.

The [preview kit](../examples/app-ui-preview/index.html), the [selection and corner study](../examples/app-ui-preview/states.html) and [`brand_colour.py`](../scripts/brand_colour.py) come from that work.

Use this page when the user wants an existing app's screens to look better: cards, lists, side panels, tabs, buttons, empty states. The design method is Playmand's: look first, compare, check the real result. The code follows the app's own stack. React, Vue, Svelte or plain CSS all stay what they are, and nothing moves to Rust for this.

## Read the app first

Before drawing anything, find:

- **The components behind the screen.** Read their props and every state they show: loading, empty, busy, failed, disabled, not installed, hidden by a privacy switch. A redesign that only covers the happy state breaks in use.
- **The theme.** Find the colour variables (often `--bg-surface-rgb` style tokens), radii, fonts, and the light and dark themes. Check whether the user can pick an accent colour. Design with these variables, not with new colours.
- **The app's own rules.** They usually live in an agent or contributor guide, for example "no custom tooltips" or "keep the system cursor". Follow them in the preview too.
- **The assets** the screen uses: icons and logos. They are the art; do not redraw them.
- **The tests** that touch these components, and what they look up (labels, roles, component types, class names). Run them once before changing anything, so later failures are yours. Note any test that already fails, and report it separately from your work.

## Look first, in a preview page

Start from a picture of the target: the user's screenshot of something they like is the best brief. Then, before touching the app, build a **standalone HTML preview page**:

- **Use the app's real look.** Copy in the app's theme variables, and use its real icons and real names (a directory file is often bundled in the repo). Show dark and light side by side. If the user can pick an accent colour, show two or three accents too: a tint that works on orange can turn muddy on green.
- **Keep the current design as one of the versions, every time.** Users do pick it. In the second upgrade the user chose the original over every new version more than once: an animation, a colour, a latency row.
- **Give open questions three or four versions.** When the direction is open, put three or four versions side by side and let the user pick, then refine the one they chose. Users choose quickly between pictures and slowly between descriptions.
- **Make one page per question.** Name each page after its question (for example selection styles, tab styles, terminal styles), so each choice stays reloadable.
- **Make the page when asked, before the code.** When the user asks for a comparison, stop editing the app and build the page first, even when you already have a favourite.
- **Change one or two things per round** and say what changed. Expect many small rounds: heights, overlap, how far a hover moves, which labels stay. Each costs seconds in a preview and minutes in the app.
- **Check every round yourself.** Keep each round reloadable (bust the module cache while iterating), and take a screenshot yourself before telling the user it is ready.

The [preview kit](../examples/app-ui-preview/index.html) is a starting point: theme variables at the top, sample rows at the bottom, a card grid and a side deck. [states.html](../examples/app-ui-preview/states.html) puts three selection styles and two corner treatments side by side under three accents. Serve the folder with `python3 -m http.server`, so the icons can be read for their colour.

## Patterns that worked

**Brand colour from the icon.** Give each provider's card its own colour, read from its icon:

- Take the most common saturated hue. White, black and grey pixels do not vote.
- Settle the lightness and saturation into a band where white text reads on every brand.
- Then darken until white text reaches 3:1 contrast (WCAG large text).
- Give black-and-white logos a neutral slate card.

There is no palette to maintain, and new entries from a remote list colour themselves. In the app this runs in the browser on the icon's pixels ([`brand.js`](../examples/app-ui-preview/brand.js)). `scripts/brand_colour.py icon.png icon.svg` gives the same colours from the command line, for a design note or a static table. For SVG it counts declared colours rather than covered area, so a multi-colour SVG logo can differ from the browser's result.

**Calm cards in a row.** A grid of cards in full brand colour looks loud once there are more than three or four.

- Keep each card mostly the plain surface colour, with the brand showing from one corner (about 22% tint in dark, 14% in light).
- Each card stays recognisable and the row stays quiet.
- A faded logo in the corner can turn upright on hover. Do not put the same faded logo in list rows: there it is just noise.

**Readability on coloured cards.** Once cards carry brand colours, rules that worked on grey break:

- Numbers stay in the text colour; the progress bar carries the meaning (green, yellow, red). A green number on a green card disappears.
- A danger value becomes a small white-on-red badge, readable on any brand colour, red and orange included.
- A one-colour logo on transparency is drawn white on its own colour, or it vanishes (a red mark on a red card). Multi-colour logos and logos on their own filled square keep their colours.
- Black-and-white logos follow the theme: white in dark, black in light.
- Check both themes every round. A tint that is rich in dark can be muddy in light, so give light a lighter tint.

**Selection without highlight borders.** Accent outlines around chosen items flicker when several are chosen in a row. Two quieter states held up:

- **Tinted rows:** the chosen row gets a clearly stronger, even tint, and no outline.
- **Neutral rows:** pills and list rows get a soft light from the top edge plus a slight lift. On a light theme the light alone barely shows, so the chosen row also sits a clear step darker.

Use one selected style per kind of element across the whole app: account pills, conversation rows and add buttons all chose the same one. Do not let one page keep an accent outline.

**Nothing moves on hover.** Users notice a one-pixel shift at once.

- Hover changes colour, shadow or a wash (`box-shadow: inset 0 0 0 999px rgb(255 255 255 / .04)`), never border width, padding or font weight.
- An element that hides in some states keeps its place with `visibility: hidden`, so its neighbours do not slide.
- Icons in a card corner stay put while the card animates (see the pitfalls below).

**Motion on hover only.** Cards that lift on hover do not also bounce, press down or swap their icon when clicked. A click on a draw-from-the-deck card adds the item and leaves the card still. Guard against double clicks with a lock in a ref, not by disabling the list: disabling it dims and re-renders the list, which reads as a flicker.

**Corners in one family.** When a shape sits inside another (a key in a well, a button in a bar), its corner radius is the outer radius minus the gap: a 9 px well with a 3 px inset gives a 6 px key. The two curves then share a centre. Equal radii look wrong when nested.

**One font for controls.** Tabs, page buttons and card actions use one font and weight. A monospace label left over on one button stands out.

**Sweep every page after a change.** When the user approves a look for one area (the list on the right, the add button, the selected state), find every other page with the same kind of area and bring it in line before saying you are done. Users find the page you missed.

**A stacked deck for directories.** For a side list of providers to add:

- Full-width cards in brand colour. Each card tucks its bottom edge (7 px) under the next, and the last card has no hidden edge.
- The hovered card lifts a little (3 px). The hidden part has no content, so a bigger lift reveals nothing.
- The "add" entry is the first card of the deck, in a light colour, so it stands out without leaving the deck.
- The whole card is the main action. Secondary actions (open the website, cancel, a region switch) are separate buttons on top of it, never buttons inside a button.

**States on a deck card.**

- **Busy:** a light sweeps across, the countdown shows, and a small cancel button appears.
- **Not installed:** the card turns grey and says so.
- **A choice** (region, provider): a small chip on the card that keeps the app's existing toggle button.

**Text the user asked to keep.** Users notice lost information. When a redesign drops field labels or puts two fields on one line, expect to be asked to put them back. Offer the cleaner version, but keep the labels one step away.

## Port into the app

There are two ways to work, and the user decides which.

**In a copy, delivered as a patch.** Use this when the app belongs to someone else or the user wants to review first.

- Work in a copy of the app.
- Commit a baseline in the copy, so the patch contains only this work, including any uncommitted changes the user already had.

**In the app's own repository.** Use this when the user hands the app over.

- Commit to the branch they name, and only when they say to. Do not push unless asked.
- If someone else (a person or another agent) is editing the same tree, stage only your own changes. Use `git add -p`, or for a file you cannot split cleanly, write the version you want with `git hash-object -w` and stage it with `git update-index --cacheinfo`.
- Commit under your own name. When asked to commit someone else's work, set them as the author.
- If someone else's commit undoes a look the user approved, tell the user and let them choose. Do not quietly change it back.

Either way:

- **Change the look, not the behaviour.** Keep the same props, handlers, accessible names, and disabled and busy rules.
- **Keep the look in shared places.** Put it in a stylesheet or the app's utility classes, with the theme variables. A small shared component (a deck card, a tinted row) is better than copying markup into several pages.
- **Keep test-visible structure** where tests depend on it (a labelled `section` per row, a class a test reads). Update a test only where the change is intended, and say why.
- **Format with the repository's own config.** Run its formatter only after its config is present, keep its line endings (CRLF repositories exist), and check the diff for reformatting noise before you deliver or commit.
- **Run the app's own checks:** type check, lint, format check, and the full test suite.

## Check in the real app, in a browser

A preview page shows the design; only the real components show what the port did. Run the app's frontend in a browser with its backend mocked, and use that as the main check, not a single screenshot at the end:

- **Mock the backend.** Replace the app's backend calls (for Tauri, `invoke`) with a small mock that answers from fixtures. Add fixtures for each state you changed: empty, many items, a failed call, a busy one.
- **Add a page switcher.** Give the harness a tiny `window.go('Page name')` that clicks the matching navigation button, so every check starts from a known page.
- **Measure, then look.** Read the computed styles and the DOM to confirm what applied, and sample pixels for colours. For motion, record an element's position on every frame through a transition; a jump shows up as one big step in the list.
- **Keep timers running.** A browser pane that is hidden pauses `requestAnimationFrame`, so canvas painting and transitions stop and a check can hang. In the harness only, route `requestAnimationFrame` through `setTimeout`.
- **Rebuild before you trust it.** Reload or restart the harness after each change, and check both themes.

## Deliver

When the work was done in a copy, deliver a **patch and a handoff note** instead of committing in the app's repository. The note covers:

- what changed in each area, and any deliberate behaviour differences;
- the apply commands and the test results;
- screenshots of the real components;
- what could not be checked, such as the real desktop window on each platform, or older web views without `color-mix()`.

Dry-run the patch (`git apply --check`) against the repository's current state. If someone applied an earlier version in the meantime, add a follow-up patch rather than a new full one.

When the work was done in the app's own repository, the note goes into the commit messages and your report to the user. Say what changed, the commit IDs, the checks you ran, and which existing failures were already there.

## Pitfalls met

- A colour written as `hsl(220 9% 30%)` put a "%" into the markup, and a test that checks a card shows no fake "0%" failed. Write colours as `rgb()`.
- A pale yellow brand gave a card too light for white text, until the contrast step was added.
- Running the formatter before copying the repository's config wrapped lines the project keeps on one line. Running it again with the right config did not undo it.
- A script that rewrote files with LF turned every line of a CRLF file into a change.
- The deck card's link button called `event.stopPropagation()`, and tests that call `onClick()` without an event failed. A sibling button does not need it.
- **WebKit hover nudge (seen in macOS WKWebView):**
  - A `filter: brightness()` on hover makes WebKit redraw the card's text and icons a sub-pixel off, so they seem to slide. Lighten with an inset `box-shadow` wash instead.
  - When a child animates on its own layer (a turning logo), WebKit ignores `overflow: hidden` on its parent. `clip-path: inset(0 round 12px)` holds in every engine.
  - The other children need a fixed layer too (`transform: translateZ(0)`), or they shift while their neighbour animates.
- **A sliding tab indicator jumped instead of sliding.** It had two causes:
  - A button that disappeared on one tab shifted the whole group. Fixed with an invisible placeholder.
  - The component remounted several times before its first paint, and each mount remembered the target position. Remember the position last painted, not the target.
- A test's mocked `getComputedStyle` returned only some properties. Helpers that read computed styles must accept `undefined`.
