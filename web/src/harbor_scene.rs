//! WebGL2 side of the harbor shot (see `harbor`): a painted plate brought to
//! life with rain, sweeping searchlights, flickering crane lamps, drifting
//! fog, lightning, and a breathing mecha that sheds water.

use wasm_bindgen::{prelude::*, JsCast};
use web_sys::{HtmlCanvasElement, HtmlImageElement, WebGl2RenderingContext as Gl, WebGlTexture};

use crate::gl::{err, texture_from_image, Mesh, Program};
use crate::harbor::*;

/// Nearer layers drift more with the camera than the plate does.
const MECHA_PARALLAX: f32 = 1.15;
const DROPS: usize = 2600;

/// GLSL version line; the scene's constants arrive as uniforms set from the
/// Rust values (see `common` in `render_at`), so the two cannot drift apart.
const HEADER: &str = "#version 300 es\nprecision highp float;\n";

// The shaders size these arrays by hand; fail the build if the data changes.
const _: () =
    assert!(LAMPS.len() == 29 && EMITTERS.len() == 12 && BEAMS.len() == 6 && BOLT_SEGMENTS == 24);

const COMMON: &str = r#"
uniform vec2 P;
uniform float LOOP;
uniform float PAR_MECHA;
uniform float WATERLINE;
uniform vec2 MIST_C;
uniform vec2 uRes;
uniform float uT;
uniform float uZoom;
uniform vec2 uDrift;
const float TAU = 6.2831853;
float W(float cycles) { return TAU * cycles * uT / LOOP; }
float scaleOf() { return max(uRes.x / P.x, uRes.y / P.y) * uZoom; }
vec2 toPlate(vec2 f, float par) { return (f - uRes * 0.5) / scaleOf() + P * 0.5 + uDrift * par; }
vec2 toScreen(vec2 p, float par) { return (p - P * 0.5 - uDrift * par) * scaleOf() + uRes * 0.5; }
float hash1(float n) { return fract(sin(n) * 43758.5453); }
float hash(vec2 p) { return fract(sin(dot(p, vec2(127.1, 311.7))) * 43758.5453); }
float noise(vec2 p) {
  vec2 i = floor(p), f = fract(p);
  vec2 u = f * f * (3.0 - 2.0 * f);
  return mix(mix(hash(i), hash(i + vec2(1, 0)), u.x), mix(hash(i + vec2(0, 1)), hash(i + vec2(1, 1)), u.x), u.y);
}
float fbm(vec2 p) {
  float v = 0.0, a = 0.5;
  for (int i = 0; i < 4; i++) { v += a * noise(p); p = p * 2.03 + 17.1; a *= 0.5; }
  return v;
}
// Noise that flows with velocity v and still loops seamlessly: two copies
// one loop apart, cross-faded.
float flowFbm(vec2 p, vec2 v) {
  float k = uT / LOOP;
  // Blend that keeps the noise's contrast (plain mixing flattens it mid-loop).
  float a = fbm(p + v * uT), b = fbm(p + v * (uT - LOOP));
  return 0.5 + ((a - 0.5) * (1.0 - k) + (b - 0.5) * k) / sqrt((1.0 - k) * (1.0 - k) + k * k);
}
"#;

const FULLSCREEN_VS: &str = r#"#version 300 es
in vec2 a_pos;
void main() { gl_Position = vec4(a_pos, 0.0, 1.0); }
"#;

