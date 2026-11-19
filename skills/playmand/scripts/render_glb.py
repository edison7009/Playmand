"""Render a glTF/GLB model from four sides into one contact sheet, so you can
look at it before building on it. Runs inside Blender, headless:

    blender -b -P render_glb.py -- model.glb sheet.png [--size 480]

Front, side, back and three-quarter views under neutral three-point light,
framed to the model's bounds, on a mid grey so dark and light parts both
read. It also prints the model's size, triangle count and materials as
Blender imported them. Blender 4.2 or newer (Eevee); it does not change the
model file.
"""
import math
import os
import sys

import bpy
from mathutils import Vector

argv = sys.argv[sys.argv.index("--") + 1:] if "--" in sys.argv else []
if len(argv) < 2:
    raise SystemExit("usage: blender -b -P render_glb.py -- model.glb sheet.png [--size 480]")
src, out = argv[0], argv[1]
size = int(argv[argv.index("--size") + 1]) if "--size" in argv else 480

bpy.ops.wm.read_factory_settings(use_empty=True)
scene = bpy.context.scene
bpy.ops.import_scene.gltf(filepath=src)
meshes = [o for o in scene.objects if o.type == "MESH"]
if not meshes:
    raise SystemExit("no meshes in " + src)
deps = bpy.context.evaluated_depsgraph_get()
corners = [o.matrix_world @ Vector(c) for o in meshes for c in o.bound_box]
lo = Vector((min(c.x for c in corners), min(c.y for c in corners), min(c.z for c in corners)))
hi = Vector((max(c.x for c in corners), max(c.y for c in corners), max(c.z for c in corners)))
centre, dims = (lo + hi) / 2, hi - lo
tris = sum(len(o.evaluated_get(deps).data.loop_triangles) for o in meshes)
mats = sorted({s.material.name for o in meshes for s in o.material_slots if s.material})
print(f"[render_glb] size {dims.x:.3f} x {dims.y:.3f} x {dims.z:.3f} m (Blender X, Y, Z; Z up), "
      f"lowest point z = {lo.z:.3f}, {tris} triangles, {len(meshes)} meshes, materials: {', '.join(mats) or 'none'}")

world = bpy.data.worlds.new("Neutral")
world.use_nodes = True
world.node_tree.nodes["Background"].inputs[0].default_value = (0.42, 0.43, 0.45, 1)
world.node_tree.nodes["Background"].inputs[1].default_value = 0.8
scene.world = world
radius = max(dims.length / 2, 0.01)
for name, d, energy in (("Key", (-1, -1.4, 1.2), 1.0), ("Fill", (1.3, -1, 0.4), 0.35), ("Rim", (0.2, 1.4, 1.1), 0.8)):
    light = bpy.data.objects.new(name, bpy.data.lights.new(name, "AREA"))
    light.data.energy = energy * 120 * radius * radius
    light.data.size = radius
    light.location = centre + Vector(d).normalized() * radius * 3
    light.rotation_euler = (centre - light.location).to_track_quat("-Z", "Y").to_euler()
    scene.collection.objects.link(light)
cam = bpy.data.objects.new("Cam", bpy.data.cameras.new("Cam"))
scene.collection.objects.link(cam)
scene.camera = cam
cam.data.type = "ORTHO"
engines = [e.identifier for e in bpy.types.RenderSettings.bl_rna.properties["engine"].enum_items]
scene.render.engine = next(e for e in ("BLENDER_EEVEE_NEXT", "BLENDER_EEVEE", "BLENDER_WORKBENCH") if e in engines)
scene.render.resolution_x = scene.render.resolution_y = size
scene.render.film_transparent = False
# glTF's front (+Z) arrives as Blender's -Y, so the front view looks along +Y.
views = (("front", (0, -1, 0)), ("side", (1, 0, 0)), ("back", (0, 1, 0)), ("threequarter", (0.8, -1, 0.35)))
tiles = []
for name, d in views:
    d = Vector(d).normalized()
    cam.location = centre + d * radius * 4
    cam.rotation_euler = (-d).to_track_quat("-Z", "Y").to_euler()
    cam.data.ortho_scale = max(dims.x, dims.y, dims.z) * 1.15
    cam.data.clip_start = radius * 0.01  # small props must not fall inside the near plane
    cam.data.clip_end = radius * 10
    scene.render.filepath = out + f".{name}.png"
    bpy.ops.render.render(write_still=True)
    tiles.append(bpy.data.images.load(scene.render.filepath))

# One sheet, four tiles side by side.
import numpy as np  # bundled with Blender

sheet = np.zeros((size, size * len(tiles), 4), dtype=np.float32)
for i, im in enumerate(tiles):
    px = np.array(im.pixels[:], dtype=np.float32).reshape(size, size, 4)
    sheet[:, i * size:(i + 1) * size] = px
img = bpy.data.images.new("sheet", size * len(tiles), size, alpha=True)
img.pixels[:] = sheet.ravel()
img.filepath_raw = out
img.file_format = "PNG"
img.save()
for im in tiles:
    path = bpy.path.abspath(im.filepath)
    bpy.data.images.remove(im)
    if os.path.exists(path):
        os.remove(path)  # keep only the sheet
print(f"[render_glb] wrote {out} ({', '.join(v for v, _ in views)})")
