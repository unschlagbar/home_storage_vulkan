use std::mem::offset_of;

use cgmath::{Matrix4, Vector2, Vector3};
use ash::vk;


#[derive(Debug, Clone, PartialEq)]
#[repr(C)]
pub struct Vertex {
    pub pos: Vector3<f32>,
    pub nrm: Vector3<f32>,
    pub uv: Vector2<f32>,
    pub padding: f32,
}

impl Vertex {
    pub const GET_BINDING_DESCRIPTION: [vk::VertexInputBindingDescription; 2] = [
        vk::VertexInputBindingDescription {
            binding: 0,
            stride: std::mem::size_of::<Vertex>() as _,
            input_rate: vk::VertexInputRate::VERTEX,
        },
        vk::VertexInputBindingDescription {
            binding: 1,
            stride: std::mem::size_of::<Matrix4<f32>>() as _,
            input_rate: vk::VertexInputRate::INSTANCE,
        }
    ];

    pub const GET_ATTRIBUTE_DESCRIPTIONS: [vk::VertexInputAttributeDescription; 8] = [
        vk::VertexInputAttributeDescription {
            binding: 0,
            location: 0,
            format: vk::Format::R32G32B32_SFLOAT,
            offset: offset_of!(Vertex, pos) as _,
        },
        vk::VertexInputAttributeDescription {
            binding: 0,
            location: 1,
            format: vk::Format::R32G32B32_SFLOAT,
            offset: offset_of!(Vertex, nrm) as _,
        },
        vk::VertexInputAttributeDescription {
            binding: 0,
            location: 2,
            format: vk::Format::R32G32_SFLOAT,
            offset: offset_of!(Vertex, uv) as _,
        },
        vk::VertexInputAttributeDescription {
            binding: 0,
            location: 3,
            format: vk::Format::R32_SFLOAT,
            offset: offset_of!(Vertex, padding) as _,
        },
        //Transformation Matrix
        vk::VertexInputAttributeDescription {
            binding: 1,
            location: 4,
            format: vk::Format::R32G32B32A32_SFLOAT,
            offset: 0,
        },
        vk::VertexInputAttributeDescription {
            binding: 1,
            location: 5,
            format: vk::Format::R32G32B32A32_SFLOAT,
            offset: 16,
        },
        vk::VertexInputAttributeDescription {
            binding: 1,
            location: 6,
            format: vk::Format::R32G32B32A32_SFLOAT,
            offset: 32,
        },
        vk::VertexInputAttributeDescription {
            binding: 1,
            location: 7,
            format: vk::Format::R32G32B32A32_SFLOAT,
            offset: 48,
        },
    ];
}

impl std::hash::Hash for Vertex {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.pos.x.to_bits().hash(state);
        self.pos.y.to_bits().hash(state);
        self.pos.z.to_bits().hash(state);
    }
}

impl Eq for Vertex {}