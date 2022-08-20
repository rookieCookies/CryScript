use std::{fs::{File, self}, env::set_current_dir};

use cry_script::run;

fn main() {
    let mut x = fs::canonicalize(std::env::current_exe().unwrap()).unwrap();
    x.pop();
    println!("{:?}", x);
    set_current_dir(x).unwrap();
    File::open("hello.txt").unwrap();
}
