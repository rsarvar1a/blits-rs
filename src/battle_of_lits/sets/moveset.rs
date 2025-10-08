
use crate::prelude::{SetOps, NUM_PIECES};
use itertools::Itertools;

type SubSet = u64;
const SUBSET_SIZE: usize = size_of::<SubSet>() * 8;
const NUM_SUBSETS: usize = (NUM_PIECES + 1) / SUBSET_SIZE + 1;
const NUM_SUBSETS_PHYSICAL: usize = (NUM_SUBSETS / 4 + 1) * 4;

#[derive(Clone, Copy, Debug)]
pub struct MoveSet([SubSet; NUM_SUBSETS_PHYSICAL]);

impl MoveSet {
    #[inline]
    fn _index(value: usize) -> (usize, usize) {
        (value / SUBSET_SIZE, value % SUBSET_SIZE)
    }

    /// Returns a MoveSet containing all possible moves (0..NUM_PIECES) in constant time.
    pub fn all() -> Self {
        let mut set = MoveSet::default();
        // Set all bits for complete pieces, and mask the last subset for NUM_PIECES
        for i in 0..(NUM_SUBSETS - 1) {
            set.0[i] = SubSet::MAX;
        }
        // Handle the last subset with proper masking
        const REMAINING_PIECES: usize = NUM_PIECES % SUBSET_SIZE;
        const MASK: u64 = (1 << REMAINING_PIECES) - 1;
        set.0[NUM_SUBSETS - 1] = MASK;
        set
    }


    /// Returns a MoveSet containing every step_by-th move for efficient sampling.
    /// Uses bit manipulation tricks for common step_by values.
    pub fn sampled(step_by: usize) -> Self {
        let mut set = MoveSet::default();        
        for piece_id in (0..NUM_PIECES).step_by(step_by) {
            set.insert(piece_id);
        }
        
        set
    }
}

impl Default for MoveSet {
    fn default() -> Self {
        MoveSet([SubSet::default(); NUM_SUBSETS_PHYSICAL])
    }
}

impl SetOps<usize, usize> for MoveSet {
    fn contains(&self, value: usize) -> bool {
        let (ia, ib) = MoveSet::_index(value);
        unsafe { (self.0.get_unchecked(ia) >> ib) & 1 == 1 }
    }

    fn len(&self) -> usize {
        // Manual unroll to eliminate iterator overhead in hot path
        let mut count = 0;
        for i in 0..NUM_SUBSETS {
            unsafe {
                count += self.0.get_unchecked(i).count_ones() as usize;
            }
        }
        count
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

    fn _extend(&mut self, iter: impl Iterator<Item = usize>) -> &mut Self {
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

    fn is_empty(&self) -> bool {
        self.iter().next().is_none()
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
    data: &'a [SubSet; NUM_SUBSETS_PHYSICAL],
    mask: SubSet,
    current_subset: usize,
}

impl<'a> MoveSetIterator<'a> {
    pub fn new<'d>(data: &'d [SubSet; NUM_SUBSETS_PHYSICAL]) -> MoveSetIterator<'d> {
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

impl std::iter::Extend<usize> for MoveSet {
    fn extend<T: IntoIterator<Item = usize>>(&mut self, iter: T) {
        iter.into_iter().for_each(|mv| {
            self.insert(mv);
        });
    }
}

impl MoveSet {
    pub fn union_3(a: &MoveSet, b: &MoveSet, c: &MoveSet) -> MoveSet {
        MoveSet(std::array::from_fn(|i| a.0[i] | b.0[i] | c.0[i]))
    }

    pub fn union_4(a: &MoveSet, b: &MoveSet, c: &MoveSet, d: &MoveSet) -> MoveSet {
        MoveSet(std::array::from_fn(|i| a.0[i] | b.0[i] | c.0[i] | d.0[i]))
    }

    pub fn union_5(a: &MoveSet, b: &MoveSet, c: &MoveSet, d: &MoveSet, e: &MoveSet) -> MoveSet {
        MoveSet(std::array::from_fn(|i| a.0[i] | b.0[i] | c.0[i] | d.0[i] | e.0[i]))
    }

    pub fn union_6(a: &MoveSet, b: &MoveSet, c: &MoveSet, d: &MoveSet, e: &MoveSet, f: &MoveSet) -> MoveSet {
        MoveSet(std::array::from_fn(|i| a.0[i] | b.0[i] | c.0[i] | d.0[i] | e.0[i] | f.0[i]))
    }

    pub fn union_7(a: &MoveSet, b: &MoveSet, c: &MoveSet, d: &MoveSet, e: &MoveSet, f: &MoveSet, g: &MoveSet) -> MoveSet {
        MoveSet(std::array::from_fn(|i| a.0[i] | b.0[i] | c.0[i] | d.0[i] | e.0[i] | f.0[i] | g.0[i]))
    }

    pub fn union_8(a: &MoveSet, b: &MoveSet, c: &MoveSet, d: &MoveSet, e: &MoveSet, f: &MoveSet, g: &MoveSet, h: &MoveSet) -> MoveSet {
        MoveSet(std::array::from_fn(|i| a.0[i] | b.0[i] | c.0[i] | d.0[i] | e.0[i] | f.0[i] | g.0[i] | h.0[i]))
    }

    pub fn union_remainder<'a>(sets: &Vec<&'a MoveSet>) -> MoveSet {
        match sets.len() {
            0 => MoveSet::default(),
            1 => sets[0].clone(),
            2 => sets[0].union(sets[1]),
            3 => MoveSet::union_3(sets[0], sets[1], sets[2]),
            4 => MoveSet::union_4(sets[0], sets[1], sets[2], sets[3]),
            5 => MoveSet::union_5(sets[0], sets[1], sets[2], sets[3], sets[4]),
            6 => MoveSet::union_6(sets[0], sets[1], sets[2], sets[3], sets[4], sets[5]),
            7 => MoveSet::union_7(sets[0], sets[1], sets[2], sets[3], sets[4], sets[5], sets[6]),
            _ => unreachable!("remainder of 8-ary tuple iterator is always 7 elements or fewer")
        }
    }

    /// Vectorized union on an arbitrary collection of MoveSets.
    pub fn union_many<'a>(iter: impl Iterator<Item = &'a MoveSet>) -> MoveSet {
        let mut set_iter = iter.into_iter().tuples::<(_,_,_,_,_,_,_,_)>();
        
        let mut sets = set_iter
            .by_ref()
            .map(|(a, b, c, d, e, f, g, h)| MoveSet::union_8(a, b, c, d, e, f, g, h))
            .collect::<Vec<_>>();
        let remainder = set_iter.into_buffer().collect();
        sets.push(MoveSet::union_remainder(&remainder));

        match sets.len() {
            1 => sets[0],
            _ => MoveSet::union_many(sets.iter())
        }
    }
}
