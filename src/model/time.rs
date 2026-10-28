use super::ModelError;

/// Canonical authoring resolution. A tempo change never rewrites event positions.
pub const TICKS_PER_BEAT: u64 = 960;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Tick(pub u64);

/// Frames in an original sample, before device-rate conversion.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct SourceFrame(pub u64);

/// Frames on the output device's clock; never used as document positions.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct DeviceFrame(pub u64);

/// Duration multiplier: 2/1 makes control material twice as long. This does
/// not change a sampler's playback rate or pitch. Reverse is not supported.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TimeScale {
    numerator: u32,
    denominator: u32,
}

impl TimeScale {
    pub const IDENTITY: Self = Self {
        numerator: 1,
        denominator: 1,
    };

    pub fn new(numerator: u32, denominator: u32) -> Result<Self, ModelError> {
        if numerator == 0 || denominator == 0 {
            return Err(ModelError::InvalidTime);
        }
        Ok(Self {
            numerator,
            denominator,
        })
    }
}

/// Exact nonnegative rational ticks during evaluation. Nested transforms are
/// composed before rounding once at the device boundary, avoiding cumulative
/// drift when a division lands between authoring ticks.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ScoreTime {
    numerator: u128,
    denominator: u128,
}

impl From<Tick> for ScoreTime {
    fn from(value: Tick) -> Self {
        Self {
            numerator: value.0.into(),
            denominator: 1,
        }
    }
}

impl ScoreTime {
    pub fn ratio(self) -> (u128, u128) {
        (self.numerator, self.denominator)
    }

    fn fraction(numerator: u128, denominator: u128) -> Self {
        let divisor = gcd(numerator, denominator);
        Self {
            numerator: numerator / divisor,
            denominator: denominator / divisor,
        }
    }

    pub fn checked_add(self, other: Self) -> Result<Self, ModelError> {
        let divisor = gcd(self.denominator, other.denominator);
        let left = self.numerator.checked_mul(other.denominator / divisor);
        let right = other.numerator.checked_mul(self.denominator / divisor);
        let numerator = left.zip(right).and_then(|(a, b)| a.checked_add(b));
        let denominator = self.denominator.checked_mul(other.denominator / divisor);
        numerator
            .zip(denominator)
            .map(|(n, d)| Self::fraction(n, d))
            .ok_or(ModelError::TimeOverflow)
    }

    pub fn checked_sub(self, other: Self) -> Result<Self, ModelError> {
        let divisor = gcd(self.denominator, other.denominator);
        let left = self.numerator.checked_mul(other.denominator / divisor);
        let right = other.numerator.checked_mul(self.denominator / divisor);
        let numerator = left.zip(right).and_then(|(a, b)| a.checked_sub(b));
        let denominator = self.denominator.checked_mul(other.denominator / divisor);
        numerator
            .zip(denominator)
            .map(|(n, d)| Self::fraction(n, d))
            .ok_or(ModelError::TimeOverflow)
    }

    pub fn scaled(self, scale: TimeScale) -> Result<Self, ModelError> {
        self.multiply(scale.numerator.into(), scale.denominator.into())
    }

    pub fn repeated(self, count: u32) -> Result<Self, ModelError> {
        self.multiply(count.into(), 1)
    }

    fn multiply(self, numerator: u128, denominator: u128) -> Result<Self, ModelError> {
        let a = gcd(self.numerator, denominator);
        let b = gcd(numerator, self.denominator);
        (self.numerator / a)
            .checked_mul(numerator / b)
            .zip((self.denominator / b).checked_mul(denominator / a))
            .map(|(n, d)| Self::fraction(n, d))
            .ok_or(ModelError::TimeOverflow)
    }
}

// Continued-fraction comparison avoids overflow from cross multiplication.
impl Ord for ScoreTime {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        let (mut a, mut b) = self.ratio();
        let (mut c, mut d) = other.ratio();
        let mut reverse = false;
        loop {
            let order = (a / b).cmp(&(c / d));
            let order = if order != std::cmp::Ordering::Equal {
                Some(order)
            } else if a % b == 0 || c % d == 0 {
                Some((a % b).cmp(&(c % d)))
            } else {
                None
            };
            if let Some(order) = order {
                return if reverse { order.reverse() } else { order };
            }
            (a, b) = (b, a % b);
            (c, d) = (d, c % d);
            reverse = !reverse;
        }
    }
}

impl PartialOrd for ScoreTime {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

fn gcd(mut a: u128, mut b: u128) -> u128 {
    while b != 0 {
        (a, b) = (b, a % b);
    }
    a
}

/// One constant tempo initially, stored exactly in thousandths of a BPM.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Tempo {
    milli_bpm: u32,
}

impl Tempo {
    pub fn from_milli_bpm(milli_bpm: u32) -> Result<Self, ModelError> {
        if milli_bpm == 0 {
            return Err(ModelError::InvalidTime);
        }
        Ok(Self { milli_bpm })
    }

    pub fn milli_bpm(self) -> u32 {
        self.milli_bpm
    }

    /// Nearest device frame, ties upward. Call at the engine boundary only.
    pub fn device_frame(
        self,
        time: ScoreTime,
        sample_rate: u32,
    ) -> Result<DeviceFrame, ModelError> {
        if sample_rate == 0 {
            return Err(ModelError::InvalidTime);
        }
        let frames = time.multiply(
            60_000 * u128::from(sample_rate),
            u128::from(TICKS_PER_BEAT) * u128::from(self.milli_bpm),
        )?;
        let (n, d) = frames.ratio();
        let rounded = (n / d)
            .checked_add(u128::from(n % d >= d / 2 + d % 2))
            .ok_or(ModelError::TimeOverflow)?;
        u64::try_from(rounded)
            .map(DeviceFrame)
            .map_err(|_| ModelError::TimeOverflow)
    }
}
