# Motion graphics and video production

Verified by the [motion-video example](../examples/motion-video/src/lib.rs): one timeline renders a 9-second promo (title, data story, end card) offscreen with Bevy 0.20.1 in 16:9, 9:16 and 1:1, with a soundtrack, SRT captions and MP4 encoding, on macOS (Metal). Each MP4 was checked for duration, frame count, codec and audio; the decoded audio was measured against the picture (every cue on its frame, 0 ms off). The timeline, captions and audio are tested in CI. The ffmpeg commands below were not run in that verification.

Use this page when the request is a video meant for people to watch: a product demo, an explainer, a data story, a logo sting, a social short, a launch clip. [Animation and video recording](video.md) covers the capture mechanics this page builds on.

## Pick the kind of video

| Kind | Shape | Length | Notes |
| --- | --- | --- | --- |
| Logo sting / title | Mark builds, name lands, hold | 3–6 s | Often a seamless loop for a README or site header |
| Product demo / launch | Problem → product → key moments → call to action | 15–60 s | Real UI captures or faithful mock-ups; one idea per shot |
| Explainer / tutorial | Steps in order, each with a caption | 30–120 s | The caption says the step; the picture shows it |
| Data story | One claim, one chart, the number lands | 5–20 s | Cite the source on screen; animate toward the point being made |
| Social short | Hook in the first second, vertical | 6–30 s | Captions burned in: many people watch without sound |

Ask for what is missing before building: the message, the audience, where it will be posted (which sets the formats), the length, the brand kit, and the facts the video may state.

## Working from a reference video

When the user gives a video to match, study it before writing the timeline:

1. Pull frames at a steady rate (`ffmpeg -i ref.mp4 -vf fps=4 ref/%04d.png`, or AVFoundation on macOS) and look at them, with the timestamps.
2. Write down the shot list: each shot's start and length, what moves and how it eases, the transitions, where text sits, and where the sound hits.
3. Rebuild that structure as your timeline with the user's own brand kit, copy, product shots and music. Match the rhythm and layout, not the other brand's logo, artwork, footage or music.
4. Compare matching moments side by side (reference frame and your frame at the same time) and list the differences that remain.

This workflow is guidance; the example does not include a reference-matching step.

## Write the timeline first

Describe the whole video as data before drawing anything: shots with start, length and caption; sound cues; the brand kit. Picture, captions and sound are then three renderings of the same numbers and cannot drift apart.

```rust
pub struct Shot { pub kind: ShotKind, pub start: f32, pub len: f32, pub caption: &'static str }
pub struct Cue { pub at: f32, pub sound: Sound }        // derived from the shots
impl Timeline {
    pub fn opacity(&self, shot: usize, t: f32) -> f32 { /* cross-fades */ }
    pub fn caption_at(&self, t: f32) -> Option<usize> { /* hard cuts */ }
    pub fn cues(&self) -> Vec<Cue> { /* a tick when each bar lands */ }
    pub fn srt(&self) -> String { /* captions file */ }
}
```

Derive cue times from the animation constants (`bar_start(i) + BAR_GROW`), never type them separately. Test the timeline without a GPU: every frame has exactly one caption, cross-fades add up to one, every cue sits where its motion ends.

## Brand kit

Keep the brand in one place: palette (background, ink, secondary, lines, one accent), fonts, logo, the product name and address, and the claims the video may make. Rules that keep it on brand:

- One accent colour, used for the one thing each shot is about (the Rust bar, the address). Everything else in ink and neutrals.
- Fonts: for a deliverable, ship the brand's font file with the project (check its licence) and load it by handle, so every machine renders the same frames. A CSS family list (`"Helvetica Neue, Arial, sans-serif"`) works for drafts but needs Bevy's `system_font_discovery` feature and depends on the fonts installed.
- Facts and numbers come from the user or a cited source, shown on screen ("Pereira et al., SLE 2017"). Do not invent statistics, customer names or quotes.

## Motion and timing

- **Ease out** for things arriving, **ease in** for things leaving, **ease in-out** for things moving across. Linear motion looks mechanical.
- **One focal move at a time.** Stagger a group by 0.1–0.3 s (the bars arrive one after another) instead of moving everything together.
- **Hold text long enough to read:** at least about 0.3 s per word plus half a second, and never cut away mid-sentence.
- **A cut is a cross-fade with complementary curves:** the incoming shot fades in over the same window the outgoing one fades out, so the two add up to one. If the new shot's own entrance animation only starts at its start time, the cross-fade shows nothing and dips to the background; start the entrance when the fade begins.
- **The first frame is the thumbnail** on many platforms: make it show the subject, not an empty background.
- **End card:** at least two seconds of name, address or call to action, held still.
- Counting numbers (1.00× → 1.04×) and growing bars should land together, and the sound cue should land on the same frame.

## Use what a game engine is good at

A video rendered by a game engine can move like a game: things with weight and spring, particles, sweeps, depth. The example uses each of these, all posed from the frame's time, so every render is identical:

