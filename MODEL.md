# Control project model

The phase 2 model in [src/model](src/model/mod.rs) represents editable composition
data without a window, graphics objects, audio device, thread queues, or decoded
sample buffers. It uses only the Rust standard library. The headless example is:

```sh
cargo run --locked --example control_score
```

It prints a chopped drum score before and after moving one source hit. It does
not play audio. The existing windowed demo still uses its frame-based sequencer;
the new editor and playback scheduler are phases 3 and 4.

## Ownership and identities

`Project` designates one main score and owns sequences, sample references,
sampler definitions, control routes, mixer channels, and audio routes. Initially,
each control row presents a sequence; instrument and mixer counts are independent
of row count. `sequence_order` is presentation order, while all references use
typed IDs. Events and placements have one owning sequence and globally unique
IDs within their respective entity kinds.

`IdGenerator` belongs to an editing session, outside project snapshots and undo
records. Deleting an entity and restoring it uses the same ID. Undo must not
rewind the generator, so a new history branch cannot accidentally reuse a deleted
entity's identity. Persistence will need to restore the allocator high-water
mark; there is no project file format yet.

The public structs currently support building candidate documents directly.
`Project::validate` checks a candidate before use, and `Project::evaluate` always
validates first. Invalid intermediate edits must not be published to the eventual
editor/engine. Phase 3 will provide that atomic mutation boundary and revision
tracking.

Three different concepts remain separate:

- A **sequence** is reusable source material: events and nested placements.
- A **placement** references a source region, an offset, a duration multiplier,
  and a finite repeat count. It does not copy the source events.
- A **runtime instance** will be one execution of that material. No playhead or
  running-instance state is stored in this model. Evaluation records the path of
  placements and repetitions for provenance; concurrent launches of the same
  path will still need different engine instance IDs.

## Timing and slice semantics

The initial authoring unit is **960 ticks per quarter-note beat**, with one
constant tempo represented in thousandths of a BPM. Event positions and source
regions use integer `Tick`s. `SourceFrame` means a frame in an original audio
sample; `DeviceFrame` means a frame on the output clock. They are separate types.

`TimeScale` is a positive rational duration multiplier: `2/1` doubles the duration
of control material, and `1/2` halves it. It does not change a sampler's audio
playback rate, pitch, or source-frame offset. Zero and reverse scales are not
representable. Musical tempo changes and absolute-time authoring remain future
experiments.

For a source event at local time `t`, the placement maps repetition `r` to:

```text
placement_start + (t - region_start + r * region_length) * time_scale
```

Selection uses the half-open interval `[region_start, region_end)`. An onset at
the start is included; an onset at the end is excluded. Repetitions are adjacent
without duplicating the boundary. Static placements, including their repetitions,
must fit within the owning sequence. Events must occur before their sequence end.

Nested material is evaluated in its own local coordinates, sliced, and then
transformed into its parent coordinates. Intermediate `ScoreTime` values retain
exact rational ticks with checked arithmetic. Only the final conversion to device
frames rounds, to the nearest frame with ties upward. Composition positions do
not change when the output device changes sample rate. Arithmetic overflow is a
reported error rather than a wrapped time or an approximate result.

The initial event set has one-shot instrument triggers, gain/playback-rate
parameter changes, launch, stop-placement, and stop-current operations. Triggers
have velocity, but no sustained duration or implicit release event yet. Slices
select onsets only: they do not reconstruct parameter state from before the slice
or manufacture a trigger for a sound that began earlier. Gates, reverse, state
chasing, and release behavior at slice boundaries require explicit extensions and
tests before they are enabled.

Events are sorted by exact time, then stop operations, parameter changes,
launches, and triggers. Ties within a kind use source ID, event ID, and placement
path. Reordering control rows therefore cannot change simultaneous-event order.
Routes are traversed in ID order. The future scheduler must preserve this ordering
when multiple control times round to the same device frame.

## Launching and routing

A placement is either arranged at a fixed time or marked `OnLaunch`. A launch
event may reference an `OnLaunch` placement owned by its sequence. Every execution
requests a new independent instance. A stop-placement operation targets children
of that placement belonging to the calling instance, while stop-current targets
the calling instance. A split jump can be expressed as a launch into a chosen
region; a transferring jump also needs stop-current. The prototype's reflection
operation stays in the compatibility sequencer until reverse semantics are defined.

The pure evaluator expands **arranged** placements. It emits launch/stop
operations as data; it does not execute them, suppress later events after a stop,
allocate voices, or simulate playheads. Child lifetimes, voice cleanup,
transport/seek, and updates during playback must be implemented and tested by the
phase 4 engine. Do not mistake this evaluator for that scheduler.

Events retain their source sequence when placed elsewhere. Control routes connect
that source's trigger, gain, or playback-rate output to a matching sampler input.
There are no per-placement routing overrides initially. Multiple compatible routes
provide fan-out without duplicating the source definition. Sequence-reference
dependencies and control routes are separate relations.

Audio routes connect sampler outputs to mixer channels or the master, and mixer
channels to other channels or the master. The initial audio topology is stereo.
Control and audio endpoints have different types; gain and trigger control ports
also cannot be connected to each other. Mixer gain, pan, mute, and solo are
document settings, but this phase does not apply DSP or resolve solo groups.
Sample paths are locators, not decoded buffers; file existence and source-frame
bounds belong to asset resolution.

Validation rejects missing references, incompatible control ports, duplicate
event/placement ownership, invalid ranges and numeric settings, recursive source
dependencies (including unused triggered placements), and audio feedback.
Disconnected sources/instruments are allowed while constructing a project. Their
evaluation has no control deliveries or no path to the master, respectively.

## Evaluation and the migration boundary

Evaluation produces event times, source identities, placement provenance,
instrument deliveries, and complete audio paths to the master. It always reads
the current source definitions. Moving a source event can move it into or out of
several slices; editing a placement changes only that use; replacing a sampler's
sample leaves the arrangement intact. The fixture and [tests](tests/model.rs)
exercise these distinctions.

Evaluation allocates and sorts. It is a preparation/reference implementation for
the main thread or a worker, **never the audio callback**. Limits cap recursion,
events per intermediate sequence, paths per delivery, and traversal steps,
including empty repeats. This is not a byte-budgeted production compiler; route
lookups and repeated schedule construction are deliberately simple. Phase 4 must
prepare bounded engine data and handle resource retirement outside the callback.
No evaluation cache exists yet; revision/dependency invalidation will be required
if caching is introduced.

The existing application's coordinator is now [App](src/app.rs). Its old
clip/junction data is owned by [PrototypeSession](src/sequencer/compatibility.rs),
which validates/queues an edit before changing its `PrototypeDocument`.
`SequencerInterface` derives drawing and hit-test geometry from document reads;
there are no `ClipInterface.model` copies to invalidate. Interaction state and
playback telemetry stay in the interface, outside document data.

The compatibility document is intentionally temporary and is not a second copy
of a tick-based `Project`. There is no automatic clip-to-event conversion or
two-way synchronization. This preserves exact frame positions and existing
junction behavior while phases 3–4 introduce the new editor and engine. The next
step is a validated event move with undo/redo, followed by gesture transactions
and playback integration through those boundaries.
