//! types and functions for retrieving (partial) grid data from Petra GRD files
//!
//! this would be a great place to tell the story of how this came into being,
//! or whatever people do here these days

use byteorder::{LittleEndian, ReadBytesExt};

use ndarray::{
    Array,
    Array2,
    Array3,
    ShapeBuilder,
};

use time::{
    macros::datetime,
    Duration,
    PrimitiveDateTime,
};

use std::{
    error,
    fmt,
    io::{self, Read, Seek, SeekFrom},
};

/// units of measure for a given dimension
#[derive(Copy, Clone, Debug)]
pub enum UnitOfMeasure {
    /// feet
    Feet,
    /// meters
    Meters,
}

impl UnitOfMeasure {
    fn from_code(code: u32) -> Option<UnitOfMeasure> {
        match code {
            0 => Some(UnitOfMeasure::Feet),
            1 => Some(UnitOfMeasure::Meters),
            _ => None,
        }
    }
}

/// the actual grid data of a Petra grid
#[derive(Clone, Debug)]
pub enum GridData {
    /// a rectangular (rows × columns) grid
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
#[derive(Clone, Debug)]
pub struct Grid {
    /// we think this is the version number; always 2, as far as we can tell
    pub version: u32,

    /// the grid name
    pub name: String,

    /// the "size" (rows × columns) for a rectangular grid; perhaps it's the
    /// pre-triangulation size for triangular grids?
    pub size: u32,

    /// the number of rows (in the *y* dimension) for a rectangular grid;
    /// perhaps it's the pre-triangulation row count for triangular grids?
    pub rows: u32,

    /// the number of columns (in the *x* dimension) for a rectangular grid;
    /// perhaps it's the pre-triangulation column count for triangular grids?
    pub columns: u32,

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

    /// units of measure in the *x* and *y* dimensions
    pub xyunits: UnitOfMeasure,

    /// units of measure in the *z* dimension
    pub zunits: UnitOfMeasure,

    /// date of creation (possibily of last modification?) as recorded by Petra
    pub created_date: PrimitiveDateTime,

    /// we think this is used to describe the source of the data used
    /// in gridding
    pub source_data: String,

    /// we don't know what this means; as far as we can tell, it's always
    /// `"C66"`
    pub unknown_metadata: String,

    /// we think this string describes the map projection (e.g. "TX-27C",
    /// which we're pretty sure corresponds to EPSG:32039)
    pub projection: String,

    /// we think this string describes the map datum (e.g. "NAD27")
    pub datum: String,

    /// we think this number is used to describe the gridding method, but
    /// we're not sure how
    pub grid_method: u32,

    /// likewise, we think this stores values of an enumerated describing the
    /// map projection, but we don't know how to decode it
    pub projection_code: u32,

    /// this value is logged by Petra as "CM": "central meridian" perhaps?
    /// (observed values look like plausible longitudes)
    pub cm: f64,

    /// this value is logged by Petra as "RLAT": "reference latitude" perhaps?
    /// (observed values look like plausible latitudes)
    pub rlat: f64,

