
use crate::prelude::{SetOps, NUM_PIECES};

type SubSet = u16;
const SUBSET_SIZE: usize = size_of::<SubSet>() * 8;
const NUM_SUBSETS: usize = (NUM_PIECES + 1) / SUBSET_SIZE + 1;

#[derive(Clone, Copy, Debug)]
pub struct MoveSet([SubSet; NUM_SUBSETS]);

impl MoveSet {
    #[inline]
    fn _index(value: usize) -> (usize, usize) {
        (value / SUBSET_SIZE, value % SUBSET_SIZE)
    }
}

impl Default for MoveSet {
    fn default() -> Self {
        MoveSet([SubSet::default(); NUM_SUBSETS])
    }
}

impl SetOps<usize, usize> for MoveSet {
    fn contains(&self, value: usize) -> bool {
        let (ia, ib) = MoveSet::_index(value);
        unsafe { (self.0.get_unchecked(ia) >> ib) & 1 == 1 }
    }

    fn len(&self) -> usize {
        self.0.iter().map(|sub| sub.count_ones() as usize).sum()
    }

    fn iter<'a>(&'a self) -> impl Iterator<Item = usize> {
        MoveSetIterator::new(&self.0)
    }

    fn insert(&mut self, value: usize) -> &mut Self {
        let (ia, ib) = MoveSet::_index(value);
        let v = (1 as SubSet) << ib;
        unsafe { 
            *self.0.get_unchecked_mut(ia) |= v;
        }
        self
    }

    fn remove(&mut self, value: usize) -> &mut Self {
        let (ia, ib) = MoveSet::_index(value);
        let v = !((1 as SubSet) << ib);
        unsafe {
            *self.0.get_unchecked_mut(ia) &= v;
        }
        self
    }

    fn extend(&mut self, iter: impl Iterator<Item = usize>) -> &mut Self {
        iter.into_iter().for_each(|i| {
            self.insert(i);
        });
        self
    }

    fn filter(&mut self, iter: impl Iterator<Item = usize>) -> &mut Self {
        iter.into_iter().for_each(|i| {
            self.remove(i);
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

impl<'a> FromIterator<&'a usize> for MoveSet {
    fn from_iter<T: IntoIterator<Item = &'a usize>>(iter: T) -> Self {
        let mut s = MoveSet::default();
        iter.into_iter().for_each(|&i| {
            s.insert(i);
        });
        s
    }
}

impl FromIterator<usize> for MoveSet {
    fn from_iter<T: IntoIterator<Item = usize>>(iter: T) -> Self {
        let mut s = MoveSet::default();
        iter.into_iter().for_each(|i| {
            s.insert(i);
        });
        s
    }
}

pub struct MoveSetIterator<'a> {
    data: &'a [SubSet; NUM_SUBSETS],
    mask: SubSet,
    current_subset: usize,
}

impl<'a> MoveSetIterator<'a> {
    pub fn new<'d>(data: &'d [SubSet; NUM_SUBSETS]) -> MoveSetIterator<'d> {
        MoveSetIterator { data, mask: SubSet::MAX, current_subset: 0 }
    }
}

impl<'a> Iterator for MoveSetIterator<'a> {
    type Item = usize;
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if self.current_subset >= NUM_SUBSETS {
                return None;
            }
            
            let subject = self.data[self.current_subset] & self.mask;
            let tz = subject.trailing_zeros() as usize;

            if tz == SUBSET_SIZE {
                self.current_subset += 1;
                self.mask = SubSet::MAX;
                continue;
            } else {
                let value = self.current_subset * SUBSET_SIZE + tz;
                self.mask ^= (1 as SubSet) << tz; // add a 0 where we found the 1 to knock it out of the next iteration
                return Some(value);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::prelude::SetOps;

    use super::MoveSet;
    use std::collections::BTreeSet;

    #[test]
    fn iterate() {
        let elements = BTreeSet::from_iter([1, 4, 21, 144, 333, 1292].into_iter());
        
        let mut s = MoveSet::default();
        elements.iter().for_each(|&i| { s.insert(i); });
        let recovered = s.iter().collect::<BTreeSet<_>>();

        assert!(elements == recovered) 
    }
}
