use std::marker::PhantomData;

pub struct CollectIterator<T, I> {
    step_size: usize,
    iter: I,
    item: PhantomData<T>,
}

impl<T, I> CollectIterator<T, I> {
    pub fn new(iter: I, step_size: usize) -> Self {
        Self {
            step_size,
            iter,
            item: PhantomData,
        }
    }
}

impl<T, I> Iterator for CollectIterator<T, I>
where
    I: Iterator<Item = T>,
{
    type Item = Vec<T>;

    fn next(&mut self) -> Option<Self::Item> {
        let mut values = Vec::with_capacity(self.step_size);

        for _ in 0..self.step_size {
            match self.iter.next() {
                Some(v) => {
                    values.push(v);
                }
                None => break,
            }
        }

        if values.is_empty() {
            None
        } else {
            Some(values)
        }
    }
}

pub struct VecGenerator<G, R> {
    key_value_pair_generator: G,
    vec_size_generator: R,
}

impl<G, R> VecGenerator<G, R> {
    pub fn new(
        key_value_pair_generator: G,
        vec_size_generator: R,
    ) -> Self {
        Self {
            key_value_pair_generator,
            vec_size_generator,
        }
    }
}

impl<G, R> IntoIterator for VecGenerator<G, R>
where
    G: IntoIterator,
    R: IntoIterator<Item = usize>,
{
    type Item = Vec<G::Item>;
    type IntoIter = VecIterator<G::IntoIter, R::IntoIter>;

    fn into_iter(self) -> Self::IntoIter {
        VecIterator {
            vec_size_iterator: self.vec_size_generator.into_iter(),
            key_value_pair_iterator: self
                .key_value_pair_generator
                .into_iter(),
        }
    }
}

pub struct VecIterator<G, R> {
    key_value_pair_iterator: G,
    vec_size_iterator: R,
}

impl<G, R> Iterator for VecIterator<G, R>
where
    G: Iterator,
    R: Iterator<Item = usize>,
{
    type Item = Vec<G::Item>;

    fn next(&mut self) -> Option<Self::Item> {
        let size = match self.vec_size_iterator.next() {
            None => return None,
            Some(s) => s,
        };

        let mut values = Vec::with_capacity(size);
        for _ in 0..size {
            match self.key_value_pair_iterator.next() {
                None => break,
                Some(kv) => values.push(kv),
            }
        }

        if values.is_empty() {
            None
        } else {
            Some(values)
        }
    }
}
