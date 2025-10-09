
use crate::prelude::*;
use itertools::Itertools;

type SubSet = u64;
const NUM_SUBSETS: usize = 2;
const BOARD_CELLS: usize = BOARD_SIZE * BOARD_SIZE; // 100 cells for 10x10 board

#[derive(Clone, Copy, Debug)]
pub struct CoordSet([SubSet; NUM_SUBSETS]);

// Mask for the second u64 to zero out unused bits (36-63)
const EXTENT_MASK: SubSet = (1u64 << (BOARD_CELLS - 64)) - 1; // Mask for bits 0-35

impl CoordSet {
    #[inline]
    fn _index(coord: &Coord) -> (usize, usize) {
        let linear_index = coord.row * BOARD_SIZE + coord.col;
        (linear_index / 64, linear_index % 64)
    }

    pub fn neg_inplace(&mut self) -> & mut Self {
        self.0[0] = !self.0[0];
        self.0[1] = (!self.0[1]) & EXTENT_MASK;
        self
    }

    /// Fast check if intersection would be empty without allocating
    #[inline]
    pub fn intersects(&self, other: &Self) -> bool {
        (self.0[0] & other.0[0]) != 0 || (self.0[1] & other.0[1]) != 0
    }

    /// Fast in-place intersection test that returns whether result would be empty
    #[inline]
    pub fn would_intersect_empty(&self, other: &Self) -> bool {
        !self.intersects(other)
    }

    /// Fast count of elements without allocation - optimized for small sets
    #[inline]
    pub fn count_fast(&self) -> usize {
        self.0[0].count_ones() as usize + self.0[1].count_ones() as usize
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
        self.count_fast()
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
        CoordSet([
            self.0[0] & other.0[0],
            self.0[1] & other.0[1],
        ])
    }

    fn intersect_inplace(&mut self, other: &Self) -> &mut Self {
        self.0[0] &= other.0[0];
        self.0[1] &= other.0[1];
        self
    }

    fn is_empty(&self) -> bool {
        self.0[0] == 0 && self.0[1] == 0
    }

    fn union(&self, other: &Self) -> Self {
        CoordSet([
            self.0[0] | other.0[0],
            self.0[1] | other.0[1],
        ])
    }

    fn union_inplace(&mut self, other: &Self) -> &mut Self {
        self.0[0] |= other.0[0];
        self.0[1] |= other.0[1];
        self
    }

    fn difference(&self, other: &Self) -> Self {
        CoordSet([
            self.0[0] & !other.0[0],
            self.0[1] & !other.0[1],
        ])
    }

    fn difference_inplace(&mut self, other: &Self) -> &mut Self {
        self.0[0] &= !other.0[0];
        self.0[1] &= !other.0[1];
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

            if tz >= 64 {
                self.current_subset += 1;
                self.mask = SubSet::MAX;
                continue;
            } else {
                let linear_index = self.current_subset * 64 + tz;
                if linear_index >= BOARD_CELLS {
                    self.current_subset += 1;
                    self.mask = SubSet::MAX;
                    continue;
                }
                let row = linear_index / BOARD_SIZE;
                let col = linear_index % BOARD_SIZE;
                let value = Coord::new(row, col);
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

            if tz >= 64 {
                self.current_subset += 1;
                self.mask = SubSet::MAX;
                continue;
            } else {
                let linear_index = self.current_subset * 64 + tz;
                if linear_index >= BOARD_CELLS {
                    self.current_subset += 1;
                    self.mask = SubSet::MAX;
                    continue;
                }
                let row = linear_index / BOARD_SIZE;
                let col = linear_index % BOARD_SIZE;
                let value = Coord::new(row, col);
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

    /// In-place union of 8 sets into an accumulator
    #[inline]
    pub fn union_8_inplace(acc: &mut CoordSet, a: &CoordSet, b: &CoordSet, c: &CoordSet, d: &CoordSet, e: &CoordSet, f: &CoordSet, g: &CoordSet, h: &CoordSet) {
        acc.0[0] |= a.0[0] | b.0[0] | c.0[0] | d.0[0] | e.0[0] | f.0[0] | g.0[0] | h.0[0];
        acc.0[1] |= a.0[1] | b.0[1] | c.0[1] | d.0[1] | e.0[1] | f.0[1] | g.0[1] | h.0[1];
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

    /// In-place union of remainder into an accumulator
    #[inline]
    pub fn union_remainder_inplace<'a>(acc: &mut CoordSet, sets: &Vec<&'a CoordSet>) {
        for set in sets {
            acc.union_inplace(set);
        }
    }

    /// Vectorized union on an arbitrary collection of CoordSets.
    pub fn union_many<'a>(iter: impl Iterator<Item = &'a CoordSet>) -> CoordSet {
        let mut result = CoordSet::default();
        let mut set_iter = iter.into_iter().tuples::<(_,_,_,_,_,_,_,_)>();

        // Process in groups of 8 for vectorization, accumulate directly in-place
        for (a, b, c, d, e, f, g, h) in set_iter.by_ref() {
            CoordSet::union_8_inplace(&mut result, a, b, c, d, e, f, g, h);
        }

        // Handle remainder in-place
        let remainder: Vec<&CoordSet> = set_iter.into_buffer().collect();
        if !remainder.is_empty() {
            CoordSet::union_remainder_inplace(&mut result, &remainder);
        }

        result
    }
}
