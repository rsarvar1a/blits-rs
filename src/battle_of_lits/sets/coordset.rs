
use crate::prelude::*;
use itertools::Itertools;

type SubSet = u16;
const SUBSET_SIZE: usize = BOARD_SIZE;
const NUM_SUBSETS: usize = (BOARD_SIZE / 4 + 1) * 4;

#[derive(Clone, Copy, Debug)]
pub struct CoordSet([SubSet; NUM_SUBSETS]);

impl CoordSet {
    #[inline]
    fn _index(coord: &Coord) -> (usize, usize) {
        (coord.row, coord.col)
    }

    pub fn neg_inplace(&mut self) -> & mut Self {
        self.0.iter_mut().for_each(|sub| {
            // we can just flip every relevant bit to change the presence of each Coord,
            // but we can't leave the high bits set from 0 -> 1, so we set them back to 0 using the mask
            *sub = !*sub & EXTENT_MASK;
        });
        self
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

    fn _extend(&mut self, iter: impl Iterator<Item = Coord>) -> &mut Self {
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

impl<'a> IntoIterator for &'a CoordSet {
    type IntoIter = CoordSetIterator<'a>;
    type Item = Coord;
    fn into_iter(self) -> Self::IntoIter {
        CoordSetIterator::new(&self.0)
    }
}

const EXTENT_MASK: SubSet = {
    let ones = ((1 as SubSet) << SUBSET_SIZE) - 1; // place a bit at the desired cap to get a bitstring like 0000010000000000, then subtract to get 0000001111111111
    ones
};

impl std::ops::Neg for CoordSet {
    type Output = CoordSet;
    fn neg(self) -> Self::Output {  
        let mut s = self.clone();
        s.neg_inplace();
        s
    }
}

impl std::ops::Not for CoordSet {
    type Output = CoordSet;
    fn not(self) -> Self::Output {
        let mut s = self.clone();
        s.neg_inplace();
        s
    }
}

impl CoordSet {
    pub fn union_3(a: &CoordSet, b: &CoordSet, c: &CoordSet) -> CoordSet {
        CoordSet(std::array::from_fn(|i| a.0[i] | b.0[i] | c.0[i]))
    }

    pub fn union_4(a: &CoordSet, b: &CoordSet, c: &CoordSet, d: &CoordSet) -> CoordSet {
        CoordSet(std::array::from_fn(|i| a.0[i] | b.0[i] | c.0[i] | d.0[i]))
    }

    pub fn union_5(a: &CoordSet, b: &CoordSet, c: &CoordSet, d: &CoordSet, e: &CoordSet) -> CoordSet {
        CoordSet(std::array::from_fn(|i| a.0[i] | b.0[i] | c.0[i] | d.0[i] | e.0[i]))
    }

    pub fn union_6(a: &CoordSet, b: &CoordSet, c: &CoordSet, d: &CoordSet, e: &CoordSet, f: &CoordSet) -> CoordSet {
        CoordSet(std::array::from_fn(|i| a.0[i] | b.0[i] | c.0[i] | d.0[i] | e.0[i] | f.0[i]))
    }

    pub fn union_7(a: &CoordSet, b: &CoordSet, c: &CoordSet, d: &CoordSet, e: &CoordSet, f: &CoordSet, g: &CoordSet) -> CoordSet {
        CoordSet(std::array::from_fn(|i| a.0[i] | b.0[i] | c.0[i] | d.0[i] | e.0[i] | f.0[i] | g.0[i]))
    }

    pub fn union_8(a: &CoordSet, b: &CoordSet, c: &CoordSet, d: &CoordSet, e: &CoordSet, f: &CoordSet, g: &CoordSet, h: &CoordSet) -> CoordSet {
        CoordSet(std::array::from_fn(|i| a.0[i] | b.0[i] | c.0[i] | d.0[i] | e.0[i] | f.0[i] | g.0[i] | h.0[i]))
    }

    pub fn union_remainder<'a>(sets: &Vec<&'a CoordSet>) -> CoordSet {
        match sets.len() {
            0 => CoordSet::default(),
            1 => sets[0].clone(),
            2 => sets[0].union(sets[1]),
            3 => CoordSet::union_3(sets[0], sets[1], sets[2]),
            4 => CoordSet::union_4(sets[0], sets[1], sets[2], sets[3]),
            5 => CoordSet::union_5(sets[0], sets[1], sets[2], sets[3], sets[4]),
            6 => CoordSet::union_6(sets[0], sets[1], sets[2], sets[3], sets[4], sets[5]),
            7 => CoordSet::union_7(sets[0], sets[1], sets[2], sets[3], sets[4], sets[5], sets[6]),
            _ => unreachable!("remainder of 8-ary tuple iterator is always 7 elements or fewer")
        }
    }

    /// Vectorized union on an arbitrary collection of CoordSets.
    pub fn union_many<'a>(iter: impl Iterator<Item = &'a CoordSet>) -> CoordSet {
        let mut set_iter = iter.into_iter().tuples::<(_,_,_,_,_,_,_,_)>();
        
        let mut sets = set_iter
            .by_ref()
            .map(|(a, b, c, d, e, f, g, h)| CoordSet::union_8(a, b, c, d, e, f, g, h))
            .collect::<Vec<_>>();
        let remainder = set_iter.into_buffer().collect();
        sets.push(CoordSet::union_remainder(&remainder));

        match sets.len() {
            1 => sets[0],
            _ => CoordSet::union_many(sets.iter())
        }
    }
}
