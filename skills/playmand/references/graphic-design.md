# Graphic design: posters, social images and slides

Verified by the [poster-design example](../examples/poster-design/src/lib.rs): one piece of content laid out and rendered on the CPU with tiny-skia and the Inter variable font into A4 and A3 print files (300 dpi, 3 mm bleed), a 4:5 feed post, a 9:16 story, a link preview and a 16:9 slide, in about 7 seconds. Every layout is checked for text inside the safe area, nothing overlapping (text, rules or shapes), legible sizes and WCAG contrast; the checks caught three real problems while the example was built (see Pitfalls). Tests run in CI on Windows and Linux.

Use this page for still graphics made in-house: posters, flyers, social images, link previews, banners, slides as images, title cards. A poster that moves (live weather or light around still pictures) is a [web effect](web-wasm.md) or a [video](motion-graphics.md); photos and detailed illustrations inside a design come from the user or a generator ([asset pipeline](asset-pipeline.md)).

## Ask first

What it is for, where it will appear (a print size, or which platform), the copy, the brand kit (colours, fonts, logo), and an example of the style they like. With no example, offer two or three directions as small sketches or prompts ([art direction](art-direction.md)).

## Formats

| Format | Pixels | Bleed | Keep clear |
| --- | --- | --- | --- |
| A4 print | 2480 x 3508 at 300 dpi | 3 mm (35 px) each side | about 7% sides, 6% top and bottom |
| A3 print | 3508 x 4961 at 300 dpi | 3 mm | same |
| Feed post 4:5 | 1080 x 1350 | none | 6-7% |
| Story 9:16 | 1080 x 1920 | none | 13% top (header), 18% bottom (reply bar), 7% sides |
| Square post | 1080 x 1080 | none | 6% |
| Link preview | 1200 x 630 | none | 6% sides, 10% top and bottom |
| Slide 16:9 | 1920 x 1080 | none | 6% sides, 8% top and bottom |

- Print: millimetres to pixels is `mm / 25.4 * dpi`. Lay the background out to the bleed edge, keep text inside the safe margin, and write the resolution into the file so print software places it at the right size.
- Lay out per format, not by scaling one image: wide formats put the data in a column beside the title, tall ones stack it.

## Layout and type

- **Hierarchy**: kicker (small, uppercase, letter-spaced), title (the largest thing, tight leading around 1.0), subtitle, body, data (big numbers with small labels), call to action. One accent colour for one or two things; everything else ink and neutrals.
- **Type scale from the short side** of the page, so a title reads the same on a phone and a wall. Let the title shrink until it fits in three lines rather than run on.
- **One family, several weights**: a variable font holds every weight in one file. Measure text with that font and its kerning; what you measure must be what you draw.
- **Content has priorities.** When a format cannot hold everything, drop the optional piece (the body paragraph) before shrinking anything below a legible size.
- **Decoration has to leave the text alone**: a large accent shape bleeding off a corner, rings, rules. Try it large first and step it down until the layout passes.

## Check every layout

Automate these and run them on every format:

- Every text block inside the safe area.
- No text overlapping other text, a rule, or a solid shape.
- Contrast against what is actually behind the text: at least 4.5:1 for body text, 3:1 for large text (WCAG). A brand's bright accent often fails for small text; keep a darker variant for type.
- Minimum sizes: 9 pt at the print resolution (37.5 px at 300 dpi); about 11 pt on a phone that shows the image 375 points wide (32 px on a 1080 px image).
- Nothing required missing, and the title in three lines or fewer.

Then look: a contact sheet of every format side by side, and each one at the size it will be seen, including as a small thumbnail.

## Export

- PNG for screens. For print, the full file with bleed and the resolution recorded, plus a trimmed preview.
- Not covered by the example: CMYK colour (many printers convert RGB themselves; ask the printer), PDF output, and editable slide files. If the user needs an editable PowerPoint or Keynote file, say so: Rust can produce slide images or a web deck, and the editable file comes from the host's document tools.
- Fonts must be licensed for embedding and distribution (the SIL Open Font License allows it; ship its licence file with the font).

Run the example:

```sh
cd examples/poster-design
cargo test --locked                          # layouts and checks for every format
cargo run --release -- out                   # PNGs and a contact sheet
python3 ../../scripts/check_asset.py out/a4-print.png out/slide.png
```

## Pitfalls

All three of the first were caught by the checks in the example:

- The brand accent used for small text: the orange kicker measured 3.7:1 on the paper; a darker accent for type fixed it.
- Decoration over text: the title ran into the accent disc, and a divider rule cut through the last line of the body. Check text against shapes and rules, not just against other text.
- Small text on tall formats: the kicker came out at 28 px on a 1080 x 1350 post, under the 32 px minimum.
- One layout scaled to every format.
- Measuring text with a different font, size or kerning than the one drawn.
- Text in the bleed or the platform's overlay area.
- A font without a licence that allows embedding.
