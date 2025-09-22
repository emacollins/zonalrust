use std::{env, process};

use zonalrust::{Config, run};


fn main() {
    let config = match Config::build(env::args()) {
        Ok(c) => c,
        Err(msg) => {
        eprintln!("Problem parsing arguments: {msg}");
        process::exit(64);
        }
    };

    if let Err(err) = run(&config) {
        eprintln!("zonalrust: {err}");
        process::exit(1)
    }

}
