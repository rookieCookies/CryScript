use std::{
    env::set_current_dir,
    fs::{self},
};

use cry_script::run;

fn main() {
    let mut x = fs::canonicalize(std::env::current_exe().unwrap()).unwrap();
    x.pop();
    set_current_dir(x).unwrap();

    // let iterations = 1;
    // let mut number = 0.0;
    // for _ in 0..iterations {
    //     number += run("script.cry") as f64 / 1_000_000.
    // }
    // println!(
    //     "Executed in {} milliseconds",
    //     number / iterations as f64
    // )

    println!(
        "Executed in {} milliseconds",
        run("script.cry") as f64 / 1_000_000.
    )
}
