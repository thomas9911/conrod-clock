//! This example behaves the same as the `all_winit_glium` example while demonstrating how to run
//! the `conrod` loop on a separate thread.
// #![windows_subsystem = "windows"]

#[macro_use]
extern crate conrod_core;
extern crate conrod_glium;
extern crate conrod_winit;
extern crate find_folder;
extern crate glium;
extern crate image;
extern crate nvml_wrapper;

use conrod_glium::Renderer;
use glium::backend::glutin;
use glium::Surface;

mod conrod_thread;
mod gui;
mod my_widgets;

use conrod_thread::run_conrod;

const WIN_W: u32 = 720;
const WIN_H: u32 = 1080;

pub struct GliumDisplayWinitWrapper(pub glutin::Display);

impl conrod_winit::WinitWindow for GliumDisplayWinitWrapper {
    fn get_inner_size(&self) -> Option<(u32, u32)> {
        self.0.gl_window().get_inner_size().map(Into::into)
    }
    fn hidpi_factor(&self) -> f32 {
        self.0.gl_window().get_hidpi_factor() as _
    }
}

fn main() {
    // Build the window.
    let mut events_loop = glium::glutin::EventsLoop::new();
    let window = glium::glutin::WindowBuilder::new()
        .with_title("Klok in Rust")
        .with_dimensions((WIN_W, WIN_H).into());
    let context = glium::glutin::ContextBuilder::new()
        .with_vsync(true)
        .with_multisampling(4);
    let display = glium::Display::new(window, context, &events_loop).unwrap();
    let display = GliumDisplayWinitWrapper(display);

    const POLL_RATE: std::time::Duration = std::time::Duration::from_millis(250);

    // A type used for converting `conrod_core::render::Primitives` into `Command`s that can be used
    // for drawing to the glium `Surface`.
    //
    // Internally, the `Renderer` maintains:
    // - a `backend::glium::GlyphCache` for caching text onto a `glium::texture::Texture2d`.
    // - a `glium::Program` to use as the shader program when drawing to the `glium::Surface`.
    // - a `Vec` for collecting `backend::glium::Vertex`s generated when translating the
    // `conrod_core::render::Primitive`s.
    // - a `Vec` of commands that describe how to draw the vertices.
    let mut renderer = Renderer::new(&display.0).unwrap();

    let image_map = conrod_core::image::Map::new();

    // A channel to send events from the main `winit` thread to the conrod thread.
    let (event_tx, event_rx) = std::sync::mpsc::channel();
    // A channel to send `render::Primitive`s from the conrod thread to the `winit thread.
    let (render_tx, render_rx) = std::sync::mpsc::channel();
    // Clone the handle to the events loop so that we can interrupt it when ready to draw.
    let events_loop_proxy = events_loop.create_proxy();

    // Spawn the conrod loop on its own thread.
    std::thread::spawn(move || run_conrod(event_rx, render_tx, events_loop_proxy));

    // Run the `winit` loop.
    // let mut last_update = std::time::Instant::now();
    let mut closed = false;
    let mut prev = std::time::Instant::now();
    let mut last_update = std::time::Instant::now();
    let mut now;
    let mut d = DoubleClicker::default();

    while !closed {
        // send every half second update time event
        now = std::time::Instant::now();
        if now.duration_since(prev) > std::time::Duration::from_millis(500) {
            event_tx.send(conrod_core::event::Input::Redraw).unwrap();
            prev = now;
        }

        // events_loop.run_forever(|event| {
        events_loop.poll_events(|event| {
            let do_fullscreen = || {
                let gl_window = display.0.gl_window();
                let window = gl_window.window();
                if window.get_fullscreen().is_none() {
                    window.set_fullscreen(Some(window.get_current_monitor()));
                } else {
                    window.set_fullscreen(None);
                }
            };

            match d.update(&event) {
                Some(ClickEvent::DoubleClick) => do_fullscreen(),
                Some(ClickEvent::Click) => (),
                None => ()
            }

            // Use the `winit` backend feature to convert the winit event to a conrod one.
            if let Some(event) = convert_event(event.clone(), &display) {
                event_tx.send(event).unwrap();
            }

            match event {
                glium::glutin::Event::WindowEvent { event, .. } => match event {
                    // Break from the loop upon `Escape`.
                    glium::glutin::WindowEvent::CloseRequested
                    | glium::glutin::WindowEvent::KeyboardInput {
                        input:
                            glium::glutin::KeyboardInput {
                                virtual_keycode: Some(glium::glutin::VirtualKeyCode::Escape),
                                ..
                            },
                        ..
                    } => {
                        closed = true;
                        // return glium::glutin::ControlFlow::Break;
                        return;
                    }
                    // We must re-draw on `Resized`, as the event loops become blocked during
                    // resize on macOS.
                    glium::glutin::WindowEvent::Resized(..) => {
                        if let Some(primitives) = render_rx.iter().next() {
                            draw(&display.0, &mut renderer, &image_map, &primitives);
                        }
                    }
                    glium::glutin::WindowEvent::KeyboardInput {
                        input:
                            glium::glutin::KeyboardInput {
                                virtual_keycode: Some(glium::glutin::VirtualKeyCode::F),
                                state: glium::glutin::ElementState::Released,
                                ..
                            },
                        ..
                    } => do_fullscreen(),
                    _ => {}
                },
                // glium::glutin::Event::Awakened => return glium::glutin::ControlFlow::Break,
                glium::glutin::Event::Awakened => return,
                _ => (),
            }

            // glium::glutin::ControlFlow::Continue
        });

        // Draw the most recently received `conrod_core::render::Primitives` sent from the `Ui`.
        if let Some(primitives) = render_rx.try_iter().last() {
            draw(&display.0, &mut renderer, &image_map, &primitives);
        }

        // let sixteen_ms = std::time::Duration::from_millis(25);
        let duration_since_last_update = now.duration_since(last_update);
        if duration_since_last_update < POLL_RATE {
            std::thread::sleep(POLL_RATE - duration_since_last_update);
        }

        last_update = std::time::Instant::now();
    }
}

