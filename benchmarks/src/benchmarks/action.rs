use byoc::{BuildingBlock, GetMut};
use std::ops::DerefMut;

pub enum Action {
    /// Push an element in the container.
    Push,
    /// Take an element out of the container.
    Take,
    /// Get an element in the container.
    GetMut,
    /// Lookup key in the container. If key is not found, push an element
    /// with the same key, else access the matching element.
    PushOrGetMut,
    /// Lookup key in the container. If key is not found, push an element
    /// with the same key, else take the matching element out.
    PushOrTake,
    /// Try to access an element in the container.
    /// If it fails, push an element with the same key in the container.
    GetMutOrPush,
}

impl Action {
    /// Run the container action with a given key on a container.
    pub fn call_once<'a, K, V, U, C>(self, key: K, container: &mut C)
    where
        K: 'a,
        V: 'a + Default,
        U: DerefMut<Target = V>,
        C: BuildingBlock<'a, K, V> + GetMut<K, V, U>,
    {
        match self {
            Self::Push => {
                container.push(vec![(key, V::default())]);
            }
            Self::Take => {
                container.take(&key);
            }
            Self::GetMut => unsafe {
                container.get_mut(&key);
            },
            Self::PushOrGetMut => {
                if container.contains(&key) {
                    unsafe {
                        container.get_mut(&key);
                    }
                } else {
                    container.push(vec![(key, V::default())]);
                }
            }
            Self::PushOrTake => {
                if container.contains(&key) {
                    container.take(&key);
                } else {
                    container.push(vec![(key, V::default())]);
                }
            }
            Self::GetMutOrPush => match unsafe { container.get_mut(&key) }
            {
                Some(_) => {}
                None => {
                    container.push(vec![(key, V::default())]);
                }
            },
        }
    }
}
