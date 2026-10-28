use super::*;

#[derive(Debug, Clone, Copy)]
pub struct EvaluationLimits {
    pub max_depth: usize,
    pub max_events: usize,
    /// Counts visits, route scans, and repetitions, including empty material.
    pub max_steps: usize,
}

impl Default for EvaluationLimits {
    fn default() -> Self {
        Self {
            max_depth: 32,
            max_events: 100_000,
            max_steps: 1_000_000,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct PlacementIteration {
    pub placement: PlacementId,
    pub repetition: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AudioPath {
    /// Ordered from instrument to master, including every edge.
    pub routes: Vec<AudioRouteId>,
    pub mixer_channels: Vec<MixerChannelId>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ControlDelivery {
    pub route: ControlRouteId,
    pub instrument: InstrumentId,
    /// Empty means disconnected. Gain/mute/solo are not rendered in this prototype.
    pub audio_paths: Vec<AudioPath>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct EvaluatedEvent {
    pub at: ScoreTime,
    pub source: SequenceId,
    pub event: EventId,
    /// Structural occurrence, not a runtime instance ID. Concurrent launches of
    /// this occurrence will receive separate IDs/playheads in the engine.
    pub placement_path: Vec<PlacementIteration>,
    pub kind: EventKind,
    pub deliveries: Vec<ControlDelivery>,
}

impl Project {
    /// Pure, allocating preparation prototype; never call on the audio thread.
    /// Re-evaluates source references on every call, so no stale schedule cache
    /// can hide source edits. Runtime launch/stop operations are emitted, not run.
    pub fn evaluate(&self, limits: EvaluationLimits) -> Result<Vec<EvaluatedEvent>, ModelError> {
        self.validate()?;
        let mut evaluator = Evaluator {
            project: self,
            steps: limits.max_steps,
            limits,
        };
        let mut events = evaluator.sequence(self.main_score, 0)?;
        events.sort_by(|a, b| {
            a.at.cmp(&b.at)
                .then(a.kind.order().cmp(&b.kind.order()))
                .then(a.source.cmp(&b.source))
                .then(a.event.cmp(&b.event))
                .then(a.placement_path.cmp(&b.placement_path))
        });
        Ok(events)
    }
}

struct Evaluator<'a> {
    project: &'a Project,
    steps: usize,
    limits: EvaluationLimits,
}

impl Evaluator<'_> {
    fn step(&mut self) -> Result<(), ModelError> {
        self.steps = self
            .steps
            .checked_sub(1)
            .ok_or(ModelError::EvaluationLimit)?;
        Ok(())
    }

    fn depth(&mut self, depth: usize) -> Result<(), ModelError> {
        // A caller cannot opt into unbounded Rust stack recursion.
        if depth >= self.limits.max_depth.min(64) {
            return Err(ModelError::EvaluationLimit);
        }
        self.step()
    }

    fn sequence(
        &mut self,
        id: SequenceId,
        depth: usize,
    ) -> Result<Vec<EvaluatedEvent>, ModelError> {
        self.depth(depth)?;
        let sequence = &self.project.sequences[&id];
        let mut result = Vec::new();
        for (&event_id, event) in &sequence.events {
            self.step()?;
            let deliveries = self.deliveries(id, event.kind.output_port())?;
            self.push(
                &mut result,
                EvaluatedEvent {
                    at: event.at.into(),
                    source: id,
                    event: event_id,
                    placement_path: Vec::new(),
                    kind: event.kind,
                    deliveries,
                },
            )?;
        }
        for (&placement_id, placement) in &sequence.placements {
            self.step()?;
            let PlacementStart::At(at) = placement.start else {
                continue;
            };
            let child_events = self.sequence(placement.source, depth + 1)?;
            let selected: Vec<_> = child_events
                .into_iter()
                .filter(|event| {
                    event.at >= placement.region.start.into()
                        && event.at < placement.region.end.into()
                })
                .collect();
            let region_length =
                ScoreTime::from(placement.region.end).checked_sub(placement.region.start.into())?;
            for repetition in 0..placement.repeats.get() {
                self.step()?;
                let offset = region_length.repeated(repetition)?;
                for event in &selected {
                    self.step()?;
                    let mut transformed = event.clone();
                    transformed.at = event
                        .at
                        .checked_sub(placement.region.start.into())?
                        .checked_add(offset)?
                        .scaled(placement.scale)?
                        .checked_add(at.into())?;
                    transformed.placement_path.insert(
                        0,
                        PlacementIteration {
                            placement: placement_id,
                            repetition,
                        },
                    );
                    self.push(&mut result, transformed)?;
                }
            }
        }
        Ok(result)
    }

    fn push(
        &self,
        result: &mut Vec<EvaluatedEvent>,
        event: EvaluatedEvent,
    ) -> Result<(), ModelError> {
        if result.len() >= self.limits.max_events {
            return Err(ModelError::EvaluationLimit);
        }
        result.push(event);
        Ok(())
    }

    fn deliveries(
        &mut self,
        source: SequenceId,
        port: Option<ControlPort>,
    ) -> Result<Vec<ControlDelivery>, ModelError> {
        let Some(port) = port else {
            return Ok(Vec::new());
        };
        let mut deliveries = Vec::new();
        for (&id, route) in &self.project.control_routes {
            self.step()?;
            if route.source == source && route.output == port {
                let mut paths = Vec::new();
                self.audio_paths(
                    AudioOutput::Instrument(route.instrument),
                    AudioPath {
                        routes: Vec::new(),
                        mixer_channels: Vec::new(),
                    },
                    0,
                    &mut paths,
                )?;
                deliveries.push(ControlDelivery {
                    route: id,
                    instrument: route.instrument,
                    audio_paths: paths,
                });
            }
        }
        Ok(deliveries)
    }

    fn audio_paths(
        &mut self,
        source: AudioOutput,
        prefix: AudioPath,
        depth: usize,
        result: &mut Vec<AudioPath>,
    ) -> Result<(), ModelError> {
        self.depth(depth)?;
        for (&id, route) in &self.project.audio_routes {
            self.step()?;
            if route.source != source {
                continue;
            }
            let mut path = prefix.clone();
            path.routes.push(id);
            match route.destination {
                AudioInput::Master => {
                    if result.len() >= self.limits.max_events {
                        return Err(ModelError::EvaluationLimit);
                    }
                    result.push(path);
                }
                AudioInput::Mixer(id) => {
                    path.mixer_channels.push(id);
                    self.audio_paths(AudioOutput::Mixer(id), path, depth + 1, result)?;
                }
            }
        }
        Ok(())
    }
}
