//! Provides combo support for two keys pressed at once.
//! E.g. Left + Right arrow at the same time => paste.
use arraydeque::ArrayDeque;
use crate::layout::Event;

/// KeyA + KeyB = KeyC
/// (For custom actions KeyC could be a virtual key off to the side and then mapped to actions via layers.)
pub type Combination = ((u8,u8),(u8,u8),(u8,u8));

/// Two keys at once!
pub struct Combo {
    /// Possible combinations
    combos: &'static [Combination],

    /// bools indicate if first or second keys are depressed.
    pub stacked: ArrayDeque<[(Combination, bool, bool); 16], arraydeque::behavior::Wrapping>,
}

impl Combo {
    /// Take the predefined combo list in.
    pub fn new(combos: &'static [Combination]) -> Self {
        Self {
            combos,
            stacked: ArrayDeque::new(),
        }
    }

    /// Consolidate events and return processed results as a result.
    pub fn tick(&mut self, mut vec: heapless::Vec<Event,heapless::consts::U8>) -> heapless::Vec<Event,heapless::consts::U4> 
    {
        let mut vec_remove: heapless::FnvIndexSet<usize,_>= heapless::FnvIndexSet::<_, heapless::consts::U8>::new();
        let mut vec_adds : heapless::Vec<Event,_>= heapless::Vec::<_, heapless::consts::U4>::new();

        for ((x1,y1),(x2,y2), (t1,t2)) in self.combos.iter() {
			let mut combo_index: Option<usize> = None;
			
			for (idx, event) in vec.iter().enumerate() {
                match event {
                    Event::Press(i,j) => {
                        let first_matches = *x1==*i && *y1 == *j;
                        let second_matches = *x2==*i && *y2 == *j; 
                        if first_matches || second_matches {
                            if let Some(first_idx) = combo_index {
                                // combo triggered on second keypress...
                                vec_remove.insert(first_idx).ok();      
                                vec_remove.insert(idx).ok(); // remove the second one.
								vec_adds.push(Event::Press(*t1,*t2)).ok();
								self.stacked.push_back((((*x1,*y1),(*x2,*y2),(*t1,*t2)), true, true));
                            }
                            combo_index = Some(idx);        
                        }
                     },
                    _ => {}
                }
			}			
        }   

		// edge case: if two combos are pressed at the same time that have a common key then
		// we need to not remove that key twice hence a Set not a Vec.
		while !vec_remove.is_empty() {
			let f: usize = *(vec_remove.iter().max().unwrap());
			vec.swap_remove(f);
			vec_remove.remove(&f);
		}

        vec.extend(vec_adds);
        
        let mut combo_remove: heapless::Vec<usize, heapless::consts::U4> = heapless::Vec::new();		
        let mut events : heapless::Vec<Event, heapless::consts::U4> = heapless::Vec::new();

        for e in vec.into_iter() {			
			let mut event_fired = false;
			for (combo_idx, (((x1,y1),(x2,y2),(x3,y3)), ref mut pressed1, ref mut pressed2)) in self.stacked.iter_mut().enumerate() {
				match e {
					Event::Release(i,j) => { 
						if *pressed1 && (*x1,*y1) == (i,j)  {
							event_fired = true;
							*pressed1 = false;
						} 
						else if *pressed2 && (*x2,*y2) == (i,j)  {
							event_fired = true;
							*pressed2 = false;
                        }
                        
						//mark finished combo for removal
						if !*pressed1 && !*pressed2 {
							events.push(Event::Release(*x3,*y3)).ok();
							combo_remove.push(combo_idx).ok();
						}
					},
					_ =>{}
				}
			}
			if !event_fired {
				events.push(e).ok();
			}
        }

		//Remove finished combos...
		for f in combo_remove.iter().rev() {
			self.stacked.remove(*f);
        }
        
        events
    }
}
