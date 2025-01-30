use crate::prelude::ImagicContext;

/// Compute shader template.
/// 
/// At preset, you have to call these methods by your self.
pub trait ComputeShader {
    /// Create shader module, bind group layout, bind group, create pipeline, dispatch and so on.
    fn execute(&mut self, imagic_context: &mut ImagicContext);
}