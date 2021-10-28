//! Provides chord support for emulating a single layout event
//! from multiple key presses. The single event press is triggered
//! once all the keys of the chord have been pressed and the chord
//! is released once all of the keys of the chord have been released.
//!
//! The chording tick should be used after debouncing, where
//! the debounce period determines the period in which all keys
//! need to be pressed to trigger the chord.
//!
//! You must use a virtual row/area of your layout to
//! define the result of the chord if the desired result is
//! not already on the layer that you want to use the chord on.

/// ## Usage
/// ```
/// use keyberon::chording::{Chording, ChordDef};
/// use keyberon::layout::{Layout, Event::*, Event};
/// use keyberon::debounce::Debouncer;
/// use keyberon::matrix::{Matrix, PressedKeys};
///
/// // The chord is defined by two or more locations in the layout
/// // that correspond to a single location in the layout
/// const CHORDS: [ChordDef; 2] = [
///     ((0, 2), &[(0, 0), (0, 1)]),
///     ((0, 0), &[(0, 1), (0, 2)])
/// ];
/// const DEBOUNCE_COUNT: u16 = 30;
///
/// pub static LAYERS: keyberon::layout::Layers = keyberon::layout::layout! {
///     { [ A B C ] }
/// };
///
/// let mut layout = Layout::new(LAYERS);
/// // Debouncer period determines chording timeout
/// let mut debouncer: Debouncer<PressedKeys<3, 1>> =
///     Debouncer::new(PressedKeys::default(), PressedKeys::default(), DEBOUNCE_COUNT);
/// let mut chording = Chording::new(&CHORDS);
///
/// // the rest of this example should be called inside a callback
/// // The PressedKeys are normally determined by calling the matrix
/// // and the for loop is just to get past the debouncer
/// for _ in 0..DEBOUNCE_COUNT {
///     assert_eq!(0, debouncer.events(PressedKeys([[true, true, false]])).count());
/// }
/// let mut events = chording
///     .tick(debouncer.events(PressedKeys([[true, true, false]])).collect())
///     .into_iter();
/// let event = events.next();
/// assert_eq!(Some(Event::Press(0, 2)), event);
/// layout.event(event.unwrap());
/// let event = events.next();
/// assert_eq!(None, event);
/// ```
use crate::layout::Event;
use heapless::Vec;

type KeyPosition = (u8, u8);

