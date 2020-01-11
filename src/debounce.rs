use core::mem;

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
            mem::swap(&mut self.cur, &mut self.new);
            self.since = 0;
            true
        } else {
            false
        }
    }
}

use crate::layout::Event;

impl<T> Debouncer<T> {
    pub fn events<'a, U>(&'a self) -> impl Iterator<Item = Event> + 'a
    where
        &'a T: IntoIterator<Item = U>,
        U: IntoIterator<Item = &'a bool>,
        U::IntoIter: 'a,
    {
        self.new
            .into_iter()
            .zip(self.cur.into_iter())
            .enumerate()
            .flat_map(move |(i, (o, n))| {
                o.into_iter()
                    .zip(n.into_iter())
                    .enumerate()
                    .filter_map(move |(j, bools)| match bools {
                        (false, true) => Some(Event::Press(i, j)),
                        (true, false) => Some(Event::Release(i, j)),
                        _ => None,
                    })
            })
    }
}
