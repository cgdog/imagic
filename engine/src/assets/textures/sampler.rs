use std::hash::{Hash, Hasher};

use ahash::AHasher;

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub enum SamplerTag {}
pub type SamplerHandle = crate::types::Handle<SamplerTag>;

pub type AddressMode = wgpu::AddressMode;
pub type FilterMode = wgpu::FilterMode;

pub struct Sampler {
    pub(crate) gpu_sampler: Option<wgpu::Sampler>,
    pub address_mode_u: AddressMode,
    pub address_mode_v: AddressMode,
    pub address_mode_w: AddressMode,
    pub mag_filter: FilterMode,
    pub min_filter: FilterMode,
    pub mipmap_filter: FilterMode,
    pub handle: SamplerHandle,
}

impl Sampler {
    pub(crate) fn compute_sampler_handle(
        address_mode_u: AddressMode,
        address_mode_v: AddressMode,
        address_mode_w: AddressMode,
        mag_filter: FilterMode,
        min_filter: FilterMode,
        mipmap_filter: FilterMode,
    ) -> SamplerHandle {
        let mut hasher = AHasher::default();
        address_mode_u.hash(&mut hasher);
        address_mode_v.hash(&mut hasher);
        address_mode_w.hash(&mut hasher);
        mag_filter.hash(&mut hasher);
        min_filter.hash(&mut hasher);
        mipmap_filter.hash(&mut hasher);
        let id = hasher.finish();
        SamplerHandle::new(id)
    }

    #[allow(unused)]
    pub(crate) fn new(
        address_mode_u: AddressMode,
        address_mode_v: AddressMode,
        address_mode_w: AddressMode,
        mag_filter: FilterMode,
        min_filter: FilterMode,
        mipmap_filter: FilterMode,
    ) -> Self {
        let handle = Self::compute_sampler_handle(address_mode_u, address_mode_v, address_mode_w, mag_filter, min_filter, mipmap_filter);
        Self {
            gpu_sampler: None,
            address_mode_u,
            address_mode_v,
            address_mode_w,
            mag_filter,
            min_filter,
            mipmap_filter,
            handle
        }
    }

    pub(crate) fn new_with_handle(
        address_mode_u: AddressMode,
        address_mode_v: AddressMode,
        address_mode_w: AddressMode,
        mag_filter: FilterMode,
        min_filter: FilterMode,
        mipmap_filter: FilterMode,
        handle: SamplerHandle,
    ) -> Self {
        Self {
            gpu_sampler: None,
            address_mode_u,
            address_mode_v,
            address_mode_w,
            mag_filter,
            min_filter,
            mipmap_filter,
            handle
        }
    }

    /// The default sampler which is linear filter and address mode of clamp to edge.
    /// If your material does not provide a sampler which is necessary, this default sampler will be used.
    pub fn default_sampler() -> SamplerHandle {
        static mut _SAMPLER_HANDLE: SamplerHandle = SamplerHandle::INVALID;
        if unsafe { _SAMPLER_HANDLE } == SamplerHandle::INVALID {
            // Note: here we just compute its handle. The init function of [`TextureSamplerManager`] will really create this sampler.
            let handle = Self::compute_sampler_handle(
                AddressMode::ClampToEdge,
                AddressMode::ClampToEdge,
                AddressMode::ClampToEdge,
                FilterMode::Linear,
                FilterMode::Linear,
                FilterMode::Linear,
            );
            unsafe { _SAMPLER_HANDLE = handle };
        }

        unsafe { _SAMPLER_HANDLE }
    }
}