    /// the actual grid data, according to its inferred format
    pub data: GridData,
}

const CM_RLAT_OFFSET: u64 = 0xb9;
const DATE_OFFSET: u64 = 0xe1;
const ROWS_COLS_OFFSET: u64 = 0x3fd;
const ZUNITS_OFFSET: u64 = 0x429;
const N_TRIANGLES_OFFSET: u64 = 0x431;
const SOURCE_OFFSET: u64 = 0x5b9;
const UNK_PROJ_DATUM_OFFSET: u64 = 0x8bf;
const GRID_OFFSET: u64 = 0x119c;

// including a null terminator; these are "fixed-width null terminated" strings
const NAME_LEN: usize = 81;
const SOURCE_LEN: usize = 246;
/* these are verrrrrry questionable and based on zero-fill in the example
 * files I had */
const UNK_LEN: usize = 2009;
const PROJ_LEN: usize = 65;
const DATUM_LEN: usize = 195;

const NAUGHTY_SPEC_REL_ERROR: f64 = 0.0001;

impl Grid { 
    /// read a Petra [Grid] from a seekable source (including a file or buffer)
    pub fn read<R: Read + Seek>(source: &mut R) -> Result<Grid, Error> {
        source.rewind()?;
        let version = source.read_u32::<LittleEndian>()?;
        let name = read_petra_string::<_, NAME_LEN>(source)?;
        let size = source.read_u32::<LittleEndian>()?;
        let xmin = source.read_f64::<LittleEndian>()?;
        let xmax = source.read_f64::<LittleEndian>()?;
        let ymin = source.read_f64::<LittleEndian>()?;
        let ymax = source.read_f64::<LittleEndian>()?;
        let xstep = source.read_f64::<LittleEndian>()?;
        let ystep = source.read_f64::<LittleEndian>()?;
        let zmin = source.read_f64::<LittleEndian>()?;
        let zmax = source.read_f64::<LittleEndian>()?;

        source.seek(SeekFrom::Start(CM_RLAT_OFFSET))?;
        let cm = source.read_f64::<LittleEndian>()?;
        let rlat = source.read_f64::<LittleEndian>()?;

        source.seek(SeekFrom::Start(DATE_OFFSET))?;
        let created_date = petra_datetime(source.read_f64::<LittleEndian>()?);

        source.seek(SeekFrom::Start(ROWS_COLS_OFFSET))?;
        let rows = source.read_u32::<LittleEndian>()?;
        let columns = source.read_u32::<LittleEndian>()?;
        let grid_method = source.read_u32::<LittleEndian>()?;
        let projection_code = source.read_u32::<LittleEndian>()?;
        let xyunits = source.read_u32::<LittleEndian>()?;
        let xyunits = UnitOfMeasure::from_code(xyunits)
          .ok_or(Error::InvalidXYUnitOfMeasure(xyunits))?;

        source.seek(SeekFrom::Start(ZUNITS_OFFSET))?;
        let zunits = source.read_u32::<LittleEndian>()?;
        let zunits = UnitOfMeasure::from_code(zunits)
          .ok_or(Error::InvalidZUnitOfMeasure(zunits))?;

        source.seek(SeekFrom::Start(N_TRIANGLES_OFFSET))?;
        let n_triangles = source.read_u32::<LittleEndian>()?;

        if rows * columns != size {
            return Err(Error::SizeMismatch(size, rows, columns));
        }

        let x_rel_err =
          (xmin + (columns - 1) as f64 * xstep - xmax).abs() / xmax;
        if x_rel_err > NAUGHTY_SPEC_REL_ERROR {
            return Err(Error::InvalidXSpec(xmin, xmax, xstep, columns));
        }

        let y_rel_err =
          (ymin + (columns - 1) as f64 * ystep - ymax).abs() / ymax;
        if y_rel_err > NAUGHTY_SPEC_REL_ERROR {
            return Err(Error::InvalidYSpec(ymin, ymax, ystep, columns));
        }

        let source_len = source.seek(SeekFrom::End(0))?;
        let data_size = source_len - GRID_OFFSET;

        if n_triangles == 0 && data_size / 8 != size as u64 {
            return Err(Error::InvalidRectangularSize(size, data_size));
        }

        if n_triangles > 0 && data_size / 72 != size as u64 {
            return Err(Error::InvalidTriangleCount(n_triangles, data_size));
        }

        source.seek(SeekFrom::Start(SOURCE_OFFSET))?;
        let source_data = read_petra_string::<_, SOURCE_LEN>(source)?;

        source.seek(SeekFrom::Start(UNK_PROJ_DATUM_OFFSET))?;
        let unknown_metadata = read_petra_string::<_, UNK_LEN>(source)?;
        let projection = read_petra_string::<_, PROJ_LEN>(source)?;
        let datum = read_petra_string::<_, DATUM_LEN>(source)?;

        source.seek(SeekFrom::Start(GRID_OFFSET))?;
        let data = if n_triangles == 0 {
            let mut buf = vec![0.0; size as usize];
            source.read_f64_into::<LittleEndian>(&mut buf[..])?;
            /* safety: we checked above that rows x columns == size and that the
             * data size matched */
            let arr = Array::from_shape_vec((rows as usize, columns as usize), buf)
              .unwrap();
            GridData::Rectangular(arr)
        } else {
            let mut buf = vec![0.0; n_triangles as usize * 9];
            source.read_f64_into::<LittleEndian>(&mut buf[..])?;
            // safety: we checked above that n_triangles x 72 was the data size
            let arr = Array::from_shape_vec(
              (n_triangles as usize, 3, 3).strides((72, 8, 24)), buf).unwrap();
            GridData::Triangular(arr)
        };

        Ok(Grid {
            version,
            name,
            size,
            rows,
            columns,
            n_triangles,
            xmin,
            xmax,
            ymin,
            ymax,
            xstep,
            ystep,
            zmin,
            zmax,
            xyunits,
            zunits,
            created_date,
            source_data,
            unknown_metadata,
            projection,
            datum,
            grid_method,
            projection_code,
            cm,
            rlat,
            data,
        })
    }
}

/// errors which may occur while reading a grid
#[derive(Debug)]
pub enum Error {
    /// an IO error
    IOError(io::Error),

