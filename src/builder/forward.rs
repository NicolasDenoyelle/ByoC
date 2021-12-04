use crate::builder::builders::{
    ArrayBuilder, AssociativeBuilder, BTreeBuilder, PolicyBuilder,
    SequentialBuilder,
};
use crate::builder::traits::{
    Array, Associative, BTree, Builder, Forward as ForwardTo, Policy,
    Sequential,
};

#[cfg(feature = "stream")]
use crate::builder::builders::ByteStreamBuilder;
#[cfg(feature = "stream")]
use crate::builder::traits::ByteStream;
#[cfg(feature = "stream")]
use crate::container::stream::{Stream, StreamFactory};
#[cfg(feature = "stream")]
use serde::{de::DeserializeOwned, Serialize};

use crate::connector::Forward;
use crate::policy::{Reference, ReferenceFactory};
use std::hash::Hasher;
use std::marker::PhantomData;

/// [Forward](../../connector/struct.Forward.html)
/// container [builder](../traits/trait.Builder.html).
///
/// This builder can be consumed later to connect two containers together
/// with a [Forward](../../connector/struct.Forward.html) connector.
/// It is built in two steps. The initial step is to wrap the left side
/// container, and later the wrap of the right side container.
///
/// After the creation of this builder, the builder is in a half initialized
/// state. Until it is fully initialized, i.e wrapping a right side
/// container, method to add a policy or connect this container to another
/// one, etc. that are not finishing the initialization will panic.
///
/// To finish initialization of this container, you have to call one of the /// methods:
/// [`array()`](../traits/trait.Array.html#tymethod.array),
/// [`btree()`](../traits/trait.BTree.html#tymethod.btree),
/// [`byte_stream()`](../traits/trait.ByteStream.html#tymethod.byte_stream),
/// while methods:
/// [`with_policy()`](../traits/trait.Policy.html#tymethod.with_policy),
/// [`forward()`](../traits/trait.Forward.html#tymethod.forward),
/// [`into_associative()`](../traits/trait.Associative.html#tymethod.into_associative),
/// [`into_sequential()`](../traits/trait.Sequential.html#tymethod.into_sequential) will panic.
///
/// Once the container is fully initialize, you can call the latter methods,
/// while the former methods will panic.
///
/// ## Examples
/// ```
/// use cache::BuildingBlock;
/// use cache::builder::traits::*;
/// use cache::policy::FIFO;
/// use cache::container::{Array, BTree as Tree};
/// use cache::builder::builders::{ArrayBuilder, BTreeBuilder, ForwardBuilder};
///
/// let array_builder = ArrayBuilder::new(2);
/// let forward_builder = ForwardBuilder::new(array_builder);
/// let mut container = forward_builder.btree(2).build();
/// container.push(vec![(1, 2)]);
///
/// // You can also chain calls.
/// let mut container = (ArrayBuilder::new(2).forward().btree(2)).build();
/// container.push(vec![(1, 2)]);
///
/// // You can for instance wrap the whole piece of connected components
/// // in a policy container.
/// let mut container = (ArrayBuilder::new(2).forward().btree(2)).with_policy(FIFO::new()).build();
/// container.push(vec![(1, 2)]);
/// ```
pub struct ForwardBuilder<L, LB, R, RB>
where
    LB: Builder<L>,
    RB: Builder<R>,
{
    lbuilder: LB,
    rbuilder: Option<RB>,
    unused: PhantomData<(L, R)>,
}

impl<L, LB, T>
    Array<
        T,
        ForwardBuilder<L, LB, crate::container::Array<T>, ArrayBuilder<T>>,
    >
    for ForwardBuilder<L, LB, crate::container::Array<T>, ArrayBuilder<T>>
where
    LB: Builder<L>,
{
    fn array(
        mut self,
        capacity: usize,
    ) -> ForwardBuilder<L, LB, crate::container::Array<T>, ArrayBuilder<T>>
    {
        if self.rbuilder.is_some() {
            panic!("Cannot append an array to a forward container already connected.");
        }
        self.rbuilder = Some(ArrayBuilder::new(capacity));
        self
    }
}

impl<L, LB, K, V>
    BTree<
        K,
        V,
        ForwardBuilder<
            L,
            LB,
            crate::container::BTree<K, V>,
            BTreeBuilder<K, V>,
        >,
    >
    for ForwardBuilder<
        L,
        LB,
        crate::container::BTree<K, V>,
        BTreeBuilder<K, V>,
    >
