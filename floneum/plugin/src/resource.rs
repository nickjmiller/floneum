use std::{
    any::{Any, TypeId},
    borrow::BorrowMut,
    collections::HashMap,
    marker::PhantomData,
};

use kalosm::language::{Node, NodeRef, Page};
use parking_lot::{RwLock, RwLockReadGuard, RwLockWriteGuard, };
use slab::Slab;
use std::sync::Arc;
use wasmtime::component::Type;

use crate::{
    embedding::LazyTextEmbeddingModel, embedding_db::VectorDBWithDocuments,
    llm::LazyTextGenerationModel, plugins::main,
};

#[derive(Default)]
pub struct ResourceStorage {
    map: Arc<RwLock<HashMap<TypeId, Slab<Box<dyn Any + Send + Sync>>>>>,
}

impl ResourceStorage {
    fn insert<T: Send + Sync + 'static>(&mut self, item: T) -> Resource<T> {
        let ty_id = TypeId::of::<T>();
        let mut binding = self.map.write();
        let mut slab = binding.entry(ty_id).or_default();
        let id = slab.insert(Box::new(item));
        Resource {
            index: id,
            owned: true,
            phantom: PhantomData,
        }
    }

    fn get<T: Send + Sync+'static>(&self, key: Resource<T>) -> Option<parking_lot::lock_api::MappedRwLockReadGuard<'_, parking_lot::RawRwLock, T>> { 
        RwLockReadGuard::try_map(self.map.read(), |r| {
            r.get(&TypeId::of::<T>())
                .and_then(|slab| slab.get(key.index))
                .and_then(|any| any.downcast_ref())
        }).ok()
    }

    fn get_mut<T: Send + Sync+'static>(&mut self, key: Resource<T>) -> Option<parking_lot::lock_api::MappedRwLockWriteGuard<'_, parking_lot::RawRwLock, T>> {
        RwLockWriteGuard::try_map(self.map.write(), |r| {
            r.get_mut(&TypeId::of::<T>())
                .and_then(|slab| slab.get_mut(key.index))
                .and_then(|any| any.downcast_mut())
        })
        .ok()
    }

    fn drop_key<T: Send + Sync+'static>(&mut self, key: Resource<T>) {
        assert!(key.owned);
        if let Some(slab) = self.map.write().get_mut(&TypeId::of::<T>()) {
            slab.remove(key.index);
        }
    }
}

struct Resource<T> {
    index: usize,
    owned: bool,
    phantom: PhantomData<T>,
}

impl From<main::types::EmbeddingModel> for Resource<LazyTextEmbeddingModel> {
    fn from(value: main::types::EmbeddingModel) -> Self {
        Self {
            index: value.id as usize,
            owned: value.owned,
            phantom: PhantomData,
        }
    }
}

impl From<main::types::TextGenerationModel> for Resource<LazyTextGenerationModel> {
    fn from(value: main::types::TextGenerationModel) -> Self {
        Self {
            index: value.id as usize,
            owned: value.owned,
            phantom: PhantomData,
        }
    }
}

impl From<main::types::EmbeddingDb> for Resource<VectorDBWithDocuments> {
    fn from(value: main::types::EmbeddingDb) -> Self {
        Self {
            index: value.id as usize,
            owned: value.owned,
            phantom: PhantomData,
        }
    }
}

impl From<main::types::Page> for Resource<Page> {
    fn from(value: main::types::Page) -> Self {
        Self {
            index: value.id as usize,
            owned: value.owned,
            phantom: PhantomData,
        }
    }
}

impl From<main::types::Node> for Resource<NodeRef> {
    fn from(value: main::types::Node) -> Self {
        Self {
            index: value.id as usize,
            owned: value.owned,
            phantom: PhantomData,
        }
    }
}
