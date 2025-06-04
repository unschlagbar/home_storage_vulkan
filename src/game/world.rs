use std::{cell::RefCell, rc::Rc};
use cgmath::Matrix4;
use iron_oxide::{graphics::SinlgeTimeCommands, physics::System, primitives::Vec3, ui::UiState};
use crate::graphics::VulkanRender;
use super::{Camera, Cube};

#[repr(C)]
#[derive()]
pub struct World {
    pub renderer: Rc<RefCell<VulkanRender>>,
    pub camera: Camera,
    pub movement_vector: Vec3,
    pub ui: Rc<RefCell<UiState>>,
    pub system: System,
    pub cubes: Vec<Cube>,
}

impl World {
    pub fn create(renderer: Rc<RefCell<VulkanRender>>, ui: Rc<RefCell<UiState>>) -> Self {
        let mut cube = Cube::new(Vec3::new(0.0, 0.0, 0.0), Vec3::new(1.0, 1.0, 1.0), true);
        cube.rigit_body.velocity.y = 10.0;
        let mut plane = Cube::new(Vec3::new(0.0, -1.0, 0.0), Vec3::new(2.0, 0.1, 2.0), false);
        let mut plane2 = Cube::new(Vec3::new(0.0, 1.0, 0.0), Vec3::new(2.0, 0.1, 2.0), true);
        plane.rigit_body.position_lock = Vec3::zero();
        plane.rigit_body.mass = 1000000.0;
        plane2.rigit_body.mass = 2.0;
        let system = System::new();

        let cubes =  vec![plane, cube, plane2];

        Self {
            camera: Camera::default(),
            movement_vector: Vec3::default(),
            ui,
            system,
            cubes,
            renderer,
        }
    }

    pub fn get_instances(&self) -> Vec<Matrix4<f32>> {
        let mut instances = Vec::with_capacity(self.cubes.len());
        
        for cube in &self.cubes {
            instances.push(cube.get_instance());
        }

        instances
    }

    pub fn update(&mut self, delta_time: f32, renderer: &mut VulkanRender) {
        if delta_time > 0.1 {
            return;
        }
        self.system.update(&mut self.cubes, delta_time);
        
        if self.movement_vector != Vec3::zero() {
            self.camera.process_movement(self.movement_vector, 0.5);
        }

        let instances = self.get_instances();

        let buffer_size = size_of::<Matrix4<f32>>() as u64 * instances.len() as u64;

        let mapped_memory = renderer.instance_staging_buffer.map_memory(&renderer.base.device, buffer_size, 0);
        unsafe {
            std::ptr::copy_nonoverlapping(instances.as_ptr() as *const u8, mapped_memory as _, buffer_size as usize);
            renderer.instance_staging_buffer.unmap_memory(&renderer.base.device);
        };

        let cmd_buf = SinlgeTimeCommands::begin(&renderer.base, &renderer.command_pool);
        renderer.instance_staging_buffer.copy(&renderer.instance_buffer, &renderer.base, buffer_size, cmd_buf);
        SinlgeTimeCommands::end(&renderer.base, &renderer.command_pool, cmd_buf);
    }

}