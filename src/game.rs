use crate::graphics::new_window;

use luminance::context::GraphicsContext;
use luminance::framebuffer::Framebuffer;
use luminance::texture::Dim2;
use luminance_glfw::{Action, GlfwSurface, Key, Surface, WindowDim, WindowEvent, WindowOpt};
use std::{collections::HashSet, time::Duration};
pub enum GameEvent {
    WindowEvent(WindowEvent),
}

pub trait Game {
    fn new<C>(context: &mut C) -> Self
    where
        C: GraphicsContext;

    fn update(&mut self, key_buffer: &HashSet<Key>);

    fn draw<C>(
        &mut self,
        context: &mut C,
        delta_time: Duration,
        frame_buffer: &Framebuffer<Dim2, (), ()>,
    ) where
        C: GraphicsContext;

    fn process_event(&mut self, event: GameEvent);
}

pub fn run<G: 'static>()
where
    G: Game,
{
    let mut surface = new_window(1600.0, 900.0);
    let mut game = G::new(&mut surface);
    let mut last_frame = std::time::Instant::now();
    let mut resize = false;
    let mut time_buffer = Duration::new(0, 0);
    let mut framebuffer = surface.back_buffer().unwrap();
    let mut keybuffer: HashSet<glfw::Key> = HashSet::new();
    'app: loop {
        let delta_time = std::time::Instant::now().duration_since(last_frame);

        last_frame = std::time::Instant::now();
        for event in surface.poll_events() {
            match event {
                // If we close the window or press escape, quit the main loop (i.e. quit the application).
                WindowEvent::Close | WindowEvent::Key(Key::Escape, _, Action::Release, _) => {
                    break 'app
                }
                WindowEvent::FramebufferSize(width, height) => {
                    game.process_event(GameEvent::WindowEvent(WindowEvent::FramebufferSize(
                        width, height,
                    )));
                    resize = true;
                }
                WindowEvent::Key(key, _, action, _) => {
                    if action == Action::Press {
                        keybuffer.insert(key);
                    } else if action == Action::Release {
                        keybuffer.remove(&key);
                    }
                }

                _ => {}
            }
        }
        if resize {
            framebuffer = surface.back_buffer().unwrap();
            surface.back_buffer().unwrap();
            resize = false;
        }
        game.draw(&mut surface, delta_time, &framebuffer);
        surface.swap_buffers();

        time_buffer += delta_time;

        if time_buffer > Duration::from_secs_f64(16.0 / 1000.0) {
            time_buffer -= Duration::from_secs_f64(16.0 / 1000.0);
            game.update(&keybuffer);
        }
    }
    // event_loop.run(move |event, target, control_flow| {
    //     let delta_time = std::time::Instant::now().duration_since(last_frame);

    //     last_frame = std::time::Instant::now();

    //     match event {
    //         Event::LoopDestroyed => return,
    //         Event::WindowEvent { event, .. } => match event {
    //             WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
    //             WindowEvent::Resized(size) => {
    //                 game.process_event(GameEvent::WindowEvent(WindowEvent::Resized(size)));
    //                 resize = true;
    //                 println!("Window Event {:?}", size);
    //             }
    //             e => {
    //                 if let Some(event) = e.to_static() {
    //                     game.process_event(GameEvent::WindowEvent(event));
    //                 }
    //             }
    //         },
    //         Event::RedrawRequested(_) => {
    //             if resize {
    //                 framebuffer = surface.back_buffer().unwrap();
    //                 surface.back_buffer().unwrap();
    //                 resize = false;
    //             }
    //             game.draw(&mut surface, delta_time, &framebuffer);
    //             surface.swap_buffers();
    //             surface.ctx.window().request_redraw();
    //         }
    //         _ => (),
    //     }

    //     time_buffer += delta_time;

    //     if time_buffer > Duration::from_secs_f64(0.16) {
    //         time_buffer -= Duration::from_secs_f64(0.16);
    //         game.update();
    //     }
    // });
}
