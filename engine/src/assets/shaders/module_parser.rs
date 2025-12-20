use std::{hash::Hash, num::NonZero};

use ahash::AHasher;
use wgpu::{
    naga::{
        self, Expression, ImageClass, ImageDimension, Module, ResourceBinding, ScalarKind, ShaderStage, Type, TypeInner, UniqueArena, VectorSize
    }, ShaderStages, TextureViewDimension
};

use crate::assets::shaders::{shader::{BuilinUniformFlags, ShaderProperties}, shader_property::{
    BuiltinShaderUniformNames, ShaderProperty, ShaderPropertyBinding, ShaderPropertyType
}};

pub(crate) struct EntryStageBindings {
    pub(crate) stage: ShaderStages,
    pub(crate) bindings: Vec<ShaderPropertyBinding>,
}

pub(crate) fn retrieve_shader_entry_stage_bindings(
    shader_module: &mut Module,
) -> Vec<EntryStageBindings> {
    let mut entries = Vec::<EntryStageBindings>::new();
    for entry in shader_module.entry_points.iter() {
        let entry_stage = match &entry.stage {
            ShaderStage::Vertex => ShaderStages::VERTEX,
            ShaderStage::Fragment => ShaderStages::FRAGMENT,
            ShaderStage::Compute => ShaderStages::COMPUTE,
            _ => {
                unreachable!("not implemented.")
            }
        };

        let mut bindings = Vec::<ShaderPropertyBinding>::new();
        for (_, expression) in entry.function.expressions.iter() {
            // let area = module.global_expressions.get_mut(expression_handle);
            match expression {
                Expression::GlobalVariable(handle) => {
                    let g_v = shader_module.global_variables.get_mut(*handle);
                    match &g_v.binding {
                        Some(resource_binding) => {
                            bindings.push(ShaderPropertyBinding::new(
                                resource_binding.group,
                                resource_binding.binding,
                            ));
                        }
                        _ => {}
                    }
                }
                _ => {
                    // TODO: others
                }
            }
        }
        entries.push(EntryStageBindings {
            stage: entry_stage,
            bindings,
        });
    }
    entries
}

fn get_binding_stages(
    name: &str,
    entries_info: &Vec<EntryStageBindings>,
    resouce_binding: &ResourceBinding,
    type_inner: &TypeInner,
) -> ShaderStages {
    let mut stages = ShaderStages::empty();
    for entry in entries_info {
        for binding in &entry.bindings {
            if binding.group == resouce_binding.group && binding.binding == resouce_binding.binding {
                stages |= entry.stage;
            }
        }
    }
    if stages.is_empty() {
        // At preset, a uniform (e.g., _features) which is used in shader stage indirectly (for example, used in a function called by shader entry),
        // can not be recongized as used by this shader stage, so we manually set its stages to VERTEX_FRAGMENT.
        // Hopefully, this can be fixed in future naga version.
        if name == BuiltinShaderUniformNames::_MATERIAL_FEATURES{
            stages = ShaderStages::VERTEX_FRAGMENT;
        } else {
            // TODO: optimize default stages
            match type_inner {
                TypeInner::Image { dim: _, arrayed: _, class: _ } => {
                    stages = ShaderStages::FRAGMENT;
                },
                TypeInner::Sampler { comparison : _ } => {
                    stages = ShaderStages::FRAGMENT;
                },
                _ => {
                    stages = ShaderStages::VERTEX_FRAGMENT;
                }

            }
        }
    }
    stages
}

