use std::io::Error;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::{env, fs};

const SHADER_DIR: &str = "shaders";
const SPV_DIR: &str = "spv";

#[cfg(target_os = "windows")]
const GLSLC: &str = "/Bin/glslc.exe";

#[cfg(not(target_os = "windows"))]
const GLSLC: &str = "/bin/glslc";

pub fn build() {
    println!("cargo:rerun-if-changed={SHADER_DIR}");

    let shader_compiler = match env::var("VULKAN_SDK") {
        Ok(vulkan_sdk) => vulkan_sdk + GLSLC,
        Err(_) => {
            println!("cargo::warning=Vulkan SDK env variable not set");
            return;
        }
    };

    for shader_path in get_shader_files(SHADER_DIR).unwrap() {
        compile_shader(&shader_path, &shader_compiler).unwrap();
    }
}

fn get_shader_files(dir: &str) -> Result<Vec<PathBuf>, Error> {
    let mut shader_files = Vec::new();

    for entry in fs::read_dir(dir)? {
        let path = entry?.path();

        if path.is_dir() {
            shader_files.extend(get_shader_files(path.to_str().unwrap())?);
        } else if let Some(extension) = path.extension()
            && (extension == "frag"
                || extension == "vert"
                || extension == "comp"
                || extension == "geom")
        {
            shader_files.push(path);
        }
    }
    Ok(shader_files)
}

fn get_spirv_output_path(shader_path: &Path) -> PathBuf {
    let extension = shader_path.file_name().unwrap().to_str().unwrap();
    PathBuf::from(format!("{SPV_DIR}/{extension}.spv"))
}

fn compile_shader(shader_path: &Path, compiler: &str) -> Result<(), Error> {
    let output_path = get_spirv_output_path(shader_path);

    let output = Command::new(compiler)
        .arg(shader_path)
        .arg("-o")
        .arg(&output_path)
        .output()?;

    if !output.status.success() {
        println!(
            "cargo::error=Shader compilation failed for: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    } else {
        println!("cargo:info=Compiled GLSL {:?}", shader_path);
    }

    Ok(())
}
