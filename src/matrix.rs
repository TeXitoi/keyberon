use core::borrow::{Borrow, BorrowMut};
use embedded_hal::digital::v2::{InputPin, OutputPin};
use generic_array::{ArrayLength, GenericArray};
use void::Void;

pub trait DynGetter<'a> {
    type DynRef: 'a;
    type DynMutRef: 'a;
    type Len;
    fn get(&'a self, i: usize) -> Option<Self::DynRef>;
    fn get_mut(&'a mut self, i: usize) -> Option<Self::DynMutRef>;
    fn len(&self) -> usize;
    fn map<T>(&'a self, f: impl FnMut(Self::DynRef) -> T) -> GenericArray<T, Self::Len>
    where
        Self::Len: ArrayLength<T>;
    fn map_mut<T>(&'a mut self, f: impl FnMut(Self::DynMutRef) -> T) -> GenericArray<T, Self::Len>
    where
        Self::Len: ArrayLength<T>;
}

#[macro_export]
macro_rules! impl_getter {
    ($s:ident, $t:ty, $len:tt, [$($idx:tt),+]) => {
        impl<'a> crate::matrix::DynGetter<'a> for $s {
            type DynRef = &'a $t;
            type DynMutRef = &'a mut $t;
            type Len = $len;
            fn get(&'a self, i: usize) -> Option<Self::DynRef> {
                match i {
                    $(
                        $idx => Some(&self.$idx as &$t),
                    )+
                        _ => None,
                }
            }
            fn get_mut(&'a mut self, i: usize) -> Option<Self::DynMutRef> {
                match i {
                    $(
                        $idx => Some(&mut self.$idx as &mut $t),
                    )+
                        _ => None,
                }
            }
            fn len(&self) -> usize {
                use generic_array::typenum::marker_traits::Unsigned;
                $len::to_usize()
            }
            fn map<T>(&'a self, mut f: impl FnMut(Self::DynRef) -> T) -> generic_array::GenericArray<T, $len> {
                generic_array::arr![T;
                    $(
                        f(&self.$idx),
                    )+
                ]
            }
            fn map_mut<T>(&'a mut self, mut f: impl FnMut(Self::DynMutRef) -> T) -> generic_array::GenericArray<T, $len> {
                generic_array::arr![T;
                    $(
                        f(&mut self.$idx),
                    )+
                ]
            }
        }
    }
}

pub struct Matrix<C, R> {
    cols: C,
    rows: R,
}

impl<C, R> Matrix<C, R> {
    pub fn new(cols: C, rows: R) -> Self {
        Self { cols, rows }
    }
}

impl<'a, C, R> Matrix<C, R>
where
    R: DynGetter<'a>,
    R::Len: ArrayLength<()>,
    R::DynMutRef: BorrowMut<dyn OutputPin<Error = Void> + 'a>,
{
    pub fn clear(&'a mut self) {
        self.rows
            .map_mut(|mut c| c.borrow_mut().set_high().unwrap());
    }
}

impl<'a, C, R> Matrix<C, R>
where
    C: DynGetter<'a>,
    R: DynGetter<'a>,
    C::Len: ArrayLength<bool>,
    R::Len: ArrayLength<GenericArray<bool, C::Len>>,
    R::DynMutRef: BorrowMut<dyn OutputPin<Error = Void> + 'a>,
    C::DynRef: Borrow<dyn InputPin<Error = Void> + 'a>,
{
    pub fn get(&'a mut self) -> PressedKeys<R::Len, C::Len> {
        let cols = &self.cols;
        PressedKeys(self.rows.map_mut(|mut c| {
            c.borrow_mut().set_low().unwrap();
            cortex_m::asm::delay(5 * 48); // 5µs
            let col = cols.map(|r| r.borrow().is_low().unwrap());
            c.borrow_mut().set_high().unwrap();
            col
        }))
    }
}

#[derive(PartialEq, Eq)]
pub struct PressedKeys<U, V>(pub GenericArray<GenericArray<bool, V>, U>)
where
    V: ArrayLength<bool>,
    U: ArrayLength<GenericArray<bool, V>>;

impl<U, V> PressedKeys<U, V>
where
    V: ArrayLength<bool>,
    U: ArrayLength<GenericArray<bool, V>>,
{
    pub fn new() -> Self {
        Self(Default::default())
    }
    pub fn iter_pressed<'a>(&'a self) -> impl Iterator<Item = (usize, usize)> + Clone + 'a {
        self.0.iter().enumerate().flat_map(|(i, r)| {
            r.iter()
                .enumerate()
                .filter_map(move |(j, &b)| if b { Some((i, j)) } else { None })
        })
    }
}
