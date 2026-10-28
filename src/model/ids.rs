//! Document identities survive moves, reordering, deletion, and restoration.
//! Keep the allocator outside undo snapshots: an undone insertion must not make
//! its identity available to a different object on a new history branch.

mod sealed {
    pub trait Sealed {}
}

pub trait DocumentId: sealed::Sealed {
    #[doc(hidden)]
    fn from_counter(counter: u64) -> Self;
}

macro_rules! ids {
    ($($name:ident),+ $(,)?) => {$(
        #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
        pub struct $name(u64);

        impl $name {
            pub fn value(self) -> u64 { self.0 }
        }

        impl sealed::Sealed for $name {}
        impl DocumentId for $name {
            fn from_counter(counter: u64) -> Self { Self(counter) }
        }
    )+};
}

ids!(
    SequenceId,
    EventId,
    PlacementId,
    InstrumentId,
    ControlRouteId,
    MixerChannelId,
    AudioRouteId,
    SampleId
);

/// One allocator per editing session, shared by every document entity kind.
/// Loading persisted IDs will need to advance this counter in phase 6.
#[derive(Debug, Default)]
pub struct IdGenerator {
    counter: u64,
}

impl IdGenerator {
    pub fn allocate<T: DocumentId>(&mut self) -> T {
        self.counter = self.counter.checked_add(1).expect("document IDs exhausted");
        T::from_counter(self.counter)
    }
}