    /// the metadata-indicated total grid size does not match the product
    /// of the indicated row and column counts
    SizeMismatch(/** grid size */ u32, /** rows */ u32, /** columns */ u32),

    /// the metadata-indicated *x* dimension spec is incoherent, because the
    /// step size and count do not match the stated bounds
    InvalidXSpec(
        /** minimum *x* */ f64,
        /** maximum *x* */ f64,
        /** *x* step */ f64,
        /** number of columns */ u32
    ),

    /// the metadata-indicated *y* dimension spec is incoherent, because the
    /// step size and count do not match the stated bounds
    InvalidYSpec(
        /** minimum *y* */ f64,
        /** maximum *y* */ f64,
        /** *y* step */ f64,
        /** number of rows */ u32
    ),

    /// the metadata-indicated total grid size (rows × columns) does not match
    /// the actual size of the portion of the file or data buffer from offset
    /// 0x119c to the end
    InvalidRectangularSize(
        /** metadata-indicated grid size */ u32,
        /** actual size of grid data (in bytes), should be 8 × size */ u64
     ),

    /// the metadata-indicated triangle count does not match the actual size of
    /// the portion of the file or data buffer from offset 0x119c to the end
    InvalidTriangleCount(
        /** metadata-indicated triangle count */ u32,
        /** actual size of grid data (in bytes), should be 72 × size */ u64
     ),

     /// the *x* and *y* unit-of-measure code in the metadata did not match a
     /// known value
     InvalidXYUnitOfMeasure(u32),

     /// the *z* unit-of-measure code in the metadata did not match a
     /// known value
     InvalidZUnitOfMeasure(u32),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::IOError(e) => write!(f, "I/O error: {}", e),
            Error::SizeMismatch(rows, columns, size) =>
                write!(f, "total size {} != {} rows x {} columns",
                  size, rows, columns),
            Error::InvalidXSpec(min, max, step, columns) =>
                write!(f, "invalid x spec: {} to {} by {} but {} columns",
                  min, max, step, columns),
            Error::InvalidYSpec(min, max, step, columns) =>
                write!(f, "invalid y spec: {} to {} by {} but {} columns",
                  min, max, step, columns),
            Error::InvalidRectangularSize(size, actual_size) =>
                write!(f,
                  "actual data length {} bytes does not match claimed grid size {}",
                  actual_size, size),
            Error::InvalidTriangleCount(count, actual_size) =>
                write!(f,
                  "actual data length {} bytes does not match claimed triangle count {}",
                  actual_size, count),
            Error::InvalidXYUnitOfMeasure(code) =>
                write!(f, "unknown XY unit-of-measure code {}", code),
            Error::InvalidZUnitOfMeasure(code) =>
                write!(f, "unknown Z unit-of-measure code {}", code),
        }
    }
}

impl error::Error for Error { }

impl From<io::Error> for Error {
    fn from(other: io::Error) -> Self {
        Self::IOError(other)
    }
}

/* produce a String from a Petra-grid "fixed width null-terminated" string;
 * in short, these things are ASCII right-padded with NUL. we use
 * from_utf8_lossy just to be on the safe side of weird/old encodings, and
 * yield a String containing everything up to the first NUL.
 */
fn petra_string(buf: &[u8]) -> String {
    let len = buf.iter().position(|&c| c == b'0').unwrap_or(buf.len());
    String::from_utf8_lossy(&buf[0..len]).into_owned()
}

// read from a file or buffer, as above, given a fixed width
fn read_petra_string<R: Read, const WIDTH: usize>(
  source: &mut R) -> Result<String, io::Error> {
    let mut buf = [0u8; WIDTH];
    source.read_exact(&mut buf)?;
    Ok(petra_string(&buf))
}

// Petra has a goofy date/time format (from Delphi)
const DELPHI_DATETIME_ORIGIN: PrimitiveDateTime = datetime!(1899-12-30 00:00);

fn petra_datetime(days_since_origin: f64) -> PrimitiveDateTime {
    DELPHI_DATETIME_ORIGIN + Duration::seconds_f64(days_since_origin / 86_400.0)
}
