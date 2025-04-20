mod ecs;

use std::{collections::HashSet, sync::Arc};
use ecs::{ArchetypeManager, Component, ComponentType, System, World};
use pixels::{Pixels, SurfaceTexture};
use winit::{application::ApplicationHandler, dpi::LogicalSize, event::WindowEvent, event_loop::{ActiveEventLoop, ControlFlow, EventLoop}, window::{Window, WindowId}};

const WIDTH: u32 = 640;
const HEIGHT: u32 = 480;

#[derive(Clone, Copy)]
struct HelloComponent;
impl Component for HelloComponent {
    fn get_type(&self) -> ComponentType {
        ComponentType::Hello
    }
}
impl HelloComponent {
    fn new() -> Self {
        Self {}
    }
}

#[derive(Clone, Copy)]
struct ValueComponent {
    value: i32
}
impl Component for ValueComponent {
    fn get_type(&self) -> ComponentType {
        ComponentType::Value
    }
}
impl ValueComponent {
    fn new(value: i32) -> Self {
        Self {
            value
        }
    }
}

struct IncrementSytem;
impl System for IncrementSytem {
    fn run(&self, manager: &mut ArchetypeManager) {
        for arch_index in manager.with(HashSet::from([ComponentType::Value])) {
            for component in manager.get_components(arch_index, &ComponentType::Value).iter_mut() {
                if let Some(value_comp) = component.as_any_mut().downcast_mut::<ValueComponent>() {
                    value_comp.value += 1;
                    println!("Value {}", value_comp.value);
                }
            }
        }
    }
}

struct HelloSystem;
impl System for HelloSystem {
    fn run(&self, manager: &mut ArchetypeManager) {
        for arch_index in manager.with(HashSet::from([ComponentType::Hello])) {
            for _ in 0..manager.get_components(arch_index, &ComponentType::Hello).len() {
                println!("Hello");
            }
        }
    }
}

#[derive(Default)]
struct App<'window> {
    window: Option<Arc<Window>>,
    pixels: Option<Pixels<'window>>,
}

impl<'window> ApplicationHandler for App<'window> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window = Arc::new(event_loop.create_window(
            Window::default_attributes()
            .with_title("Civsim")
            .with_inner_size(LogicalSize::new(WIDTH as f64, HEIGHT as f64))
        ).unwrap());
        let pixels = {
            let window_size = window.inner_size();
            let surface_texture = SurfaceTexture::new(window_size.width, window_size.height, window.clone());
            Pixels::new(WIDTH, HEIGHT, surface_texture).unwrap()
        };

        self.window = Some(window);
        self.pixels = Some(pixels);
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => { event_loop.exit(); },
            WindowEvent::Resized(size) => {
                self.pixels.as_mut().unwrap().resize_buffer(size.width, size.height).unwrap();
                self.pixels.as_mut().unwrap().resize_surface(size.width, size.height).unwrap();
            },
            WindowEvent::RedrawRequested => {
                for (_i, pixel) in self.pixels.as_mut().unwrap().frame_mut().chunks_exact_mut(4).enumerate() {
                    //let x = (i % WIDTH as usize) as i16;
                    //let y = (i / WIDTH as usize) as i16;
                    pixel.copy_from_slice(&[0xff, 0x50, 0x00, 0xff]);
                }
                self.pixels.as_mut().unwrap().render().unwrap();
            }
            _ => (),
        }
    }
}

fn main() {
    let mut world = World::new();
    for i in 0..3 {
        world.add_component(i, &ValueComponent::new((i as i32) * 100));
    }
    for i in 0..3 {
        world.add_component(i + 3, &HelloComponent::new());
    }
    for i in 0..3 {
        world.add_component(i + 6, &ValueComponent::new((i as i32) * 100));
        world.add_component(i + 6, &ValueComponent::new((i as i32) * 100 + 50));
        world.add_component(i + 6, &HelloComponent::new());
        world.add_component(i + 6, &HelloComponent::new());
    }
    world.add_system(Box::new(HelloSystem));
    world.add_system(Box::new(IncrementSytem));
    world.run();

    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(ControlFlow::Wait);
    event_loop.run_app(&mut App::default()).unwrap();
}
