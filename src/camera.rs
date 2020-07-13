use crate::graphics::orthographic_projection_matrix;
use nalgebra::{Matrix4, Vector3, Vector2};
pub struct Camera {
    position: Vector2<f32>,
    width: u32,
    height: u32,
}

impl Camera {
    pub fn new(width: f32, height: f32) -> Self{
        Camera{
            width: width as u32,
            height: height as u32,
            position: Vector2::new(0.0, 0.0)
        }
    }
    pub fn set_position(&mut self, pos: Vector2<f32>) {
        self.position = pos;
    }
    pub fn set_size(&mut self, width: u32, height: u32){
        self.width = width;
        self.height = height;
    }
    pub fn translate(&mut self, x: f32, y:f32){
        self.position += Vector2::new(x, y);
    }
    pub fn as_matrix(&self) -> [f32; 16] {
        let mut matrix = orthographic_projection_matrix(self.width, self.height);
        let transform_matrix = Matrix4::new_translation(&Vector3::new(self.position.x, self.position.y, 0.0));

        let mut final_matrix: [f32; 16] = [0.0; 16];
        final_matrix.copy_from_slice((matrix * transform_matrix).as_slice());
        final_matrix
    }
}
