use std::{env, fs, io::Write, path::Path};

const DIRS: &[(&str, &str, &str)] = &[("assets/textures", "TEXTURES", "png")];

pub fn build() {
    for &(path, _, _) in DIRS {
        println!("cargo:rerun-if-changed={path}");
    }

    let out_dir = env::var("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("include_dirs.rs");
    let mut file = fs::File::create(&dest_path).unwrap();

    for &(path, name, extention) in DIRS {
        let mut entries = Vec::new();
        for entry in fs::read_dir(Path::new(env!("CARGO_MANIFEST_DIR")).join(path)).unwrap() {
            let entry = entry.unwrap();
            let path = entry.path();
            if let Some(ext) = path.extension()
                && ext == extention
            {
                let path = path.to_string_lossy().to_string();
                entries.push(path);
            }
        }

        if entries.is_empty() {
            continue;
        }

        let mut code = String::new();
        code.push_str(&format!(
            "pub const {}: [(&str, &[u8]); {}] = [",
            name,
            entries.len()
        ));
        for entry in entries {
            let name = entry.split("/").last().unwrap();
            code.push_str(&format!("(r\"{}\", include_bytes!(r\"{}\")),", name, entry));
        }
        code.push_str("];");

        file.write_all(code.as_bytes()).unwrap();
    }
}
