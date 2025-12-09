use std::{env, fs, io::Write, path::Path};

const ASSET_FOLDER: &str = "assets";

#[macro_export]
macro_rules! generate_assets {
    ($($path:expr => $enum_name:ident),* $(,)?) => {
        pub const BUILD_JOBS: &[(&str, &str)] = &[
            $( (concat!("assets/", $path), stringify!($enum_name)), )*
        ];
    };
}

generate_assets! {
    "textures" => UiIcons,
    "sprites"  => Sprites,
}

pub fn build() {
    println!("cargo:rerun-if-changed={ASSET_FOLDER}");

    let out_dir = env::var("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("gen_assets.rs");
    let mut file = fs::File::create(&dest_path).unwrap();

    for (path, enum_name) in BUILD_JOBS {
        let mut icons = Vec::new();
        for entry in fs::read_dir(path).unwrap() {
            let entry = entry.unwrap();
            let path = entry.path();
            if let Some(ext) = path.extension()
                && ext == "png"
                && let Some(stem) = path.file_stem()
            {
                let name = stem.to_string_lossy();
                icons.push(name.to_string());
            }
        }

        if icons.is_empty() {
            continue;
        }

        let mut code = String::new();
        code.push_str("#[repr(u32)]\n#[derive(Clone, Copy, Debug)]\n");
        code.push_str("pub enum ");
        code.push_str(enum_name);
        code.push_str(" {\n");
        for icon in &icons {
            let variant = to_camel_case(icon);
            code.push_str(&format!("    {},\n", variant));
        }
        code.push_str("}\n");

        file.write_all(code.as_bytes()).unwrap();
    }
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
