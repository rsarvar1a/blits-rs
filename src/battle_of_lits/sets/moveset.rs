use crate::prelude::*;

const EXTENT: usize = NUM_PIECES + 1;

#[derive(Clone, Debug)]
/// A purpose-built implementation of a moveset.
pub struct MoveSet(Box<[bool; EXTENT]>, usize);

impl Default for MoveSet {
    fn default() -> Self {
        MoveSet(Box::new([false; EXTENT]), 0)
    }
}

impl MoveSet {
    fn _order<'a>(lhs: &'a MoveSet, rhs: &'a MoveSet) -> (&'a MoveSet, &'a MoveSet) {
        if lhs.len() < rhs.len() {
            (lhs, rhs)
        } else {
            (rhs, lhs)
        }
    }

    fn _into_iter(self) -> impl Iterator<Item = usize> {
        self.0.into_iter().enumerate()
            .filter_map(|(i, v)| if v { Some(i) } else { None })
    }

    pub fn new() -> MoveSet {
        MoveSet::default()
    }

    pub fn iter_neg(&self) -> impl Iterator<Item = usize> {
        self.0.iter().enumerate()
            .filter_map(|(i, &v)| if v { None } else { Some(i) })
    }
}

impl SetOps<usize, usize> for MoveSet {
    fn len(&self) -> usize {
        self.1
    }

    fn insert(&mut self, value: usize) -> &mut Self {
        let el = unsafe { self.0.get_unchecked_mut(value) };
        self.1 += !*el as usize;
        *el = true;
        self
    }

    fn remove(&mut self, value: usize) -> &mut Self {
        let el = unsafe { self.0.get_unchecked_mut(value) };
        self.1 -= *el as usize;
        *el = false;
        self
    }

    fn contains(&self, value: usize) -> bool {
        unsafe { *self.0.get_unchecked(value) }
    }

    fn iter(&self) -> impl Iterator<Item = usize> {
        self.0.iter().enumerate()
            .filter_map(|(i, &v)| if v { Some(i) } else { None })
    }

    fn intersect(&self, other: &Self) -> Self {
        let (smaller, larger) = MoveSet::_order(self, other); 
        let mut smaller = smaller.clone();
        smaller.intersect_inplace(larger);
        smaller
    }

    fn intersect_inplace(&mut self, other: &Self) -> &mut Self {
        other.iter_neg().for_each(|mv| { self.remove(mv); });
        self
    }

    fn union(&self, other: &Self) -> Self {
        let (smaller, larger) = MoveSet::_order(self, other);
        let mut larger = larger.clone();
        larger.union_inplace(smaller);
        larger
    }

    fn union_inplace(&mut self, other: &Self) -> &mut Self {
        other.iter().for_each(|mv| { self.insert(mv); });
        self
    }

    fn difference(&self, other: &Self) -> Self {
        let (smaller, larger) = MoveSet::_order(self, other);
        let mut larger = larger.clone();
        larger.difference_inplace(smaller);
        larger
    }

    fn difference_inplace(&mut self, other: &Self) -> &mut Self {
        other.iter().for_each(|mv| { self.remove(mv); });
        self
    }
}

impl IntoIterator for MoveSet {
    type Item = usize;
    type IntoIter = impl Iterator<Item = usize>;
    fn into_iter(self) -> Self::IntoIter {
        self._into_iter()
    }
}

impl IntoIterator for &MoveSet {
    type Item = usize;
    type IntoIter = impl Iterator<Item = usize>;
    fn into_iter(self) -> Self::IntoIter {
        self.clone()._into_iter()
    }
}

impl<'a> FromIterator<&'a usize> for MoveSet {
    fn from_iter<T: IntoIterator<Item = &'a usize>>(iter: T) -> Self {
        let mut arr = [false; EXTENT];
        let mut count = 0;
        iter.into_iter().for_each(|i| {
            if *i >= EXTENT { panic!("out of bounds"); }
            unsafe { *arr.get_unchecked_mut(*i) = true; }
            count += 1;
        });
        MoveSet(Box::new(arr), count)
    }
}

impl FromIterator<usize> for MoveSet {
    fn from_iter<T: IntoIterator<Item = usize>>(iter: T) -> Self {
        let mut arr = [false; EXTENT];
        let mut count = 0;
        iter.into_iter().for_each(|i| {
            if i >= EXTENT { panic!("out of bounds"); }
            unsafe { *arr.get_unchecked_mut(i) = true; }
            count += 1;
        });
        MoveSet(Box::new(arr), count)
    }
}

impl std::ops::Neg for &MoveSet {
    type Output = MoveSet;
    fn neg(self) -> Self::Output {
        MoveSet(Box::new(self.0.map(|v| !v)), EXTENT - self.1)
    }
}
