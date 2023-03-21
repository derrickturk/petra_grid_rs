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

#### Available under the [MIT license](LICENSE)

#### (c) 2023 [dwt](https://www.github.com/derrickturk) | [terminus, LLC](https://terminusdatascience.com)
