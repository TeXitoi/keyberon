//! Provides chord support for two keys pressed at once.
//! E.g. Left + Right arrow at the same time => paste.
use crate::layout::Event;
use bitvec::prelude::*;
use heapless::{Vec, consts::{U16, U8, U4}};

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
    keys_pressed: u8
}

impl Chord {
    /// Create new chord from user data.
    pub fn new(def: &'static ChordDef) -> Self {
        let mut me = Self {
            def,
            in_progress: false,
            keys_pressed: 0b000_0000,
        };
        me.set_high_bits(true);
        me
    }

    // Set to true when looking to trigger, false for when looking to release.
    fn set_high_bits(&mut self, value: bool) {
        let len = self.def.keys.len();
        self.in_progress = !value;
        for k in 0..8 {
            self.keys_pressed.view_bits_mut::<Msb0>().set(k, k >= len);
        }
    }

    fn process(&mut self, event: Event) -> Option<Event> {
        match event {
            e @ Event::Press(_,_) => {
                if !self.in_progress {
                    for (k, _) in self.def.keys.iter().enumerate().filter(|(_,key)| **key == e.coord()) {
                        self.keys_pressed.view_bits_mut::<Msb0>().set(k, true);
                    }
                    if self.keys_pressed.view_bits_mut::<Msb0>().all() {
                        self.set_high_bits(false);
                        return Some(Event::press_from_coord(self.def.result));
                    }
                }
            },
            e @ Event::Release(_,_) => {
                for (k,_) in self.def.keys.iter().enumerate().filter(|(_,key)| **key == e.coord()) {
                   self.keys_pressed.view_bits_mut::<Msb0>().set(k, false);
                }
                if self.in_progress {
                    if self.keys_pressed.view_bits_mut::<Msb0>().not_any() {
                        self.set_high_bits(true);
                        return Some(Event::release_from_coord(self.def.result));
                    }
                }
            }
        }
        None
    }
}

/// Two keys at once!
pub struct Chording {
    /// Defined chords
    chords: Vec<Chord, U16>,
}

impl Chording {
    /// Take the predefined chord list in.
    pub fn new(chords: &'static [ChordDef]) -> Self {
        let mut v = Vec::<Chord, U16>::new();
        for ch in chords { v.push(Chord::new(ch)).ok().unwrap(); }
        Self { chords: v }
    }

    /// Consolidate events and return processed results as a result.
    pub fn tick(&mut self, vec: Vec<Event, U8>) -> Vec<Event, U8> {
        let mut vec_remove = Vec::<Event,U8>::new();

        // If the event is the last in a chord, map it to the result (and remove any assisting events.)
        let events : Vec<Event, U4> = vec.into_iter().map(|event|{
            for chord in self.chords.iter_mut() {
                match chord.process(event) {
                    Some(e @ Event::Press(_, _)) => {
                        vec_remove.extend(chord.def.keys.iter().copied().map(Event::press_from_coord));
                        return e;
                    },
                    Some(e @ Event::Release(_, _)) => {
                        vec_remove.extend(chord.def.keys.iter().copied().map(Event::release_from_coord));
                        return e;
                    },
                    None => {}
                }
            }
            event
        }).collect();

        events.into_iter().filter(|event| !vec_remove.contains(event)).collect()
    }
}
