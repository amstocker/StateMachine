# Development and validation

Run commands from the repository root so the demo can find `assets/samples/`.
`rust-toolchain.toml` pins Rust 1.99.1, including rustfmt and Clippy; `Cargo.lock`
pins dependencies. This is the tested compiler, not a minimum supported version.

```sh
cargo run --locked
cargo fmt --all -- --check
cargo check --locked --all-targets
cargo test --locked --all-targets
cargo clippy --locked --all-targets
cargo build --locked --release
```

On this development machine, the selected Xcode installation has an unaccepted
license. Prefix commands with
`DEVELOPER_DIR=/Library/Developer/CommandLineTools` to use the installed command
line tools. For example:

```sh
DEVELOPER_DIR=/Library/Developer/CommandLineTools cargo run --locked
```

This does not change the machine's global developer-tool selection. `--offline`
can also be added once dependencies have been downloaded. CI runs formatting,
build, tests, and Clippy on macOS. Existing warnings remain visible; warnings are
not yet treated as errors. GPU tests are explicitly opt-in.

## Current demo

The four tracks contain snare, flute, kick, and hi-hat samples, from top to bottom.
All playheads start stopped. The timeline still uses 500,000 device frames rather
than a musical timebase. This is the original sample-clip prototype, alongside the
new headless control-oriented model described in [MODEL.md](MODEL.md).

- Click empty track space outside the narrow junction lane to start a playhead.
  Left click plays forward; right click plays backward. It stops at a track end.
- Drag an existing clip horizontally to move it on its current track.
- Drag from the narrow lane at the top of a track to another track to create a
  split jump: the source keeps playing and starts the destination playhead.
  Releasing on the same track creates a reflection at the release position.
- A rejected edit appears in the diagnostic label. Each track currently supports
  32 clips and 32 junctions; the sample bank supports 32 sounds.

Cross-track clip moves, undo, saving, source offset/rate playback, a mixer, and
interactive use of the new control model are still future phases. Input routing, layering,
text metrics, and demand-driven redraw are also still on the plan.

## Regression checks

`src/lib.rs` exposes the prototype modules so tests can construct the sequencer,
sample bank, import pipeline, and interaction state without opening a window or
audio device. Debug audio tests use the existing allocation guard. Coverage
includes:

- Ten seconds of generated audio with an undrained status queue, followed by
  resumed status consumption.
- Ordered control delivery under queue pressure, bounded rejection, invalid
  indices, and UI state remaining unchanged when an edit is rejected.
- Clip/junction capacity and cross-track junction ownership.
- Identity, up/downsampling, short/empty input, invalid ratios, and duration at
  44.1/48/96 kHz. The temporary linear interpolator still has no anti-alias filter;
  Rubato remains the planned replacement.
- Mono 16-bit PCM and 32-bit float import, explicit rejection of other formats,
  malformed/truncated input, non-finite samples, and bundled sample loading.

Status updates may be dropped when the audio-to-UI queue is full. Control edits
use the existing 1,024-entry audio queue plus a 1,024-entry UI pending queue,
retried during UI updates. An edit is rejected before local mutation if both are
full or the engine is disconnected. Pending edits retain FIFO order. Each audio
sync consumes at most 1,024 messages. This is a bridge to the revision-based
engine protocol in phase 4, not an acknowledgement or preview-coalescing system.

Sample-bank slots are append-only for now: additions are validated and metadata
is published only after queue acceptance. Replacement and retirement still
require phase 4's ownership protocol.

The rendering checks are separate from the device-independent suite:

```sh
cargo test --locked --test rendering -- --ignored --nocapture
cargo run --locked --example render_stress
```

The first reads GPU pixels to verify shapes beyond the former 128-instance
capacity, discarded text, and an empty following frame. The second opens a
window briefly, renders 1,024 rectangles and 1,024 lines per frame, exercises
resize and zero-size suspension, and exits after 120 redraws. Neither opens an
audio device. They retain the existing 4x MSAA and three-pass rendering path.
Recoverable surface loss/outdated/timeout handling is implemented, but these
driver errors are not forcibly injected by either check.

## Initial implementation baseline — 2026-10-28

Starting checkout: `e02690b` (`codex init 2 (pre-impl)`). The original locked build
passed with no tests and 53 compiler warnings; Clippy had two errors. Rustfmt was
applied as a mechanical step before the behavioral changes. No dependency
versions were migrated in this batch.

After stabilization, 19 device-independent tests passed, as did the explicit GPU
test and the window smoke test. Formatting, all-target checking, Clippy, and the
release build were checked locally. CI configuration is added; hosted CI has not
been run from this working tree. Prototype warnings and the upstream `block
0.1.6` future-compatibility warning remain.

Release reference, **after this stabilization batch** (no pre-change performance
capture was made):

| Setting / measurement | Value |
| --- | --- |
| Machine / OS | Apple M2, integrated Metal GPU, macOS 26.6.2 |
| Build | Rust 1.99.1, `cargo build --locked --release` |
| Workload | Four demo clips, all playheads stopped, no interaction |
| Window | 1,600 × 1,200 physical pixels, 4x MSAA, FIFO presentation |
| Audio output | MacBook Air Speakers, 48 kHz, stereo F32 |
| Redraw frequency | 60 redraws/second after startup |
| Process CPU | Mean 5.5%; range 4.4–7.2% |
| Final resident memory | 88.9 MiB |

CPU was sampled using `ps -o %cpu=,rss=` once per second for 12 samples after a
three-second warmup. This short idle run is a reference, not a performance claim
or a memory-growth test. Redraw counts were collected with:

```sh
STATE_MACHINE_RENDER_STATS=1 cargo run --locked --release
```

Remaining manual baseline work: reference screenshots, listening/interacting
with the demo during playback, and actual OS minimize/restore checks. Screen
capture permission was unavailable during this run; no permission settings were
changed. The smoke test covers programmatic zero-size suspension and resize.

## Phase 2 model validation

The project model is now in `src/model`, independently of the current demo's
frame-based document. [MODEL.md](MODEL.md) records timing, slice, routing,
launch/stop, identity, and migration semantics. Run its example without a window
or audio device:

```sh
cargo run --locked --example control_score
cargo test --locked --test model
```

The example prints six arranged drum hits, then shows the repeated source hit
moving in both affected uses. Sixteen model tests cover source/placement edits,
stable identities, instrument replacement, control/audio routes, cycles, nested
transforms, slice boundaries, device-rate conversion, and evaluation limits.
The full headless suite now has 36 tests, including an additional check that the
prototype's hit testing follows document edits without cached geometry. Tests
and Clippy pass locally; the model also compiles directly with `rustc` without
any external libraries. Formatting and the application build pass. A four-second
debug startup check opened the existing demo, initialized 48 kHz stereo output,
and rendered at approximately 60 redraws/second without an early exit; the check
then terminated its own process. This was not a listening or interaction test.
Existing prototype and upstream warnings remain.

The old application coordinator is renamed from `Instrument` to `App`. Its
`PrototypeSession` owns clip/junction data and queue publication, while the UI
reads the document and derives geometry. There is no automatic conversion from
the legacy demo to the new model. The phase 2 fixture is not yet audible, and
this batch does not change the render passes or dependency versions.

Next is phase 3: validated project edits, revisions, undo/redo, and gesture
transactions. Keep the ID generator outside history, prove event moves between
sequences without changing identity, and then wire the editor to the interface.
