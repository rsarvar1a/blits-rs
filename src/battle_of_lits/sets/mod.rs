
mod coordset;
mod moveset;

pub use coordset::CoordSet;
pub use moveset::MoveSet;

pub trait SetOps<T: Clone + Copy + std::fmt::Debug> {
    fn contains(&self, value: &T) -> bool;
    fn len(&self) -> usize;

    fn iter(&self) -> impl Iterator<Item = T>;

    fn insert(&mut self, value: &T) -> &mut Self;
    fn remove(&mut self, value: &T) -> &mut Self;

    fn intersect(&self, other: &Self) -> Self;
    fn intersect_inplace(&mut self, other: &Self) -> &mut Self;
    
    fn union(&self, other: &Self) -> Self;
    fn union_inplace(&mut self, other: &Self) -> &mut Self;

    fn difference(&self, other: &Self) -> Self;
    fn difference_inplace(&mut self, other: &Self) -> &mut Self;
}
