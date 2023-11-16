//! Debouncer definition.
//!
//! When pressed, switches don't give a clear state change: they
//! bounce. A debouncer filter these bounces. The current
//! implementation validate the state change when the state is stable
//! during a configurable number of update. 5 ms is the recommended
//! duration for keyboard switches.

use crate::layout::Event;
use either::Either::*;

/// The debouncer type.
pub struct Debouncer<T> {
    cur: T,
    new: T,
    since: u16,
    nb_bounce: u16,
}

impl<T> Debouncer<T> {
    /// Create a new debouncer.
    ///
    /// `cur` and `new` corresponds to the initial state, they should
    /// be equal at start. taking the 2 states allow `new` to be a
    /// `const fn` and allow non clonable types to be used.
    ///
    /// `nb_bounce` correspond to the number of update with same state
    /// needed to validate the new state.
    pub const fn new(cur: T, new: T, nb_bounce: u16) -> Self {
        Self {
            cur,
            new,
            since: 0,
            nb_bounce,
        }
    }
}

impl<T: PartialEq> Debouncer<T> {
    /// Gets the current state.
    pub fn get(&self) -> &T {
        &self.cur
    }

    /// Updates the current state.  Returns `true` if the state changes.
    pub fn update(&mut self, new: T) -> bool {
        if self.cur == new {
            self.since = 0;
            return false;
        }

        if self.new != new {
            self.new = new;
            self.since = 1;
        } else {
            self.since += 1;
        }

        if self.since > self.nb_bounce {
            core::mem::swap(&mut self.cur, &mut self.new);
            self.since = 0;
            true
        } else {
            false
        }
    }

    /// Iterates on the `Event`s generated by the update.
    ///
    /// `T` must be some kind of array of array of bool.
    ///
    /// Panics if the coordinates doesn't fit in a `(u8, u8)`.
    ///
    /// # Example
    ///
    /// ```
    /// use keyberon::debounce::Debouncer;
    /// use keyberon::layout::Event;
    /// let mut debouncer = Debouncer::new(
    ///     [[false, false], [false, false]],
    ///     [[false, false], [false, false]],
    ///     2,
    /// );
    ///
    /// // no changes
    /// assert_eq!(0, debouncer.events([[false, false], [false, false]]).count());
    ///
    /// // `(0, 1)` pressed, but debouncer is filtering
    /// assert_eq!(0, debouncer.events([[false, true], [false, false]]).count());
    /// assert_eq!(0, debouncer.events([[false, true], [false, false]]).count());
    ///
    /// // `(0, 1)` stable enough, event appear.
    /// assert_eq!(
    ///     vec![Event::Press(0, 1)],
    ///     debouncer.events([[false, true], [false, false]]).collect::<Vec<_>>(),
    /// );
    /// ```
    pub fn events<'a, U>(&'a mut self, new: T) -> impl Iterator<Item = Event> + 'a
    where
        &'a T: IntoIterator<Item = U>,
        U: IntoIterator<Item = &'a bool>,
        U::IntoIter: 'a,
    {
        if self.update(new) {
            Left(
                self.new
                    .into_iter()
                    .zip(&self.cur)
                    .enumerate()
                    .flat_map(move |(i, (o, n))| {
                        o.into_iter()
                            .zip(n)
                            .enumerate()
                            .filter_map(move |(j, bools)| match bools {
                                (false, true) => Some(Event::Press(i as u8, j as u8)),
                                (true, false) => Some(Event::Release(i as u8, j as u8)),
                                _ => None,
                            })
                    }),
            )
        } else {
            Right(core::iter::empty())
        }
    }
}
