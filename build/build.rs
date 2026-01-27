use std::env;

mod asset_gen;
mod include_dir;
mod shaderc;

fn main() {
    shaderc::build();
    asset_gen::build();
    include_dir::build();

    if env::var("CARGO_CFG_TARGET_OS").unwrap() == "android" {
        if let Some(ndk) = option_env!("ANDROID_NDK_HOME") {
            println!(
                r"cargo:rustc-link-search=native={}\toolchains\llvm\prebuilt\windows-x86_64\sysroot\usr\lib\aarch64-linux-android\34",
                ndk
            );
        } else {
            println!("cargo::warning=ANDROID_NDK_HOME env variable not set");
        }
    }
}
