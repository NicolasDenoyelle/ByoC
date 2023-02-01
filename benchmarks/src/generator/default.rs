use std::marker::PhantomData;

pub struct DefaultValueGenerator<T: Default> {
    unused: PhantomData<T>,
}

impl<T: Default> DefaultValueGenerator<T> {
    pub fn new() -> Self {
        Self {
            unused: PhantomData,
        }
    }
}

impl<T: Default> Iterator for DefaultValueGenerator<T> {
    type Item = T;
    fn next(&mut self) -> Option<Self::Item> {
        Some(T::default())
    }
}
