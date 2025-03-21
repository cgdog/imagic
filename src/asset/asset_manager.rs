use std::{
    any::{Any, TypeId},
    collections::HashMap,
};

use super::asset::{Asset, Handle};

/// Asset manager.
/// 
/// All assets ,for example, Meshes, Materials, Textures, are stored in and managed by AssetManager.
pub struct AssetManager {
    asset_stores: HashMap<TypeId, Box<dyn AssetStore>>,
}

impl Default for AssetManager {
    fn default() -> Self {
        Self {
            asset_stores: HashMap::new(),
        }
    }
}

impl AssetManager {
    pub fn new() -> Self {
        Self {
            asset_stores: HashMap::new(),
        }
    }
    /// Add an asset.
    pub fn add<T: Asset>(&mut self, asset: T) -> Handle<T> {
        let type_id = TypeId::of::<T>();
        match self.asset_stores.get_mut(&type_id) {
            Some(asset_store) => {
                if let Some(concrete_asset_store) = asset_store
                    .as_any_mut()
                    .downcast_mut::<ConcreteAssetStore<T>>()
                {
                    return concrete_asset_store.add(asset);
                } else {
                    panic!("Failed to add asset {:?}", type_id);
                }
            }
            None => {
                let mut concrete_asset_store = ConcreteAssetStore::<T>::new();
                let handle = concrete_asset_store.add(asset);
                if let None = self
                    .asset_stores
                    .insert(type_id, Box::new(concrete_asset_store))
                {
                    panic!("Failed to create asset store");
                }
                return handle;
            }
        }
    }

    /// Get the asset reference.
    pub fn get<T: Asset>(&self, handle: &Handle<T>) -> Option<&T> {
        let type_id = TypeId::of::<T>();
        if let Some(asset_store) = self.asset_stores.get(&type_id) {
            if let Some(concrete_asset_store) = asset_store.as_any().downcast_ref::<ConcreteAssetStore<T>>() {
                concrete_asset_store.get(handle)
            } else {
                None
            }
        } else {
            None
        }
    }

    /// Get the asset mutable reference.
    pub fn get_mut<T: Asset>(&mut self, handle: &Handle<T>) -> Option<&mut T> {
        let type_id = TypeId::of::<T>();
        if let Some(asset_store) = self.asset_stores.get_mut(&type_id) {
            if let Some(concrete_asset_store) = asset_store.as_any_mut().downcast_mut::<ConcreteAssetStore<T>>() {
                concrete_asset_store.get_mut(handle)
            } else {
                None
            }
        } else {
            None
        }
    }
}

/// The asset store trait.
/// 
/// Every type of asset has its own asset tore to store all the assets with the same type.
trait AssetStore: Any {
    fn as_any(&self) -> &dyn Any;

    fn as_any_mut(&mut self) -> &mut dyn Any;
}

/// The concrete asset store.
struct ConcreteAssetStore<T: Asset> {
    assets: HashMap<Handle<T>, T>,
}

impl<T: Asset> ConcreteAssetStore<T> {
    fn new() -> Self {
        Self {
            assets: HashMap::new(),
        }
    }

    fn add(&mut self, asset: T) -> Handle<T> {
        let asset_id = Handle::<T>::generate();
        self.assets.insert(asset_id.clone(), asset);
        asset_id
    }

    fn get(&self, handel: &Handle<T>) -> Option<&T> {
        self.assets.get(handel)
    }

    fn get_mut(&mut self, handel: &Handle<T>) -> Option<&mut T> {
        self.assets.get_mut(handel)
    }
}

impl<T: Asset> AssetStore for ConcreteAssetStore<T> {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}
