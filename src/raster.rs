use std::fs::{read_to_string, read};
use std::mem::size_of;

pub struct Raster {
    width: usize,
    height: usize,
    data: Vec<f32>, //row-major order
}

impl Raster {
    pub fn from_files(meta_path: &str, bin_path: &str) -> Result<Self, String> {
        let metadata: Vec<usize> = match read_to_string(meta_path) {
            Ok(wh) => wh.split_whitespace()
                                .map(|str| str.parse::<usize>()
                                                    .map_err(|_| format!("Invalid number: {str}")))
                                .collect::<Result<_, _>>()?,
            _ => return Err(format!("Error reading metadata string from path: {}", meta_path))
        };

        let binary_data: Vec<u8> = read(bin_path).map_err(|_|format!("Error reading binary from path: {}", bin_path))?;
        if binary_data.len() % size_of::<f32>() != 0 {
            return Err(format!("Binary length {} is not divisble by 4", binary_data.len()))
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

        let expected_data_length: usize = width * height;
        
        if data.len() != expected_data_length {
            return Err(format!("Length of data does not match expected length found in metadata; Expected: {}, Actual {}", expected_data_length, data.len()))
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
        // TODO: iterate over the window and compute mean
        todo!()
    }
}