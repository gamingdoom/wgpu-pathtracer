use glam::Vec3;
use winit::dpi::PhysicalSize;

use crate::shaders::shader_definitions;

pub struct Camera {
    pub width: u32,
    pub height: u32,

    pub fov: f32,

    pub position: Vec3,
    pub lookat: Vec3,
    
    pub frame: u32,

    pub samples_per_pixel: u32,
    pub max_bounces: u32,

    pub pixel_space_x: Vec3,
    pub pixel_space_y: Vec3,
    pub first_pixel_pos: Vec3,

    pub theta_x: f32,
    pub theta_y: f32,
    pub camera_speed: f32

}

impl Camera {
    pub fn new(size: PhysicalSize<u32>, fov: f32, position: Vec3) -> Self {
        let mut cam = Self {
            width: size.width,
            height: size.height,
            fov,
            position,
            lookat: position + Vec3::new(0.0, 0.0, 1.0),
            samples_per_pixel: shader_definitions::SAMPLES_PER_PIXEL,
            max_bounces: shader_definitions::MAX_BOUNCES,
            pixel_space_x: Vec3::new(0.0, 0.0, 0.0),
            pixel_space_y: Vec3::new(0.0, 0.0, 0.0),
            first_pixel_pos: Vec3::new(0.0, 0.0, 0.0),
            theta_x: 0.0,
            theta_y: 0.0,
            camera_speed: 10.0,
            frame: 0
        };

        cam.update();

        cam
    }

    pub fn update(&mut self) {
        let viewport_height = 2.0 * (self.fov * (std::f32::consts::PI / 180.0) / 2.0).tan();
        let viewport_width = viewport_height * ((self.width as f32)/(self.height as f32));
    
        let viewport_z = (self.position - self.lookat).normalize();
        let viewport_x = Vec3::new(0.0, 1.0, 0.0).cross(viewport_z).normalize();
        let viewport_y = viewport_z.cross(viewport_x);
    
        let viewport_x_vec = viewport_x * viewport_width;
        let viewport_y_vec = -viewport_y * viewport_height;
    
        self.pixel_space_x = viewport_x_vec / (self.width as f32);
        self.pixel_space_y = viewport_y_vec / (self.height as f32);
    
        let viewport_top_left = self.position - viewport_z - viewport_x_vec/2.0 - viewport_y_vec/2.0;
    
        self.first_pixel_pos = viewport_top_left + 0.5 * (self.pixel_space_x + self.pixel_space_y);

        self.frame += 1;

        self.lookat[0] = self.position[0] + self.theta_x.cos() * self.theta_y.cos();
        self.lookat[1] = self.position[1] + self.theta_y.sin();
        self.lookat[2] = self.position[2] + self.theta_x.sin() * self.theta_y.cos();

        // println!("{} {} {} {} {}", self.lookat[0], self.lookat[1], self.lookat[2], self.theta_x, self.theta_y);
    }
}