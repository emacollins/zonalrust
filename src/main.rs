use std::{env, process};
use std::fs::File;
use std::io::Write;
use std::path::Path;

use zonalrust::{Config, run};

fn create_sample_binary_file(zarr_dir: &Path) -> std::io::Result<()> {
    let values: Vec<f32> = vec![1.0, 2.0, 3.0, 4.0];
    let example_file_path = zarr_dir.join("example.bin");
    let mut f = File::create(example_file_path)?;
    for v in values {
        f.write_all(&v.to_le_bytes())?
    }
    Ok(())
}   


fn main() {
    let config = match Config::build(env::args()) {
        Ok(c) => c,
        Err(msg) => {
        eprintln!("Problem parsing arguments: {msg}");
        process::exit(64);
        }
    };
    let _ = create_sample_binary_file(config.zarr_path());
    if let Err(err) = run(&config) {
        eprintln!("zonalrust: {err}");
        process::exit(1)
    }

}
