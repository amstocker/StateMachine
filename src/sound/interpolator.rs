use dasp::Sample;

use crate::sound::{Float, OutputSample};

pub struct LinearInterpolator<I>
where
    I: Iterator,
{
    iterator: I,
    prev: Option<I::Item>,
    next: Option<I::Item>,
    ratio: f64,
    source_frame: u64,
    output_frame: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InvalidRatio;

impl std::fmt::Display for InvalidRatio {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("resampling ratio must be finite and positive")
    }
}

impl std::error::Error for InvalidRatio {}

impl<I> LinearInterpolator<I>
where
    I: Iterator,
    I::Item: OutputSample,
{
    /// `ratio` is output frames / input frames. The final sample interpolates
    /// toward silence until the source duration ends (the prototype's policy).
    pub fn new(mut iterator: I, ratio: f64) -> Result<Self, InvalidRatio> {
        if !ratio.is_finite() || ratio <= 0.0 {
            return Err(InvalidRatio);
        }
        let prev = iterator.next();
        let next = iterator.next();
        Ok(Self {
            iterator,
            prev,
            next,
            ratio,
            source_frame: 0,
            output_frame: 0,
        })
    }

    fn interpolate(&self, t: Float) -> Option<I::Item> {
        if let Some(a) = self.prev {
            let a = a.to_sample::<Float>();
            if let Some(b) = self.next {
                let b = b.to_sample::<Float>();
                return Some(((1.0 - t) * a + t * b).to_sample::<I::Item>());
            } else {
                return Some(((1.0 - t) * a).to_sample::<I::Item>());
            }
        }
        None
    }
}

impl<I> Iterator for LinearInterpolator<I>
where
    I: Iterator,
    I::Item: OutputSample,
{
    type Item = I::Item;

    fn next(&mut self) -> Option<Self::Item> {
        // Derive position from an integer clock rather than repeatedly adding
        // and subtracting a fractional step (which drifts over long samples).
        let mut position = self.output_frame as f64 / self.ratio;
        // A rate such as 48000/44100 is not exactly representable. Snap a
        // position within two rounding units of a frame boundary so one second
        // of input does not gain an extra output frame at the end.
        let nearest_frame = position.round();
        if (position - nearest_frame).abs() <= 2.0 * f64::EPSILON * position.abs().max(1.0) {
            position = nearest_frame;
        }
        while self.prev.is_some() && position - self.source_frame as f64 >= 1.0 {
            self.prev = self.next;
            self.next = self.iterator.next();
            self.source_frame += 1;
        }
        let result = self.interpolate((position - self.source_frame as f64) as Float);
        self.output_frame += 1;
        result
    }
}
