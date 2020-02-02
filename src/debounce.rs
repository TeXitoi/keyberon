use crate::layout::Event;
use either::Either::*;

pub struct Debouncer<T> {
    cur: T,
    new: T,
    since: u16,
    nb_bounce: u16,
}

impl<T> Debouncer<T> {
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
    pub fn get(&self) -> &T {
        &self.cur
    }
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
                    .zip(self.cur.into_iter())
                    .enumerate()
                    .flat_map(move |(i, (o, n))| {
                        o.into_iter().zip(n.into_iter()).enumerate().filter_map(
                            move |(j, bools)| match bools {
                                (false, true) => Some(Event::Press(i, j)),
                                (true, false) => Some(Event::Release(i, j)),
                                _ => None,
                            },
                        )
                    }),
            )
        } else {
            Right(core::iter::empty())
        }
    }
}
