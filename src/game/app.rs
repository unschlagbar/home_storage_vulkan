use std::{
    cell::RefCell,
    mem::{forget, MaybeUninit},
    rc::Rc,
    thread::sleep,
    time::{Duration, Instant}
};
use iron_oxide::{primitives::Vec2, ui::{DirtyFlags, UiEvent, UiState}};
use log::info;
use winit::{
    application::ApplicationHandler, dpi::{PhysicalPosition, PhysicalSize}, event::{ElementState, MouseButton, TouchPhase, WindowEvent}, event_loop::{ActiveEventLoop, ControlFlow}, keyboard::{KeyCode, PhysicalKey}, window::{Theme, Window, WindowId}
};
use crate::graphics::VulkanRender;
use super::{states::build_main, World};

const WIDTH: u32 = 1280;
const HEIGHT: u32 = 720;
pub const FPS_LIMIT: bool = true;

#[allow(dead_code)]
pub struct App {
    pub init: bool,
    pub window: MaybeUninit<Window>,
    pub renderer: Rc<RefCell<VulkanRender>>,
    pub ui: Rc<RefCell<UiState>>,
    pub cursor_pos: PhysicalPosition<f64>,
    pub world: World,
    pub time: Instant,
    pub last_cursor_location: PhysicalPosition<f64>,
    pub touch_id: u64,
    pub mouse_pressed: bool,
    pub sim_speed: f32,
    pub target_frame_time: f32,
}

impl App {
    #[allow(unused)]
    pub fn run() -> Self {
        #[allow(invalid_value)]
        let renderer = Rc::new(RefCell::new(unsafe { MaybeUninit::uninit().assume_init() }));
        let ui: Rc<RefCell<UiState>> = Rc::new(RefCell::new(build_main()));
        let world = World::create(renderer.clone(), ui.clone());

        Self {
            window: MaybeUninit::uninit(),
            renderer,
            init: false,
            cursor_pos: PhysicalPosition { x: 0.0, y: 0.0 },
            world, 
            time: Instant::now(),
            ui,
            last_cursor_location: PhysicalPosition { x: 0.0, y: 0.0 },
            touch_id: 0,
            mouse_pressed: false,
            sim_speed: 1.0,
            target_frame_time: 1.0 / 144.0,
        }
    }

    pub fn window(&self) -> &Window {
        unsafe { self.window.assume_init_ref() }
    }
}

impl ApplicationHandler for App {
    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        if !self.init {
            return;
        }
        let mut renderer = self.renderer.borrow_mut();

