use {
    chrono::NaiveDate,
    ndarray::{Array2, Array3,},
};

/// units of measure for a given dimension
pub enum UnitOfMeasure {
    /// feet
    Feet,
    /// meters
    Meters,
}

/// the actual grid data of a Petra grid
pub enum GridData {
    /// a rectangular (rows × cols) grid
    ///
    /// each data element is a measurement in the *z* dimension; the *x* and *y*
    /// values are implicit (see the `xmin`, ...) values in [Grid]
    Rectangular(Array2<f64>),

    /// a triangular (n_triangles × 3 vertices × 3 dimensions) grid
    ///
    /// each triangle is represented as (*x*, *y*, *z*) triplets; we think
    /// (but haven't verified) that triangles are stored with their vertices in
    /// counterclockwise order, because they seem to work over in Python-land
    /// with `matplotlib.tri.Triangulation`
    Triangular(Array3<f64>),
}

/// a Petra grid
pub struct Grid {
    /// we think this is the version number; always 2, as far as we can tell
    pub version: u32,

    /// the grid name
    pub name: String,

    /// the "size" (rows × cols) for a rectangular grid; perhaps it's the
    /// pre-triangulation size for triangular grids?
    pub size: u32,

    /// the number of rows (in the *y* dimension) for a rectangular grid;
    /// perhaps it's the pre-triangulation row count for triangular grids?
    pub rows: u32,

    /// the number of columns (in the *x* dimension) for a rectangular grid;
    /// perhaps it's the pre-triangulation column count for triangular grids?
    pub cols: u32,

    /// the number of triangles; zero for rectangular grids
    pub n_triangles: u32,

    /// minimum bound in the *x* dimension
    pub xmin: f64,

    /// maximum bound in the *x* dimension
    pub xmax: f64,

    /// minimum bound in the *y* dimension
    pub ymin: f64,

    /// maximum bound in the *y* dimension
    pub ymax: f64,

    /// step in the *x* dimension
    pub xstep: f64,

    /// step in the *y* dimension
    pub ystep: f64,

    /// minimum value in the *z* dimension
    pub zmin: f64,

    /// maximum value in the *z* dimension
    pub zmax: f64,

    pub xyunits: UnitOfMeasure,
    pub zunits: UnitOfMeasure,

    pub created_date: NaiveDate,
    pub source_data: String,
    pub unknown_metadata: String,

    pub projection: String,
    pub datum: String,
    pub grid_method: u32,
    pub projection_code: u32,
    pub cm: f64,
    pub rlat: f64,
}
