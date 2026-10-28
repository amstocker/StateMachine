# StateMachine
StateMachine is an experimental music application written in Rust for composing
with samples and reusable control sequences. Tracks represent control: a main
score triggers events and auxiliary sequences, which can be transformed and
routed to instruments whose audio feeds a mixer. The current code is an early prototype
of this working vision. The following screenshot is a mockup of the prototype
interface:

![screenshot](/assets/screenshot.png)

# Concepts

The central idea is that control material can itself be sampled. A melody or
rhythm can be referenced, chopped, looped, rearranged, or retimed while retaining
its events. Those events can then drive a sample instrument through a control
routing graph. Retiming a melody and changing an audio sample's playback rate
are separate operations.

For example, a drum pattern written as trigger events can be chopped and
rearranged while remaining linked to its editable source. Changing a source hit
updates the relevant uses; changing its instrument changes the sound. This
workflow should preserve the live control data throughout the arrangement.

The main control track is the score. It can trigger sample playback, launch
auxiliary control sequences, and eventually trigger other kinds of events.
Moving event triggers between control tracks is an interaction the prototype
explores.

The intended flow is main score, auxiliary scores and transformations, control
routing, instruments, and an audio mixer. Routing, instrument configuration, and
mixing may share a tab; sample organization also has its own view. The exact
interface and musical semantics remain open to experimentation.

- [Project vision](VISION.md) records the direction and open design questions.
- [Implementation plan](IMPLEMENTATION_PLAN.md) orders the architecture changes,
  dependency migrations, and correctness fixes.
- [Control model](MODEL.md) documents the implemented project types, timing and
  slice semantics, routing, and the headless composition example.
- [Visual references](design/README.md) collect ideas for later colors, textures,
  styles, and shader experiments.
- [Development and validation](DEVELOPMENT.md) explains how to run the demo,
  execute the regression and rendering checks, and reproduce baseline measurements.

# Running

From the repository root, with Rust installed through rustup:

```sh
cargo run --locked
```

The repository pins the tested Rust toolchain. See the development notes for the
local macOS command-line-tools workaround if Xcode reports a license error.

# Technology

The user interface, so far, is handmade using `wgpu` and `winit`.  The audio stack is
also custom built on top of `cpal`.
