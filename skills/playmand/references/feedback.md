# Game feel: rewards, level-ups and feedback

Verified by the [feedback example](../examples/feedback-tiers/src/lib.rs) (plain Rust, no dependencies, tested in CI) and by the level-up effects first built for EchoBird's pets, checked frame by frame in a browser. Engine-agnostic: the same beats drive a Bevy system, a WebGL2 page or a Macroquad loop.

Use this page when a game needs a reward to *feel* like one: levelling up, collecting, unlocking, evolving, finishing.

## Tier the moments

Decide the tiers first, then make each one read as clearly stronger than the one before. A typical ladder:

| Tier | When | Length | What it adds |
| --- | --- | --- | --- |
| Gain | Progress that is not yet a reward | ~0.45 s | Energy flowing in, a soft glow, a small gulp. **No impact, ring or burst**, so it never looks like a level |
| Level | The common reward, often repeated | ~0.7 s | Crouch, hop, one ground ring, a few sparks. Short enough to chain |
| Milestone | Rare (every tenth level, a boss) | ~1.3 s | A light column, a bigger hop, two rings, particles, a slight camera push |
| Finale | Once (max level, the ending) | ~2.4 s | Everything, then **stillness**: it must not loop |

Repeated rewards stay short; rare ones may be long. If progress can be partial, the partial case needs its own, quieter feedback: reusing the level animation tells players they levelled when they did not.

## Shape one moment

- **Anticipation, action, follow-through**: crouch (scale y below 1) just before the impact, stretch while rising, squash on landing. Scale about the feet, and keep volume by scaling x and z by 1/√(scale y).
- **One impact moment**: rings, particles, the flash and the sound all start there. Expose it as an event (`on_impact`) so audio plays exactly on it instead of after a timer.
- **Do not let the camera follow the hop.** If the camera frames the character every frame, apply hop, squash and spin *after* framing, or the jump becomes invisible. A small push-in (a few percent) at the impact is enough emphasis.
- **Form changes land after the change.** When a transformation finishes at the very end of its animation, add a short landing (rings, particles) after it, then report completion.
- Colours: the product's accent for common tiers, gold for rare ones, and the character's own colour in particles so the effect belongs to it.

## Describe it as data, test it once

Write the effect as a function of `(tier, t)` that returns plain numbers: hop, squash, glow, ring ages, particle ages. Renderers only read them. Then test what matters, without a GPU: each tier is stronger than the last; the gain tier never has an impact; the hop crouches before it rises; at `t = 1` everything is back to rest (hop 0, squash 1, no rings, no particles); reduced motion keeps only a glow.

## Run it safely

- Give every started effect a **serial number**; cancel by bumping it. A frame that was already queued for an old effect checks the serial and does nothing, so it can never finish a cancelled effect or fire its callbacks.
- Fire the impact and the completion **at most once each**, and never after a cancel. Switching characters, leaving the screen, hiding the window and unmounting all cancel.
- Settle game state (currency, level, save) in the game's own logic, not in the animation. The animation reports completion; the game decides whether the reward still applies.
- Stop the frame loop when nothing is animating, and free particles, buffers and timers on cancel.

## Reduced motion

Replace motion with a short, steady glow: no hop, spin, flash, camera push or particles, and keep it under about half a second. Keep real progress and the final result visible: reduced motion must still show what happened.

## Pitfalls

- Partial progress playing the level animation.
- A celebration that loops, or one that blocks the next action for seconds on a reward players repeat.
- The sound firing after the visuals on a fixed timer, or before the reward is actually granted.
- Effects left mid-way at the end of the span (a ring still expanding, the character still squashed).
- A cancelled effect finishing anyway because one queued frame still ran.