const PLATE_FS: &str = r#"
uniform sampler2D uPlate;
uniform vec2 uLamps[29];
uniform float uFlash;
out vec4 o;
void main() {
  vec2 f = vec2(gl_FragCoord.x, uRes.y - gl_FragCoord.y);
  vec2 p = toPlate(f, 1.0);
  // Harbour water: ripples bend the reflections.
  float w = smoothstep(748.0, 800.0, p.y);
  vec2 circle = vec2(cos(W(3.0)), sin(W(3.0))) * 4.0;
  vec2 q = p;
  q.x += w * (2.2 * sin(p.y * 0.55 + W(40.0)) + 3.0 * (noise(vec2(p.x * 0.015, p.y * 0.22) + circle) - 0.5));
  q.y += w * 1.2 * sin(p.x * 0.03 + W(28.0));
  vec3 col = texture(uPlate, q / P).rgb;
  float glint = pow(noise(vec2(p.x * 0.09, p.y * 0.7) + circle * 2.0), 9.0);
  col += w * glint * vec3(0.55, 0.7, 1.0) * 1.4;
  // Crane lamps: warm cores and halos that flicker; a few blink red.
  for (int i = 0; i < 29; i++) {
    float fi = float(i);
    float d = length(p - uLamps[i]);
    float flick = 0.6 + 0.4 * sin(W(16.0 + mod(fi, 5.0) * 8.0) + fi * 2.3);
    if (mod(fi, 6.0) == 0.0) flick = step(0.55, fract(uT * 0.75 + fi * 0.13));
    float glow = exp(-d * d / 70.0) * 0.9 + exp(-d / 34.0) * 0.16;
    col += vec3(1.0, 0.52, 0.16) * glow * flick;
  }
  // Lightning lights the sky and the cloud base.
  float sky = 1.0 - smoothstep(120.0, 720.0, p.y);
  col = col * (1.0 + uFlash * 0.7) + uFlash * sky * vec3(0.32, 0.4, 0.62);
  o = vec4(col, 1.0);
}
"#;

const MECHA_VS: &str = r#"
in vec2 a_pos;
uniform vec2 uMechaPos;
uniform vec2 uMechaSize;
uniform vec2 uCore;
uniform float uBreath;
uniform float uSway;
out vec2 vUv;
out vec2 vLocal;
void main() {
  vec2 L = a_pos * uMechaSize;
  vLocal = L;
  vUv = a_pos;
  // The chest swells around the core as it breathes; the shoulders lift.
  float torso = (1.0 - smoothstep(110.0, 380.0, L.y)) * (1.0 - smoothstep(150.0, 300.0, abs(L.x - uCore.x)));
  L += (L - uCore) * 0.014 * uBreath * torso;
  L.y -= 5.0 * uBreath * (1.0 - smoothstep(0.0, 280.0, L.y));
  // A slow sway about the feet, and a slight dip on each breath.
  vec2 pivot = vec2(uMechaSize.x * 0.5, uMechaSize.y);
  vec2 r = L - pivot;
  float c = cos(uSway), s = sin(uSway);
  L = pivot + vec2(r.x * c - r.y * s, r.x * s + r.y * c);
  L.y += 2.0 * uBreath;
  vec2 sp = toScreen(uMechaPos + L, PAR_MECHA);
  gl_Position = vec4(sp.x / uRes.x * 2.0 - 1.0, 1.0 - sp.y / uRes.y * 2.0, 0.0, 1.0);
}
"#;

const MECHA_FS: &str = r#"
uniform sampler2D uMecha;
uniform sampler2D uMasks;
uniform float uFlash;
uniform float uRim;
uniform float uCorePulse;
in vec2 vUv;
in vec2 vLocal;
out vec4 o;
void main() {
  vec4 c = texture(uMecha, vUv);
  vec3 m = texture(uMasks, vUv).rgb;
  // Lightning strikes behind it: the body stays dark, a silhouette.
  vec3 col = c.rgb * (1.0 + uFlash * 0.08);
  // Searchlight rim, brighter in a lightning strike.
  col += m.r * vec3(0.55, 0.72, 1.0) * (uRim + uFlash * 0.6);
  // Water running down the wet armour.
  float flow = flowFbm(vec2(vLocal.x * 0.11, vLocal.y * 0.022), vec2(0.0, -0.9));
  col += m.b * vec3(0.55, 0.72, 1.0) * pow(flow, 3.0) * 1.4;
  // The core breathes light.
  col += m.g * vec3(1.0, 0.6, 0.2) * (uCorePulse - 0.6) * 1.6;
  // Below the waterline the body fades into the sea.
  float edge = WATERLINE + 26.0 * (noise(vec2(vLocal.x * 0.03, W(6.0))) - 0.5);
  float a = c.a * (1.0 - smoothstep(edge - 45.0, edge + 18.0, vLocal.y));
  o = vec4(col * a, a);
}
"#;

