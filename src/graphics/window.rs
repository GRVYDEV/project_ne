use glutin::event_loop::EventLoop;
use luminance_glutin::GlutinError;
use glutin::window::WindowBuilder;
use luminance_glutin::GlutinSurface;
use glutin::dpi::LogicalSize;
use glutin::dpi::Size;

pub fn new_window(width: f64, height: f64) -> Result<(GlutinSurface, EventLoop<()>), GlutinError>{
   let window = WindowBuilder::new().with_inner_size(Size::from(LogicalSize::new(width, height)));
   GlutinSurface::new(window, 2)
}