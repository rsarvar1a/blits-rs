
mod coordset;
mod moveset;

pub use coordset::CoordSet;
pub use moveset::MoveSet;

pub trait SetOps<T: Clone + Copy + std::fmt::Debug, I> {
    /// Determines whether the given element is in this set.
    fn contains(&self, value: T) -> bool;
    
    /// Returns the length of the set.
    /// 
    /// WARNING: it is highly recommended that this operations
    /// is constant time, as it is used internally to back
    /// optimizations on the pairwise set operations.
    fn len(&self) -> usize;

    /// Iterates over the elements in the set.
    fn iter(&self) -> impl Iterator<Item = I>;

    /// Inserts a value into the set, if it does not already exist.
    fn insert(&mut self, value: T) -> &mut Self;

    /// Removes a value from the set, if it exists.
    fn remove(&mut self, value: T) -> &mut Self;

    /// Returns a new set consisting of all the elements of self
    /// that are also in the other set.
    fn intersect(&self, other: &Self) -> Self;

    /// Removes all elements from self that are not also in the
    /// other set.
    fn intersect_inplace(&mut self, other: &Self) -> &mut Self;
    
    /// Returns a new set consisting of all the elements present
    /// in self or the other set.
    fn union(&self, other: &Self) -> Self;
    
    /// Inserts all elements in other into self.
    fn union_inplace(&mut self, other: &Self) -> &mut Self;

    /// Returns a set consisting of all elements in self that are
    /// not present in the other set.
    fn difference(&self, other: &Self) -> Self;

    /// Removes all elements in other from self.
    fn difference_inplace(&mut self, other: &Self) -> &mut Self;
}
