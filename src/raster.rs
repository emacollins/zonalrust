use std::fs::{read_to_string, read, File, remove_file};
use std::path::Path;
use std::mem::size_of;
use std::io::Write;

#[derive(Debug, PartialEq)]
pub struct Raster {
    width: usize,
    height: usize,
    x_min: f64,
    y_max: f64,
    x_res: f64,
    y_res: f64,
    data: Vec<f32>, //row-major order
}

impl Raster {
    pub fn from_files(meta_path: &Path, bin_path: &Path) -> Result<Self, String> {
        let metadata: Vec<f64> = match read_to_string(meta_path) {
            Ok(wh) => wh.split_whitespace()
                                .map(|str| str.parse::<f64>()
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

        let width: usize = match metadata.get(0) {
            Some(s) => *s as usize,
            None => return Err(format!("No width value in metadata"))
        };

        let height: usize = match metadata.get(1) {
            Some(s) => *s as usize,
            None => return Err(format!("No height value in metadata"))
        };

        let x_min: f64 = match metadata.get(2) {
            Some(s) => *s,
            None => return Err(format!("No x_min value in metadata"))
        };

        let y_max: f64 = match metadata.get(3) {
            Some(s) => *s,
            None => return Err(format!("No y_max value in metadata"))
        };

        let x_res: f64 = match metadata.get(4) {
            Some(s) => *s,
            None => return Err(format!("No x_res value in metadata"))
        };
        let y_res: f64 = match metadata.get(5) {
            Some(s) => *s,
            None => return Err(format!("No y_res value in metadata"))
        };


        let expected_length = width.checked_mul(height).ok_or(format!("Overflow with width and height: {} x {}", width, height))?;
        
        if data.len() != expected_length as usize {
            return Err(format!("Length of data does not match expected length found in metadata; Expected: {}, Actual {}", expected_length, data.len()))
        }
        
        Ok(Raster {width, height, x_min, y_max, x_res, y_res, data})
    }

    pub fn get(&self, x: f64, y: f64) -> Result<f32, String> {
        let (i, j) = Self::coords_to_index(&self, x, y)?;
        Ok(self.data[j * self.width + i])
    }

    pub fn mean_in_bbox(&self, x_min: f64, y_min: f64, x_max: f64, y_max: f64) -> Result<f32, String> {

        let (i0, i1, j0, j1) = Self::bbox_coords_to_index_ranges(&self, x_min, y_min, x_max, y_max)?;

        let mut sum: f32 = 0.0;
        let mut count: i32 = 0;

        for y in j0..j1 {
            let row_start = y * self.width + i0;
            let row_end = y * self.width + i1;
            let row_slice = &self.data[row_start..row_end];

            for &value in row_slice {
                sum += value;
                count += 1;
            }
        }

        if count > 0 {
            return Ok(sum / count as f32)
        } else {
            return Ok(0.0)
        }
    }


    fn coords_to_index(&self, x: f64, y:f64) -> Result<(usize, usize), String> {

        let i = ((x - self.x_min) / self.x_res).floor() as isize;
        let j = ((y - self.y_max) / self.y_res).floor() as isize;

        if i < 0 || j < 0 {
            return Err("Invalid coords: Negative indices".to_string())
        }

        let i = i as usize;
        let j = j as usize;

        if i > self.width - 1 || j > self.height - 1 {
            return Err("Invalid coords: Out of raster bounds".to_string())
        }

        Ok((i, j))
    }

    /// Returns (i0, i1, j0, j1) where i1 and j1 are exclusive upper bounds.
    /// Loops should use `for i in i0..i1`.
    /// assume: x_min, y_max, x_res>0, y_res<0
    fn bbox_coords_to_index_ranges(&self, x_min: f64, y_min: f64, x_max: f64, y_max: f64) -> Result<(usize, usize, usize, usize), String>{
        if !(x_min < x_max && y_min < y_max) { return Err("Bounding box coordinates not valid".to_string()); }

        let (i0, j0) = Self::coords_to_index(&self, x_min, y_max)?;
        let (mut i1, mut j1) = Self::coords_to_index(&self, x_max, y_min)?;

        // Since we want to return exclusive upper bounds, we add one to the end index
        i1 += 1;
        j1 += 1;


        Ok((i0 as usize, i1 as usize, j0 as usize, j1 as usize))
    }

}

mod tests {

    use super::*;

    #[test]
    fn from_files_loads_ok() {
        let values: Vec<f32> = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0];
        let metadata: Vec<f64> = vec![3.0, 2.0, 0.0, 2.0, 1.0, -1.0];
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

        assert_eq!(good_result, Raster {width: 3, height: 2, x_min: 0.0, y_max: 2.0, x_res: 1.0, y_res: -1.0, data: vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0]});
        remove_file(bin_path).unwrap();
        remove_file(meta_path).unwrap();

    }

    #[test]
    fn mean_in_bbox_ok() {
        let raster = Raster {width: 3, height: 2, x_min: 0.0, y_max: 0.0, x_res: 0.25, y_res: -0.25, data: vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0]};

        // Bounding Box within raster
        let mean_1 = raster.mean_in_bbox(0.25, -0.25, 0.50, 0.0).unwrap();

        // Bounding Box outside raster bounds, expect error
        let bad_mean_1 = raster.mean_in_bbox(0.25, -0.25, 0.75, 0.0);

        // Case where x min > x max, expect error message
        let bad_mean_2 = raster.mean_in_bbox(0.50, -0.25, 0.25, 0.25);

        assert_eq!(mean_1, 4.0);
        assert_eq!(bad_mean_1, Err("Invalid coords: Out of raster bounds".to_string()));
        assert_eq!(bad_mean_2, Err("Bounding box coordinates not valid".to_string()));
    }
}