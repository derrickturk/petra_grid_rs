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
                eprintln!("Error: {}", e);
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
