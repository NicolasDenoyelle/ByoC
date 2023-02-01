use crate::decorator::DecorationFactory;
use std::marker::PhantomData;

//------------------------------------------------------------------------//
// Decoration wrapper                                                      //
//------------------------------------------------------------------------//

/// `BuildingBlock` wrapping its values in a `Decoration` cell.
///
/// [`Decorator`] is a [`BuildingBlock`](trait.BuildingBlock.html) wrapper that
/// wraps its values inside of
/// a [`Decoration`](utils/decorator/trait.Decoration.html) cell when they are
/// inserted and unwraps them out of the cell when they are taken out.
///
/// The [`Decorator`] [`BuildingBlock`](trait.BuildingBlock.html) simply
/// forwards methods calls to the
/// [`BuildingBlock`](trait.BuildingBlock.html) it wraps. It is associated with
/// a [`DecorationFactory`](utils/decorator/trait.DecorationFactory.html) that
/// instantiates the wrapping
/// [`Decoration`](utils/decorator/trait.Decoration.html) cell of the element
/// to insert when the latter is inserted.
///
/// Decorating a building block values may allow to customize its behavior when
/// the latter relies on the implementation of a trait carried by its values and
/// implemented by the decoration.
/// For instance, for a container evicting values based on their order,
/// decoration cells may provide a specific implementation of values order,
/// therefore dictating the eviction policy.
///
/// ## Examples
///
/// See [`decorator`](utils/decorator/index.html) module for examples.
pub struct Decorator<C, V, F>
where
    F: DecorationFactory<V>,
{
    pub(super) container: C,
    pub(super) factory: F,
    pub(super) unused: PhantomData<V>,
}

impl<C, V, F> Decorator<C, V, F>
where
    F: DecorationFactory<V>,
{
    /// Construct a new decorator wrapper.
    pub fn new(container: C, factory: F) -> Self {
        Decorator {
            container,
            factory,
            unused: PhantomData,
        }
    }
}
impl<C, V, F> Clone for Decorator<C, V, F>
where
    F: DecorationFactory<V> + Clone,
    C: Clone,
{
    fn clone(&self) -> Self {
        Decorator {
            container: self.container.clone(),
            factory: self.factory.clone(),
            unused: PhantomData,
        }
    }
}

impl<'a, K, V, C, F> From<Decorator<C, V, F>>
    for crate::DynBuildingBlock<'a, K, V>
where
    K: 'a + Ord,
    V: 'a,
    F: 'a + DecorationFactory<V>,
    C: 'a + crate::BuildingBlock<K, F::Item>,
{
    fn from(decorator: Decorator<C, V, F>) -> Self {
        crate::DynBuildingBlock::new(decorator, false)
    }
}

//------------------------------------------------------------------------//
//  Tests
//------------------------------------------------------------------------//

// TODO
