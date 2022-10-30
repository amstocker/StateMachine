use std::io;
use std::fs;
use std::convert::AsRef;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;


pub type SoundID = usize;

fn generate_id() -> SoundID {
    static COUNTER:AtomicUsize = AtomicUsize::new(0);
    COUNTER.fetch_add(1, Ordering::Relaxed)
}

pub struct Sound {
    pub id: SoundID,
    pub path: PathBuf,
    pub name: String,
    pub data: SoundData
}

impl Sound {
    pub fn new(path: String) -> Self {
        let path = PathBuf::from(path);
        let name =  path.file_name().unwrap()
            .to_str().unwrap()
            .to_owned();
        let data = SoundData::load(&path).unwrap();
        Self {
            id: generate_id(),
            path,
            name,
            data
        }
    }
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
    pub fn load(path: &PathBuf) -> io::Result<SoundData> {
        let buf = fs::read(path)?;
        Ok(SoundData(Arc::new(buf)))
    }
    pub fn cursor(self: &Self) -> io::Cursor<SoundData> {
        io::Cursor::new(SoundData(self.0.clone()))
    }
    pub fn decoder(self: &Self) -> rodio::Decoder<io::Cursor<SoundData>> {
        rodio::Decoder::new(self.cursor()).unwrap()
    }
}