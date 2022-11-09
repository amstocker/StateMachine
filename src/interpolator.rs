use dasp::Sample;

use crate::output::OutputSample;


pub type InterpolatorFloat = f64;

pub struct LinearInterpolator<I> where I: Iterator {
    iterator: I,
    prev: Option<I::Item>,
    next: Option<I::Item>,
    ratio: InterpolatorFloat,
    step: InterpolatorFloat
}

impl<I> LinearInterpolator<I>
where
    I: Iterator,
    I::Item: OutputSample
{
    pub fn new(mut iterator: I, ratio: InterpolatorFloat) -> Self {
        let prev = iterator.next();
        let next = iterator.next();
        Self {
            iterator,
            prev,
            next,
            ratio,
            step: 0.0
        }
    }

    fn interpolate(&self, t: InterpolatorFloat) -> Option<I::Item> {
        if let Some(a) = self.prev {
            let a = a.to_sample::<InterpolatorFloat>();
            if let Some(b) = self.next {
                let b = b.to_sample::<InterpolatorFloat>();
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
    I::Item: OutputSample
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
