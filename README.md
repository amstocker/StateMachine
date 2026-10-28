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

# Technology

The user interface, so far, is handmade using `wgpu` and `winit`.  The audio stack is
also custom built on top of `cpal`.
