#[derive(Clone)]
pub struct ZipGenerator<K, V> {
    pub(super) key_generator: K,
    pub(super) value_generator: V,
}

impl<K, V> ZipGenerator<K, V> {
    pub fn new(key_generator: K, value_generator: V) -> Self {
        Self {
            key_generator,
            value_generator,
        }
    }
}

impl<K: IntoIterator, V: IntoIterator> IntoIterator
    for ZipGenerator<K, V>
{
    type Item = (K::Item, V::Item);
    type IntoIter = std::iter::Zip<K::IntoIter, V::IntoIter>;

    fn into_iter(self) -> Self::IntoIter {
        self.key_generator
            .into_iter()
            .zip(self.value_generator.into_iter())
    }
}