const DROPS_VS: &str = r#"
in float a_pos;
uniform vec4 uEmit[12];
uniform vec2 uMechaPos;
uniform float uBreath;
out float vA;
void main() {
  int id = gl_VertexID / 2;
  bool tail = (gl_VertexID % 2) == 1;
  float fi = float(id);
  float h1 = hash1(fi * 1.37 + 0.1), h2 = hash1(fi * 7.13 + 1.0), h3 = hash1(fi * 3.71 + 2.0);
  vec4 seg = uEmit[id % 12];
  vec2 start = mix(seg.xy, seg.zw, h1);
  // A whole number of lives per loop keeps the stream seamless.
  float life = LOOP / (14.0 + floor(h2 * 10.0));
  float age = mod(uT + h3 * life, life);
  vec2 v0 = vec2((h2 - 0.5) * 34.0, 18.0 + h3 * 34.0);
  float g = 300.0;
  vec2 pos = start + v0 * age + vec2(0.0, 0.5 * g * age * age);
  vec2 vel = v0 + vec2(0.0, g * age);
  vec2 L = tail ? pos - vel * 0.05 : pos;
  L.y += 2.0 * uBreath;
  vA = (1.0 - smoothstep(WATERLINE - 45.0, WATERLINE, pos.y)) * smoothstep(0.0, 0.06, age) * (0.22 + 0.4 * h1);
  vec2 sp = toScreen(uMechaPos + L, PAR_MECHA);
  gl_Position = vec4(sp.x / uRes.x * 2.0 - 1.0, 1.0 - sp.y / uRes.y * 2.0, 0.0, 1.0);
}
"#;

const DROPS_FS: &str = r#"
in float vA;
out vec4 o;
void main() { o = vec4(vec3(0.72, 0.84, 1.0) * vA, 0.0); }
"#;

const FX_FS: &str = r#"
uniform vec4 uBeams[6];
uniform vec4 uBolt[24];
uniform float uBoltN;
uniform vec2 uCoreP;
uniform float uCorePulse;
uniform float uFlash;
uniform float uDpr;
out vec4 o;
float beam(vec2 p, vec4 b) {
  vec2 d = p - b.xy;
  vec2 dir = vec2(cos(b.z), sin(b.z));
  float along = dot(d, dir);
  if (along < 0.0) return 0.0;
  float perp = abs(d.x * dir.y - d.y * dir.x);
  float width = along * tan(b.w) + 5.0;
  float k = 1.0 - smoothstep(0.0, width, perp);
  return k * k * exp(-along / 1500.0) * smoothstep(0.0, 30.0, along);
}
float rain(vec2 f, float cell, float len, float cycles, float slant, float seed) {
  vec2 q = vec2(f.x + f.y * slant, f.y);
  float column = floor(q.x / cell);
  float h = hash1(column * 1.7 + seed), h2 = hash1(column * 3.1 + seed * 2.0);
  if (h2 < 0.3) return 0.0;
  float period = 240.0 + h * 260.0;
  float speed = period * (cycles + floor(h2 * 5.0)) / LOOP;
  float y = mod(q.y - uT * speed + h * period, period);
  float x = abs((fract(q.x / cell) - 0.5) * cell);
  float streak = smoothstep(0.0, len * 0.25, y) * (1.0 - smoothstep(len * 0.5, len, y));
  return streak * (1.0 - smoothstep(0.35, 1.1, x)) * (0.5 + 0.5 * h);
}
void main() {
  vec2 f = vec2(gl_FragCoord.x, uRes.y - gl_FragCoord.y);
  vec2 p = toPlate(f, 1.0);
  vec2 css = f / uDpr;
  float B = 0.0;
  for (int i = 0; i < 6; i++) B += beam(p, uBeams[i]);
  vec3 col = vec3(0.62, 0.74, 0.98) * B * 0.12;
  // Dust and spray caught in the beams.
  vec2 circle = vec2(cos(W(2.0)), sin(W(2.0))) * 3.0;
  col += vec3(0.75, 0.85, 1.0) * B * pow(noise(p * 0.06 + circle), 5.0) * 0.5;
  // Warm light from the core.
  float dc = length(p - uCoreP);
  float coreLight = exp(-dc / 300.0) * uCorePulse;
  // Three layers of rain, lit by the beams, the core and lightning.
  float r = rain(css, 9.0, 70.0, 64.0, 0.16, 1.0) * 0.55
          + rain(css * 0.7, 12.0, 46.0, 48.0, 0.14, 2.0) * 0.35
          + rain(css * 1.6, 7.0, 26.0, 36.0, 0.12, 3.0) * 0.22;
  vec3 rainLight = vec3(0.5, 0.6, 0.8) * (0.14 + B * 1.6 + uFlash * 1.2) + vec3(1.0, 0.6, 0.25) * coreLight * 0.6;
  col += r * rainLight;
  // Fog drifting through the harbour, thicker low down.
  float fog = flowFbm(p * 0.0035, vec2(0.9, -0.08));
  col += vec3(0.32, 0.42, 0.58) * smoothstep(0.35, 0.85, fog) * (0.05 + 0.08 * smoothstep(300.0, 780.0, p.y)) * (1.0 + B * 2.0);
  // Spray and mist churning round the mecha's legs.
  vec2 m = (p - MIST_C) / vec2(340.0, 95.0);
  if (dot(m, m) < 1.0) {
    float mist = flowFbm(vec2(p.x * 0.012, p.y * 0.02), vec2(0.15, 1.1));
    col += vec3(0.6, 0.7, 0.86) * smoothstep(1.0, 0.0, length(m)) * smoothstep(0.3, 0.8, mist) * 0.42;
  }
  // The bolt itself, far behind the cranes: a hot core and a wide glow.
  if (uFlash > 0.01) {
    float d = 1e4;
    for (int i = 0; i < 24; i++) {
      if (float(i) >= uBoltN) break;
      vec2 a = uBolt[i].xy, b = uBolt[i].zw, pa = p - a, ba = b - a;
      d = min(d, length(pa - ba * clamp(dot(pa, ba) / dot(ba, ba), 0.0, 1.0)));
    }
    col += vec3(0.82, 0.88, 1.0) * uFlash * (exp(-d / 1.6) * 2.2 + exp(-d / 20.0) * 0.35);
  }
  // Core bloom.
  col += vec3(1.0, 0.6, 0.22) * uCorePulse * (exp(-dc / 22.0) * 0.8 + exp(-dc / 110.0) * 0.2);
  o = vec4(col, 0.0);
}
"#;

