# StateMachine
StateMachine is music software written in Rust with the goal of creating music
using audio samples arranged in non-linear timelines.  The follow screenshot is
a mockup of what the interface currently looks like:

![screenshot](/assets/screenshot.png)

# Concepts

The main idea is that a timeline for music is data that can be referred to and
manipulated.  For example, imagine creating a drum pattern and being able to
loop parts of it, play it at half speed, etc.  In StateMachine, that drum pattern
would exist on its own timeline, and you would be able to refer to that drum
timeline from a "main" timeline, so you can sequence things like fills, etc.

Timelines can also trigger other timelines using markers (as shown in the screenshot above), and
can control the playback of other timelines.  Thus, StateMachine can be used as
a standard timeline as in a typical Digital Audio Workstation (DAW), or it can be
configured as a more experimental sound engine.

# Technology

The user interface, so far, is handmade using `wgpu` and `winit`.  The audio stack is
also custom built on top of `cpal`.