use dasp::{Sample, sample::{FromSample, ToSample}};


type Float = f64;

pub struct LinearInterpolator<I> where I: Iterator {
    iterator: I,
    prev: Option<I::Item>,
    next: Option<I::Item>,
    ratio: Float,
    step: Float,
    remainder: Float
}

impl<I> LinearInterpolator<I>
where
    I: Iterator,
    I::Item: Sample + ToSample<Float> + FromSample<Float> + std::fmt::Debug
{
    pub fn new(iterator: I, ratio: Float) -> Self {
        let mut interpolator = LinearInterpolator {
            iterator,
            prev: None,
            next: None,
            ratio,
            step: 0.0,
            remainder: 0.0
        };
        interpolator.prev = interpolator.iterator.next();
        interpolator.next = interpolator.iterator.next();
        interpolator
    }

    fn interpolate(&self, t: Float) -> Option<I::Item> {
        if let (Some(a), Some(b)) = (self.prev, self.next) {
            return Some(Sample::from_sample((1.0 - t) * a.to_sample::<Float>() + t * b.to_sample::<Float>()));
        }
        self.prev
    }
}

impl<I> Iterator for LinearInterpolator<I>
where
    I: Iterator,
    I::Item: Sample + ToSample<Float> + FromSample<Float> + std::fmt::Debug
{
    type Item = I::Item;

    fn next(&mut self) -> Option<Self::Item> {
        if self.step >= self.ratio {
            self.step -= self.ratio;
            self.prev = self.next;
            self.next = self.iterator.next();
        }
        let result = self.interpolate(self.step / self.ratio);
        self.step += 1.0;
        result
    }
}
