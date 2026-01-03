use crate::core::{LogicContext, NodeHandle};

/// Trait for loading models from various formats, for example, .gltf, .glb, .fbx.
pub trait ModelLoaderTrait {
    /// Load a model from the specified path.
    /// Returns a Result containing the loaded Node or an error message.
    /// # Errors
    /// Returns an error if the model file cannot be opened or processed.
    /// # Arguments
    /// * `logic_context` - The logic context to use for loading the model.
    /// * `path` - The path to the model file.
    /// # Returns
    /// A Result containing the loaded Node or an error message.
    fn load(&self, logic_context: &mut LogicContext<'_>, path: &str) -> Result<NodeHandle, Box<dyn std::error::Error>>;
}

/// A model loader that can load models from different formats.
/// Currently, it supports GLTF format.
pub struct ModelLoader {
}

impl ModelLoaderTrait for ModelLoader {
    fn load(&self, logic_context: &mut LogicContext<'_>, path: &str) -> Result<NodeHandle, Box<dyn std::error::Error>> {
        if path.ends_with(".gltf") || path.ends_with(".glb") {
            let gltf_loader = crate::assets::loaders::gltf_loader::GLTFLoader::new();
            return gltf_loader.load(logic_context, path);
        }
        Err("ModelLoaderTrait not implemented".to_string().into())
    }
}

impl ModelLoader {
    /// Create a new ModelLoader instance.
    pub fn new() -> Self {
        ModelLoader {}
    }
}