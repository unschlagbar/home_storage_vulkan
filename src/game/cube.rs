use crate::graphics::Vertex;
use cgmath::{vec3, Matrix4, Vector2};
use iron_oxide::{physics::{Collision, ImplRigitBody, RigitBody}, primitives::Vec3};
#[derive(Debug)]
pub struct Cube {
    pub rigit_body: RigitBody,
}

impl Cube {
    pub const fn new(position: Vec3, size: Vec3, gravity:bool) -> Self {
        Self {
            rigit_body: RigitBody {
                on_ground: false,
                gravity,
                position,
                velocity: Vec3::zero(),
                mass: 1.,
                size,
                position_lock: Vec3::one(),
            },
        }
    }

    pub fn _apply_force(&mut self, force: Vec3) {
        self.rigit_body.velocity += force / self.rigit_body.mass;
    }

    pub fn get_instance(&self) -> Matrix4<f32> {
        Matrix4::from_translation(vec3(self.rigit_body.position.x, self.rigit_body.position.y, self.rigit_body.position.z))
        * Matrix4::from_nonuniform_scale(self.rigit_body.size.x, self.rigit_body.size.y, self.rigit_body.size.z)
    }

    pub fn generate_vertices() -> (Vec<Vertex>, Vec<u32>) {
        let half_size = Vec3::new(0.5, 0.5, 0.5);
        let min = -half_size;
        let max = half_size;

        // Define the 8 corners of the cube
        let corners = [
            Vec3::new(min.x, min.y, min.z), // 0
            Vec3::new(max.x, min.y, min.z), // 1
            Vec3::new(max.x, max.y, min.z), // 2
            Vec3::new(min.x, max.y, min.z), // 3
            Vec3::new(min.x, min.y, max.z), // 4
            Vec3::new(max.x, min.y, max.z), // 5
            Vec3::new(max.x, max.y, max.z), // 6
            Vec3::new(min.x, max.y, max.z), // 7
        ];

        // Define the indices for each face (two triangles per face)
        let face_indices: [[u32; 6]; 6] = [
            // Front face
            [0, 1, 2, 2, 3, 0],
            // Back face
            [4, 5, 6, 6, 7, 4],
            // Left face
            [0, 3, 7, 7, 4, 0],
            // Right face
            [1, 2, 6, 6, 5, 1],
            // Top face
            [3, 2, 6, 6, 7, 3],
            // Bottom face
            [0, 1, 5, 5, 4, 0],
        ];

        // Normals for each face
        let normals = [
            Vec3::new(0.0, 0.0, -1.0), // Front
            Vec3::new(0.0, 0.0, 1.0),  // Back
            Vec3::new(-1.0, 0.0, 0.0), // Left
            Vec3::new(1.0, 0.0, 0.0),  // Right
            Vec3::new(0.0, 1.0, 0.0),  // Top
            Vec3::new(0.0, -1.0, 0.0), // Bottom
        ];

        let mut vertices = Vec::with_capacity(8);
        let mut indices = Vec::with_capacity(36);

        for (face, indice) in face_indices.iter().enumerate() {
            for &index in indice {
                let position = corners[index as usize];
                let normal = normals[face];
                let uv = Vector2::new(
                    (position.x - min.x) / 1.0, // Simple UV mapping
                    (position.y - min.y) / 1.0,
                );

                vertices.push(Vertex {
                    pos: position.into(),
                    nrm: normal.into(),
                    uv,
                    padding: 0.0,
                });
            }

            let v_len = vertices.len() as u32;
            indices.extend_from_slice(&[v_len - 6, v_len - 5, v_len - 4, v_len - 3, v_len - 2, v_len - 1]);
        }

        (vertices, indices)
    }

}


impl ImplRigitBody for Cube {
    fn velocity(&mut self) -> &mut Vec3 {
        &mut self.rigit_body.velocity
    }

    fn position(&mut self) -> &mut Vec3 {
        &mut self.rigit_body.position
    }

    fn collision(&mut self) -> Collision {
        Collision::Cube { center: self.rigit_body.position, size: self.rigit_body.size }
    }

    fn rigit_body(&mut self) -> &mut RigitBody {
        &mut self.rigit_body
    }
}