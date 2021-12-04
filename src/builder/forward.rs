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