const POST_FS: &str = r#"
out vec4 o;
void main() {
  vec2 f = vec2(gl_FragCoord.x, uRes.y - gl_FragCoord.y);
  vec2 n = f / uRes - 0.5;
  float vignette = 1.0 - dot(n * vec2(1.0, 1.25), n * vec2(1.0, 1.25)) * 0.95;
  float grain = 0.955 + 0.09 * hash(f + floor(uT * 24.0) * 13.7);
  o = vec4(vec3(vignette * grain), 1.0);
}
"#;

#[wasm_bindgen]
pub struct HarborScene {
    gl: Gl,
    canvas: HtmlCanvasElement,
    plate: Program,
    mecha: Program,
    drops: Program,
    fx: Program,
    post: Program,
    tri: Mesh,
    grid: Mesh,
    drop_lines: Mesh,
    textures: [Option<WebGlTexture>; 3],
    // Data that never changes, flattened once for the uniforms.
    lamps: Vec<f32>,
    emit: Vec<f32>,
    bolts: [(Vec<f32>, f32); 2],
    beams: Vec<f32>,
    dpr: f32,
    start_ms: Option<f64>,
    reduced: bool,
}

#[wasm_bindgen]
impl HarborScene {
    /// `recording` keeps each finished frame readable for a frame grabber; the
    /// live page leaves it off, since it costs a copy every frame on mobile GPUs.
    #[wasm_bindgen(constructor)]
    pub fn new(canvas: HtmlCanvasElement, recording: bool) -> Result<HarborScene, JsValue> {
        let options = js_sys::Object::new();
        js_sys::Reflect::set(&options, &"antialias".into(), &true.into())?;
        js_sys::Reflect::set(&options, &"alpha".into(), &false.into())?;
        js_sys::Reflect::set(&options, &"preserveDrawingBuffer".into(), &recording.into())?;
        let gl: Gl = canvas
            .get_context_with_context_options("webgl2", &options)?
            .ok_or_else(|| err("WebGL2 is not available"))?
            .dyn_into()?;
        let head = HEADER;
        let fullscreen =
            |fs: &str| Program::new(&gl, FULLSCREEN_VS, &format!("{head}{COMMON}{fs}"));
        let plate = fullscreen(PLATE_FS)?;
        let fx = fullscreen(FX_FS)?;
        let post = fullscreen(POST_FS)?;
        let mecha = Program::new(
            &gl,
            &format!("{head}{COMMON}{MECHA_VS}"),
            &format!("{head}{COMMON}{MECHA_FS}"),
        )?;
        let drops = Program::new(
            &gl,
            &format!("{head}{COMMON}{DROPS_VS}"),
            &format!("{head}{COMMON}{DROPS_FS}"),
        )?;
        let tri = Mesh::new(&gl, &[-1.0, -1.0, 3.0, -1.0, -1.0, 3.0], 2, false)?;
        let grid = Mesh::new(&gl, &crate::gl::grid(36, 52), 2, false)?;
        let drop_lines = Mesh::new(&gl, &vec![0.0; DROPS * 2], 1, false)?;
        Ok(HarborScene {
            gl,
            canvas,
            plate,
            mecha,
            drops,
            fx,
            post,
            tri,
            grid,
            drop_lines,
            textures: [None, None, None],
            lamps: LAMPS.iter().flatten().copied().collect(),
            emit: EMITTERS.iter().flatten().copied().collect(),
            bolts: [0, 1].map(|k| {
                let segments = bolt(k);
                let n = segments.len() as f32;
                let mut flat: Vec<f32> = segments.iter().flatten().copied().collect();
                flat.resize(BOLT_SEGMENTS * 4, 0.0);
                (flat, n)
            }),
            beams: Vec::with_capacity(BEAMS.len() * 4),
            dpr: 1.0,
            start_ms: None,
            reduced: false,
        })
    }

