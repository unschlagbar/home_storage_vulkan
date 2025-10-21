mod asset_gen;
mod shaderc;

fn main() {
    shaderc::build();
    asset_gen::build();
}
