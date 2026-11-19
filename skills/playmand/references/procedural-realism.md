# Making code-built 3D look real

Guidance, not yet verified by an example in this repository. It distils what makes fully code-built 3D games look realistic: no image models, no downloaded assets, every model, texture and sound written as code. Read it before building hard-surface 3D (props, buildings, vehicles, machines, weapons) from code, and see [Art direction](art-direction.md) for when code is the right source at all.

Code-built 3D looks like a toy when every part is a clean primitive with one flat material under neutral light. It starts to look real when it carries the evidence of a real object: wear, dirt, damage, labels, water, and light that comes from somewhere. None of that needs an artist; all of it needs deliberate work, layer by layer.

## 1. A modelling kit, then detail density

Write a small kit first and build everything from it, so detail is cheap to add:

- Rounded boxes with a set corner radius (sharp enough to read as metal, not pillows), lathes with arc-length UVs (barrels, tyres, cones, pipes), polygon extrusions, swept profiles (corrugated walls, rails, trims), oriented cylinders, double-sided ribbons (straps, cables).
- A builder with a transform stack (push, pop) that collects parts into one bucket per material and merges each bucket into one mesh at the end. Parts that animate stay separate.
- Cache and reuse: a few seeded variants per prop (`seed % 4`), cloned, rather than thousands of unique meshes.

Then add density. A real object has more parts than it needs: panel seams, bolts, trims, brackets, cables, vents, labels. A real place has clutter. The most common failure is too few, too large, too clean parts.

## 2. Imperfection in the geometry

Perfect shapes read as fake. Add, from a seed:

- Dents (radial pushes in a barrel or panel), sag (sandbags, tarps, cables under gravity), crush (boxes), bulges (containers under load).
- Draped cloth as a height field with a hem; cables as curves with slack.
- Slight random rotation and offset on repeated parts, so rows are not ruler-straight.

## 3. Wear and dirt baked in

Bake grime and occlusion into vertex colours (or a texture) when the mesh is built, from noise and the shape:

- Two or three octaves of 3D noise for blotchy grime.
- Darker and browner near the ground (the bottom 0.2 m), lighter on upward faces, darker in creases and under overhangs.
- Rust bands at rims and seams, damp lines on porous materials, dust on top faces.

This costs nothing at runtime and does more for realism than another light.

## 4. Decals and markings

Real things are labelled and stained. Draw decals on a canvas or in a shader and lay them on surfaces with a small depth offset:

- Stencilled text (weights, codes, warnings) with a distressed edge.
- Rust runs and grime streaks that follow gravity down corrugations and from bolts.
- Hazard stripes, numbers, logos of your own invention.

## 5. Materials that respond to the weather

Patch the engine's standard material (an `onBeforeCompile` patch in three.js, a material extension in Bevy) instead of writing a whole new one:

- Wetness: halve roughness where wet, darken the base colour a little, add raindrop beads on up-facing surfaces only (by the world normal), with a slight normal bump.
- Roughness variation from noise, so highlights break up instead of sliding across a perfect surface.
- Separate looks for metal finishes (bare, anodised, painted, polymer), each with its own wear.

## 6. Light that comes from things

- Lamps are emissive meshes that also report where real lights belong; the scene places lights there from a small pool (only the nearest few are live).
- Strong emissive values with bloom for lamps and glows; additive cones for beams in fog, with fog off on the cone itself.
- One key light with soft shadows, fog for depth, tone mapping (ACES, AgX or similar) and a little grain and vignette.
- Quality presets (low to ultra) that scale shadows, reflections and resolution.

## 7. Sound made in code

Realistic sound is part of realistic looks. Synthesize it (Web Audio, or offline into buffers):

- Layered noise filtered into rain, wind and sea, plus hundreds of tiny random droplet clicks; thunder as low-passed noise with several rumble swells and a low tail.
- Impacts from noise bursts, thumps and resonant partials; a different tail indoors and outdoors.
- A master chain with a compressor and soft clipping; convolution reverb with impulse responses you synthesize; 3D panning; a voice limit that steals the lowest-priority sound.
- Generate long sounds in time slices so the page never freezes, and test the output offline (peak level, silence, NaN).

## 8. Check it like a critic

- Build showcase hooks into the code: named camera shots (side, close-up, wide), a fixed exposure, freezing or pre-simulating moving things, so any view can be rendered and inspected on demand.
- Log what the scene costs (triangles, meshes, draw calls) per asset.
- Render those shots, put them next to the target picture, and list what reads as fake: clean surfaces, floating or intersecting parts, repeated patterns, flat light. Fix the biggest and render again.
- Better still, let a fresh reviewer that did not build the asset judge the shots; a builder grading its own work stops too early.

## 9. Budget honestly

This level of realism is many hours of iteration, not one pass: the convincing public examples ran for hours and hundreds of millions of tokens. Tell the user what that costs before starting, and keep the scope small (one place, a few kinds of things) so the detail can be deep.

## What code does not do well

Organic characters, faces, creatures and cloth-covered people still look wrong when built from primitives. Get those from a generator (image-to-3D, image generation) or an artist, then rig, animate, light and stage them in code: see [Art direction](art-direction.md#who-makes-the-art).
