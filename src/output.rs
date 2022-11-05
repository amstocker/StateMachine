use dasp::Sample;


pub enum StereoChannel {
    Left,
    Right
}

pub struct MonoToStereo<I> where I: Iterator {
    iterator: I,
    buffer: Option<I::Item>,
    first: bool
}

impl<I> MonoToStereo<I> where I: Iterator {
    pub fn new(iterator: I) -> Self {
        Self {
            iterator,
            buffer: None,
            first: true
        }
    }
}

impl<I> Iterator for MonoToStereo<I> where I: Iterator, I::Item: Copy {
    type Item = I::Item;

    fn next(&mut self) -> Option<Self::Item> {
        if self.first {
            self.buffer = self.iterator.next();
            self.first = false;
            self.buffer
        } else {
            self.first = true;
            self.buffer
        }
    }
}


pub struct StereoOutput<I> {
    iterator: I,
    num_channels: u16,
    output_channels: (u16, u16),
    current_channel: u16,
    next_channel: StereoChannel
}

impl<I> StereoOutput<I> where I: Iterator {
    pub fn new(iterator: I, num_channels: u16, output_channels: (u16, u16)) -> Self {
        assert!(output_channels.0 != output_channels.1);
        Self {
            iterator,
            num_channels,
            output_channels,
            current_channel: 0,
            next_channel: StereoChannel::Left
        }
    }
}

impl<I> Iterator for StereoOutput<I> where I: Iterator, I::Item: Sample {
    type Item = I::Item;

    fn next(&mut self) -> Option<Self::Item> {
        self.current_channel = (self.current_channel + 1) % self.num_channels;
        if (self.current_channel == self.output_channels.0) || 
           (self.current_channel == self.output_channels.1)
        {
            self.next_channel = match self.next_channel {
                StereoChannel::Left => StereoChannel::Right,
                StereoChannel::Right => StereoChannel::Left
            };
            self.iterator.next()
        } else {
            Some(Self::Item::EQUILIBRIUM)
        }
    }
}
