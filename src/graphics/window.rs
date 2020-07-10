
use luminance_glfw::{Action, GlfwSurface, Key, Surface, WindowDim, WindowEvent, WindowOpt};
use glfw;

pub fn new_window(width: f64, height: f64) -> GlfwSurface{
    let mut surface = GlfwSurface::new(
        WindowDim::Windowed(1920, 1080),
        //WindowDim::Fullscreen,
        "Title",
        WindowOpt::default().set_num_samples(2),
    )
    .expect("GLFW surface creation");
    surface
}