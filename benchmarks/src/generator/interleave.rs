use std::collections::VecDeque;

#[derive(Clone)]
pub struct InterleaveGenerator<G> {
    generators: Vec<G>,
}

impl<G> InterleaveGenerator<G> {
    pub fn new(generators: Vec<G>) -> Self {
        Self { generators }
    }
}

impl<G, I> IntoIterator for InterleaveGenerator<G>
where
    I: Iterator,
    G: IntoIterator<IntoIter = I, Item = I::Item>,
{
    type IntoIter = InterleaveIterator<I>;
    type Item = I::Item;

    fn into_iter(self) -> Self::IntoIter {
        let mut iterators = VecDeque::with_capacity(self.generators.len());
        for iter in self.generators.into_iter().map(|i| i.into_iter()) {
            iterators.push_back(iter);
        }

        InterleaveIterator::<I> { iterators }
    }
}

pub struct InterleaveIterator<I> {
    iterators: VecDeque<I>,
}

impl<I> Iterator for InterleaveIterator<I>
where
    I: Iterator,
{
    type Item = I::Item;

    fn next(&mut self) -> Option<Self::Item> {
        match self.iterators.pop_front() {
            None => None,
            Some(mut iter) => match iter.next() {
                None => self.next(),
                Some(item) => {
                    self.iterators.push_back(iter);
                    Some(item)
                }
            },
        }
    }
}
