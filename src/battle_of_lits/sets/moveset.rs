use crate::prelude::*;

const EXTENT: usize = NUM_PIECES + 1;

#[derive(Clone, Debug)]
/// A purpose-built implementation of a moveset.
pub struct MoveSet(Box<[bool; EXTENT]>);

impl Default for MoveSet {
    fn default() -> Self {
        MoveSet(Box::new([false; EXTENT]))
    }
}

impl MoveSet {
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

impl SetOps<usize> for MoveSet {
    fn len(&self) -> usize {
        self.0.iter().filter(|&v| *v).count()
    }

    fn insert(&mut self, value: &usize) -> &mut Self {
        self.0[*value] = true;
        self
    }

    fn remove(&mut self, value: &usize) -> &mut Self {
        self.0[*value] = false;
        self
    }

    fn contains(&self, value: &usize) -> bool {
        self.0[*value]
    }

    fn iter(&self) -> impl Iterator<Item = usize> {
        self.0.iter().enumerate()
            .filter_map(|(i, &v)| if v { Some(i) } else { None })
    }

    fn intersect(&self, other: &Self) -> Self {
        let mut s = self.clone();
        s.intersect_inplace(other);
        s
    }

    fn intersect_inplace(&mut self, other: &Self) -> &mut Self {
        other.iter_neg().for_each(|mv| { self.remove(&mv); });
        self
    }

    fn union(&self, other: &Self) -> Self {
        let mut s = self.clone();
        s.union_inplace(other);
        s
    }

    fn union_inplace(&mut self, other: &Self) -> &mut Self {
        other.iter().for_each(|mv| { self.insert(&mv); });
        self
    }

    fn difference(&self, other: &Self) -> Self {
        let mut s = self.clone();
        s.difference_inplace(other);
        s
    }

    fn difference_inplace(&mut self, other: &Self) -> &mut Self {
        other.iter().for_each(|mv| { self.remove(&mv); });
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
        iter.into_iter().for_each(|i| {
            arr[*i] = true;
        });
        MoveSet(Box::new(arr))
    }
}

impl FromIterator<usize> for MoveSet {
    fn from_iter<T: IntoIterator<Item = usize>>(iter: T) -> Self {
        let mut arr = [false; EXTENT];
        iter.into_iter().for_each(|i| {
            arr[i] = true;
        });
        MoveSet(Box::new(arr))
    }
}

impl std::ops::Neg for &MoveSet {
    type Output = MoveSet;
    fn neg(self) -> Self::Output {
        MoveSet(Box::new(self.0.map(|v| !v)))
    }
}
