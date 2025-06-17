pub mod app;
pub mod state;

use once_cell::sync::Lazy;

pub static STATE_MANAGER: Lazy<state::StateManager> = Lazy::new(|| state::StateManager::new());

pub fn manage<T: Send + Sync + 'static>(value: T) -> Option<state::State<'static, T>> {
    if !STATE_MANAGER.set(value) {
        return Some(STATE_MANAGER.get());
    }
    None
}

pub fn manage_or_replace<T: Send + Sync + 'static>(value: T) {
    unsafe {
        STATE_MANAGER.unmanage::<T>();
    }
    STATE_MANAGER.set(value);
}

#[allow(unused)]
pub fn unmanage<T: Send + Sync + 'static>() -> Option<T> {
    unsafe { STATE_MANAGER.unmanage() }
}

pub fn get_state<T: Send + Sync + 'static>() -> state::State<'static, T> {
    STATE_MANAGER.get()
}

pub fn try_get_state<T: Send + Sync + 'static>() -> Option<state::State<'static, T>> {
    STATE_MANAGER.try_get()
}
