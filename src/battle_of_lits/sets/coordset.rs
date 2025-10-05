use itertools::Itertools;

use crate::prelude::*;

#[derive(Clone, Debug)]
/// A purpose-built implementation of a coordset.
pub struct CoordSet(Box<[bool; 100]>);

impl Default for CoordSet {
    fn default() -> Self {
        CoordSet(Box::new([false; 100]))
    }
}

impl CoordSet {
    fn _coord(index: usize) -> Coord {
        Coord { row: index / BOARD_SIZE, col: index % BOARD_SIZE }
    }

    fn _index(coord: &Coord) -> usize {
        coord.row * BOARD_SIZE + coord.col
    }

    fn _into_iter(self) -> impl Iterator<Item = Coord> {
        self.0.into_iter().enumerate()
            .filter_map(|(i, v)| if v { Some(CoordSet::_coord(i)) } else { None })
    }

    pub fn new() -> CoordSet {
        CoordSet::default()
    }
}

impl SetOps<&Coord, Coord> for CoordSet {
    fn len(&self) -> usize {
        self.iter().count()
    }

    fn insert(&mut self, coord: &Coord) -> &mut Self {
        let idx = CoordSet::_index(coord);
        self.0[idx] = true;
        self
    }

    fn remove(&mut self, coord: &Coord) -> &mut Self {
        let idx = CoordSet::_index(coord);
        self.0[idx] = false;
        self   
    }

    fn contains(&self, coord: &Coord) -> bool {
        let idx = CoordSet::_index(coord);
        self.0[idx]
    }

    fn iter(&self) -> impl Iterator<Item = Coord> {
        self.0.iter().enumerate()
            .filter_map(|(i, &v)| if v { Some(CoordSet::_coord(i)) } else { None })
    }

    fn intersect(&self, other: &CoordSet) -> CoordSet {
        CoordSet(Box::new(
            self.0.iter()
                .zip(other.0.iter())
                .map(|(l, r)| l & r)
                .collect_array::<100>().unwrap()
        ))
    }

    fn intersect_inplace(&mut self, other: &CoordSet) -> &mut Self {
        self.0.iter_mut()
            .zip(other.0.iter())
            .for_each(|(el, r)| { *el &= r; });
        self
    }

    fn union(&self, other: &CoordSet) -> CoordSet {
        CoordSet(Box::new(
            self.0.iter()
                .zip(other.0.iter())
                .map(|(l, r)| l | r)
                .collect_array::<100>().unwrap()
        ))
    }

    fn union_inplace(&mut self, other: &CoordSet) -> &mut Self {
        self.0.iter_mut()
            .zip(other.0.iter())
            .for_each(|(el, r)| { *el |= r; });
        self
    }

    fn difference(&self, other: &CoordSet) -> CoordSet {
        CoordSet(Box::new(
            self.0.iter()
                .zip(other.0.iter())
                .map(|(l, r)| l & !r)
                .collect_array::<100>().unwrap()
        ))
    }

    fn difference_inplace(&mut self, other: &CoordSet) -> &mut Self {
        self.0.iter_mut()
            .zip(other.0.iter())
            .for_each(|(el, r)| { *el &= !r; });
        self
    }
}

impl IntoIterator for CoordSet {
    type Item = Coord;
    type IntoIter = impl Iterator<Item = Coord>;
    fn into_iter(self) -> Self::IntoIter {
        self._into_iter()
    }
}

impl<'a> FromIterator<&'a Coord> for CoordSet {
    fn from_iter<T: IntoIterator<Item = &'a Coord>>(iter: T) -> Self {
        let mut arr = [false; 100];
        iter.into_iter().for_each(|c| {
            arr[CoordSet::_index(c)] = true;
        });
        CoordSet(Box::new(arr))
    }
}

impl FromIterator<Coord> for CoordSet {
    fn from_iter<T: IntoIterator<Item = Coord>>(iter: T) -> Self {
        let mut arr = [false; 100];
        iter.into_iter().for_each(|c| {
            arr[CoordSet::_index(&c)] = true;
        });
        CoordSet(Box::new(arr))
    }
}