/// parse shader (naga) module and get all [`ShaderProperty`]s.
pub(crate) fn parse_shader_module(
    wgsl_source: &str,
    hasher: &mut AHasher,
) -> (ShaderProperties, u32, Module, BuilinUniformFlags) {
    let mut max_bind_group = 0;
    // material shader properties
    let mut shader_properties = ShaderProperties::new();
    let mut module: naga::Module = naga::front::wgsl::parse_str(wgsl_source).unwrap();
    let mut builtin_uniform_flags = BuilinUniformFlags::new();
    let entries_info = retrieve_shader_entry_stage_bindings(&mut module);
    for (_, global_var) in module.global_variables.iter() {
        // let global_var = module.global_variables.get_mut(global_variable_handle);
        let name = global_var.name.as_ref().unwrap().clone();
        let resouce_binding = global_var.binding.unwrap();
        let type_inner = &module.types.get_handle(global_var.ty).unwrap().inner;
        let stages = get_binding_stages(&name, &entries_info, &resouce_binding, type_inner);
        if cfg!(debug_assertions) {
            log::debug!(
                "global_var name: {}, space: {:?}, binding: {:?}, type inner: {:?}, init: {:?}, stages: {:?}",
                name,
                global_var.space,
                resouce_binding,
                // global_var.ty,
                type_inner,
                global_var.init,
                stages,
            );
        }

        max_bind_group = max_bind_group.max(resouce_binding.group);

        match type_inner {
            TypeInner::Scalar(_scalar) => {
                let shader_property = ShaderProperty::new(
                    name.clone(),
                    resouce_binding.into(),
                    ShaderPropertyType::Float(NonZero::new(4)),
                    stages,
                    // 4,
                );
                insert_shader_properties(name, shader_property, &mut shader_properties, hasher, &mut builtin_uniform_flags);
            }
            TypeInner::Vector { size, scalar } => {
                // let vector_component_size = scalar.width;
                // info!("vector_component_size: {}", vector_component_size);
                match (size, scalar.kind) {
                    (VectorSize::Quad, ScalarKind::Float) => {
                        // vec4
                        let shader_property = ShaderProperty::new(
                            name.clone(),
                            resouce_binding.into(),
                            ShaderPropertyType::Vec4(NonZero::new(16)),
                            stages,
                        );
                        insert_shader_properties(name, shader_property, &mut shader_properties, hasher, &mut builtin_uniform_flags);
                    }
                    (VectorSize::Quad, ScalarKind::Uint) => {
                        // vec4u
                        let shader_property = ShaderProperty::new(
                            name.clone(),
                            resouce_binding.into(),
                            ShaderPropertyType::UVec4(NonZero::new(16)),
                            stages,
                        );
                        insert_shader_properties(name, shader_property, &mut shader_properties, hasher, &mut builtin_uniform_flags);
                    }
                    (VectorSize::Quad, ScalarKind::Sint) => {
                        // vec4i
                        let shader_property = ShaderProperty::new(
                            name.clone(),
                            resouce_binding.into(),
                            ShaderPropertyType::IVec4(NonZero::new(16)),
                            stages,
                        );
                        insert_shader_properties(name, shader_property, &mut shader_properties, hasher, &mut builtin_uniform_flags);
                    }
                    _ => {
                        // TODO: other vectors
                    }
                }
            }
            TypeInner::Matrix {
                columns,
                rows,
                scalar,
            } => {
                match (columns, rows, scalar.kind) {
                    (VectorSize::Quad, VectorSize::Quad, ScalarKind::Float) => {
                        // matrix4x4f
                        let shader_property = ShaderProperty::new(
                            name.clone(),
                            resouce_binding.into(),
                            ShaderPropertyType::Matrix4x4(NonZero::new(64)),
                            stages,
                        );
                        insert_shader_properties(name, shader_property, &mut shader_properties, hasher, &mut builtin_uniform_flags);
                    }
                    _ => {
                        // TODO: implement others, e.g., matrix3x3, matrix2x3.
                    }
                }
            }
            TypeInner::Struct {
                members: _,
                span: _,
            } => {
                let struct_size = _calculate_type_size(&module.types, type_inner);
                if cfg!(debug_assertions) {
                    log::debug!("size of struct {}: {}", name, struct_size);
                }
                let shader_property = ShaderProperty::new(
                    name.clone(),
                    resouce_binding.into(),
                    ShaderPropertyType::Struct(NonZero::new(struct_size as u64)),
                    stages,
                );
                insert_shader_properties(name, shader_property, &mut shader_properties, hasher, &mut builtin_uniform_flags);
            }
            TypeInner::Image { dim, arrayed, class } => {
                let shader_property = construct_image_shader_property(dim, name.clone(), resouce_binding, stages, class, arrayed);
                insert_shader_properties(name, shader_property, &mut shader_properties, hasher, &mut builtin_uniform_flags);
            }
            TypeInner::Sampler { comparison } => {
                let shader_property = ShaderProperty::new(
                    name.clone(),
                    resouce_binding.into(),
                    ShaderPropertyType::Sampler(*comparison),
                    stages,
                );
                insert_shader_properties(name, shader_property, &mut shader_properties, hasher, &mut builtin_uniform_flags);
            }
            _ => {
                // TODO: other types
            }
        }
    }
    (shader_properties, max_bind_group, module, builtin_uniform_flags)
}