where
    LB: Builder<L>,
    K: Copy + Ord,
    V: Ord,
{
    fn btree(
        mut self,
        capacity: usize,
    ) -> ForwardBuilder<
        L,
        LB,
        crate::container::BTree<K, V>,
        BTreeBuilder<K, V>,
    > {
        if self.rbuilder.is_some() {
            panic!("Cannot append a btree to a forward container already connected.");
        }
        self.rbuilder = Some(BTreeBuilder::new(capacity));
        self
    }
}

impl<L, LB, R, RB> Clone for ForwardBuilder<L, LB, R, RB>
where
    LB: Builder<L> + Clone,
    RB: Builder<R> + Clone,
{
    fn clone(&self) -> Self {
        ForwardBuilder {
            lbuilder: self.lbuilder.clone(),
            rbuilder: match &self.rbuilder {
                None => None,
                Some(rb) => Some(rb.clone()),
            },
            unused: PhantomData,
        }
    }
}

impl<L, LB, R, RB> ForwardBuilder<L, LB, R, RB>
where
    LB: Builder<L>,
    RB: Builder<R>,
{
    pub fn new(lbuilder: LB) -> Self {
        Self {
            lbuilder: lbuilder,
            rbuilder: None,
            unused: PhantomData,
        }
    }
}

impl<L, LB, R, RB> Sequential<Forward<L, R>>
    for ForwardBuilder<L, LB, R, RB>
where
    LB: Builder<L>,
    RB: Builder<R>,
{
    fn into_sequential(self) -> SequentialBuilder<Forward<L, R>, Self> {
        if self.rbuilder.is_none() {
            panic!(
                "Cannot transform into sequential containers until connected to another block."
            )
        }
        SequentialBuilder::new(self)
    }
}

impl<L, LB, R, RB> Associative<Forward<L, R>>
    for ForwardBuilder<L, LB, R, RB>
where
    LB: Builder<L> + Clone,
    RB: Builder<R> + Clone,
{
    fn into_associative<H: Hasher + Clone>(
        self,
        n_sets: usize,
        key_hasher: H,
    ) -> AssociativeBuilder<Forward<L, R>, H, Self> {
        if self.rbuilder.is_none() {
            panic!(
                "Cannot transform into associative containers until connected to another block."
            )
        }
        AssociativeBuilder::new(self, n_sets, key_hasher)
    }
}

impl<L, LB, R, RB> ForwardTo<Forward<L, R>, R, RB>
    for ForwardBuilder<L, LB, R, RB>
where
    LB: Builder<L>,
    RB: Builder<R>,
{
    fn forward(self) -> ForwardBuilder<Forward<L, R>, Self, R, RB> {
        if self.rbuilder.is_none() {
            panic!(
                "Cannot connect forward until connected to another block."
            )
        }

        ForwardBuilder::new(self)
    }
}

impl<L, LB, R, RB> Policy<Forward<L, R>> for ForwardBuilder<L, LB, R, RB>
where
    LB: Builder<L>,
    RB: Builder<R>,
{
    fn with_policy<V, I: Reference<V>, F: ReferenceFactory<V, I>>(
        self,
        policy: F,
    ) -> PolicyBuilder<Forward<L, R>, V, I, F, Self>
    where
        Self: Sized,
    {
        if self.rbuilder.is_none() {
            panic!(
                "Cannot build a policy for a Forward block that is not connected."
            )
        }
        PolicyBuilder::new(self, policy)
    }
}

#[cfg(feature = "stream")]
impl<T, S, F, L, LB>
    ByteStream<
        T,
        S,
        F,
        ForwardBuilder<
            L,
            LB,
            crate::container::ByteStream<T, S, F>,
            ByteStreamBuilder<T, S, F>,
        >,
    >
    for ForwardBuilder<
        L,
        LB,
        crate::container::ByteStream<T, S, F>,
        ByteStreamBuilder<T, S, F>,
    >
where
    T: DeserializeOwned + Serialize,
    S: Stream,
    F: StreamFactory<S> + Clone,
    LB: Builder<L>,
{
    fn byte_stream(mut self, factory: F, capacity: usize) -> Self {
        if self.rbuilder.is_some() {
            panic!(
                "Cannot append a stream container to an already connected forward block."
            )
        }
        self.rbuilder = Some(ByteStreamBuilder::new(factory, capacity));

        self
    }
}

impl<L, LB, R, RB> Builder<Forward<L, R>> for ForwardBuilder<L, LB, R, RB>
where
    LB: Builder<L>,
    RB: Builder<R>,
{
    fn build(self) -> Forward<L, R> {
        Forward::new(self.lbuilder.build(), self.rbuilder.expect("ForwardBuilder requires to connect a container on the right hand side.").build())
    }
}
