use std::{hash::{Hash, Hasher}, marker::PhantomData, u32};

/// The asset trait.
/// 
/// Every concrete asset type (e.g., [`Mesh`]`) must implement this trait.
pub trait Asset: 'static {
}

// enum AssetID {

// }

/// The handle of a type of asset.
/// 
/// It will be a ECS component for entities which use the coressponding asset.
pub struct Handle<T: Asset> {
    id: u32,
    phantom_data: PhantomData<T>,
}

impl<T: Asset> Handle<T> {
    const INVALID_ID: u32 = u32::MAX;
    pub const INVALID: Handle<T> = Self {
        id: Self::INVALID_ID,
        phantom_data: PhantomData,
    };
    
    fn next_valid_id() -> u32 {
        static mut CUR_AVAIABLE_ID: u32 = 0;

        unsafe {
            let next_id = CUR_AVAIABLE_ID;
            CUR_AVAIABLE_ID += 1;
            next_id
        }
    }

    pub fn generate() -> Handle<T> {
        Handle::<T> {
            id: Self::next_valid_id(),
            phantom_data: PhantomData,
        }
    }
}

impl<T: Asset> PartialEq for Handle<T> {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl<T: Asset> Eq for Handle<T> {}

impl<T: Asset> Hash for Handle<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
        // self.phantom_data.hash(state);
    }
}

impl<T: Asset> Clone for Handle<T> {
    fn clone(&self) -> Self {
        Self {
            id: self.id,
            phantom_data: PhantomData,
        }
    }
}