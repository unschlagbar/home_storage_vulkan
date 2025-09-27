mod game;
pub mod graphics;
#[cfg(target_os = "android")]
mod android {
    use crate::game::app::App;
    use activity::AndroidApp;
    use log::info;
    use std::panic;
    use winit::event_loop::{EventLoop, EventLoopBuilder};
    use winit::platform::android::EventLoopBuilderExtAndroid;
    use winit::platform::android::*;

    #[unsafe(no_mangle)]
    pub fn android_main(app: AndroidApp) {
        panic::set_hook(Box::new(|info| {
            log::error!("Panic occurred: {:?}", info);
        }));
        android_logger::init_once(
            android_logger::Config::default().with_max_level(log::LevelFilter::max()),
        );
        log::info!("Running mainloop...");

        let event_loop: EventLoop<()> = EventLoopBuilder::default()
            .with_android_app(app)
            .build()
            .unwrap();

        let mut app = App::run();

        info!("between");
        event_loop.run_app(&mut app).unwrap();
    }
}
