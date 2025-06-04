
pub mod graphics;
mod game;

#[allow(non_snake_case, unused_variables)]
#[cfg(target_os = "android")]
mod android {
    use activity::AndroidApp;
    use winit::platform::android::EventLoopBuilderExtAndroid;
    use winit::platform::android::*;
    use log::info;
    use crate::game::app::App;
    use winit::event_loop::{EventLoop, EventLoopBuilder};
    use std::panic;

    #[unsafe(no_mangle)]
    pub fn android_main(app: AndroidApp) {
        panic::set_hook(Box::new(|info| {
            log::error!("Panic occurred: {:?}", info);
        }));
        android_logger::init_once(android_logger::Config::default().with_max_level(log::LevelFilter::max()));
        log::info!("Running mainloop...");

        let event_loop: EventLoop<()> = EventLoopBuilder::default().with_android_app(app).build().unwrap();

        let mut application = App::run();
        
        info!("between");
        event_loop.run_app(&mut application).unwrap();
    }
}