# Art direction: start from a picture

Verified parts: [`scripts/cutout.swift`](../scripts/cutout.swift) was tested on generated-style images on plain and gradient backgrounds; its masks matched the true alpha with an IoU of 0.99, semi-transparent wings and feathers kept. The rest of this page is workflow guidance.

Code and engines are the easy part for an agent: given a picture of what the result should look like, matching its layout, palette, light and effects in Bevy, a video or a page is straightforward. Inventing the look from words is the hard part, and code-built art (boxes, primitives) rarely looks finished. So for new visual work, get a picture first.

## When to do it

Do it for new visual work where the look matters and no reference exists yet: a new game, a promo video, a web effect, a character, a scene.

Skip it, and build straight away, when:

- the user already gave references, art, a style guide or an existing project to match;
- the task is a fix, a refactor, logic, performance, or a feature in a project that already has its look;
- the user says to just build it, or declines. Then pick a sensible default and say which.

Never block on it: it is an offer that saves a wasted build, not a gate.

## Where the picture comes from

If the user gave references or art, use them. Otherwise there are four ways to get a picture; tell the user which apply, and do not pick a paid one for them:

1. **The host's own image generation.** Some agent hosts generate images themselves (Codex, for example), and a session may have an image tool or MCP server. Check what this session can call; if it can, use it.
2. **The user's image app.** Write the prompts and hand them over: *"I think you want something like this. Paste this into your image app (Gemini, Doubao, ChatGPT, Midjourney...) and send me the one you like."* Many of these apps are free. Simplest, and the default when nothing else is set up.
3. **An image API the user connects.** For batches or a fully automatic pipeline. Kinds of service: image generation and editing (Google Gemini, OpenAI, xAI and others), image-to-3D with rigging (Tripo, Meshy and others), sound and music. The user picks one and puts its key in an environment variable; then read that provider's current API docs and wire it in. Say what a call costs before the first one, keep keys out of code and the repository, and never sign up or pay on the user's behalf.

4. **Existing pictures, found rather than made.** For a well-known kind of thing (a giant robot, a cyberpunk street, a cosy farm), plenty of pictures already exist. Ask the user to search, or search by image, and send a few they like; or, if this session can search the web, find some and ask "is this the look you mean?". Found pictures are **references only**: use them to agree on style, palette, proportions and mood, not as assets in the work, unless their licence allows it. And a named franchise (a specific robot series, game or character) belongs to someone else: design an original one in that genre instead of copying it.

Whichever way, the rest is the same: settle the style, make the assets, build, and check against the picture.

## Step 1: two or three directions

Offer 2–3 clearly different directions, each a short label and one prompt, so the user chooses by looking rather than by describing:

```text
A. Bright storybook: hand-painted fantasy, warm daylight, soft shadows, saturated
B. Dark crystal: night scene, glowing crystals and runes, deep blues, strong bloom
C. Clean toy: smooth glossy materials, pastel colours, studio lighting
```

Write each prompt in this order, in the user's language (image apps handle Chinese and English; Doubao is strongest in Chinese):

1. **What it is and how it will be used**: "key art for a fantasy auto-battler game, a hexagonal arena seen from a 3/4 top-down camera".
2. **Subject and composition**: what is in frame, where, camera angle, distance.
3. **Style**: medium and finish (hand-painted, 3D render, pixel art, flat vector), detail level, references to *genres and moods*, never to another company's game, characters or artist.
4. **Palette and light**: 3–5 colours, time of day, key light direction, glow.
5. **Format**: aspect ratio (16:9 for a scene, 9:16 for a phone, 1:1 for a card), no text or logos in the image unless wanted.

## Step 2: settle the style

When the chosen image comes back, look at it and write a short **style sheet** in the project (`art/STYLE.md`): palette as hex values, light (direction, warmth, contrast), materials, edges and outlines, level of detail, mood words, and what to avoid. Keep the chosen image as `art/target.png`. Every later prompt and every check refers to these two files.

## Step 3: assets that match

Ask for assets in prompts that *include the target image as a reference* (most apps take an image plus text), so they stay in one family. For each kind:

- **Characters and props**: one per image, full body, centred, facing three-quarters, **on a plain flat background** of a colour not in the subject, no shadow on the floor, no text. A plain background makes the cut-out clean.
- **Variations**: generate one hero first, then ask for the next with the hero attached: "same style and proportions as the attached image; change only: a mage in purple robes". Prompting only the difference keeps the family together.
- **Backgrounds and key art**: the exact aspect ratio, nothing important near the edges.
- **Icons and small sprites**: draw them big (512–1024 px) with bold, simple shapes; detail vanishes when scaled to 64 px. Several icons in one image as a grid is fine; slice it afterwards.
- **Facing**: generators get left and right wrong; generate one facing and mirror in the engine.

Then cut them out and normalise them:

```sh
swiftc -O scripts/cutout.swift -o cutout          # macOS 14+, offline, free
./cutout art/incoming -o art/cut --trim --pad 16 --max 1024
```

It writes a transparent PNG per image and prints each subject's share of the frame; a share near 0% or 100% means the cut-out failed (busy background, subject touching the edges): ask for that image again on a plain background. Look at every result on a contrasting colour before using it. On other systems, any background-removal tool works the same way.

Keep an asset list in `art/README.md`: name, file, **size in the game** (metres, pixels or seconds), and where it came from. Without the in-game size, code scales assets wrongly.

## Step 4: build, then compare with the target

Build the scene in the engine with the assets, then render it and put the frame **next to `art/target.png`**. Compare and write down the gaps: palette, light direction and warmth, contrast, detail, composition, mood. Fix the biggest gap and compare again. "It compiles" or "it runs" is not the finish line; looking like the target is.

Engines add what a still picture cannot: motion, light that moves, particles, camera, timing. See [game feel](feedback.md) and [motion graphics](motion-graphics.md).

## 2D characters in a 3D scene

Generated illustrations look better than code-built models. A scene can be 3D (board, terrain, lights, particles, camera) with the characters as cut-out 2D art standing in it: a quad facing the camera, a soft blob shadow under it, a rim of light, a little bob and squash. Full-screen cut-ins of the same art make skills and wins feel big. Turning images into 3D models needs a paid or GPU-heavy service; offer it only if the user wants it.

## Pitfalls

- Building first and asking about the look afterwards: a whole build thrown away.
- Each asset prompted from scratch: five styles in one game. Attach the hero or target image.
- Busy or gradient backgrounds and floor shadows in asset images: dirty cut-outs.
- Small icons generated small: muddy at game size.
- Prompts naming another company's game, characters or artist. Describe the genre and mood instead.
- No in-game sizes recorded: assets scaled wrongly in code.
