use std::{env, fs};
use std::io::Error;
use std::path::{Path, PathBuf};
use std::process::Command;

const SHADER_DIR: &str = "shaders";
const SPV_DIR: &str = "spv";

fn main() -> Result<(), Error> {
    println!("cargo:rerun-if-changed={}", SHADER_DIR);

    let shader_compiler = env::var("VULKAN_SDK").unwrap() + "/Bin/glslc.exe";

    let shader_files = get_shader_files(SHADER_DIR)?;
    for shader_path in shader_files {
        compile_shader(&shader_path, &shader_compiler)?;
    }

    Ok(())
}

fn get_shader_files(dir: &str) -> Result<Vec<PathBuf>, Error> {
    let mut shader_files = Vec::new();

    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_dir() {
            shader_files.extend(get_shader_files(path.to_str().unwrap())?);

        } else if let Some(extension) = path.extension() {
            if extension == "frag" || extension == "vert" || extension == "comp" || extension == "geom" {
                shader_files.push(path);
            }
        }
    }

    Ok(shader_files)
}


fn get_spirv_output_path(shader_path: &Path) -> PathBuf {
    let extension = shader_path.file_name().unwrap().to_str().unwrap();
    PathBuf::from(format!("{}/{}.spv", SPV_DIR, extension))
}

fn compile_shader(shader_path: &Path, compiler: &str) -> Result<(), Error> {
    let output_path = get_spirv_output_path(shader_path);

    let output = Command::new(compiler)
        .arg(shader_path)
        .arg("-o")
        .arg(&output_path)
        .output()?;

    if !output.status.success() {
        println!("cargo::error=Shader compilation failed for: {}", String::from_utf8_lossy(&output.stderr));
    }

    Ok(())
}