# Assets: getting, checking and bringing them in

Verified parts: [`scripts/check_asset.py`](../scripts/check_asset.py) is unit-tested on GLB, PNG and WAV files and was run on a 4.5-million-triangle Blender export, an audio track and the harbor art; [`scripts/render_glb.py`](../scripts/render_glb.py) was run with Blender 5.3 on that export. The engine notes come from Bevy 0.20.1's glTF extension table and default features. The rest of the page is workflow guidance.

Polished characters, creatures, scenes and footage come from outside: the user, an artist, an asset library or a generator the user runs. Code then makes them move, lights them, adds effects, keeps them fast and ships them. First ask what kind of work it is and what the user has (see [the skill's entry](../SKILL.md#ask-what-the-user-wants-and-what-they-have)); this page is what happens next.

## Where assets come from

In this order:

1. **The user's own files**: art, models, footage, product shots, logos, fonts, music, voice. Use them; ask about their licence if they are not the user's own work.
2. **Open-licence libraries.** Always read the licence on the asset's own page; a site's general reputation is not a licence.
   - CC0 (no conditions, commercial use allowed): Poly Haven (HDRIs, textures, models), ambientCG (PBR textures), Kenney and Quaternius (game models, sprites, UI).
   - Mixed licences, checked per asset: Freesound (from CC0 to non-commercial), OpenGameArt, Sketchfab's downloadable models.
   - Fonts: Google Fonts (mostly the SIL Open Font License).
3. **Generators the user runs or connects**: image generation and editing, image-to-3D (a picture or a three-view sheet to a textured GLB, some with auto-rigging), sound effects, music, voice. The user runs a web app, or connects one with their own key; say what a call costs before the first one. Check each provider's terms for commercial use.
4. **Code**: effects, shaders, procedural textures, synthesized sound, motion graphics, simple stylised geometry. Not detailed characters or scenes ([who makes the art](art-direction.md#who-makes-the-art)).

Candidates are not assets yet. Generated or downloaded options go in a scratch folder; only the ones the user picks move into the project, and the rest are deleted. Downloading is the user's call: say the file, the source and the size first.

## Keep a manifest

One row per asset in `assets/ASSETS.md` (or the project's equivalent), written when the asset arrives, not at the end:

| Name | File | Size in the game | Source | Licence | Author | Changes |
| --- | --- | --- | --- | --- | --- | --- |
| hero | models/hero.glb | 1.8 m tall | image-to-3D from art/hero.png | the provider's terms | generated | decimated to 40k, split hands |
| rain loop | audio/rain.ogg | 24 s loop | synthesized in code | project licence | | |
| crate | models/crate.glb | 0.8 m | Kenney, Survival Kit | CC0 | Kenney | none |

- **Size in the game** (metres, pixels or seconds) is what code scales to; without it assets come out the wrong size.
- Licences with conditions (CC BY: credit; CC BY-SA: share alike) need the credit shipped with the work, for example in a credits screen or a `CREDITS.md`. Non-commercial (NC) assets cannot go into commercial work.
- Keep keys and tokens out of the manifest and the repository.

## Check every asset before building on it

A file that loads is not a file that works. Measure it and look at it:

```sh
python3 scripts/check_asset.py models/hero.glb --expect-height 1.8 --max-tris 60000 --max-texture 2048
python3 scripts/check_asset.py art/hero.png --require-alpha
python3 scripts/check_asset.py audio/hit.wav --max-peak -1
blender -b -P scripts/render_glb.py -- models/hero.glb hero-sheet.png
```

`check_asset.py` (Python standard library only) reports what is really inside and exits 1 when a limit fails, so it can gate a build:

- **Models (glTF/GLB)**: triangles, vertices, meshes, materials, embedded images and their sizes, skins and joint counts, animation clips and their length, and the model's size in metres from its node transforms. It warns when the model looks like centimetres (taller than 100 m), when its lowest point is not at the origin, when normals or texture coordinates are missing, when images live outside a GLB, and when an extension does not load in Bevy (Draco and meshopt compression, quantized meshes, the basisu and WebP texture extensions) or needs a feature.
- **Images**: size, whether there is an alpha channel, and for PNGs how much is really transparent: an alpha channel with every pixel opaque is a baked background.
- **Audio**: WAV length, peak and loudness, clipped samples and leading silence; Ogg codec and length.

`render_glb.py` renders the model from the front, side, back and three-quarter view onto one sheet. Look at it next to the target picture: proportions, missing or inside-out faces, floating parts, the wrong way round, materials that came in grey.

Image-to-3D results in particular: expect hundreds of thousands of triangles, one fused mesh, fingers stuck together, the texture baked with lighting, and sometimes Draco compression. Check before deciding what to fix.

## Fix and prepare in Blender

Blender runs headless from a script (`blender -b -P fix.py -- in.glb out.glb`), so fixes are repeatable and live in the repository:

- Scale to metres and put the origin at the feet or base; apply the transforms on a static prop, not on a rigged character.
- Reduce triangles to the project's budget (a Decimate modifier, applied), and check the silhouette afterwards.
- For code-driven motion, split the model at its joints and parent the pieces to empties placed on the real pivots, named for the joint (`Head`, `Shoulder.L`, `Elbow.L`, ...). The engine rotates the empties; a rigid machine needs no skinning.
- Separate what must move on its own: hands that close, a door, a turret, a wheel.
- Mark glowing parts with an emissive material (and `KHR_materials_emissive_strength` for strong glow), and give glass and decals the right alpha mode.
- Re-export without Draco or meshopt compression for Bevy, then check the new file again.

For a humanoid that needs a full skeleton and walk cycles, use an auto-rigging service or tool the user runs; then check the joint count and clips with `check_asset.py`.

## Bring it into the engine

Bevy 0.20.1:

- Load the scene: `commands.spawn(WorldAssetRoot(asset_server.load(GltfAssetLabel::Scene(0).from_asset("models/hero.glb"))))`. In 0.20 a glTF scene is a `WorldAsset`; older guides and examples say `SceneRoot`, which no longer exists. glTF is Y-up and metres, like Bevy; a model that faces the wrong way gets a rotation on its root entity, not a re-export.
- Drive joints from code by name: after the scene spawns, find the entities with `Name` ("Head", "Elbow.L") and set their `Transform` every frame from your motion functions. Keyframed clips play through an `AnimationGraph` and `AnimationPlayer`.
- Feature flags for formats: the default build loads PNG and HDR images and plays Ogg Vorbis. JPEG, WebP, KTX2 textures and WAV, MP3, FLAC audio each need their feature (`jpeg`, `webp`, `ktx2`, `wav`, `mp3`, `flac`).
- Emissive parts need bloom on the camera to glow; transparent decals need `AlphaMode::Blend`.

Scene readiness, animation clips, graphs and blending in detail: [Bevy assets and animation](assets-animation.md).

Web pages with Rust and WebAssembly: a full Bevy build is several megabytes. For a page, prefer pictures and effects (see [web effects](web-wasm.md)); a 3D model on a page is a separate decision (Bevy on WebGL2, or a small glTF loader in your own renderer) to weigh against size.

## Pitfalls

- Building on a model nobody looked at: it was 100 times too big, faced backwards, or its hands were one lump.
- Draco-compressed GLBs from web tools: they do not load in Bevy; re-export.
- Assets with no recorded licence, or a non-commercial one in commercial work.
- Generated candidates left lying in the project, or kept without the user picking them.
- Sculpting a detailed character from code because no asset was asked for.
