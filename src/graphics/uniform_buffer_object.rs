#![allow(unused)]

use cgmath::Matrix4;

#[repr(align(16))]
pub struct UniformBufferObject {
    pub view_proj: Matrix4<f32>,
}

#[repr(align(16))]
pub struct UiUniformBufferObject {
    pub view_proj: Matrix4<f32>,
} 