    /// 0 = plate, 1 = mecha cut-out, 2 = mecha light masks.
    pub fn set_image(&mut self, slot: usize, image: HtmlImageElement) -> Result<(), JsValue> {
        let texture = texture_from_image(&self.gl, &image)?;
        if let Some(old) = self.textures.get_mut(slot).and_then(Option::take) {
            self.gl.delete_texture(Some(&old));
        }
        self.textures[slot] = Some(texture);
        Ok(())
    }

    pub fn ready(&self) -> bool {
        self.textures.iter().all(Option::is_some)
    }

    pub fn set_reduced_motion(&mut self, reduced: bool) {
        self.reduced = reduced;
    }

    pub fn resize(&mut self, css_width: f32, css_height: f32, dpr: f32) {
        // Cap the backing store: the shot is full of soft light, and a
        // 4K canvas would only cost frame rate.
        let dpr = dpr.min(2.0).min(2400.0 / css_width.max(1.0));
        self.dpr = dpr;
        self.canvas
            .set_width((css_width * dpr).round().max(1.0) as u32);
        self.canvas
            .set_height((css_height * dpr).round().max(1.0) as u32);
    }

    /// Draws the frame for wall-clock time `now_ms` (requestAnimationFrame).
    pub fn frame(&mut self, now_ms: f64) {
        let start = *self.start_ms.get_or_insert(now_ms);
        let t = if self.reduced {
            6.0
        } else {
            // Modulo in f64 first: an f32 of hours of elapsed time would step.
            ((now_ms - start) / 1000.0 % LOOP as f64) as f32
        };
        self.render_at(t);
    }

