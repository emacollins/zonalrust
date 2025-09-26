use std::{path::{Path, PathBuf}};

pub mod wkt;
pub mod raster;

#[derive(Debug)]
pub struct Config {
    zarr_path: PathBuf,
    vector_path: PathBuf,
    verbose: bool,
}

impl Config {
    pub fn build(mut args: impl Iterator<Item = String>) -> Result<Self, String> {
        let program = args.next().unwrap_or_else(|| "zonalrust".into()); // First value is always the name of the program

        let mut zarr_path: Option<PathBuf> = None;
        let mut vector_path: Option<PathBuf> = None;
        let mut verbose = false;

        while let Some(arg) = args.next() {
            match arg.as_str() {
                "-h" | "--help" => return Err(Self::usage(&program)),
                "-v" | "--verbose" => verbose = true,
                "--zarr" => {
                    let v = next_or("--zarr", &mut args, &program)?;
                    zarr_path = Some(PathBuf::from(v));
                }
                 "--vector" => {
                    let v = next_or("--vector", &mut args, &program)?;
                    vector_path = Some(PathBuf::from(v));
                }
                _=> {
                    if zarr_path.is_none() {
                        zarr_path = Some(PathBuf::from(arg));
                    } else if vector_path.is_none() {
                        vector_path = Some(PathBuf::from(arg));
                    } else {
                        return Err(format!(
                            "Unexpected extra argument: {arg}\n{usage}",
                            usage=Self::usage(&program),
                        ));
                    }
                }  
            }
        }

        let zarr_path = zarr_path.ok_or_else(|| format!("Missing Zarr path\n{}", Self::usage(&program)))?;
        let vector_path = vector_path.ok_or_else(||format!("Missing vector path \n{}", Self::usage(&program)))?;

        Ok(Self {
            zarr_path,
            vector_path,
            verbose
        })
    }
    
    pub fn zarr_path(&self) -> &Path {
        &self.zarr_path
    }

    pub fn vector_path(&self) -> &Path {
        &self.vector_path
    }

    pub fn verbose(&self) -> bool {
        self.verbose
    }

    fn usage(program: &str) -> String {
            format!(
    "Usage:
    {program} [OPTIONS] <ZARR_PATH> <VECTOR_PATH>
    {program} [OPTIONS] --zarr <PATH> --vector <PATH>

    Options:
    --zarr <PATH>      Path to Zarr store directory
    --vector <PATH>    Path to vector feature (we'll define format soon)
    -v, --verbose      Extra logging to stderr
    -h, --help         Show this help
    ",
    program=program)
    }
}

fn next_or(
    flag: &str,
    args: &mut impl Iterator<Item = String>,
    program: &str,
) -> Result<String, String> {
    args.next()
        .ok_or_else(||format!("Expected a value after {flag}\n{}", Config::usage(program)))
}
    
pub fn run(cfg: &Config) -> Result<(), String> {
    if !cfg.zarr_path().exists() {
        return Err(format!(
            "Zarr path does not exist: {}",
            cfg.zarr_path().display()
        ));
    }

    if !cfg.vector_path().exists() {
        return Err(format!(
            "Vector path does not exist: {}",
            cfg.vector_path().display()
        ));
    }

    if cfg.verbose() {
        eprintln!("Config: {:?}", cfg);
    }


    println!("Input looks good. Ready for next step.");
    Ok(())


}

mod tests {
    use super::*;

    #[test]
    fn parses_positional_ok() {
        let args = vec!["prog", "zarr_dir", "vec.wkt"]
            .into_iter()
            .map(String::from);

        let cfg = Config::build(args).unwrap();
        assert_eq!(cfg.zarr_path(), Path::new("zarr_dir"));
        assert_eq!(cfg.vector_path(), Path::new("vec.wkt"));
        assert!(!cfg.verbose());


    }

    #[test]
    fn parses_flag_ok() {
        let args = vec!["prog", "zarr_dir", "vec.wkt", "--verbose"]
            .into_iter()
            .map(String::from);
        let cfg = Config::build(args).unwrap();
        assert_eq!(cfg.zarr_path(), Path::new("zarr_dir"));
        assert_eq!(cfg.vector_path(), Path::new("vec.wkt"));
        assert!(cfg.verbose());

    }

    #[test]
    fn help_triggers_usage_err() {
        let args = vec!["prog", "--help"]
            .into_iter()
            .map(String::from);
        let err = Config::build(args).unwrap_err();
        assert!(err.contains("Usage:"));
    }
}