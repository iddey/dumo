use std::env;

use fs_extra::copy_items;
use fs_extra::dir::CopyOptions;
use fs_extra::error::Result;

fn main() -> Result<()> {
    let paths = ["memory.x"];
    let out_dir = env::var("OUT_DIR").expect("expected `OUT_DIR` to be set");
    copy_items(&paths, &out_dir, &CopyOptions::new().overwrite(true))?;

    println!("cargo:rustc-link-search={}", out_dir);

    for path in paths {
        println!("cargo:rerun-if-changed={}", path);
    }

    Ok(())
}