    /// Draws the frame at shot time `t` seconds (for recording).
    pub fn render_at(&mut self, t: f32) {
        if !self.ready() {
            return;
        }
        let gl = &self.gl;
        let (w, h) = (self.canvas.width() as f32, self.canvas.height() as f32);
        gl.viewport(0, 0, w as i32, h as i32);
        let (zoom, drift) = camera(t);
        let flash = lightning(t);
        let b = breath(t);
        let core_pulse = 0.82 + 0.14 * wave(t, 12.0, 0.0) + 0.04 * wave(t, 97.0, 1.3);
        let rim = 0.22 + 0.14 * wave(t, 2.0, 0.5).max(0.0);
        let common = |p: &mut Program| {
            p.bind(gl);
            p.f2(gl, "uRes", w, h);
            p.f1(gl, "uT", t);
            p.f1(gl, "uZoom", zoom);
            p.f2(gl, "uDrift", drift[0], drift[1]);
            p.f2(gl, "P", PLATE[0], PLATE[1]);
            p.f1(gl, "LOOP", LOOP);
            p.f1(gl, "PAR_MECHA", MECHA_PARALLAX);
            p.f1(gl, "WATERLINE", WATERLINE);
            p.f2(
                gl,
                "MIST_C",
                MECHA_POS[0] + MECHA_SIZE[0] / 2.0,
                MECHA_POS[1] + WATERLINE,
            );
        };
        let bind_tex = |unit: u32, tex: &Option<WebGlTexture>| {
            gl.active_texture(Gl::TEXTURE0 + unit);
            gl.bind_texture(Gl::TEXTURE_2D, tex.as_ref());
        };

        // 1. The plate.
        gl.disable(Gl::BLEND);
        common(&mut self.plate);
        bind_tex(0, &self.textures[0]);
        self.plate.i1(gl, "uPlate", 0);
        self.plate.f1(gl, "uFlash", flash);
        self.plate.f2v(gl, "uLamps", &self.lamps);
        self.tri.draw(gl, Gl::TRIANGLES);

        // 2. The mecha, premultiplied over the plate.
        gl.enable(Gl::BLEND);
        gl.blend_func(Gl::ONE, Gl::ONE_MINUS_SRC_ALPHA);
        common(&mut self.mecha);
        bind_tex(0, &self.textures[1]);
        bind_tex(1, &self.textures[2]);
        self.mecha.i1(gl, "uMecha", 0);
        self.mecha.i1(gl, "uMasks", 1);
        self.mecha.f2(gl, "uMechaPos", MECHA_POS[0], MECHA_POS[1]);
        self.mecha
            .f2(gl, "uMechaSize", MECHA_SIZE[0], MECHA_SIZE[1]);
        self.mecha.f2(gl, "uCore", CORE[0], CORE[1]);
        self.mecha.f1(gl, "uBreath", b);
        self.mecha.f1(gl, "uSway", 0.003 * wave(t, 3.0, 0.3));
        self.mecha.f1(gl, "uFlash", flash);
        self.mecha.f1(gl, "uRim", rim);
        self.mecha.f1(gl, "uCorePulse", core_pulse);
        self.grid.draw(gl, Gl::TRIANGLES);

        // 3. Water pouring off the armour, added as light.
        gl.blend_func(Gl::ONE, Gl::ONE);
        common(&mut self.drops);
        self.drops.f4v(gl, "uEmit", &self.emit);
        self.drops.f2(gl, "uMechaPos", MECHA_POS[0], MECHA_POS[1]);
        self.drops.f1(gl, "uBreath", b);
        self.drop_lines.draw(gl, Gl::LINES);

        // 4. Beams, rain, fog, mist and the core's glow.
        common(&mut self.fx);
        self.beams.clear();
        for (i, bm) in BEAMS.iter().enumerate() {
            self.beams.extend_from_slice(&[
                bm.origin[0],
                bm.origin[1],
                beam_angle(i, t),
                bm.spread,
            ]);
        }
        self.fx.f4v(gl, "uBeams", &self.beams);
        // The core as seen in plate space: the mecha layer has more parallax.
        let core = [
            MECHA_POS[0] + CORE[0] - drift[0] * (MECHA_PARALLAX - 1.0),
            MECHA_POS[1] + CORE[1] + 2.0 * b - drift[1] * (MECHA_PARALLAX - 1.0),
        ];
        self.fx.f2(gl, "uCoreP", core[0], core[1]);
        self.fx.f1(gl, "uCorePulse", core_pulse);
        self.fx.f1(gl, "uFlash", flash);
        self.fx.f1(gl, "uDpr", self.dpr);
        let (bolt_data, bolt_n) = &self.bolts[strike_index(t)];
        self.fx.f4v(gl, "uBolt", bolt_data);
        self.fx
            .f1(gl, "uBoltN", if flash > 0.0 { *bolt_n } else { 0.0 });
        self.tri.draw(gl, Gl::TRIANGLES);

        // 5. Vignette and film grain, multiplied in.
        gl.blend_func(Gl::DST_COLOR, Gl::ZERO);
        common(&mut self.post);
        self.tri.draw(gl, Gl::TRIANGLES);
        gl.disable(Gl::BLEND);
    }
}
