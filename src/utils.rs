use std::{
    iter::{FusedIterator, Peekable},
    marker::PhantomData,
    ops::Range,
};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CartesianProduct<T, U, I1, I2>
where
    I1: Iterator<Item = T>,
    T: Clone,
    I2: Iterator<Item = U> + Clone,
{
    a_iter: I1,
    a_curr: Option<T>,
    b_iter: I2,
    b_curr: I2,
}

impl<T, U, I1, I2> CartesianProduct<T, U, I1, I2>
where
    I1: Iterator<Item = T>,
    T: Clone,
    I2: Iterator<Item = U> + Clone,
{
    pub fn new(mut a: I1, b: I2) -> Self {
        Self {
            a_curr: a.next(),
            a_iter: a,
            b_iter: b.clone(),
            b_curr: b,
        }
    }
}

impl<T, U, I1, I2> Iterator for CartesianProduct<T, U, I1, I2>
where
    I1: Iterator<Item = T>,
    T: Clone,
    I2: Iterator<Item = U> + Clone,
{
    type Item = (T, U);

    fn next(&mut self) -> Option<Self::Item> {
        let b = match self.b_curr.next() {
            Some(next) => next,
            None => {
                self.b_curr = self.b_iter.clone();
                self.a_curr = self.a_iter.next();
                self.b_curr.next()?
            }
        };
        let a = self.a_curr.clone()?;
        Some((a, b))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let (bc_min, bc_max) = self.b_curr.size_hint();
        let (b_min, b_max) = self.b_iter.size_hint();
        let (a_min, a_max) = self.a_iter.size_hint();
        let min = bc_min + b_min * a_min;
        if bc_max.is_none() || b_max.is_none() || a_max.is_none() {
            return (min, None);
        }
        (min, Some(bc_max.unwrap() + b_max.unwrap() * a_max.unwrap()))
    }
}

impl<T, U, I1, I2> FusedIterator for CartesianProduct<T, U, I1, I2>
where
    I1: Iterator<Item = T> + FusedIterator,
    T: Clone,
    I2: Iterator<Item = U> + Clone,
{
}

pub fn cartesian_product<T, U, I1, I2>(a: I1, b: I2) -> CartesianProduct<T, U, I1, I2>
where
    I1: Iterator<Item = T>,
    T: Clone,
    I2: Iterator<Item = U> + Clone,
{
    CartesianProduct::new(a, b)
}

#[derive(Debug, Clone)]
pub struct ChunkIter<T, I, F>
where
    I: Iterator<Item = T> + FusedIterator,
    F: FromIterator<T>,
{
    iter: Peekable<I>,
    chunk_size: usize,
    from_iter_type: PhantomData<F>,
}

impl<T, I, F> ChunkIter<T, I, F>
where
    I: Iterator<Item = T> + FusedIterator,
    F: FromIterator<T>,
{
    pub fn new(iter: I, chunk_size: usize) -> Self {
        Self {
            iter: iter.peekable(),
            chunk_size,
            from_iter_type: PhantomData,
        }
    }
}

impl<T, I, F> Iterator for ChunkIter<T, I, F>
where
    I: Iterator<Item = T> + FusedIterator,
    F: FromIterator<T>,
{
    type Item = F;

    fn next(&mut self) -> Option<Self::Item> {
        if self.iter.peek().is_some() {
            Some((&mut self.iter).take(self.chunk_size).collect())
        } else {
            None
        }
    }
}

impl<T, I, F> FusedIterator for ChunkIter<T, I, F>
where
    I: Iterator<Item = T> + FusedIterator,
    F: FromIterator<T>,
{
}

#[derive(Debug, Clone, PartialEq)]
pub struct RangeChunks {
    range: Range<usize>,
    chunk_size: usize,
}

impl Iterator for RangeChunks {
    type Item = Range<usize>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.range.start >= self.range.end {
            return None;
        }
        let new_end = self.range.end.min(self.range.start + self.chunk_size);
        let item = self.range.start..new_end;
        self.range.start = new_end;
        Some(item)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        if self.range.start == self.range.end {
            return (0, Some(0));
        }
        let count = (self.range.end - self.range.start - 1) / self.chunk_size + 1;
        (count, Some(count))
    }
}

impl RangeChunks {
    pub fn new(range: Range<usize>, chunk_size: usize) -> Self {
        Self { range, chunk_size }
    }
}

impl FusedIterator for RangeChunks {}

impl ExactSizeIterator for RangeChunks {}

#[cfg(test)]
mod tests {
    use crate::utils::RangeChunks;

    use super::ChunkIter;

