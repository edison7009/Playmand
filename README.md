<div align="center">

<img src="assets/logo.svg" alt="Playmand logo" width="112">

# Playmand

### Rust + Bevy Skill for AI Agents · Games & Videos

Make your AI agent fluent in **Rust + Bevy**,<br/>
so the games and videos it builds actually look good.

[![Bevy 0.20.1](https://img.shields.io/badge/Bevy-0.20.1-eb6834)](https://bevy.org/)
[![Rust 1.97.1](https://img.shields.io/badge/Rust-1.97.1-b7410e?logo=rust)](https://rust-lang.org/)
[![Claude Code](https://img.shields.io/badge/Claude%20Code-skill-f5f5f5)](https://code.claude.com/docs/en/skills)
[![Codex](https://img.shields.io/badge/Codex-skill-f5f5f5)](https://learn.chatgpt.com/docs/build-skills)
[![MIT License](https://img.shields.io/badge/license-MIT-blue)](LICENSE)
[![skills.sh](https://skills.sh/b/edison7009/Playmand)](https://skills.sh/edison7009/Playmand)

**English** | [简体中文](README.zh-CN.md)

</div>

<p align="center">
  <img src="assets/demo-icons.webp" alt="Two rolling 3D star nebulae, a ball, a globe, a ringed planet and a torus, assemble into the Claude and ChatGPT icons in four styles: vortex, rain, big bang and scan." width="100%">
</p>

<p align="center"><a href="https://github.com/edison7009/Playmand/releases/download/demo-videos/playmand-icons-1080p.mp4"><b>▶ Watch in 1080p (MP4)</b></a></p>

<p align="center"><sub>Made with Playmand: 3,600 Bevy particles roll as round 3D nebulae, then assemble into the Claude and ChatGPT icons four different ways.<br/>Fan-made tribute. Not affiliated with or endorsed by Anthropic or OpenAI; Claude and ChatGPT are their respective trademarks.</sub></p>

## Why Bevy, now that agents write the code

<picture>
  <source media="(prefers-color-scheme: dark)" srcset="assets/scorecard-en-dark.svg">
  <img alt="Engine scorecard for the AI coding era: Bevy leads on AI coding friendliness (9.5) and ECS / data-driven design (10), and trails on the visual editor (5)." src="assets/scorecard-en-light.svg" width="100%">
</picture>

- **Top score for AI coding friendliness (9.5) and ECS / data-driven design (10).** In Bevy the whole game is plain Rust: components, systems and state, with no scene files or inspector settings hidden from the agent. Cargo's compiler and tests tell the agent right away when it got something wrong.
- **The weak spot is the visual editor (5), and agents don't need one.** Editors are built for humans clicking around. An agent works in code, and Playmand fills the visual side with Blender + MCP for assets and screenshot checks for what's on screen.
- **2D and 3D are solid (8–8.5).** That's enough for full games, not just prototypes.

<details>
<summary>Scores as a table</summary>

| Dimension | Bevy | Godot | Unity | Unreal |
| --- | ---: | ---: | ---: | ---: |
| AI coding friendliness | **9.5** | 8.5 | 8 | 7.5 |
| ECS / data-driven | **10** | 7 | 8 | 8 |
| 2D games | **8.5** | 9.5 | 8.5 | 7 |
| 3D games | **8** | 8 | 9 | 10 |
| Visual editor | **5** | 9.5 | 10 | 10 |

Scores are a subjective comparison produced by an AI assistant, not a benchmark. Regenerate the chart with `python assets/make_scorecard.py`.
</details>

## Why Rust: C speed, and a compiler that checks the agent's work

<picture>
  <source media="(prefers-color-scheme: dark)" srcset="assets/langspeed-en-dark.svg">
  <img alt="Execution time relative to C: Rust 1.04x, C++ 1.56x, Java 1.89x, Go 2.83x, C# 3.14x, JavaScript 6.52x, TypeScript 46.20x, Python 71.90x, Lua 82.91x." src="assets/langspeed-en-light.svg" width="100%">
</picture>

- **2nd fastest and 2nd most energy-efficient of 27 languages.** In the study above, Rust needed 1.04× C's time and 1.03× its energy. That puts it ahead of C++ (1.56× / 1.34×) and far ahead of common game scripting languages: C# 3.14×, JavaScript 6.52×, Python 71.90×, Lua 82.91×.
- **No garbage collector.** Nothing pauses a frame to collect memory, so frame times stay steady in games and long renders.
- **The compiler reviews the agent's code.** Types, ownership and the borrow checker reject null, data-race and memory errors before the program runs. The agent gets an exact error message to fix instead of a crash at runtime. That's the feedback loop agents work best with.
- **One toolchain.** `cargo` builds, tests and formats, and `Cargo.lock` pins every dependency. No project has to pick its own build system.
- **The costs are real: slow compiles and a steep learning curve.** For people, that's the main reason to avoid Rust. For an agent, the strictness is what helps, and Playmand supplies the Bevy specifics.

| | Rust | C++ | C# | GDScript | Python | TypeScript |
| --- | --- | --- | --- | --- | --- | --- |
| Time vs C (study) | **1.04×** | 1.56× | 3.14× | not measured¹ | 71.90× | 46.20× |
| Memory model | **ownership, no GC** | manual | GC | reference counting | ref counting + GC | GC |
| Errors caught before running | **types, ownership, borrows** | types | types | optional types | none by default | types, erased at runtime |
| Standard build tool | **cargo** | none (CMake, …) | dotnet | Godot editor | pip, uv, … | npm, pnpm, … |
| Compile speed | slow | slow | fast | none needed | none needed | fast |

¹ GDScript is interpreted and was not part of the study. Benchmarks are the study's 2017 Computer Language Benchmarks Game programs; regenerate the chart with `python assets/make_langchart.py`.

## Made with Playmand

Games and videos come from the same code: a Bevy scene you can play in real time can also be recorded frame by frame into a seamless animation. Both demos below were written by an AI coding agent (Claude Code) using this skill.

| Pelican riding a bicycle | ECS galaxy |
| :---: | :---: |
| <img src="assets/demo-pelican.webp" alt="A pelican pedaling a bicycle along a road, with scrolling clouds and hills" width="100%"> | <img src="assets/demo-galaxy.webp" alt="A glowing spiral galaxy of 4,000 stars rotating" width="100%"> |
| The well-known "pelican riding a bicycle" LLM test, popularized by Simon Willison. Here it's an animated Bevy scene instead of an SVG: wheel spin matches road speed, and the legs follow the pedals with two-bone IK. | 4,000 star entities with HDR bloom, moved by one parallel ECS system. Inner stars do more laps, so the arms wind up and come back to their starting shape each loop. |
| [▶ Watch in 1080p (MP4)](https://github.com/edison7009/Playmand/releases/download/demo-videos/playmand-pelican-1080p.mp4) | [▶ Watch in 1080p (MP4)](https://github.com/edison7009/Playmand/releases/download/demo-videos/playmand-galaxy-1080p.mp4) |

Source: [`pelican.rs`](demos/src/bin/pelican.rs), [`galaxy.rs`](demos/src/bin/galaxy.rs) and the header animation [`icons.rs`](demos/src/bin/icons.rs). Run them from `demos/` with `cargo run --bin pelican`, `galaxy` or `icons`. Add `-- --record frames` to save one seamless loop, then `python demos/encode.py frames out.webp`.

### Live in your browser: card pack and voxel pets

<a href="https://edison7009.github.io/Playmand/"><img src="assets/demo-cards.webp" alt="Five holographic cards with cartoon characters, fanned out side by side, one tilted toward the pointer" width="100%"></a>

<p align="center"><a href="https://edison7009.github.io/Playmand/"><b>▶ Play it live</b></a> · tear the pack, hover the cards, hatch and grow the pets</p>

Rust compiled to WebAssembly, drawing straight into WebGL2 in an ordinary web page: no engine and no JavaScript 3D library, 235 KB of WebAssembly (about 92 KB gzipped). Tear a foil pack along a random line; five cards rise out of the slit and turn face up, each with real thickness, a clear coat that catches the light as it tilts and a character floating above the art. Below, six voxel pets hatch from an egg and grow through three forms. The characters were modelled and rendered with Bevy. It began as the TokenPet feature of [EchoBird](https://github.com/edison7009/EchoBird). Source and build notes: [`web/`](web/); what we learned is in the skill's [web effects guide](skills/playmand/references/web-wasm.md).

## Make a game or a video

```text
/playmand Build a top-down 2D action game with dashing, melee combat,
enemies that chase you, loot and leveling. Bright pixel art, punchy hit feedback.
```

```text
/playmand Make a 6-second seamless looping animation of our logo assembling
from glowing particles, 1920x1080, exported as MP4 and as a WebP for the README.
```

Use `$playmand` in Codex. In Claude Code, install it in one line (see [Installation](#installation) for other agents):

```text
/plugin install playmand --marketplace edison7009/Playmand
```

## What Playmand is

**Playmand is a Rust + Bevy skill for games and videos. It helps AI coding agents work fluently with this stack and turn designs into playable 2D / 3D games, or into animations recorded frame by frame.**

At its core is hands-on Rust + Bevy expertise: using the API of the version you actually have, structuring ECS and gameplay state, wiring up model animation, building UI and rendering, tracking down performance problems, and shipping builds. Codex, Claude Code and other skill-capable tools are the hosts that run it.

**Blender + MCP is a key part of asset creation**: the agent drives Blender through a connected MCP server to create and modify models, materials, rigs and animations, checks the result, exports GLB and brings it into Bevy. The skill covers the full technical path from editable asset to in-game result.

New games default to Rust + Bevy; existing Bevy projects keep their version and structure. If the user explicitly picks another engine, that choice is respected, and existing projects are never migrated automatically. The skill does not impose a fixed development loop, and it does not shrink the user's request into a minimal demo.

## Why Rust + Bevy + Agents

Rust's main appeal is performance and reliability: it compiles to native code with no garbage collector and gives fine-grained control over memory and resources, while its type system and ownership checks catch many memory and concurrency bugs at compile time. These traits suit games that care about frame time, resource usage and long-term maintainability. ([Rust](https://rust-lang.org/))

Bevy's ECS architecture organizes game logic around data, runs systems in parallel where possible, and its modular design is easy to extend. Gameplay, state and system behavior are all expressed in Rust code, which agents can read and modify, and get fast feedback on through Cargo builds and tests. ([Bevy introduction](https://bevy.org/learn/quick-start/introduction/))

The cost is a steep learning curve: ownership, borrowing, lifetimes, plus Bevy's queries, scheduling and resource management all have to be understood. Build times, version churn and hard-to-diagnose errors also slow iteration. Fast at runtime does not mean fast to develop.

**That is exactly where Playmand and agents fit together: keep the performance and engineering strengths of Rust + Bevy, and let the agent carry more of the implementation, API verification and debugging, lowering the cost of working with this stack.** You focus on gameplay, style and feel; Playmand gives the agent engine and asset-pipeline knowledge so it guesses fewer APIs and hits fewer pitfalls. Actual development speed and game quality are still measured on real projects and controlled evaluations.

## What's Included

| Content | How it helps |
| --- | --- |
| [Rust / Bevy toolchain](skills/playmand/references/bevy.md) | New project setup, version/API lookup, feature and plugin compatibility, builds and Windows linking |
| [ECS and gameplay state](skills/playmand/references/ecs.md) | Query borrow conflicts, deferred commands, system ordering, fixed-timestep input, pause and restart |
| [Blender and MCP](skills/playmand/references/blender-mcp.md) | Connection and capability discovery, scene editing, modeling/materials/animation, GLB export and Bevy integration |
| [Assets and animation](skills/playmand/references/assets-animation.md) | GLB sub-scenes, instance readiness, animation graph/player wiring, per-instance material and skeleton isolation |
| [Rendering, UI and performance](skills/playmand/references/rendering.md) | 2D/3D cameras, lighting, layers, UI layout, screenshots and preview camera lifecycle |
| [Animation and video recording](skills/playmand/references/video.md) | Fixed-clock frame capture, seamless loops, bloom, WebP/GIF/MP4 encoding |
| [Version inspection script](skills/playmand/scripts/inspect_bevy.py) | Reports Bevy version, features, source locations and docs links from Cargo's actual resolved graph |
| [Runnable Rust example](skills/playmand/examples/ecs-patterns/src/lib.rs) | Four real Bevy ECS behavior tests, for understanding the API or narrowing down a bug |

The technical baseline is **Bevy 0.20.1 / Rust 1.97.1**. For other versions, check that version's source; existing projects should not be upgraded just to match the examples. Reference material is read on demand per problem; the full entry point is [SKILL.md](skills/playmand/SKILL.md).

> A Chinese copy of the skill and its references is in [docs/zh-CN/playmand](docs/zh-CN/playmand/README.md). Agents load the English version, and answer in whatever language you prompt in.

## Installation

**Claude Code (2.1.275 or later).** Install the plugin straight from this repository:

```text
/plugin install playmand --marketplace edison7009/Playmand
```

On older versions, run `/plugin marketplace add edison7009/Playmand`, then `/plugin install playmand@playmand`.

**Any agent that reads skills (via [skills.sh](https://skills.sh)).** This needs Node.js:

```sh
npx skills add edison7009/Playmand
```

**Codex and Claude Code with the bundled installer.** Use this to choose personal or project scope, or to install without Node.js:

Requires Codex or Claude Code, and Python 3.9+. The installer uses only the Python standard library. The engine, compiler, graphics and input tools a game needs are prepared per project; the installer never installs or purchases them.

To use Blender MCP, you also need to configure an MCP server in your host and enable the Blender-side add-on; installing the skill does not set up that connection. See the [Blender MCP guide](skills/playmand/references/blender-mcp.md) for details and boundaries.

```sh
git clone https://github.com/edison7009/Playmand.git
cd Playmand
python scripts/install_skill.py --user --target both
```

You can also download the GitHub ZIP, extract it, enter the folder and run the same Python command. To install for a single host, replace `both` with `codex` or `claude`. Without Python, copy the whole `skills/playmand` folder into the directory shown below.

| Host | Personal install (`--user`) | Project install | Explicit invocation |
| --- | --- | --- | --- |
| Codex | `~/.agents/skills/playmand/` | `.agents/skills/playmand/` | `$playmand help me ...` |
| Claude Code | `~/.claude/skills/playmand/` | `.claude/skills/playmand/` | `/playmand help me ...` |

`~` is the current user's home directory. A personal install is available across that user's projects; to share the skill with a team alongside a game project, use a project install:

```sh
python scripts/install_skill.py --project "/absolute/path/to/your/game" --target both
```

Choose one scope, not both, to avoid duplicate installs with the same name. A project install only copies into the given directory; the installer never touches game source or host configuration files. It refuses to proceed if a regular file occupies the install path, a same-named directory has different contents, or a path points outside the project.

Directory conventions follow the [Codex skill docs](https://learn.chatgpt.com/docs/build-skills) and [Claude Code skill docs](https://code.claude.com/docs/en/skills), checked on 2026-09-13. The shared skill body uses `name` / `description`, an MIT license marker and relative paths, with no host-specific directive syntax; `agents/openai.yaml` is extra display metadata for Codex.

After installing, open a new session in your game project, confirm the skill is discovered, then invoke it explicitly. Older hosts or ones restricted by organization policy need their loading behavior checked against their actual version; a successful file copy does not mean the host has loaded the skill. This applies to local Codex / Claude Code; cloud tasks need the skill brought into their actual execution environment.

The installer skips identical content and exits without overwriting when it finds a same-named directory with different contents. Build output (`target`) and Python caches are not copied and do not count as conflicts. To update: `git pull`, review the diff, move the old install outside the skill discovery directory for safekeeping, then reinstall; installed copies do not sync automatically. The source of truth is `skills/playmand` in this repository.

## Usage

New game in Codex:

```text
$playmand Build a top-down 2D action game with Rust + Bevy.
It needs dashing, melee attacks, enemy pursuit, loot drops and leveling,
plus complete start, pause and results screens.
Use a bright pixel-art style, with strong hit feedback and tight controls.
```

Existing project:

```text
$playmand Fix the current Bevy project: the GLB character renders, but its animation doesn't play.
Keep the versions in Cargo.lock, and check the scene instance, AnimationPlayer and animation graph wiring.
```

Asset creation:

```text
$playmand Use the connected Blender MCP to make a low-poly treasure chest with a lid-opening animation.
Keep the editable .blend, export GLB, bring it into the current Bevy project, and play the animation on click.
```

In Claude Code, replace `$playmand` with `/playmand`. Other hosts can load the whole `skills/playmand` folder according to their own skill mechanism; this repository's installer only targets Codex and Claude Code and does not claim every tool has been tested.

## Validation and Maintenance

- [Validation record](VALIDATION.md): install, host and distribution checks performed, and the current validation scope.
- [Evaluation cases](docs/evaluation.md): evaluate correctness, development time and human intervention on real game tasks.

Maintainers can check and package locally:

```sh
python -B -m unittest discover -s tests -v
python scripts/build_package.py
```

Run the real Bevy example separately (requires the Rust 1.97.1 toolchain):

```sh
cd skills/playmand/examples/ecs-patterns
cargo test --locked
```

The packaging script uses an explicit public-file allowlist and produces `dist/playmand-0.3.0.zip` plus a SHA-256 manifest. GitHub Actions runs the same tests on Windows and Linux. Host behavior and engine/GPU validation scopes are documented in [VALIDATION.md](VALIDATION.md); passing installation does not imply any game will be completed automatically.

## License

Playmand is released under the [MIT License](LICENSE).
