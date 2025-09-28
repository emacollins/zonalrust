use std::fs::{read_to_string, read, File, remove_file};
use std::path::Path;
use std::mem::size_of;
use std::io::Write;

#[derive(Debug, PartialEq)]
pub struct Raster {
    width: usize,
    height: usize,
    data: Vec<f32>, //row-major order
}

impl Raster {
    pub fn from_files(meta_path: &Path, bin_path: &Path) -> Result<Self, String> {
        let metadata: Vec<usize> = match read_to_string(meta_path) {
            Ok(wh) => wh.split_whitespace()
                                .map(|str| str.parse::<usize>()
                                                    .map_err(|_| format!("Invalid number: {str}")))
                                .collect::<Result<_, _>>()?,
            _ => return Err(format!("Error reading metadata string from path: {}", meta_path.display()))
        };

        let binary_data: Vec<u8> = read(bin_path).map_err(|_|format!("Error reading binary from path: {}", bin_path.display()))?;
        if binary_data.len() % size_of::<f32>() != 0 {
            return Err(format!("Binary length {} is not divisible by 4", binary_data.len()))
        }

        let data: Vec<f32> = binary_data.chunks_exact(size_of::<f32>())
                                        .map(|chunk| {
                                            let bytes: [u8; 4] = chunk.try_into().unwrap();
                                            f32::from_le_bytes(bytes)
                                        })
                                        .collect();

        let width = match metadata.first() {
            Some(s) => *s,
            None => return Err(format!("No width value in metadata"))
        };

        let height = match metadata.last() {
            Some(s) => *s,
            None => return Err(format!("No height value in metadata"))
        };

        let expected_length = width.checked_mul(height).ok_or(format!("Overflow with width and height: {} x {}", width, height))?;
        
        if data.len() != expected_length {
            return Err(format!("Length of data does not match expected length found in metadata; Expected: {}, Actual {}", expected_length, data.len()))
        }
        
        Ok(Raster {width, height, data})
    }

    pub fn get(&self, x: usize, y: usize) -> Option<f32> {
        if x < self.width && y < self.height {
            Some(self.data[y * self.width + x])
        } else {
            None
        }
    }

    pub fn mean_in_bbox(&self, minx: usize, miny: usize, maxx: usize, maxy: usize) -> Option<f32> {
        
        todo!()
    }
}

mod tests {

    use super::*;

    #[test]
    fn from_files_loads_ok() {
        let values: Vec<f32> = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0];
        let metadata: Vec<usize> = vec![3, 2];
        let dir = std::env::temp_dir();
        let bin_path = dir.join("values_test.bin");
        let meta_path = dir.join("metadata_test.txt");


        let mut binary_file = File::create(&bin_path).unwrap();
        for v in values {
            binary_file.write_all(&v.to_le_bytes()).unwrap();
            }
        let mut metadata_file = File::create(&meta_path).unwrap();
        for (i, val) in metadata.iter().enumerate() {
            if i > 0 {
                write!(metadata_file, " ").unwrap(); // add space before every number after the first
            }
                write!(metadata_file, "{val}").unwrap();
        }
        writeln!(metadata_file).unwrap(); // final newline

        let good_result = Raster::from_files(&meta_path, &bin_path).unwrap();

        assert_eq!(good_result, Raster {width: 3, height: 2, data: vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0]});

        remove_file(bin_path).unwrap();
        remove_file(meta_path).unwrap();

    }
}