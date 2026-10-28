# Visual references

This folder collects ideas for colors, textures, atmosphere, and visual styles to explore in the StateMachine editor. Visual design is deferred while the composition model and core editor take shape. These images are references for experimentation; they do not establish a fixed palette, layout, or finished interface style.

Some visible themes to revisit:

- Fine black lines, labels, arrows, and geometric relationships in technical and mathematical diagrams.
- Compact, information-dense audio software interfaces and clearly differentiated event or signal colors.
- Light neutral hardware surfaces with warm yellow controls and restrained dark details.
- Amber/red spectral imagery, saturated blue and turquoise, yellow accents, and strong light/shadow shapes.
- Print grain, patterned surfaces, irregular linework, and rippled or textured glass.

Shader effects are an explicit area of interest for later experiments. Possible studies include procedural grain, material shading, glow, patterned transparency or refraction, and animated light or signal activity. These are candidates to try, with no effect selected yet.

Keep visual styling and effect parameters separate from composition data and audio processing. A plain rendering baseline should make it easy to compare an effect's appearance, readability, motion, and performance. Effects may use additional rendering passes when the result warrants them; the simple UI rendering path remains useful alongside those experiments.

See the [project vision](../VISION.md) and [implementation plan](../IMPLEMENTATION_PLAN.md). Renderer structure is addressed in phase 7; visual/shader experiments can follow once the core workflow is usable, with performance comparison in phase 9.
