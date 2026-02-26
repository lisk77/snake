// build.rs example

use anyhow::*;
use fs_extra::copy_items;
use fs_extra::dir::CopyOptions;
use std::env;
use std::path::PathBuf;

fn main() -> Result<()> {
    // Watch resource directories for changes
    println!("cargo:rerun-if-changed=res/textures/*");
    println!("cargo:rerun-if-changed=res/fonts/*");
    println!("cargo:rerun-if-changed=res/sounds/*");
    println!("cargo:rerun-if-changed=res/shaders/*");

    let profile = env::var("PROFILE")?;
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR")?);
    let target_dir = manifest_dir.join("target").join(&profile);

    let dest_resources_dir = target_dir.join("res");

    std::fs::create_dir_all(&dest_resources_dir)?;

    let mut copy_options = CopyOptions::new();
    copy_options.overwrite = true;
    copy_options.copy_inside = true;

    let resource_folders = vec!["res/textures/", "res/fonts/", "res/sounds/", "res/shaders/"];

    let resource_paths: Vec<PathBuf> = resource_folders
        .iter()
        .map(|p| manifest_dir.join(p))
        .collect();

    copy_items(&resource_paths, &dest_resources_dir, &copy_options)?;

    Ok(())
}
