use {
    crate::{
        auth::AuthSet,
        providers::{ProviderId, RpcProvider},
        types::PrincipalStorable,
    },
    ic_stable_structures::{
        memory_manager::{MemoryId, MemoryManager, VirtualMemory},
        DefaultMemoryImpl, StableBTreeMap,
    },
    std::cell::RefCell,
};

const AUTH_MEMORY_ID: MemoryId = MemoryId::new(1);
const PROVIDERS_MEMORY_ID: MemoryId = MemoryId::new(2);

pub type StableMemory = VirtualMemory<DefaultMemoryImpl>;
pub type AuthMemory = StableBTreeMap<PrincipalStorable, AuthSet, StableMemory>;
pub type ProvidersMemory = StableBTreeMap<ProviderId, RpcProvider, StableMemory>;

thread_local! {
    // Stable static data: these are preserved when the canister is upgraded.
    pub static MEMORY_MANAGER: RefCell<MemoryManager<DefaultMemoryImpl>> =
        RefCell::new(MemoryManager::init(DefaultMemoryImpl::default()));
}

pub fn get_memory(memory_id: MemoryId) -> StableMemory {
    MEMORY_MANAGER.with(|m| m.borrow().get(memory_id))
}

pub fn init_auth_memory() -> AuthMemory {
    AuthMemory::init(get_memory(AUTH_MEMORY_ID))
}

pub fn init_providers_memory() -> ProvidersMemory {
    ProvidersMemory::init(get_memory(PROVIDERS_MEMORY_ID))
}
