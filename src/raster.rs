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
    y_res: f64, //north-up
    data: Vec<f32>, //row-major order
}

impl Raster {
    pub fn from_files(meta_path: &Path, bin_path: &Path) -> Result<Self, String> {
        let metadata: String = read_to_string(meta_path).map_err(|e| format!("Error reading metadata file: {e}"))?;
        let tokens: Vec<&str> = metadata.split_whitespace().collect::<Vec<&str>>();
        if tokens.len() != 6 {
            return Err(format!("Expected 6 metadata fields, got {}", tokens.len()))
        }
        let width: usize = tokens[0].parse().map_err(|_| "invalid width".to_string())?;
        let height: usize = tokens[1].parse().map_err(|_| "invalid height".to_string())?;
        let x_min: f64 = tokens[2].parse().map_err(|_| "invalid x_min".to_string())?;
        let y_max: f64 = tokens[3].parse().map_err(|_| "invalid y_max".to_string())?;
        let x_res: f64 = tokens[4].parse().map_err(|_| "invalid x_res".to_string())?;
        let y_res: f64 = tokens[5].parse().map_err(|_| "invalid y_res".to_string())?;

        // Validation of fields here should eventually be moved to their own method
        if x_res == 0. {
            return Err("X resolution is zero!".to_string())
        }
        if y_res == 0. {
            return Err("Y resolution is zero!".to_string())
        }
        if y_res > 0. {
            return Err("Y resolution should be negative, we are assuming north-up!".to_string())
        }

        if width <= 0 {
            return Err("Width not valid".to_string())
        }

        if height <= 0 {
            return Err("Height not valid".to_string())
        }

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
        let mut count: usize = 0;

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
            return Err("no overlapping cells between provided bounding box and raster".to_string())
        }
    }

    fn coords_to_index(&self, x: f64, y:f64) -> Result<(usize, usize), String> {
        
        let dy = self.y_res.abs();

        let i = ((x - self.x_min) / self.x_res).floor() as isize;
        let j = ((self.y_max - y) / dy).floor() as isize;

        if i < 0 || j < 0 {
            return Err("Invalid coords: Negative indices".to_string())
        }

        let i = i as usize;
        let j = j as usize;

        if i >= self.width || j >= self.height {
            return Err("Invalid coords: Out of raster bounds".to_string())
        }

        Ok((i, j))
    }

    /// Returns (i0, i1, j0, j1) where i1 and j1 are exclusive upper bounds.
    /// If bounding box is out of raster bounds, we clamp to raster edges
    /// Loops should use `for i in i0..i1`.
    /// assume: x_min, y_max, x_res>0, y_res<0
    fn bbox_coords_to_index_ranges(&self, x_min: f64, y_min: f64, x_max: f64, y_max: f64) -> Result<(usize, usize, usize, usize), String>{
        if !(x_min < x_max && y_min < y_max) { return Err("Bounding box coordinates not valid".to_string()); }

        let x_max_grid = self.x_min + ((self.width) as f64) * self.x_res;
        let y_min_grid = self.y_max + ((self.height) as f64) * self.y_res; // assume y_res < 0

        // clamp to overlap
        let x0 = x_min.clamp(self.x_min, x_max_grid);
        let x1 = x_max.clamp(self.x_min, x_max_grid);
        let y0 = y_min.clamp(y_min_grid, self.y_max);
        let y1 = y_max.clamp(y_min_grid, self.y_max);

        if !(x0 < x1 && y0 < y1) {
            return Err("no overlapping cells between provided bounding box and raster".to_string());
        }

        // Want exlcusive upper bounds, so use ceiling for x1 & y1
        let dy = self.y_res.abs();
        let i0 = ((x0 - self.x_min) / self.x_res).floor() as usize;
        let i1 = ((x1 - self.x_min) / self.x_res).ceil() as usize;
        let j0 = ((self.y_max - y1) / dy).floor() as usize;
        let j1 = ((self.y_max - y0) / dy).ceil() as usize;

        if i0 >= i1 || j0 >= j1 {
            return Err("no overlapping cells between provided bounding box and raster".into());
        }

        Ok((i0, i1, j0, j1))
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
        let r = Raster {width: 3, height: 2, x_min: 0.0, y_max: 0.0, x_res: 0.25, y_res: -0.25, data: vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0]};

        assert_eq!(r.mean_in_bbox(0.25, -0.25, 0.50, 0.0), Ok(2.0 / 1.0));
        assert_eq!(r.mean_in_bbox(0.25, -0.25, 1.0, 0.0), Ok( (3.0 + 2.0) / 2.0 )); // bbox partially in raster (x_max is out of bounds)
        assert_eq!(r.mean_in_bbox(0.26, -0.24, 0.49, 0.01), Ok(2.0 / 1.0));
        assert_eq!(r.mean_in_bbox(0.50, -0.25, 0.25, 0.25), Err("Bounding box coordinates not valid".to_string())); // Case where x min > x max, expect error message
        assert_eq!(r.mean_in_bbox(0.50, -0.25, 0.75, 0.0), Ok( 3.0 / 1.0 )); // bbox whose right edge == raster right edge (0.75) should not include column 2 beyond it
        assert_eq!(r.mean_in_bbox(0.0, -0.50, 0.25, -0.25), Ok(4.0)); // only j=1,i=0
        assert!(r.mean_in_bbox(0.80, -0.10, 0.90, 0.10).is_err()); // bbox fully outside grid
        assert!(r.mean_in_bbox(0.80, -0.4, 0.80, -0.3).is_err()); // x_min and x_max are equal, violating that x_max must be greater
        assert_eq!(r.mean_in_bbox(0.0, -0.25, 0.01, 0.0), Ok(1.0)); // top-left cell (i=0,j=0) should be included if bbox touches its top/left edges


    }
}