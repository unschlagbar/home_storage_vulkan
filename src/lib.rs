#![cfg(target_os = "android")]
include!(concat!(env!("OUT_DIR"), "/gen_icons.rs"));

mod app;
mod explorer;
mod vulkan_render;

mod android {
    use crate::app::App;
    use android::activity::AndroidApp;
    use std::panic;
    use winit::event_loop::{EventLoop, EventLoopBuilder};
    use winit::platform::android::{self, EventLoopBuilderExtAndroid};

    #[unsafe(no_mangle)]
    pub fn android_main(app: AndroidApp) {
        panic::set_hook(Box::new(|info| {
            log::error!("Panic occurred: {:?}", info);
        }));

        android_logger::init_once(
            android_logger::Config::default().with_max_level(log::LevelFilter::max()),
        );

        let mut apk = App::new(app.asset_manager());

        let event_loop: EventLoop<()> = EventLoopBuilder::default()
            .with_android_app(app)
            .build()
            .unwrap();

        event_loop.run_app(&mut apk).unwrap();
    }
}
