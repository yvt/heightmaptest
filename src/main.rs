#![warn(rust_2018_idioms)]

use cgmath::{prelude::*, vec2, vec3, Matrix2, Matrix4, Point3, Rad, Vector3};
use sdl2;

mod perfcounter;
mod renderer;
mod terrain;

#[derive(Debug)]
struct State {
    center: Vector3<f32>,
    yaw: f32,
    pitch: f32,
    scale: f32,
    drag: bool,
    translate: bool,
}

impl State {
    fn new() -> Self {
        let center = (terrain::SIZE / 2) as f32;
        Self {
            center: vec3(center, center, 128.0),
            yaw: 0.4,
            pitch: 0.4,
            scale: 1000.0 / terrain::SIZE as f32,
            drag: false,
            translate: false,
        }
    }

    fn handle_event(&mut self, e: &sdl2::event::Event) {
        use sdl2::event::Event;
        use sdl2::keyboard::Keycode;
        match e {
            &Event::KeyDown {
                keycode: Some(Keycode::LShift),
                ..
            }
            | &Event::KeyDown {
                keycode: Some(Keycode::RShift),
                ..
            } => {
                self.translate = true;
            }
            &Event::KeyUp {
                keycode: Some(Keycode::LShift),
                ..
            }
            | &Event::KeyUp {
                keycode: Some(Keycode::RShift),
                ..
            } => {
                self.translate = false;
            }
            &Event::MouseButtonDown { .. } => {
                self.drag = true;
            }
            &Event::MouseButtonUp { .. } => {
                self.drag = false;
            }
            &Event::MouseWheel { y, .. } => {
                self.scale *= (y as f32 * 0.01).exp();
            }
            &Event::MouseMotion { xrel, yrel, .. } => {
                if self.drag {
                    if self.translate {
                        let params = self.render_params(0, 0);
                        let m = Matrix2::from_cols(params.axes[0], params.axes[1])
                            .invert()
                            .unwrap();
                        let offset = m * vec2(xrel as f32, yrel as f32);
                        self.center -= offset.extend(0.0);
                    } else {
                        self.pitch += yrel as f32 * 0.003;
                        self.yaw += xrel as f32 * 0.003;

                        self.yaw = self
                            .yaw
                            .max(std::f32::consts::PI * -0.4)
                            .min(std::f32::consts::PI * 0.4);
                        self.pitch = self
                            .pitch
                            .max(std::f32::consts::PI * -0.5)
                            .min(std::f32::consts::PI * 0.5);
                    }
                }
            }
            _ => {}
        }
    }

    fn render_params(&self, w: u32, h: u32) -> renderer::RenderParams {
        let mat = Matrix4::from_scale(self.scale)
            * Matrix4::from_angle_x(Rad(self.pitch + std::f32::consts::FRAC_PI_2))
            * Matrix4::from_angle_z(Rad(self.yaw))
            * Matrix4::from_nonuniform_scale(1.0, 1.0, terrain::SIZE as f32 / 1024.0)
            * Matrix4::from_translation(-self.center);

        let origin = mat.transform_point(Point3::new(0.0, 0.0, 0.0));

        renderer::RenderParams {
            origin: vec2(w, h).cast::<f32>().unwrap() * 0.5 + vec2(origin.x, origin.y),
            axes: [
                mat.transform_vector(Vector3::unit_x()).truncate(),
                mat.transform_vector(Vector3::unit_y()).truncate(),
                mat.transform_vector(Vector3::unit_z()).truncate(),
            ],
        }
    }
}

fn main() {
    let sdl_context = sdl2::init().unwrap();
    let mut event_pump = sdl_context.event_pump().unwrap();
    let video = sdl_context.video().unwrap();

    let mut window = video
        .window("heightmap rendering test", 1280, 720)
        .position_centered()
        .resizable()
        .build()
        .unwrap();

    let mut surface = {
        let window_surface = window.surface(&event_pump).unwrap();
        sdl2::surface::Surface::new(
            window_surface.width(),
            window_surface.height(),
            sdl2::pixels::PixelFormatEnum::ARGB8888,
        ).unwrap()
    };

    let terrain = terrain::new_terrain_random();
    let mut state = State::new();

    use sdl2::event::{Event, WindowEvent};
    use sdl2::keyboard::Keycode;

    let mut fps_counter = crate::perfcounter::PerfCounter::new();

    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'running,
                Event::Window {
                    win_event: WindowEvent::Resized(w, h),
                    ..
                } => {
                    surface = sdl2::surface::Surface::new(
                        w as u32,
                        h as u32,
                        sdl2::pixels::PixelFormatEnum::ARGB8888,
                    ).unwrap();
                }
                e => {
                    state.handle_event(&e);
                }
            }
        }

        fps_counter.log(1.0);

        let render_params = state.render_params(surface.width(), surface.height());

        renderer::render_to(&mut surface, &terrain, &render_params);

        {
            let mut window_surface = window.surface(&event_pump).unwrap();
            surface.blit(None, &mut window_surface, None).unwrap();
            window_surface.update_window().unwrap();
        }

        let title = format!("heightmap rendering test [{:.2} fps]", fps_counter.rate(),);
        window.set_title(&title).unwrap();
    }
}
