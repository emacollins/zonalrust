#[derive(Debug, PartialEq)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}

#[derive(Debug, PartialEq)]
pub struct LineString {
    pub points: Vec<Point>,
}

#[derive(Debug, PartialEq)]
pub struct Polygon {
    pub rings: Vec<LineString>,
}

#[derive(Debug, PartialEq)]
pub struct MultiPolygon {
    pub polygons: Vec<Polygon>
}

#[derive(Debug, PartialEq)]
pub enum Geometry {
    Point(Point),
    LineString(LineString),
    Polygon(Polygon),
    MultiPolygon(MultiPolygon)
}

impl Geometry {
    pub fn from_wkt(wkt: &str) -> Result<Self, String> {
        let wkt = wkt.trim();

        if wkt.starts_with("POINT") {
            return Self::parse_point(wkt)
        } else if wkt.starts_with("LINESTRING") {
            // call a helper: parse_linestring(&str)
            return Self::parse_linestring(wkt)
        } else {
            Err(format!("Unsupported WKT: {wkt}"))
        }
    }

    fn parse_xy(pair: &str) -> Result<Point, String> {
        let nums: Vec<f64> = pair
                             .split_whitespace()
                             .map(|x| x.parse::<f64>().map_err(|_| format!("Invalid number: {x}")))
                             .collect::<Result<_, _>>()?;
        if nums.len() != 2 {
            return Err(format!("Expected two numbers for coordinate pair, got: {}", nums.len()));
        }

        Ok(Point {x: nums[0], y: nums[1]})
    }

    fn parse_point(wkt: &str) -> Result<Geometry, String>{
        
        // Strip prefix first, then suffix to get string of numeric values.
        let inner = wkt.strip_prefix("POINT (").ok_or_else(|| format!("Bad prefix: {}", wkt).to_string())?
                                     .strip_suffix(")").ok_or_else(|| format!("Bad suffix: {}", wkt).to_string())?;

        let point= Self::parse_xy(inner)?;

        Ok(Geometry::Point(point))

    }

    fn parse_linestring(wkt: &str) -> Result<Geometry, String>{
        
        // Strip prefix first, then suffix to get string of numeric values.
        let inner = wkt.strip_prefix("LINESTRING (").ok_or_else(|| format!("Bad prefix: {}", wkt).to_string())?
                             .strip_suffix(")").ok_or_else(|| format!("Bad suffix: {}", wkt).to_string())?;

        let points: Result<Vec<Point>, String> = inner.split(",").map(|pair| Self::parse_xy(pair.trim())).collect();


        Ok(Geometry::LineString(LineString { points: points? }))

    }

}

mod tests {
    use super::*;

    #[test]
    fn parses_xy_ok() {
        let xy_good: &str = "10.0 1.0";
        let xy_bad_1: &str = "100";
        let xy_bad_2: &str = "a";
        let xy_bad_3: &str = "10.0, 1.0";

        assert_eq!(Geometry::parse_xy(xy_good).unwrap(), Point {x: 10.0, y: 1.0});
        assert_eq!(Geometry::parse_xy(xy_bad_1), Err("Expected two numbers for coordinate pair, got: 1".to_string()));
        assert_eq!(Geometry::parse_xy(xy_bad_2), Err("Invalid number: a".to_string()));
        assert_eq!(Geometry::parse_xy(xy_bad_3), Err("Invalid number: 10.0,".to_string()))

    }

    #[test]
    fn parses_point_ok() {
        let point_str_good = "POINT (10.0 1.0)";
        let point_str_bad_1 = "POINT(10.0 1.0)";
        let point_str_bad_2 = "POINT (10.0 1.0";
        let point_str_bad_3 = "POINT (1.0)";
        
        assert_eq!(Geometry::parse_point(point_str_good), Ok(Geometry::Point(Point {x: 10.0, y: 1.0})));
        assert_eq!(Geometry::parse_point(point_str_bad_1), Err(format!("Bad prefix: {}", point_str_bad_1)));
        assert_eq!(Geometry::parse_point(point_str_bad_2), Err(format!("Bad suffix: {}", point_str_bad_2)));
        assert_eq!(Geometry::parse_point(point_str_bad_3), Err("Expected two numbers for coordinate pair, got: 1".to_string()));
    }

    #[test]
    fn parses_linstring_ok() {
        let linsetring_str_good = "LINESTRING (1.0 1.0, 2.0 2.0, 3.0 2.0)";
        let linestring_str_bad_1 = "LINESTRING(1.0 1.0, 2.0 2.0, 3.0 2.0)";
        let linestring_str_bad_2 = "LINESTRING (1.0 1.0, 2.0 2.0, 3.0 2.0";
        let linestring_str_bad_3 = "LINESTRING (1.0 1.0 2.0 2.0, 3.0 2.0)";

        assert_eq!(Geometry::parse_linestring(linsetring_str_good), Ok(Geometry::LineString(LineString{points: vec![Point {x: 1.0, y: 1.0}, Point {x: 2.0, y: 2.0}, Point {x: 3.0, y: 2.0}]})));
        assert_eq!(Geometry::parse_linestring(linestring_str_bad_1), Err(format!("Bad prefix: {}", linestring_str_bad_1)));
        assert_eq!(Geometry::parse_linestring(linestring_str_bad_2), Err(format!("Bad suffix: {}", linestring_str_bad_2)));
        assert_eq!(Geometry::parse_linestring(linestring_str_bad_3), Err("Expected two numbers for coordinate pair, got: 4".to_string()));

    }

}