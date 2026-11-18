# Card character art

Original artwork: Echo (the pack mascot), Sprout, Twin, Zip and Puff. Each
character is modelled and rendered with Bevy 0.20 from procedural shapes, one
layer at a time (body, eyes, joy, wince, smile, open), then cut out by
rendering every layer on black and on white and recovering the alpha from the
difference. Nothing here is taken from third-party sites or packs.

The generator is in [`web/art-gen`](../../art-gen):

```sh
cd web/art-gen
cargo run -- --char sprout --layer body --bg black --out frames/sprout-body-black.png
cargo run -- --char sprout --layer body --bg white --out frames/sprout-body-white.png
python3 matte.py frames layers   # pairs every *-black.png with its *-white.png
```
