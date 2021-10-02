//! Provides chord support for two keys pressed at once.
//! E.g. Left + Right arrow at the same time => paste.

/// ## Usage
/// ``` no_run
/// use keyberon::chording::{Chording, ChordDef};
/// use keyberon::layout::{Layout, Event::*};
/// use keyberon::debounce::Debouncer;
/// use keyberon::matrix::{Matrix, PressedKeys};
///
/// // The chord is defined by two or more locations in the layout
/// // that correspond to a single location in the layout
/// const CHORD0: ChordDef = ChordDef::new((0, 2), &[(0, 0), (0, 1)]);
/// const CHORD1: ChordDef = ChordDef::new((0, 0), &[(0, 1), (0, 2)]);
/// const CHORDS: [ChordDef; 2] = [CHORD0, CHORD1];
///
/// pub static LAYERS: keyberon::layout::Layers = keyberon::layout::layout! {
///     { [ A B C ] }
/// };
///
/// let mut layout = Layout::new(LAYERS);
/// // Debouncer period determines chording timeout
/// let mut debouncer: Debouncer<PressedKeys<3, 1>> =
///     Debouncer::new(PressedKeys::default(), PressedKeys::default(), 30);
/// let mut chording = Chording::new(&CHORDS);
///
/// // the rest of this example should be called inside a callback
/// // The PressedKeys are normall determined by calling the matrix
/// let keys_pressed = PressedKeys([[true, true, false]]);
/// let events = chording.tick(debouncer.events(keys_pressed).collect()).into_iter();
///
/// events.for_each(|e| layout.event(e));
/// ```
use crate::layout::Event;
use heapless::Vec;

type KeyPosition = (u8, u8);

/// KeyA + KeyB = KeyC
/// (For custom actions KeyC could be a virtual key off to the side and then mapped to actions via layers.)
#[derive(Clone)]
pub struct ChordDef {
    keys: &'static [KeyPosition],
    result: KeyPosition,
}

/// Warning: Chording home mod keys can leave the mod on.
impl ChordDef {
    /// Create new chord
    pub const fn new(result: KeyPosition, keys: &'static [KeyPosition]) -> Self {
        Self { keys, result }
    }
}

/// Runtime data for a chord
#[derive(Clone)]
pub struct Chord {
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
            keys_pressed: Vec::<bool, 8>::new(),
        };
        me.in_progress = false;
        def.keys
            .iter()
            .for_each(|_| me.keys_pressed.push(false).unwrap());
        me
    }

    fn process(&mut self, event: Event) -> Option<Event> {
        match event {
            e @ Event::Press(_, _) => {
                if !self.in_progress {
                    for (k, _) in self
                        .def
                        .keys
                        .iter()
                        .enumerate()
                        .filter(|(_, key)| **key == e.coord())
                    {
                        self.keys_pressed[k] = true;
                    }
                    if self.keys_pressed.iter().all(|&k| k) {
                        self.in_progress = true;
                        return Some(Event::press_from_coord(self.def.result));
                    }
                }
            }
            e @ Event::Release(_, _) => {
                for (k, _) in self
                    .def
                    .keys
                    .iter()
                    .enumerate()
                    .filter(|(_, key)| **key == e.coord())
                {
                    self.keys_pressed[k] = false;
                }
                if self.in_progress && self.keys_pressed.iter().all(|&k| !k) {
                    self.in_progress = false;
                    for k in self.keys_pressed.iter_mut() {
                        *k = false;
                    }
                    return Some(Event::release_from_coord(self.def.result));
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
                            vec_remove.extend(
                                chord.def.keys.iter().copied().map(Event::press_from_coord),
                            );
                            return e;
                        }
                        Some(e @ Event::Release(_, _)) => {
                            vec_remove.extend(
                                chord
                                    .def
                                    .keys
                                    .iter()
                                    .copied()
                                    .map(Event::release_from_coord),
                            );
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
        const CHORD: ChordDef = ChordDef::new((0, 2), &[(0, 0), (0, 1)]);
        let mut chording = Chording::new(&[CHORD]);

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
        const CHORD: ChordDef = ChordDef::new((0, 2), &[(0, 0), (0, 1)]);
        let mut chording = Chording::new(&[CHORD]);

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
        const CHORD: ChordDef = ChordDef::new((0, 2), &[(0, 0), (0, 1)]);
        let mut chording = Chording::new(&[CHORD]);

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