- **Springy arrivals:** build a title from one entity per letter and let each rise with an overshooting ease (`ease_out_back`), a small tilt that straightens, and a stagger of about 0.05 s. Measure each letter's laid-out width (`TextLayoutInfo.size`) during warm-up and place the letters from those widths.
- **Particles on the beat:** a burst of 20–30 dots thrown out with drag and a little gravity when the title lands, at the same moment as its sound.
- **Wipes:** a slanted band in the accent colour sweeping across on each cut, centred on the cut, over the cross-fade, with the whoosh peaking there.
- **Charts that land:** bars overshoot and settle, the number counts up with the bar, the grid fades in line by line, a glint sweeps the key bar, the key value pulses once.
- **Ripples and pops:** an end-card mark that pops in, a play triangle that spins into place, rings spreading as the chime sounds.
- **Depth:** a few large, faint circles drifting slowly behind everything.
- **Determinism:** every "random" layout comes from a seeded generator. Scramble the seed first: neighbouring seeds (100, 101, 102…) in a plain xorshift give nearly the same first numbers, and the example's background circles all bunched on one side until the seed was scrambled.

Keep it in proportion: one hero move per shot, and motion that supports the message instead of competing with it.

## Formats and safe areas

Render every format from the same timeline; lay out against a safe area, not the full frame.

| Format | Size | Keep clear (left, top, right, bottom) | Typical use |
| --- | --- | --- | --- |
| 16:9 | 1920×1080 | 5% each side | YouTube, sites, presentations |
| 9:16 | 1080×1920 | 6%, 12%, 14%, 22% | Shorts, Reels, TikTok: the app draws buttons on the right and its caption and account name over the bottom |
| 1:1 | 1080×1080 | 6%, 6%, 6%, 8% | Feeds |

- Size type from the short side, and add about 30% for 9:16, which is watched full-screen on a phone. Cap the title's size by the safe width so long names cannot overflow.
- Change the layout, not just the scale: in 9:16 the example puts chart labels above the bars so the bars get the full width.
- Width and height must be even for H.264 with yuv420p.

## Captions

- Burn captions in for social video (many viewers watch muted); also write an SRT file for platforms that take one (YouTube).
- Captions **cut**, they never cross-fade: two half-visible lines are unreadable. Use the same times as the SRT.
- Keep them inside the safe area, about 40 characters a line, and let the text wrap to the safe width.

## Sound

- Render the soundtrack from the timeline: a music bed that fades in and out with the video, plus a cue for each event (a whoosh on a cut, a tick as a bar lands, a chime on the end card). The example mixes it in plain Rust and writes a WAV.
- Normalize so the peak is about -1 dBFS and nothing clips. Platforms normalize loudness (YouTube to around -14 LUFS), so do not push it louder.
- Voice-over and licensed music come from the user or a provider they choose; place them at cue times like any other sound. Do not use music without a licence for the platform.

## Render offscreen

Render into an image, read each frame back once, and advance the clock only after that frame is saved. No window means no black frames when a window is covered, and it runs on a machine with no display. See [video.md](video.md#offscreen-rendering-recommended-for-video) for the code and the row-padding pitfall.

## Encode and check

macOS, no ffmpeg: the skill's [encoder](../scripts/encode_mp4.swift) writes H.264 with an AAC soundtrack and can check a finished file.

```sh
swiftc -O scripts/encode_mp4.swift -o encode_mp4
./encode_mp4 frames/ out.mp4 --fps 30 --mbps 12 --audio soundtrack.wav
./encode_mp4 --verify out.mp4   # duration, size, fps, frame count, codec, audio track
```

With ffmpeg (not run in this verification):

```sh
ffmpeg -framerate 30 -i frame-%04d.png -i soundtrack.wav -c:v libx264 -pix_fmt yuv420p -crf 18 \
  -c:a aac -b:a 192k -shortest out.mp4
ffmpeg -i out.mp4 -i captions.srt -c copy -c:s mov_text with-subs.mp4   # soft subtitles
ffprobe -v error -show_entries stream=codec_name,width,height,r_frame_rate,duration out.mp4
```

Check every export, not just that the command succeeded:

- Duration, frame count, frame rate and size match the timeline; the audio track exists and is as long as the video.
- A contact sheet of key frames (first frame, the middle of each cross-fade, each shot's peak, the end card), viewed at the size people will watch it. Look for empty frames, overlapping captions, text outside the safe area, and overflow.
- Sync: decode the audio back and check cues land on their frames.

## Pitfalls

- Typing cue times separately from the animation: sound and picture drift.
- Captions cross-fading, or two captions on screen at once.
- A cross-fade that dips to the background because the next shot's entrance starts late.
- An empty first frame used as the thumbnail.
- One layout scaled to every format: 9:16 ends up tiny with empty space, or text runs under the platform's buttons.
- Odd frame sizes (H.264 refuses them) and skewed 9:16 frames from unstripped readback padding.
- Numbers or claims on screen with no source.
- Mesh2d shapes that will fade made with `ColorMaterial::from_color(opaque colour)`: it picks an opaque blend mode, which ignores alpha later, so the shape never fades and stays on screen in every shot. Set `alpha_mode: AlphaMode2d::Blend` yourself.
