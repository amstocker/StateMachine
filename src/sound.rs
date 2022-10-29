use std::io;
use std::fs;
use std::convert::AsRef;
use std::sync::Arc;


pub type SoundID = usize;

pub struct Sound {
    pub id: SoundID,
    pub filename: String,
}

#[derive(Clone)]
pub enum Delay {
    Milliseconds(u32),
    Tempo {
        count: u32,
        division: u32,
        swing: f32
    }
}

#[derive(Clone)]
pub struct TriggerInfo {
    pub target: SoundID,
    pub delay: Delay
}


/* Source:
 *  https://github.com/RustAudio/rodio/issues/141
 */
pub struct SoundData(Arc<Vec<u8>>);

impl AsRef<[u8]> for SoundData {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl SoundData {
    pub fn load(filename: &str) -> io::Result<SoundData> {
        let buf = fs::read(filename).unwrap();
        Ok(SoundData(Arc::new(buf)))
    }
    pub fn cursor(self: &Self) -> io::Cursor<SoundData> {
        io::Cursor::new(SoundData(self.0.clone()))
    }
    pub fn decoder(self: &Self) -> rodio::Decoder<io::Cursor<SoundData>> {
        rodio::Decoder::new(self.cursor()).unwrap()
    }
}