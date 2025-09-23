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
            return Self::parse_point_wkt(wkt)
        } else if wkt.starts_with("LINESTRING") {
            // call a helper: parse_linestring(&str)
            return Self::parse_linestring_wkt(wkt)
        } else if wkt.starts_with("POLYGON") {
            return Self::parse_polygon_wkt(wkt)
        } else {
            Err(format!("Unsupported WKT: {wkt}"))
        }
    }

    fn parse_point(pair: &str) -> Result<Point, String> {
        let nums: Vec<f64> = pair
                             .split_whitespace()
                             .map(|x| x.parse::<f64>().map_err(|_| format!("Invalid number: {x}")))
                             .collect::<Result<_, _>>()?;
        if nums.len() != 2 {
            return Err(format!("Expected two numbers for coordinate pair, got: {}", nums.len()));
        }

        Ok(Point {x: nums[0], y: nums[1]})
    }

    // Example pairs: (20 10, 10 20, 30 30)
    fn parse_linestring(pairs: &str) -> Result<LineString, String> {
        let inner = pairs.strip_prefix("(").ok_or_else(|| format!("Bad prefix: {}", pairs))?
                                     .strip_suffix(")").ok_or_else(|| format!("Bad suffix: {}", pairs))?;

        let points: Result<Vec<Point>, String> = inner.split(",").map(|pair| Self::parse_point(pair.trim())).collect();

        Ok(LineString { points: points? })

    }

    fn parse_point_wkt(wkt: &str) -> Result<Geometry, String>{
        
        // Strip prefix first, then suffix to get string of numeric values.
        let inner = wkt.strip_prefix("POINT (").ok_or_else(|| format!("Bad prefix: {}", wkt))?
                                     .strip_suffix(")").ok_or_else(|| format!("Bad suffix: {}", wkt))?;

        let point= Self::parse_point(inner)?;

        Ok(Geometry::Point(point))

    }

    fn parse_linestring_wkt(wkt: &str) -> Result<Geometry, String>{
        
        // Strip prefix first, then suffix to get string of numeric values.
        let inner = wkt.strip_prefix("LINESTRING").ok_or_else(|| format!("Bad prefix: {}", wkt))?;

        let linestring = Self::parse_linestring(inner.trim())?;


        Ok(Geometry::LineString(linestring))

    }

    fn parse_polygon_wkt(wkt: &str) -> Result<Geometry, String>{
        // Strip prefix first, then suffix to get string of numeric values.
        let inner = wkt.strip_prefix("POLYGON (").ok_or_else(|| format!("Bad prefix: {}", wkt))?
                             .strip_suffix(")").ok_or_else(|| format!("Bad suffix: {}", wkt))?;
        
        
        let linestrings: Vec<LineString> = inner.split_inclusive("), ")
                                                .map(|coords| Self::parse_linestring(coords.trim().strip_suffix(",").unwrap_or(coords)))
                                                .collect::<Result<_, _>>()?;
                                        
        
        for linestring in linestrings.iter() {
            if &linestring.points[0] != linestring.points.last().unwrap() {
                return Err(format!("Polygon line not closed: {}", wkt))
            }
        }

        Ok(Geometry::Polygon(Polygon { rings: linestrings }))


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

        assert_eq!(Geometry::parse_point(xy_good).unwrap(), Point {x: 10.0, y: 1.0});
        assert_eq!(Geometry::parse_point(xy_bad_1), Err("Expected two numbers for coordinate pair, got: 1".to_string()));
        assert_eq!(Geometry::parse_point(xy_bad_2), Err("Invalid number: a".to_string()));
        assert_eq!(Geometry::parse_point(xy_bad_3), Err("Invalid number: 10.0,".to_string()))

    }

    #[test]
    fn parses_point_ok() {
        let point_str_good = "POINT (10.0 1.0)";
        let point_str_bad_1 = "POINT(10.0 1.0)";
        let point_str_bad_2 = "POINT (10.0 1.0";
        let point_str_bad_3 = "POINT (1.0)";
        
        assert_eq!(Geometry::parse_point_wkt(point_str_good), Ok(Geometry::Point(Point {x: 10.0, y: 1.0})));
        assert_eq!(Geometry::parse_point_wkt(point_str_bad_1), Err(format!("Bad prefix: {}", point_str_bad_1)));
        assert_eq!(Geometry::parse_point_wkt(point_str_bad_2), Err(format!("Bad suffix: {}", point_str_bad_2)));
        assert_eq!(Geometry::parse_point_wkt(point_str_bad_3), Err("Expected two numbers for coordinate pair, got: 1".to_string()));
    }

    #[test]
    fn parses_linstring_ok() {
        let linsetring_str_good = "LINESTRING (1.0 1.0, 2.0 2.0, 3.0 2.0)";
        let linestring_str_bad_1 = "LINESTRING (1.0 1.0, 2.0 2.0, 3.0 2.0";
        let linestring_str_bad_2 = "LINESTRING (1.0 1.0 2.0 2.0, 3.0 2.0)";

        assert_eq!(Geometry::parse_linestring_wkt(linsetring_str_good), Ok(Geometry::LineString(LineString{points: vec![Point {x: 1.0, y: 1.0}, Point {x: 2.0, y: 2.0}, Point {x: 3.0, y: 2.0}]})));
        assert_eq!(Geometry::parse_linestring_wkt(linestring_str_bad_1), Err(format!("Bad suffix: {}", "(1.0 1.0, 2.0 2.0, 3.0 2.0")));
        assert_eq!(Geometry::parse_linestring_wkt(linestring_str_bad_2), Err("Expected two numbers for coordinate pair, got: 4".to_string()));

    }

    #[test]
    fn parses_polygon_ok() {
        let polygon_str_good_1 = "POLYGON ((1.0 1.0, 2.0 2.0, 3.0 2.0, 1.0 1.0))";
        let linestring_vec_good_1 = vec![LineString{points: vec![Point {x: 1.0, y: 1.0}, Point {x: 2.0, y: 2.0}, Point {x: 3.0, y: 2.0}, Point {x: 1.0, y: 1.0}]}];

        let polygon_str_good_2 = "POLYGON ((3.0 0.0, 3.0 3.0, 0.0 3.0, 0.0 0.0, 3.0 0.0), (2.0 1.0, 2.0 2.0, 1.0 2.0, 1.0 1.0, 2.0 1.0))";
        let linestring_vec_good_2 = vec![LineString{points: vec![Point {x: 3.0, y: 0.0}, Point {x: 3.0, y: 3.0}, Point {x: 0.0, y: 3.0}, Point {x: 0.0, y: 0.0}, Point {x: 3.0, y: 0.0}]},
                                                          LineString{points: vec![Point {x: 2.0, y: 1.0}, Point {x: 2.0, y: 2.0}, Point {x: 1.0, y: 2.0}, Point {x: 1.0, y: 1.0}, Point {x: 2.0, y: 1.0}]}];

        assert_eq!(Geometry::parse_polygon_wkt(polygon_str_good_1), Ok(Geometry::Polygon(Polygon{rings: linestring_vec_good_1})));
        assert_eq!(Geometry::parse_polygon_wkt(polygon_str_good_2), Ok(Geometry::Polygon(Polygon{rings: linestring_vec_good_2})));
    }

}