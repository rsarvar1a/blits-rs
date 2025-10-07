
use crate::prelude::*;

type SubSet = u16;
const SUBSET_SIZE: usize = BOARD_SIZE;
const NUM_SUBSETS: usize = BOARD_SIZE;

#[derive(Clone, Copy, Debug)]
pub struct CoordSet([SubSet; NUM_SUBSETS]);

impl CoordSet {
    #[inline]
    fn _index(coord: &Coord) -> (usize, usize) {
        (coord.row, coord.col)
    }
}

impl Default for CoordSet {
    fn default() -> Self {
        CoordSet([SubSet::default(); NUM_SUBSETS])
    }
}

impl SetOps<&Coord, Coord> for CoordSet {
    fn contains(&self, value: &Coord) -> bool {
        let (ia, ib) = CoordSet::_index(value);
        unsafe { (self.0.get_unchecked(ia) >> ib) & 1 == 1 }
    }

    fn len(&self) -> usize {
        self.0.iter().map(|sub| sub.count_ones() as usize).sum()
    }

    fn iter<'a>(&'a self) -> impl Iterator<Item = Coord> {
        CoordSetIterator::new(&self.0)
    }

    fn insert(&mut self, value: &Coord) -> &mut Self {
        let (ia, ib) = CoordSet::_index(value);
        let v = (1 as SubSet) << ib;
        unsafe {
            *self.0.get_unchecked_mut(ia) |= v;
        }
        self
    }

    fn remove(&mut self, value: &Coord) -> &mut Self {
        let (ia, ib) = CoordSet::_index(value);
        let v = !((1 as SubSet) << ib);
        unsafe {
            *self.0.get_unchecked_mut(ia) &= v;
        }
        self
    }

    fn extend(&mut self, iter: impl Iterator<Item = Coord>) -> &mut Self {
        iter.into_iter().for_each(|c| {
            self.insert(&c);
        });
        self
    }

    fn filter(&mut self, iter: impl Iterator<Item = Coord>) -> &mut Self {
        iter.into_iter().for_each(|c| {
            self.remove(&c);
        });
        self
    }

    fn intersect(&self, other: &Self) -> Self {
        let mut s = self.clone();
        s.intersect_inplace(other);
        s
    }

    fn intersect_inplace(&mut self, other: &Self) -> &mut Self {
        self.0.iter_mut().zip(other.0.iter()).for_each(|(l, r)| {
            *l &= r;
        });
        self
    }

    fn union(&self, other: &Self) -> Self {
        let mut s = self.clone();
        s.union_inplace(other);
        s
    }

    fn union_inplace(&mut self, other: &Self) -> &mut Self {
        self.0.iter_mut().zip(other.0.iter()).for_each(|(l, r)| {
            *l |= r;
        });
        self
    }

    fn difference(&self, other: &Self) -> Self {
        let mut s = self.clone();
        s.difference_inplace(other);
        s
    }

    fn difference_inplace(&mut self, other: &Self) -> &mut Self {
        self.0.iter_mut().zip(other.0.iter()).for_each(|(l, r)| {
            *l &= !r;
        });
        self
    }
}

impl<'a> FromIterator<&'a Coord> for CoordSet {
    fn from_iter<T: IntoIterator<Item = &'a Coord>>(iter: T) -> Self {
        let mut s = CoordSet::default();
        iter.into_iter().for_each(|i| {
            s.insert(i);
        });
        s
    }
}

impl FromIterator<Coord> for CoordSet {
    fn from_iter<T: IntoIterator<Item = Coord>>(iter: T) -> Self {
        let mut s = CoordSet::default();
        iter.into_iter().for_each(|i| {
            s.insert(&i);
        });
        s
    }
}

pub struct CoordSetIterator<'a> {
    data: &'a [SubSet; NUM_SUBSETS],
    mask: SubSet,
    current_subset: usize,
}

impl<'a> CoordSetIterator<'a> {
    pub fn new<'d>(data: &'d [SubSet; NUM_SUBSETS]) -> CoordSetIterator<'d> {
        CoordSetIterator { data, mask: SubSet::MAX, current_subset: 0 }
    }
}

impl<'a> Iterator for CoordSetIterator<'a> {
    type Item = Coord;
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if self.current_subset >= NUM_SUBSETS {
                return None;
            }

            let subject = self.data[self.current_subset] & self.mask;
            let tz = subject.trailing_zeros() as usize;

            if tz >= SUBSET_SIZE {
                self.current_subset += 1;
                self.mask = SubSet::MAX;
                continue;
            } else {
                let value = Coord::new(self.current_subset, tz);
                self.mask ^= (1 as SubSet) << tz;
                return Some(value);
            }
        }
    }
}

pub struct CoordSetIntoIterator {
    data: [SubSet; NUM_SUBSETS],
    mask: SubSet,
    current_subset: usize
}

impl CoordSetIntoIterator {
    pub fn new(data: &[SubSet; NUM_SUBSETS]) -> CoordSetIntoIterator {
        CoordSetIntoIterator { data: data.clone(), mask: SubSet::MAX, current_subset: 0 }
    }
}

impl Iterator for CoordSetIntoIterator {
    type Item = Coord;
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if self.current_subset >= NUM_SUBSETS {
                return None;
            }

            let subject = self.data[self.current_subset] & self.mask;
            let tz = subject.trailing_zeros() as usize;

            if tz >= SUBSET_SIZE {
                self.current_subset += 1;
                self.mask = SubSet::MAX;
                continue;
            } else {
                let value = Coord::new(self.current_subset, tz);
                self.mask ^= (1 as SubSet) << tz;
                return Some(value);
            }
        }
    }
}

impl IntoIterator for CoordSet {
    type IntoIter = CoordSetIntoIterator;
    type Item = Coord;
    fn into_iter(self) -> Self::IntoIter {
        CoordSetIntoIterator::new(&self.0)
    }
}