/// Description of the virtual key corresponding to a given chord.
/// keys are the coordinates of the multiple keys that make up the chord
/// result is the outcome of the keys being pressed
pub type ChordDef = (KeyPosition, &'static [KeyPosition]);

/// Runtime data for a chord
#[derive(Clone)]
struct Chord {
    def: &'static ChordDef,
    in_progress: bool,
    keys_pressed: Vec<bool, 8>,
}

impl Chord {
    /// Create new chord from user data.
    pub fn new(def: &'static ChordDef) -> Self {
        let mut me = Self {
            def,
            in_progress: false,
            keys_pressed: Vec::new(),
        };
        for _ in def.1 {
            me.keys_pressed.push(false).unwrap()
        }
        me
    }

    fn process(&mut self, event: Event) -> Option<Event> {
        match event {
            Event::Press(_, _) => {
                if !self.in_progress {
                    for (k, _) in self
                        .def
                        .1
                        .iter()
                        .enumerate()
                        .filter(|(_, key)| **key == event.coord())
                    {
                        self.keys_pressed[k] = true;
                    }
                    if self.keys_pressed.iter().all(|&k| k) {
                        self.in_progress = true;
                        return Some(Event::press_from_coord(self.def.0));
                    }
                }
            }
            Event::Release(_, _) => {
                for (k, _) in self
                    .def
                    .1
                    .iter()
                    .enumerate()
                    .filter(|(_, key)| **key == event.coord())
                {
                    self.keys_pressed[k] = false;
                }
                if self.in_progress && self.keys_pressed.iter().all(|&k| !k) {
                    self.in_progress = false;
                    self.keys_pressed.iter_mut().for_each(|k| *k = false);
                    return Some(Event::release_from_coord(self.def.0));
                }
            }
        }
        None
    }
}

/// Two keys at once!
pub struct Chording {
    /// Defined chords
    chords: Vec<Chord, 16>,
}

impl Chording {
    /// Take the predefined chord list in.
    pub fn new(chords: &'static [ChordDef]) -> Self {
        let mut v = Vec::<Chord, 16>::new();
        for ch in chords {
            v.push(Chord::new(ch)).ok().unwrap();
        }
        Self { chords: v }
    }

    /// Consolidate events and return processed results as a result.
    pub fn tick(&mut self, vec: Vec<Event, 8>) -> Vec<Event, 8> {
        let mut vec_remove = Vec::<Event, 8>::new();

        // If the event is the last in a chord, map it to the result (and remove any assisting events.)
        let events: Vec<Event, 4> = vec
            .into_iter()
            .map(|event| {
                for chord in self.chords.iter_mut() {
                    match chord.process(event) {
                        Some(e @ Event::Press(_, _)) => {
                            vec_remove
                                .extend(chord.def.1.iter().copied().map(Event::press_from_coord));
                            return e;
                        }
                        Some(e @ Event::Release(_, _)) => {
                            vec_remove
                                .extend(chord.def.1.iter().copied().map(Event::release_from_coord));
                            return e;
                        }
                        None => {}
                    }
                }
                event
            })
            .collect();

        events
            .into_iter()
            .filter(|event| !vec_remove.contains(event))
            .collect()
    }
}

#[cfg(test)]
mod test {
    use super::{ChordDef, Chording};
    use crate::layout::{Event, Event::*};
    use heapless::Vec;

    #[test]
    fn single_press_release() {
        const CHORDS: [ChordDef; 1] = [((0, 2), &[(0, 0), (0, 1)])];
        let mut chording = Chording::new(&CHORDS);

        // Verify a single press goes through chording unchanged
        let mut single_press = Vec::<Event, 8>::new();
        single_press.push(Press(0, 0)).ok();
        assert_eq!(chording.tick(single_press), &[Press(0, 0)]);
        let mut single_release = Vec::<Event, 8>::new();
        single_release.push(Release(0, 0)).ok();
        assert_eq!(chording.tick(single_release), &[Release(0, 0)]);
    }

    #[test]
    fn chord_press_release() {
        const CHORDS: [ChordDef; 1] = [((0, 2), &[(0, 0), (0, 1)])];
        let mut chording = Chording::new(&CHORDS);

        // Verify a chord is converted to the correct key
        let mut double_press = Vec::<Event, 8>::new();
        double_press.push(Press(0, 0)).ok();
        double_press.push(Press(0, 1)).ok();
        assert_eq!(chording.tick(double_press), &[Press(0, 2)]);
        let mut double_release = Vec::<Event, 8>::new();
        double_release.push(Release(0, 0)).ok();
        double_release.push(Release(0, 1)).ok();
        let chord_double_release = chording.tick(double_release);
        assert_eq!(chord_double_release, &[Release(0, 2)]);
    }

    #[test]
    fn chord_press_half_release() {
        const CHORDS: [ChordDef; 1] = [((0, 2), &[(0, 0), (0, 1)])];
        let mut chording = Chording::new(&CHORDS);

        // Verify a chord is converted to the correct key
        let mut double_press = Vec::<Event, 8>::new();
        double_press.push(Press(0, 0)).ok();
        double_press.push(Press(0, 1)).ok();
        assert_eq!(chording.tick(double_press), &[Press(0, 2)]);
        let mut first_release = Vec::<Event, 8>::new();
        first_release.push(Release(0, 0)).ok();
        // we will see the key release pass through, but this won't matter
        assert_eq!(chording.tick(first_release), &[Release(0, 0)]);
        let mut second_release = Vec::<Event, 8>::new();
        second_release.push(Release(0, 1)).ok();
        // once all keys of the combo are released, the combo is released
        assert_eq!(chording.tick(second_release), &[Release(0, 2)]);
    }
}