conrod_winit::conversion_fns!();

// Draws the given `primitives` to the given `Display`.
fn draw(
    display: &glium::Display,
    renderer: &mut Renderer,
    image_map: &conrod_core::image::Map<glium::Texture2d>,
    primitives: &conrod_core::render::OwnedPrimitives,
) {
    renderer.fill(display, primitives.walk(), &image_map);
    let mut target = display.draw();
    target.clear_color(0.0, 0.0, 0.0, 1.0);
    renderer.draw(display, &mut target, &image_map).unwrap();
    target.finish().unwrap();
}


#[derive(Debug, Copy, Clone)]
struct DoubleClicker {
    between_pressed_released: std::time::Duration,
    between_clicks: std::time::Duration,
    prev_click: std::time::Instant,
    prev_pressed: std::time::Instant,
    prev_released: std::time::Instant,
}

impl Default for DoubleClicker {
    fn default() -> Self {
        DoubleClicker {
            between_pressed_released: std::time::Duration::from_millis(115),
            between_clicks: std::time::Duration::from_millis(500),
            prev_click: std::time::Instant::now(),
            prev_pressed: std::time::Instant::now()
                .checked_sub(std::time::Duration::from_millis(50))
                .unwrap(),
            prev_released: std::time::Instant::now(),
        }
    }
}

enum ClickEvent {
    DoubleClick,
    Click,
}

impl DoubleClicker {
    pub fn update(&mut self, event: &glium::glutin::Event) -> Option<ClickEvent> {
        if self.handle_event(event).is_some() {
            if self.is_click() {
                let event = match self.is_double_click() {
                    true => ClickEvent::DoubleClick,
                    false => ClickEvent::Click,
                };
                self.prev_click = std::time::Instant::now();
                return Some(event);
            };
        };
        None
    }

    pub fn handle_event(&mut self, event: &glium::glutin::Event) -> Option<()> {
        match &event {
            glium::glutin::Event::WindowEvent {
                event: window_event,
                ..
            } => match window_event {
                glium::glutin::WindowEvent::MouseInput {
                    state: mouse_state,
                    button,
                    ..
                } => match (button, mouse_state) {
                    (glium::glutin::MouseButton::Left, glium::glutin::ElementState::Pressed) => {
                        self.prev_pressed = std::time::Instant::now();
                        return Some(());
                    }
                    (glium::glutin::MouseButton::Left, glium::glutin::ElementState::Released) => {
                        self.prev_released = std::time::Instant::now();
                        return Some(());
                    }
                    (_, _) => (),
                },
                _ => (),
            },
            _ => (),
        };
        None
    }

    fn is_click(&self) -> bool {
        if self.prev_pressed < self.prev_released {
            self.prev_pressed + self.between_pressed_released > self.prev_released
        } else {
            false
        }
    }

    fn is_double_click(&self) -> bool {
        self.prev_click + self.between_clicks > std::time::Instant::now()
    }
}
