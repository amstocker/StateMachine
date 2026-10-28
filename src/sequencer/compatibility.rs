//! Temporary editing boundary for the original frame-based demo. It preserves
//! its exact clip/junction behavior until the tick-based project is wired to
//! the editor and scheduler in phases 3–4. No geometry or interaction state.
use super::*;

#[derive(Default)]
pub struct PrototypeTrack {
    clips: Vec<Clip>,
    junctions: Vec<Junction>,
}

impl PrototypeTrack {
    pub fn clips(&self) -> &[Clip] {
        &self.clips
    }
    pub fn junctions(&self) -> &[Junction] {
        &self.junctions
    }
}

pub struct PrototypeDocument {
    tracks: [PrototypeTrack; NUM_CHANNELS],
}

impl PrototypeDocument {
    pub fn tracks(&self) -> &[PrototypeTrack; NUM_CHANNELS] {
        &self.tracks
    }
    pub fn length(&self) -> u64 {
        DEFAULT_CHANNEL_LENGTH
    }
}

pub struct PrototypeSession {
    document: PrototypeDocument,
    controller: SequencerController,
}

impl PrototypeSession {
    pub fn new(controller: SequencerController) -> Self {
        Self {
            document: PrototypeDocument {
                tracks: Default::default(),
            },
            controller,
        }
    }

    pub fn document(&self) -> &PrototypeDocument {
        &self.document
    }

    pub fn add_clip(&mut self, track: usize, clip: Clip) -> Result<(), EditError> {
        let target = self
            .document
            .tracks
            .get_mut(track)
            .ok_or(EditError::InvalidChannel)?;
        if target.clips.len() >= MAX_CLIPS_PER_CHANNEL {
            return Err(EditError::ClipsFull);
        }
        self.controller.send(SequencerControlMessage::SyncClip {
            index: ChannelItemIndex {
                channel_index: track,
                item_index: target.clips.len(),
            },
            clip,
        })?;
        target.clips.push(clip);
        Ok(())
    }

    pub fn move_clip(&mut self, track: usize, index: usize, start: u64) -> Result<(), EditError> {
        let target = self
            .document
            .tracks
            .get_mut(track)
            .ok_or(EditError::InvalidChannel)?;
        let clip = target.clips.get_mut(index).ok_or(EditError::InvalidClip)?;
        let length = clip
            .channel_location_end
            .checked_sub(clip.channel_location_start)
            .filter(|length| *length <= DEFAULT_CHANNEL_LENGTH)
            .ok_or(EditError::InvalidLocation)?;
        let start = start.min(DEFAULT_CHANNEL_LENGTH - length);
        let updated = Clip {
            channel_location_start: start,
            channel_location_end: start + length,
            ..*clip
        };
        self.controller.send(SequencerControlMessage::SyncClip {
            index: ChannelItemIndex {
                channel_index: track,
                item_index: index,
            },
            clip: updated,
        })?;
        *clip = updated;
        Ok(())
    }

    pub fn add_junction(&mut self, track: usize, junction: Junction) -> Result<(), EditError> {
        let target = self
            .document
            .tracks
            .get_mut(track)
            .ok_or(EditError::InvalidChannel)?;
        if target.junctions.len() >= MAX_JUNCTIONS_PER_CHANNEL {
            return Err(EditError::JunctionsFull);
        }
        self.controller
            .send(SequencerControlMessage::SyncJunction {
                index: ChannelItemIndex {
                    channel_index: track,
                    item_index: target.junctions.len(),
                },
                junction,
            })?;
        target.junctions.push(junction);
        Ok(())
    }

    pub fn set_playhead(&mut self, track: usize, playhead: Playhead) -> Result<(), EditError> {
        self.controller.send(SequencerControlMessage::SyncPlayhead {
            index: ChannelItemIndex {
                channel_index: track,
                item_index: 0,
            },
            playhead,
        })
    }

    pub fn flush_pending(&mut self) -> Result<(), EditError> {
        self.controller.flush_pending()
    }

    pub fn latest_summary(&mut self) -> Option<SequencerSummary> {
        let mut latest = None;
        while let Ok(SequencerEvent::Tick(summary)) = self.controller.event_receiver.pop() {
            latest = Some(summary);
        }
        latest
    }
}
