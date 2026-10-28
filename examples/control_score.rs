use state_machine::model::{fixtures::chopped_drums, EvaluationLimits, IdGenerator, Tick};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut ids = IdGenerator::default();
    let mut fixture = chopped_drums(&mut ids);
    for label in [
        "Original pattern",
        "After moving the second source hit by half a beat",
    ] {
        println!("{label}:");
        for event in fixture.project.evaluate(EvaluationLimits::default())? {
            let frame = fixture.project.tempo.device_frame(event.at, 48_000)?;
            println!(
                "  tick {:?}, output frame {}, event {}, uses {:?}, instrument {}",
                event.at.ratio(),
                frame.0,
                event.event.value(),
                event.placement_path,
                event.deliveries[0].instrument.value()
            );
        }
        fixture
            .project
            .sequences
            .get_mut(&fixture.source)
            .unwrap()
            .events
            .get_mut(&fixture.hits[1])
            .unwrap()
            .at = Tick(1_440);
    }
    Ok(())
}
