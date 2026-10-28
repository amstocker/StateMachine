use super::*;
use std::collections::{BTreeMap, BTreeSet, VecDeque};

impl Project {
    /// Validate a candidate document before publishing it. Public document data
    /// makes prototypes/fixtures easy to build; phase 3 will own atomic edits.
    pub fn validate(&self) -> Result<(), ModelError> {
        if !self.sequences.contains_key(&self.main_score) {
            return Err(ModelError::MissingSequence(self.main_score));
        }
        let order: BTreeSet<_> = self.sequence_order.iter().copied().collect();
        if order.len() != self.sequence_order.len()
            || order != self.sequences.keys().copied().collect()
        {
            return Err(ModelError::InvalidSequenceOrder);
        }
        let mut event_ids = BTreeSet::new();
        let mut placement_ids = BTreeSet::new();
        let mut dependencies = Vec::new();
        for (&owner, sequence) in &self.sequences {
            if sequence.length.0 == 0 {
                return Err(ModelError::InvalidTime);
            }
            for (&id, placement) in &sequence.placements {
                if !placement_ids.insert(id) {
                    return Err(ModelError::DuplicatePlacement(id));
                }
                let source = self
                    .sequences
                    .get(&placement.source)
                    .ok_or(ModelError::MissingSequence(placement.source))?;
                if placement.region.start >= placement.region.end
                    || placement.region.end > source.length
                {
                    return Err(ModelError::InvalidRegion);
                }
                let duration = placement.duration()?;
                if let PlacementStart::At(at) = placement.start {
                    if ScoreTime::from(at).checked_add(duration)? > sequence.length.into() {
                        return Err(ModelError::OutOfBounds);
                    }
                }
                // Include triggered references, even if no launch currently uses
                // them, so enabling a launch cannot introduce hidden recursion.
                dependencies.push((owner, placement.source));
            }
            for (&id, event) in &sequence.events {
                if !event_ids.insert(id) {
                    return Err(ModelError::DuplicateEvent(id));
                }
                if event.at >= sequence.length {
                    return Err(ModelError::OutOfBounds);
                }
                match event.kind {
                    EventKind::Trigger { velocity } => {
                        if !velocity.is_finite() || !(0.0..=1.0).contains(&velocity) {
                            return Err(ModelError::InvalidValue);
                        }
                    }
                    EventKind::SetParameter { parameter, value } => {
                        validate_parameter(parameter, value)?;
                    }
                    EventKind::Launch { placement } | EventKind::Stop { placement } => {
                        let target = sequence
                            .placements
                            .get(&placement)
                            .ok_or(ModelError::MissingPlacement(placement))?;
                        if matches!(event.kind, EventKind::Launch { .. })
                            && target.start != PlacementStart::OnLaunch
                        {
                            return Err(ModelError::LaunchRequiresTriggeredPlacement(placement));
                        }
                    }
                    EventKind::StopCurrent => {}
                }
            }
        }
        if cyclic(self.sequences.keys().copied(), dependencies) {
            return Err(ModelError::SequenceCycle);
        }
        for instrument in self.instruments.values() {
            if !self.samples.contains_key(&instrument.sample) {
                return Err(ModelError::MissingSample(instrument.sample));
            }
            validate_parameter(Parameter::Gain, instrument.gain)?;
            validate_parameter(Parameter::PlaybackRate, instrument.playback_rate)?;
        }
        for (&id, route) in &self.control_routes {
            if !self.sequences.contains_key(&route.source) {
                return Err(ModelError::MissingSequence(route.source));
            }
            if !self.instruments.contains_key(&route.instrument) {
                return Err(ModelError::MissingInstrument(route.instrument));
            }
            if route.output != route.input {
                return Err(ModelError::IncompatibleControlPorts(id));
            }
        }
        for channel in self.mixer_channels.values() {
            validate_parameter(Parameter::Gain, channel.gain)?;
            if !channel.pan.is_finite() || !(-1.0..=1.0).contains(&channel.pan) {
                return Err(ModelError::InvalidValue);
            }
        }
        let mut audio_dependencies = Vec::new();
        for route in self.audio_routes.values() {
            match route.source {
                AudioOutput::Instrument(id) if !self.instruments.contains_key(&id) => {
                    return Err(ModelError::MissingInstrument(id));
                }
                AudioOutput::Mixer(id) if !self.mixer_channels.contains_key(&id) => {
                    return Err(ModelError::MissingMixer(id));
                }
                _ => {}
            }
            if let AudioInput::Mixer(id) = route.destination {
                if !self.mixer_channels.contains_key(&id) {
                    return Err(ModelError::MissingMixer(id));
                }
                if let AudioOutput::Mixer(source) = route.source {
                    audio_dependencies.push((source, id));
                }
            }
        }
        if cyclic(self.mixer_channels.keys().copied(), audio_dependencies) {
            return Err(ModelError::AudioCycle);
        }
        Ok(())
    }
}

fn validate_parameter(parameter: Parameter, value: f32) -> Result<(), ModelError> {
    if !value.is_finite()
        || match parameter {
            Parameter::Gain => value < 0.0,
            Parameter::PlaybackRate => value <= 0.0,
        }
    {
        return Err(ModelError::InvalidValue);
    }
    Ok(())
}

/// Topological validation uses no recursion, including on malformed documents.
fn cyclic<K: Copy + Ord>(nodes: impl Iterator<Item = K>, edges: Vec<(K, K)>) -> bool {
    let mut incoming: BTreeMap<K, usize> = nodes.map(|id| (id, 0)).collect();
    let mut outgoing: BTreeMap<K, Vec<K>> = BTreeMap::new();
    for (source, dest) in edges {
        *incoming.get_mut(&dest).expect("references validated") += 1;
        outgoing.entry(source).or_default().push(dest);
    }
    let mut ready: VecDeque<_> = incoming
        .iter()
        .filter_map(|(&id, &count)| (count == 0).then_some(id))
        .collect();
    let mut visited = 0;
    while let Some(id) = ready.pop_front() {
        visited += 1;
        for dest in outgoing.get(&id).into_iter().flatten() {
            let count = incoming.get_mut(dest).expect("references validated");
            *count -= 1;
            if *count == 0 {
                ready.push_back(*dest);
            }
        }
    }
    visited != incoming.len()
}
