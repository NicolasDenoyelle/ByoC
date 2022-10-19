use super::{Inclusive, InclusiveCell};
use crate::utils::get::LifeTimeGuard;
use crate::{BuildingBlock, Get, GetMut};
use std::ops::{Deref, DerefMut};

/// Try to clone an element from a `from` building block to another `to`
/// building block. If `to` building block pops elements that are not present
/// in `from` or elements updated from this container, they are pushed back to
/// `from` container. If `from` container pops elements as a result, we cannot
/// recover and panic.
/// Return whether the element was found in `from` building
/// block.
fn downgrade<'b, K, V, L, R>(from: &mut R, to: &mut L, key: &K) -> bool
where
    K: 'b + Clone,
    V: 'b + Clone,
    L: BuildingBlock<'b, K, InclusiveCell<V>>,
    R: Get<K, InclusiveCell<V>> + BuildingBlock<'b, K, InclusiveCell<V>>,
{
    let c = match from.get(key) {
        None => return false,
        Some(c) => c,
    };

    let popped: Vec<(K, InclusiveCell<V>)> = to
        .push(vec![(key.clone(), c.clone())])
        .into_iter()
        .filter_map(|(k, c)| match (c.is_updated(), c.is_cloned()) {
            (false, true) => None, // The value is already in the from container
            _ => Some((k, c)),
        })
        .collect();

    if popped.is_empty() {
        return true;
    }

    let overflow = from.push(popped);
    if overflow.is_empty() {
        return true;
    }

    panic!("Downgrading InclusiveCell from back container to front container resulted in an overflow in the back container.");
}

pub struct InclusiveGetCell<V> {
    value: V,
}

impl<T, V> Deref for InclusiveGetCell<T>
where
    T: Deref<Target = InclusiveCell<V>>,
{
    type Target = V;
    fn deref(&self) -> &Self::Target {
        let inclusive_cell = self.value.deref();
        inclusive_cell.deref()
    }
}

impl<T, V> DerefMut for InclusiveGetCell<T>
where
    T: DerefMut<Target = InclusiveCell<V>>,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        let inclusive_cell = self.value.deref_mut();
        inclusive_cell.deref_mut()
    }
}

impl<'b, K, V, L, R> Get<K, V> for Inclusive<'b, K, V, L, R>
where
    K: 'b + Clone,
    V: 'b + Clone,
    L: Get<K, InclusiveCell<V>> + BuildingBlock<'b, K, InclusiveCell<V>>,
    R: Get<K, InclusiveCell<V>> + BuildingBlock<'b, K, InclusiveCell<V>>,
{
    type Target = InclusiveGetCell<L::Target>;

    fn get(&mut self, key: &K) -> Option<LifeTimeGuard<Self::Target>> {
        if self.front.contains(key)
            || downgrade(&mut self.back, &mut self.front, key)
        {
            self.front.get(key).map(|t| {
                LifeTimeGuard::new(InclusiveGetCell { value: t.unwrap() })
            })
        } else {
            None
        }
    }
}

impl<'b, K, V, L, R> GetMut<K, V> for Inclusive<'b, K, V, L, R>
where
    K: 'b + Clone,
    V: 'b + Clone,
    L: GetMut<K, InclusiveCell<V>>
        + BuildingBlock<'b, K, InclusiveCell<V>>,
    R: Get<K, InclusiveCell<V>> + BuildingBlock<'b, K, InclusiveCell<V>>,
{
    type Target = InclusiveGetCell<L::Target>;

    fn get_mut(&mut self, key: &K) -> Option<LifeTimeGuard<Self::Target>> {
        if self.front.contains(key)
            || downgrade(&mut self.back, &mut self.front, key)
        {
            self.front.get_mut(key).map(|t| {
                LifeTimeGuard::new(InclusiveGetCell { value: t.unwrap() })
            })
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Inclusive;
    use crate::tests::{test_get, test_get_mut};
    use crate::Array;

    #[test]
    fn get() {
        test_get(Inclusive::new(Array::new(10), Array::new(0)));
        test_get(Inclusive::new(Array::new(10), Array::new(100)));
        test_get_mut(Inclusive::new(Array::new(10), Array::new(0)));
        test_get_mut(Inclusive::new(Array::new(10), Array::new(100)));
    }
}
