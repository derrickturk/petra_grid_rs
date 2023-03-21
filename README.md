# petra_grid

**`petra_grid` is a library for reading grid data in the `.GRD` file
format produced by the Petra™ geological interpretation application**

---

This library is based on a lot of time spent in a hex editor looking at
grid files and poring over publicly-available documentation. It is by necessity
incomplete and incorrect, and will remain so until the file format is properly
and publicly specified.

However, the library is able to successfully read rectangular and triangulated
grids and a good portion of their metadata. Please open an issue if you have
trouble reading a grid and are able to share the grid (in GRD and exported form)
with the developer.

---

### Example usage

This library can be used to read `.GRD` grid data from a file, buffer, or
any other source implementing `std::io::Read` and `std::io::Seek`. Here's a
short program for dumping "debug" representations of grid files provided on the
command line:
```rust
use std::{
    env,
    fs::File,
    process::ExitCode,
};

use petra_grid::{Error, Grid};

fn process_grid_file(path: &String) -> Result<(), Error> {
    let mut f = File::open(path)?;
    let grid = Grid::read(&mut f)?;
    println!("{}:\n{:?}", path, grid);
    Ok(())
}

fn main() -> ExitCode {
    let args = env::args().collect::<Vec<_>>();
    match &args[..] {
        [] => {
            eprintln!("Usage: read_grid <grd-files>");
            return ExitCode::from(2);
        },

        [prog] => {
            eprintln!("Usage: {} <grd-files>", prog);
            return ExitCode::from(2);
        },

        _ => {},
    }

    let mut any_error = false;
    for path in &args[1..] {
        match process_grid_file(path) {
            Ok(()) => { },
            Err(e) => {
                eprintln!("Error reading {}: {}", path, e);
                any_error = true;
            },
        };
    }

    if any_error {
        ExitCode::from(1)
    } else {
        ExitCode::from(0)
    }
}
```

As another example, we can use [`plotters`](https://plotters-rs.github.io/home) to draw `matplotlib.pyplot.imshow`-style greyscale renders of rectangular or triangular grids. (Axis labels and so forth are left as an exercise to the reader!)
```rust
use std::{
    env,
    error::Error,
    fs::File,
    process::ExitCode,
};

use plotters::prelude::*;

use petra_grid::{Grid, GridData};

const PIXELS_PER_XY_UNIT: f64 = 1e-3;

fn greyscale(val: f64, min: f64, span: f64) -> RGBColor {
    let frac = ((val - min) / span * 255.0) as u8;
    RGBColor(frac, frac, frac)
}

fn plot_grid(grid: &Grid, path: &String) -> Result<(), Box<dyn Error>> {
    let xspan = grid.xmax - grid.xmin;
    let yspan = grid.ymax - grid.ymin;
    let zspan = grid.zmax - grid.zmin;

    let xres = (xspan * PIXELS_PER_XY_UNIT) as u32;
    let yres = (yspan * PIXELS_PER_XY_UNIT) as u32;

    let root = BitMapBackend::new(path, (xres, yres)).into_drawing_area();
    root.fill(&WHITE)?;

    match &grid.data {
        GridData::Rectangular(arr) => {
            let (rows, cols) = arr.dim();

            let mut chart = ChartBuilder::on(&root)
              .build_cartesian_2d(0..cols, 0..rows)?;

            chart.configure_mesh()
              .disable_x_mesh()
              .disable_y_mesh()
              .draw()?;

            chart.draw_series(
              arr.indexed_iter().filter_map(|((j, i), &z)| {
                  if z.is_nan() {
                      None
                  } else {
                      Some(Rectangle::new(
                        [(i, j), ((i + 1), (j + 1))],
                        greyscale(z, grid.zmin, zspan).filled()))
                  }
              })
            )?;
        },

        GridData::Triangular(arr) => {
            let mut chart = ChartBuilder::on(&root)
              .build_cartesian_2d(grid.xmin..grid.xmax, grid.ymin..grid.ymax)?;

            chart.configure_mesh()
              .disable_x_mesh()
              .disable_y_mesh()
              .draw()?;

            chart.draw_series(
              arr.outer_iter().filter_map(|tri| {
                  let mut verts = vec![];
                  let mut z_avg = 0.0;
                  for vert in tri.outer_iter() {
                      verts.push((vert[0], vert[1]));
                      z_avg += vert[2];
                  }
                  z_avg /= 3.0;

                  if z_avg.is_nan() {
                      None
                  } else {
                      Some(Polygon::new(verts,
                        greyscale(z_avg, grid.zmin, zspan).filled()))
                  }
              })
            )?;
        },
    };

    Ok(())
}

fn process_grid_file(path: &String) -> Result<(), Box<dyn Error>> {
    let mut f = File::open(path)?;
    let grid = Grid::read(&mut f)?;

    let output_path = match path.to_lowercase().strip_suffix(".grd") {
        Some(base) => format!("{}.png", base),
        None => format!("{}.png", path),
    };

    plot_grid(&grid, &output_path)
}

fn main() -> ExitCode {
    let args = env::args().collect::<Vec<_>>();
    match &args[..] {
        [] => {
            eprintln!("Usage: read_grid <grd-files>");
            return ExitCode::from(2);
        },

        [prog] => {
            eprintln!("Usage: {} <grd-files>", prog);
            return ExitCode::from(2);
        },

        _ => {},
    }

    let mut any_error = false;
    for path in &args[1..] {
        match process_grid_file(path) {
            Ok(()) => { },
            Err(e) => {
                eprintln!("Error reading {}: {}", path, e);
                any_error = true;
            },
        };
    }

    if any_error {
        ExitCode::from(1)
    } else {
        ExitCode::from(0)
    }
}
```

#### Available under the [MIT license](LICENSE)

#### (c) 2023 [dwt](https://www.github.com/derrickturk) | [terminus, LLC](https://terminusdatascience.com)