        match event {
            WindowEvent::CursorMoved { device_id: _,  position } => {
                let in_ui;

                {
                    let mut ui = renderer.ui_state.borrow_mut();
                    in_ui = ui.update_cursor(position.into(), UiEvent::Move);
                }

                if in_ui.is_none() && self.mouse_pressed {
                    let delta = Vec2::new(self.cursor_pos.x as f32 - position.x as f32, self.cursor_pos.y as f32 - position.y as f32);
                    self.world.camera.process_mouse_movement(delta, 0.25);
                }

                self.cursor_pos = position;
            },
            WindowEvent::MouseInput { device_id: _, state, button } => {
                match button {
                    MouseButton::Left => {
                        self.mouse_pressed = state == ElementState::Pressed;
                        renderer.ui_state.borrow_mut().update_cursor(self.cursor_pos.into(), 
                            match state {
                                ElementState::Pressed => UiEvent::Press,
                                ElementState::Released => UiEvent::Release,
                            }
                        );
                    }
                    _ => ()
                }
            },
            WindowEvent::Touch(touch) => {
                let cursor_pos = touch.location.into();
                match touch.phase {
                    TouchPhase::Started => {
                        if touch.id != 0 || self.touch_id != touch.id { return }
                        self.touch_id = touch.id;
                        renderer.ui_state.borrow_mut().update_cursor(cursor_pos, UiEvent::Press);
                        self.last_cursor_location = touch.location;
                    },
                    TouchPhase::Moved => {
                        self.last_cursor_location = touch.location;
                        renderer.ui_state.borrow_mut().update_cursor(cursor_pos, UiEvent::Move);
                    },
                    TouchPhase::Ended | TouchPhase::Cancelled => {
                        self.touch_id = 0;
                        renderer.ui_state.borrow_mut().update_cursor(cursor_pos, UiEvent::Release);
                    }
                }
            },
            WindowEvent::RedrawRequested => {
                let time_stamp = self.time.elapsed().as_secs_f32();
                if !FPS_LIMIT || time_stamp > self.target_frame_time * 0.93 {
                    self.time = Instant::now();
                    self.world.update(self.sim_speed * time_stamp, &mut renderer);
                    renderer.draw_frame();
                } else {
                    sleep(Duration::from_nanos(800_000));
                };
            },
            WindowEvent::KeyboardInput { device_id: _, event, is_synthetic: _ } => {
                if let PhysicalKey::Code(key_code) = event.physical_key {

                    match key_code {
                        KeyCode::F1 => {
                            if event.state.is_pressed() {
                                {
                                    let mut value = renderer.ui_state.borrow_mut();
                                    value.visible = !value.visible;
                                    value.dirty = DirtyFlags::Size;
                                }
                            }
                        },
                        KeyCode::KeyX => {
                            if event.state.is_pressed() {
                                if self.sim_speed == 0.0 {
                                    self.sim_speed = 1.0;
                                } else {
                                    self.sim_speed = 0.0;
                                }
                            }
                        },
                        KeyCode::KeyA => {
                            if event.state.is_pressed() {
                                self.world.movement_vector.x = -1.0;
                            } else if self.world.movement_vector.x == -1.0 {
                                self.world.movement_vector.x = 0.0;
                            }
                        },
                        KeyCode::KeyD => {
                            if event.state.is_pressed() {
                                self.world.movement_vector.x = 1.0;
                            } else if self.world.movement_vector.x == 1.0 {
                                self.world.movement_vector.x = 0.0;
                            }
                        },
                        KeyCode::KeyW => {
                            if event.state.is_pressed() {
                                self.world.movement_vector.z = 1.0;
                            } else if self.world.movement_vector.z == 1.0 {
                                self.world.movement_vector.z = 0.0;
                            }
                         },
                        KeyCode::KeyS => {
                            if event.state.is_pressed() {
                                self.world.movement_vector.z = -1.0;
                            } else if self.world.movement_vector.z == -1.0 {
                                self.world.movement_vector.z = 0.0;
                            }
                        },
                        KeyCode::Space => {
                            if event.state.is_pressed() {
                                self.world.movement_vector.y = 1.0;
                            } else if self.world.movement_vector.y == 1.0 {
                                self.world.movement_vector.y = 0.0;
                            }
                        },
                        KeyCode::ShiftLeft => {
                            if event.state.is_pressed() {
                                self.world.movement_vector.y = -1.0;
                            } else if self.world.movement_vector.y == -1.0 {
                                self.world.movement_vector.y = 0.0;
                            }
                        },
                        _ => ()
                    }
                }
            },
            WindowEvent::Resized(new_size) => {
                if !self.init {
                    return;
                }
                let size = self.window().inner_size();
                if new_size != size || new_size == renderer.window_size {
                    return;
                }
                renderer.recreate_swapchain(size);
                self.world.camera.moved = true;
            },
            WindowEvent::CloseRequested => {
                event_loop.exit();
                unsafe { renderer.base.device.device_wait_idle().unwrap_unchecked() };
            },
            _ => (),
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        if self.init {
            self.window().request_redraw();
        }
    }

    fn suspended(&mut self, event_loop: &ActiveEventLoop) {
        println!("suspended");
        if !self.init {
            return;
        }
        self.init = false;
        let mut renderer = self.renderer.borrow_mut();
        unsafe { renderer.base.device.device_wait_idle().unwrap_unchecked(); };
        renderer.destroy();
        event_loop.set_control_flow(ControlFlow::Wait);
    }
    
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        println!("resumed");
        if self.init {
            return;
        } else {
            self.init = true;
        }

        let window_attributes = Window::default_attributes()
            .with_title("Vulkan Homeserver")
            .with_inner_size(PhysicalSize {width: WIDTH, height: HEIGHT})
            .with_visible(false)
            .with_theme(Some(Theme::Dark));

        let window = event_loop.create_window(window_attributes).unwrap();
        if let Some(monitor) = window.current_monitor() {
            if let Some(refresh_rate) = monitor.refresh_rate_millihertz() {
                self.target_frame_time = 1000.0 / refresh_rate as f32;
                println!("target pfs: {}", refresh_rate / 1000);
            } else {
                println!("Refresh rate not available");
            }
        }
        forget(self.renderer.replace(VulkanRender::create(&window, &self.world)));

        let mut renderer = self.renderer.borrow_mut();
        
        let shaders = (include_bytes!("../../spv/basic.vert.spv").as_ref(), include_bytes!("../../spv/basic.frag.spv").as_ref());
        let font_shaders = (include_bytes!("../../spv/bitmap.vert.spv").as_ref(), include_bytes!("../../spv/bitmap.frag.spv").as_ref());
        
        {
            let mut ui = self.ui.borrow_mut();
            ui.init_graphics(&renderer.base, renderer.window_size, renderer.render_pass, renderer.ui_descriptor_set_layout, shaders, font_shaders);
        }
        
        renderer.draw_frame();
        window.set_visible(true);
        
        self.window.write(window);
        event_loop.set_control_flow(ControlFlow::Poll);
        println!("window time: {:?}", self.time.elapsed());
        self.time = Instant::now();
    }

    fn exiting(&mut self, _event_loop: &ActiveEventLoop) {
        info!("exiting");
        if !self.init {
            return;
        }
        self.init = false;
    }
}