fn insert_shader_properties(name: String, shader_property: ShaderProperty, shader_properties: &mut ShaderProperties, hasher: &mut AHasher, builtin_uniform_flags: &mut BuilinUniformFlags) {
    shader_property.hash(hasher);
    if BuiltinShaderUniformNames::is_per_object_uniform(&name, builtin_uniform_flags) {
        shader_properties.per_object_properties.insert(name, shader_property);
    } else if BuiltinShaderUniformNames::is_per_camera_uniform(&name, builtin_uniform_flags) {
        shader_properties.per_camera_properties.insert(name, shader_property);
    } else if BuiltinShaderUniformNames::is_per_scene_uniform(&name, builtin_uniform_flags) {
        shader_properties.per_scene_properties.insert(name, shader_property);
    } else {
        shader_properties.per_material_properties.insert(name, shader_property);
    }
}

fn get_sample_type(kind: &ScalarKind) -> wgpu::TextureSampleType{
    match kind {
        ScalarKind::Sint => {
            wgpu::TextureSampleType::Sint
        },
        ScalarKind::Uint => {
            wgpu::TextureSampleType::Uint
        },
        ScalarKind::Float => {
            wgpu::TextureSampleType::Float { filterable: true }
        },
        ScalarKind::Bool => unimplemented!(),
        ScalarKind::AbstractInt => unimplemented!(),
        ScalarKind::AbstractFloat => unimplemented!(),
    }
}

fn get_view_dimension(dim: &ImageDimension, is_array: bool) -> TextureViewDimension{
    match dim {
        ImageDimension::D1 => {
            TextureViewDimension::D1
        },
        ImageDimension::D2 => {
            if is_array {
                TextureViewDimension::D2Array
            } else {
                TextureViewDimension::D2
            }
        },
        ImageDimension::D3 => {
            TextureViewDimension::D3
        },
        ImageDimension::Cube => {
            if is_array {
                TextureViewDimension::CubeArray
            } else {
                TextureViewDimension::Cube
            }
        },
    }
}

fn construct_image_shader_property(
    dim: &ImageDimension,
    name: String,
    resouce_binding: ResourceBinding,
    stages: ShaderStages,
    image_class: &ImageClass,
    arrayed: &bool,
) -> ShaderProperty {
    let view_dimension = get_view_dimension(dim, *arrayed);
    match image_class {
        ImageClass::Sampled{kind, multi} => {
            let sample_type = self::get_sample_type(kind);
            let shader_property = ShaderProperty::new(
                name,
                resouce_binding.into(),
                ShaderPropertyType::Image(view_dimension, sample_type, *multi),
                stages,
            );
            shader_property
        }
        _ => {unimplemented!()}
    }
}

fn _calculate_type_size(module_types: &UniqueArena<Type>, type_inner: &TypeInner) -> u32 {
    match type_inner {
        TypeInner::Scalar(scalar) => scalar.width as u32,
        TypeInner::Vector { size, scalar } => match (size, scalar.kind) {
            (VectorSize::Quad, ScalarKind::Float) => 4 * 4,
            (VectorSize::Quad, ScalarKind::Uint) => 4 * 4,
            (VectorSize::Quad, ScalarKind::Sint) => 4 * 4,
            (VectorSize::Tri, ScalarKind::Float) => 3 * 4,
            (VectorSize::Tri, ScalarKind::Uint) => 3 * 4,
            (VectorSize::Tri, ScalarKind::Sint) => 3 * 4,
            (VectorSize::Bi, ScalarKind::Float) => 2 * 4,
            (VectorSize::Bi, ScalarKind::Uint) => 2 * 4,
            (VectorSize::Bi, ScalarKind::Sint) => 2 * 4,
            _ => {
                unimplemented!()
            }
        },
        TypeInner::Matrix {
            columns,
            rows,
            scalar,
        } => (*columns as u32) * (*rows as u32) * (scalar.width as u32),
        TypeInner::Array {
            base,
            size,
            stride: _,
        } => {
            let base_type_innser = &module_types.get_handle(*base).unwrap().inner;
            let elemtn_size = _calculate_type_size(module_types, base_type_innser);
            // elemtn_size * size
            match size {
                naga::ArraySize::Constant(non_zero) => elemtn_size * non_zero.get(),
                naga::ArraySize::Pending(_handle) => unimplemented!(),
                naga::ArraySize::Dynamic => unimplemented!(),
            }
        }
        TypeInner::Struct { members, span: _ } => {
            let mut size = 0;
            for member in members {
                let member_type_inner = &module_types.get_handle(member.ty).unwrap().inner;
                size += _calculate_type_size(module_types, member_type_inner);
            }
            size
        }
        _ => {
            unimplemented!()
        }
    }
}
