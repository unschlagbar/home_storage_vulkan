use std::{env, fs, io::Write, path::Path};

const ASSET_FOLDER: &str = "textures";

pub fn build() {
    println!("cargo:rerun-if-changed={ASSET_FOLDER}");
    let icon_dir = Path::new(ASSET_FOLDER);

    let out_dir = env::var("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("gen_icons.rs");

    let mut icons = Vec::new();
    for entry in fs::read_dir(icon_dir).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        if let Some(ext) = path.extension() {
            if ext == "png" {
                if let Some(stem) = path.file_stem() {
                    let name = stem.to_string_lossy();
                    icons.push(name.to_string());
                }
            }
        }
    }

    let mut code = String::new();
    code.push_str("#[repr(u32)]\n#[derive(Clone, Copy, Debug)]\n");
    code.push_str("pub enum Icon {\n");
    for icon in &icons {
        let variant = to_camel_case(icon);
        code.push_str(&format!("    {},\n", variant));
    }
    code.push_str("}\n");

    let mut file = fs::File::create(&dest_path).unwrap();
    file.write_all(code.as_bytes()).unwrap();
}

fn to_camel_case(s: &str) -> String {
    s.split('_')
        .map(|w| {
            let mut chars = w.chars();
            match chars.next() {
                None => String::new(),
                Some(f) => f.to_uppercase().collect::<String>() + chars.as_str(),
            }
        })
        .collect()
}
