#![allow(clippy::collapsible_match)]
#![allow(clippy::single_match)]
use crate::app::{App, DEBUG_PERF, LIMIT_FPS, ONLY_DRAW_ON_UPDATE};
use crate::thread_event::RenderEvent;
use crate::vulkan_render::VulkanRender;

use std::{cell::RefCell, rc::Rc, time::Instant};
use winit::{
    application::ApplicationHandler,
    event::{ElementState, MouseButton, WindowEvent},
    event_loop::{ActiveEventLoop, ControlFlow},
    keyboard::{KeyCode, PhysicalKey},
    window::{Fullscreen, WindowId},
};

impl ApplicationHandler<RenderEvent> for App {
    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        let (renderer, window) = if let Some(window) = &self.window
            && let Some(renderer) = &self.renderer
        {
            (renderer, window)
        } else {
            return;
        };

        let input_consumed;

        let ui_events = {
            let mut ui = self.ui.borrow_mut();
            input_consumed = ui.window_event(&event, window).is_new();
            [ui.get_event(), ui.get_event()]
        };

        for event in ui_events.into_iter().flatten() {
            self.explorer.proceed_event(event)
        }

        match event {
            WindowEvent::MouseInput {
                device_id: _,
                state,
                button,
            } => {
                let mut ui = self.ui.borrow_mut();
                match (button, state) {
                    (MouseButton::Left, ElementState::Released) => {
                        self.explorer.mouse_click(&mut ui);
                    }
                    (MouseButton::Right, ElementState::Pressed) => {
                        self.explorer.right_click(&mut ui);
                    }
                    _ => (),
                }
                if ui.is_dirty() {
                    window.request_redraw();
                }
            }
            _ => (),
        }

        if input_consumed {
            return;
        }

        match event {
            WindowEvent::KeyboardInput {
                device_id: _,
                event,
                is_synthetic: _,
            } => {
                if event.state == ElementState::Pressed
                    && let PhysicalKey::Code(key_code) = event.physical_key
                {
                    match key_code {
                        KeyCode::F11 => {
                            if window.fullscreen().is_none() {
                                window.set_fullscreen(Some(Fullscreen::Borderless(None)));
                            } else {
                                window.set_fullscreen(None);
                            }
                        }
                        KeyCode::F3 => {
                            let mut ui = self.ui.borrow_mut();
                            let elem = ui.get_element(self.explorer.properties_view.id).unwrap();
                            println!("f3 Debug: {:?}", elem);
                        }
                        _ => (),
                    }
                }
            }
            WindowEvent::RedrawRequested => {
                if DEBUG_PERF {
                    let start = Instant::now();
                    renderer.borrow_mut().draw_frame();
                    println!("Draw: {:?}", start.elapsed());
                } else {
                    renderer.borrow_mut().draw_frame();
                }
            }
            WindowEvent::Resized(new_size) => {
                let mut renderer = renderer.borrow_mut();
                if new_size == renderer.window_size {
                    return;
                }
                renderer.recreate_swapchain(new_size);
            }
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Destroyed => event_loop.exit(),
            _ => (),
        }
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        if ONLY_DRAW_ON_UPDATE {
            if let Some(window) = &self.window {
                let mut ui = self.ui.borrow_mut();
                if ui.needs_ticking() {
                    ui.process_ticks();
                    if ui.is_dirty() {
                        window.request_redraw();
                    }
                }
                event_loop.set_control_flow(ControlFlow::wait_duration(self.target_frame_time));
            }
        } else {
            if LIMIT_FPS {
                event_loop.set_control_flow(ControlFlow::wait_duration(self.target_frame_time));
            }

            if let Some(window) = &self.window {
                let mut ui = self.ui.borrow_mut();
                if ui.needs_ticking() {
                    ui.process_ticks();
                }
                window.request_redraw();
            }
        }
    }

    fn suspended(&mut self, _: &ActiveEventLoop) {
        println!("suspended");

        if let Some(renderer) = &self.renderer {
            let mut renderer = renderer.borrow_mut();
            renderer.destroy_ressources(&mut self.render_assets);
        }

        self.renderer = None;
    }

    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window = if let Some(window) = self.window.take() {
            window
        } else {
            let window = self.create_window(event_loop);
            self.get_framerate(&window);
            window
        };

        if let Some(renderer) = &self.renderer {
            let mut renderer = renderer.borrow_mut();
            self.render_assets.init(&mut renderer);
        } else if DEBUG_PERF {
            let start_time = Instant::now();

            let mut renderer = VulkanRender::create(&window, self.ui.clone());
            self.render_assets.init(&mut renderer);
            self.renderer = Some(Rc::new(RefCell::new(renderer)));

            println!("Vulkan time: {:?}", start_time.elapsed());
        } else {
            let mut renderer = VulkanRender::create(&window, self.ui.clone());
            self.render_assets.init(&mut renderer);
            self.renderer = Some(Rc::new(RefCell::new(renderer)));
        }

        let mut ui = self.ui.borrow_mut();
        ui.scale_factor = window.scale_factor() as f32;
        ui.resize(window.inner_size().into());

        // Wayland only shows the window after the first frame is rendered
        // Windows shows BLACK window until we draw the first frame, so we hide the window until its done
        #[cfg(target_os = "windows")]
        window.set_visible(true);

        self.window = Some(window);
        if DEBUG_PERF {
            println!("window + Vulkan time: {:?}", self.time.elapsed());
        }
        self.time = Instant::now();
    }

    fn exiting(&mut self, _: &ActiveEventLoop) {
        println!("exiting");

        if let Some(renderer) = &self.renderer {
            let mut renderer = renderer.borrow_mut();
            renderer.destroy(&mut self.render_assets);
        }
    }

    fn user_event(&mut self, _: &ActiveEventLoop, event: RenderEvent) {
        self.explorer.proceed_message(event);
        let ui = self.ui.borrow();
        if ui.is_dirty()
            && let Some(window) = &self.window
        {
            window.request_redraw();
        }
    }
}
