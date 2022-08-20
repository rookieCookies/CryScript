use std::{
    env::set_current_dir,
    fs::{self},
};

use cry_script::run;

fn main() {
    let mut x = fs::canonicalize(std::env::current_exe().unwrap()).unwrap();
    x.pop();
    set_current_dir(x).unwrap();
    println!(
        "Executed in {} milliseconds",
        run("script.cry") as f64 / 1_000_000.
    )
}
