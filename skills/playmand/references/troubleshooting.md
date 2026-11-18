# Troubleshooting by symptom

Read only the relevant entry. This records the order of checks and verifiable conclusions; the actual implementation still follows the current project.

## UI state, navigation and saving

| Symptom | Confirm first | Smallest fix direction | Verify |
| --- | --- | --- | --- |
| Rounded corners show seams, text jumps when selected | Are background and border drawn twice; does an interaction state change the reserved size | Simplify duplicated fills; keep layout size when the border changes, rather than forcing every design to drop borders | Capture the default, selected, hovered and other affected combinations, and compare text position and hit areas |
| Navigation disappears after opening a page, or several panels stack | Does navigation belong to a temporary page that gets cleaned up; overlays, z-order and which object receives events | Entry points the project wants to stay up get their own lifecycle; keep page exclusivity rules explicit | Open from each entry point, switch, close and reopen; check entry points exist, are unique and work |
| Switching pages loses drafts, or an old page's save overwrites newer data | Are saved state, per-page drafts and each page's fields mixed together | Keep or drop drafts per the product's meaning; on save, merge only the fields this page owns into current data | Leave a draft on the old page → change another field on a new page and save → save the old page → restart; check both changes survive |
| An error message disappears as soon as focus leaves | Do text changes and focus changes share a clearing path; submit order | Clear errors on real content changes or an explicit product rule; do not copy another project's rule | Invalid submit → change focus → return → edit → valid submit; check the message and the saved result |
| Long text or the last page pushes the panel out of the window | Final layout bounds, parent constraints, scrolling and pagination placeholders | Fix the runaway container and limit the content area if needed; do not just move the screenshot camera to hide overflow | Long names, empty lists, the last page and the supported languages/resolutions; check actual bounds and that buttons are clickable |

Saving a page is not automatically a transaction: first confirm who may change which fields, then decide between merging, a conflict prompt or another strategy. Do not turn "always overwrite the whole object" or "always keep drafts" into a general rule.

## The model is loaded but displays wrong

Narrow it down along this chain:

1. Are the path and target scene correct, are external textures all there, and is the scene instance really ready?
2. Do the bounding box, units, axes, scale and world transform fit the camera? Is it clipped, occluded or excluded by render layers?
3. Are material, alpha, colour space, normals and lighting reasonable? Check the current camera first, and avoid changing exposure, materials and lights at once and losing track of the cause.
4. Does the expected pose come from the static bind pose or an animation? Does the model carry props it should not show, and do instance and bone references point at the current model?

Verify one model in a small scene before adding more. Keep the editable source, the export and source information; record triangle count, material/texture count, size and instance count when they matter to the task. When wiring animation, check the start, mid-action, the end/loop seam, feet touching the ground and held-object alignment.

When a preview needs its own pose, reuse shareable assets and create a separate instance, so the main scene's character is not modified. Restoring the bind pose is only a candidate for a specific model; it cannot guarantee a natural standing pose for any skeleton. After replacing a model, check that old bone references and child entities are released.

## Resources still used when static or hidden, opening gets slower each time

- Measure active cameras, draws, entity/asset counts and background tasks first; do not judge only by whether the image is still.
- Be clear about when objects are created, reused, refreshed, hidden and released, and how many may exist at once. Compare after opening, switching and closing repeatedly and returning to the same state; one delayed release is not immediately a leak.
- Static previews can refresh after content, rotation or theme changes; previews that need animation keep the updates they need. Wait for assets and the actual render to complete before turning the camera off and keeping the output; "wait 32 frames and it works" is not a general rule.
- When hidden, skip unnecessary rendering and pose updates while still advancing clocks the game needs; on showing again, correct to the current state. Zero opacity does not necessarily stop drawing.
- Share reusable meshes, materials and textures, and avoid loading or generating them every frame. Only same-condition measurements support a frame-rate gain; stable counts prove only that round's lifecycle check.

## Screenshot timeouts, black images, stale frames, or an icon change that does not show

1. Check the running program's absolute path, build identifier, working folder, session/PID and config folder. An old program or an old screenshot is the first suspect.
2. Check the exit code, error logs, asset readiness, the target camera and the output-completion signal; open the image and confirm its content matches the scene, rather than only checking that a PNG exists.
3. Check for old render instances or build tasks interfering. Stop only processes this task owns; do not kill instances whose ownership you cannot confirm, and note interference in performance measurements.
4. One successful controlled re-run proves only that this run worked; it does not establish a root cause such as drivers or VRAM. For path problems, compare normalized results and keep minimal repros with spaces or Chinese characters; swapping slashes is not a complete fix.
5. For native icons, check the source asset, build resources, the real window and the distribution package separately. An off-screen game image cannot prove the title bar or taskbar is right, and a platform API request does not prove the OS applied it.

## Time and state interpreted inconsistently

First separate system time, simulation time, pause time, user input facts and their business meaning. Test time boundaries before, exactly at and after the threshold, plus the affected branches for pause/resume, crossing midnight and clock jumps. Do not read a single signal, such as a background process existing or no mouse movement, as the user working or away; the game defines that meaning.
