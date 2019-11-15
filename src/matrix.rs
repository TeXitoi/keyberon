use embedded_hal::digital::v2::{InputPin, OutputPin};
use generic_array::{ArrayLength, GenericArray};
use void::Void;

pub trait HeterogenousMap {
    type Item;
    type Len;
    fn map<T>(self, f: impl FnMut(Self::Item) -> T) -> GenericArray<T, Self::Len>
    where
        Self::Len: ArrayLength<T>;
}

#[macro_export]
macro_rules! impl_heterogenous_map {
    ($s:ident, $t:ty, $len:tt, [$($idx:tt),+]) => {
        impl<'a> IntoIterator for &'a $s {
            type Item = &'a $t;
            type IntoIter = generic_array::GenericArrayIter<&'a $t, $len>;
            fn into_iter(self) -> Self::IntoIter {
                generic_array::arr![
                    Self::Item;
                    $( &self.$idx as &$t, )+
                ].into_iter()
            }
        }
        impl<'a> IntoIterator for &'a mut $s {
            type Item = &'a mut $t;
            type IntoIter = generic_array::GenericArrayIter<&'a mut $t, $len>;
            fn into_iter(self) -> Self::IntoIter {
                generic_array::arr![
                    Self::Item;
                    $( &mut self.$idx as &mut $t, )+
                ].into_iter()
            }
        }
        impl<'a> $crate::matrix::HeterogenousMap for &'a mut $s {
            type Item = &'a mut $t;
            type Len = $len;
            fn map<T>(self, mut f: impl FnMut(Self::Item) -> T) -> generic_array::GenericArray<T, Self::Len>
            {
                generic_array::arr![T; $( f(&mut self.$idx), )+]
            }
        }
        impl<'a> $crate::matrix::HeterogenousMap for &'a $s {
            type Item = &'a $t;
            type Len = $len;
            fn map<T>(self, mut f: impl FnMut(Self::Item) -> T) -> generic_array::GenericArray<T, Self::Len>
            {
                generic_array::arr![T; $( f(&self.$idx), )+]
            }
        }
    }
}

pub struct Matrix<C, R> {
    cols: C,
    rows: R,
}

impl<C, R> Matrix<C, R>
where
    for<'a> &'a mut R: IntoIterator<Item = &'a mut dyn OutputPin<Error = Void>>,
{
    pub fn new(cols: C, rows: R) -> Self {
        let mut res = Self { cols, rows };
        res.clear();
        res
    }
}

impl<C, R> Matrix<C, R>
where
    for<'a> &'a mut R: IntoIterator<Item = &'a mut dyn OutputPin<Error = Void>>,
{
    pub fn clear(&mut self) {
        for r in self.rows.into_iter() {
            r.set_high().unwrap();
        }
    }
}

impl<'a, C: 'a, R: 'a> Matrix<C, R>
where
    &'a mut R: HeterogenousMap<Item = &'a mut dyn OutputPin<Error = Void>>,
    <&'a mut R as HeterogenousMap>::Len:
        ArrayLength<GenericArray<bool, <&'a C as HeterogenousMap>::Len>>,
    &'a C: HeterogenousMap<Item = &'a dyn InputPin<Error = Void>>,
    <&'a C as HeterogenousMap>::Len: ArrayLength<bool>,
{
    pub fn get(
        &'a mut self,
    ) -> PressedKeys<<&'a mut R as HeterogenousMap>::Len, <&'a C as HeterogenousMap>::Len> {
        let cols = &self.cols;
        PressedKeys(self.rows.map(|r| {
            r.set_low().unwrap();
            let col = cols.map(|r| r.is_low().unwrap());
            r.set_high().unwrap();
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