    #[test]
    fn test_cartesian_product() {
        let a = vec![1, 2, 3];
        let b = vec![4, 5, 6];
        let mut product = super::cartesian_product(a.iter(), b.iter());
        assert_eq!(product.size_hint(), (9, Some(9)));
        assert_eq!(product.next(), Some((&1, &4)));
        assert_eq!(product.size_hint(), (8, Some(8)));
        assert_eq!(product.next(), Some((&1, &5)));
        assert_eq!(product.size_hint(), (7, Some(7)));
        assert_eq!(product.next(), Some((&1, &6)));
        assert_eq!(product.size_hint(), (6, Some(6)));
        assert_eq!(product.next(), Some((&2, &4)));
        assert_eq!(product.size_hint(), (5, Some(5)));
        assert_eq!(product.next(), Some((&2, &5)));
        assert_eq!(product.size_hint(), (4, Some(4)));
        assert_eq!(product.next(), Some((&2, &6)));
        assert_eq!(product.size_hint(), (3, Some(3)));
        assert_eq!(product.next(), Some((&3, &4)));
        assert_eq!(product.size_hint(), (2, Some(2)));
        assert_eq!(product.next(), Some((&3, &5)));
        assert_eq!(product.size_hint(), (1, Some(1)));
        assert_eq!(product.next(), Some((&3, &6)));
        assert_eq!(product.size_hint(), (0, Some(0)));
        assert_eq!(product.next(), None);
    }

    #[test]
    fn test_ranges() {
        let a = 0..9;
        let b = 0..10;
        let a_iter = ChunkIter::new(a, 3);
        let b_iter = ChunkIter::new(b, 3);
        let mut product = super::cartesian_product(a_iter, b_iter);
        assert_eq!(product.next(), Some((vec![0, 1, 2], vec![0, 1, 2])));
        assert_eq!(product.next(), Some((vec![0, 1, 2], vec![3, 4, 5])));
        assert_eq!(product.next(), Some((vec![0, 1, 2], vec![6, 7, 8])));
        assert_eq!(product.next(), Some((vec![0, 1, 2], vec![9])));
        assert_eq!(product.next(), Some((vec![3, 4, 5], vec![0, 1, 2])));
        assert_eq!(product.next(), Some((vec![3, 4, 5], vec![3, 4, 5])));
        assert_eq!(product.next(), Some((vec![3, 4, 5], vec![6, 7, 8])));
        assert_eq!(product.next(), Some((vec![3, 4, 5], vec![9])));
        assert_eq!(product.next(), Some((vec![6, 7, 8], vec![0, 1, 2])));
        assert_eq!(product.next(), Some((vec![6, 7, 8], vec![3, 4, 5])));
        assert_eq!(product.next(), Some((vec![6, 7, 8], vec![6, 7, 8])));
        assert_eq!(product.next(), Some((vec![6, 7, 8], vec![9])));
    }

    #[test]
    fn test_empty_range() {
        let r = 0..0;
        let mut iter: ChunkIter<_, _, Vec<_>> = ChunkIter::new(r, 3);
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn test_unbounded_range() {
        let a = 0..;
        let b = 0..10;
        let a_iter = ChunkIter::new(a, 3);
        let b_iter = ChunkIter::new(b, 3);
        let mut product = super::cartesian_product(a_iter, b_iter);
        assert_eq!(product.next(), Some((vec![0, 1, 2], vec![0, 1, 2])));
        assert_eq!(product.next(), Some((vec![0, 1, 2], vec![3, 4, 5])));
        assert_eq!(product.next(), Some((vec![0, 1, 2], vec![6, 7, 8])));
        assert_eq!(product.next(), Some((vec![0, 1, 2], vec![9])));
        assert_eq!(product.next(), Some((vec![3, 4, 5], vec![0, 1, 2])));
        assert_eq!(product.next(), Some((vec![3, 4, 5], vec![3, 4, 5])));
        assert_eq!(product.next(), Some((vec![3, 4, 5], vec![6, 7, 8])));
        assert_eq!(product.next(), Some((vec![3, 4, 5], vec![9])));
        assert_eq!(product.next(), Some((vec![6, 7, 8], vec![0, 1, 2])));
        assert_eq!(product.next(), Some((vec![6, 7, 8], vec![3, 4, 5])));
        assert_eq!(product.next(), Some((vec![6, 7, 8], vec![6, 7, 8])));
        assert_eq!(product.next(), Some((vec![6, 7, 8], vec![9])));
        assert_eq!(product.next().unwrap().0, vec![9, 10, 11]);
    }

    #[test]
    fn test_range_chunks() {
        let mut chunks = RangeChunks::new(0..10, 3);
        assert_eq!(chunks.size_hint(), (4, Some(4)));
        assert_eq!(chunks.next(), Some(0..3));
        assert_eq!(chunks.size_hint(), (3, Some(3)));
        assert_eq!(chunks.next(), Some(3..6));
        assert_eq!(chunks.size_hint(), (2, Some(2)));
        assert_eq!(chunks.next(), Some(6..9));
        assert_eq!(chunks.size_hint(), (1, Some(1)));
        assert_eq!(chunks.next(), Some(9..10));
        assert_eq!(chunks.size_hint(), (0, Some(0)));
        assert_eq!(chunks.next(), None);
    }
}
