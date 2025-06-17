use std::{
    any::{Any, TypeId},
    collections::HashMap,
    hash::BuildHasherDefault,
    pin::Pin,
    sync::Mutex,
};

/// A guard for a state value.
///
/// See [`Manager::manage`](`crate::Manager::manage`) for usage examples.
pub struct State<'r, T: Send + Sync + 'static>(&'r T);

impl<'r, T: Send + Sync + 'static> State<'r, T> {
    /// Retrieve a borrow to the underlying value with a lifetime of `'r`.
    /// Using this method is typically unnecessary as `State` implements
    /// [`std::ops::Deref`] with a [`std::ops::Deref::Target`] of `T`.
    #[inline(always)]
    #[allow(unused)]
    pub fn inner(&self) -> &'r T {
        self.0
    }
}

impl<T: Send + Sync + 'static> std::ops::Deref for State<'_, T> {
    type Target = T;

    #[inline(always)]
    fn deref(&self) -> &T {
        self.0
    }
}

impl<T: Send + Sync + 'static> Clone for State<'_, T> {
    fn clone(&self) -> Self {
        State(self.0)
    }
}

impl<T: Send + Sync + 'static + PartialEq> PartialEq for State<'_, T> {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl<T: Send + Sync + std::fmt::Debug> std::fmt::Debug for State<'_, T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("State").field(&self.0).finish()
    }
}

// Taken from: https://github.com/SergioBenitez/state/blob/556c1b94db8ce8427a0e72de7983ab5a9af4cc41/src/ident_hash.rs
// This is a _super_ stupid hash. It just uses its input as the hash value. This
// hash is meant to be used _only_ for "prehashed" values. In particular, we use
// this so that hashing a TypeId is essentially a noop. This is because TypeIds
// are already unique integers.
#[derive(Default)]
struct IdentHash(u64);

impl std::hash::Hasher for IdentHash {
    fn finish(&self) -> u64 {
        self.0
    }

    fn write(&mut self, bytes: &[u8]) {
        for byte in bytes {
            self.write_u8(*byte);
        }
    }

    fn write_u8(&mut self, i: u8) {
        self.0 = (self.0 << 8) | (i as u64);
    }

    fn write_u64(&mut self, i: u64) {
        self.0 = i;
    }
}

/// Safety:
/// - The `key` must equal to `(*value).type_id()`, see the safety doc in methods of [StateManager] for details.
/// - Once you insert a value, you can't remove/mutated/move it anymore, see [StateManager::try_get] for details.
type TypeIdMap = HashMap<TypeId, Pin<Box<dyn Any + Sync + Send>>, BuildHasherDefault<IdentHash>>;

/// The Tauri state manager.
#[derive(Debug)]
pub struct StateManager {
    map: Mutex<TypeIdMap>,
}

impl StateManager {
    pub(crate) fn new() -> Self {
        Self {
            map: Default::default(),
        }
    }

    pub(crate) fn set<T: Send + Sync + 'static>(&self, state: T) -> bool {
        let mut map = self.map.lock().unwrap();
        let type_id = TypeId::of::<T>();
        let already_set = map.contains_key(&type_id);
        if !already_set {
            let ptr = Box::new(state) as Box<dyn Any + Sync + Send>;
            let pinned_ptr = Box::into_pin(ptr);
            map.insert(
                type_id,
                // SAFETY: keep the type of the key is the same as the type of the value，
                // see [try_get] methods for details.
                pinned_ptr,
            );
        }
        !already_set
    }

    /// SAFETY: Calling this method will move the `value`,
    /// which will cause references obtained through [Self::try_get] to dangle.
    pub(crate) unsafe fn unmanage<T: Send + Sync + 'static>(&self) -> Option<T> {
        let mut map = self.map.lock().unwrap();
        let type_id = TypeId::of::<T>();
        let pinned_ptr = map.remove(&type_id)?;
        // SAFETY: The caller decides to break the immovability/safety here, then OK, just let it go.
        let ptr = unsafe { Pin::into_inner_unchecked(pinned_ptr) };
        let value = unsafe {
            ptr.downcast::<T>()
                // SAFETY: the type of the key is the same as the type of the value
                .unwrap_unchecked()
        };
        Some(*value)
    }

    /// Gets the state associated with the specified type.
    pub fn get<T: Send + Sync + 'static>(&self) -> State<'_, T> {
        self.try_get()
            .unwrap_or_else(|| panic!("state not found for type {}", std::any::type_name::<T>()))
    }

    /// Gets the state associated with the specified type.
    pub fn try_get<T: Send + Sync + 'static>(&self) -> Option<State<'_, T>> {
        let map = self.map.lock().unwrap();
        let type_id = TypeId::of::<T>();
        let ptr = map.get(&type_id)?;
        let value = unsafe {
            ptr.downcast_ref::<T>()
                // SAFETY: the type of the key is the same as the type of the value
                .unwrap_unchecked()
        };
        // SAFETY: We ensure the lifetime of `value` is the same as [StateManager] and `value` will not be mutated/moved.
        let v_ref = unsafe { &*(value as *const T) };
        Some(State(v_ref))
    }
}
