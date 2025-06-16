use database::{prefix::PrefixDB, Database};
use kv_store::{
    bank::multi::ApplicationMultiBank,
    store::kv::{immutable::KVStore, mutable::KVStoreMut},
    StoreKey,
};

use crate::{
    baseapp::ConsensusParams,
    types::store::kv::{mutable::StoreMut, Store},
};
use tendermint::types::{
    chain_id::ChainId,
    proto::{event::Event, header::Header},
    time::timestamp::Timestamp,
};

use super::{InfallibleContext, InfallibleContextMut, QueryableContext, TransactionalContext};

#[derive(Debug)]
pub struct BlockContext<'a, DB, SK> {
    multi_store: &'a mut ApplicationMultiBank<DB, SK>,
    pub(crate) height: u32,
    pub header: Header,
    pub(crate) consensus_params: ConsensusParams,
    pub events: Vec<Event>,
}

impl<'a, DB, SK> BlockContext<'a, DB, SK> {
    pub fn new(
        multi_store: &'a mut ApplicationMultiBank<DB, SK>,
        height: u32,
        header: Header,
        consensus_params: ConsensusParams,
    ) -> Self {
        BlockContext {
            multi_store,
            height,
            events: Vec::new(),
            consensus_params,
            header,
        }
    }

    pub fn chain_id(&self) -> &ChainId {
        &self.header.chain_id
    }

    pub fn consensus_params(&self) -> &ConsensusParams {
        &self.consensus_params
    }
}

impl<DB: Database, SK: StoreKey> BlockContext<'_, DB, SK> {
    pub fn kv_store(&self, store_key: &SK) -> KVStore<'_, PrefixDB<DB>> {
        KVStore::from(self.multi_store.kv_store(store_key))
    }

    pub fn kv_store_mut(&mut self, store_key: &SK) -> KVStoreMut<'_, PrefixDB<DB>> {
        KVStoreMut::from(self.multi_store.kv_store_mut(store_key))
    }
}

impl<DB: Database, SK: StoreKey> QueryableContext<DB, SK> for BlockContext<'_, DB, SK> {
    fn height(&self) -> u32 {
        self.height
    }

    fn chain_id(&self) -> &ChainId {
        &self.header.chain_id
    }

    fn kv_store(&self, store_key: &SK) -> Store<'_, PrefixDB<DB>> {
        Store::from(self.kv_store(store_key))
    }
}

impl<DB: Database, SK: StoreKey> InfallibleContext<DB, SK> for BlockContext<'_, DB, SK> {
    fn infallible_store(&self, store_key: &SK) -> KVStore<'_, PrefixDB<DB>> {
        self.kv_store(store_key)
    }
}

impl<DB: Database, SK: StoreKey> InfallibleContextMut<DB, SK> for BlockContext<'_, DB, SK> {
    fn infallible_store_mut(&mut self, store_key: &SK) -> KVStoreMut<'_, PrefixDB<DB>> {
        self.kv_store_mut(store_key)
    }
}

impl<DB: Database, SK: StoreKey> TransactionalContext<DB, SK> for BlockContext<'_, DB, SK> {
    fn push_event(&mut self, event: Event) {
        self.events.push(event);
    }

    fn append_events(&mut self, mut events: Vec<Event>) {
        self.events.append(&mut events);
    }

    fn events_drain(&mut self) -> Vec<Event> {
        std::mem::take(&mut self.events)
    }

    fn get_time(&self) -> Timestamp {
        self.header.time
    }

    fn kv_store_mut(&mut self, store_key: &SK) -> StoreMut<'_, PrefixDB<DB>> {
        StoreMut::from(self.kv_store_mut(store_key))
    }

    fn tx_index(&self) -> u32 {
        0
    }

    fn tx_hash(&self) -> [u8; 32] {
        [0u8; 32]
    }
}
