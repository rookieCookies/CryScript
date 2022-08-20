use cry_script::run;

fn main() {
    let file_path = String::from("script.cry");
    println!(
        "\nExecuted in {}ms",
        run(file_path.as_str()) as f64 / 1_000_000_f64,
    );
}
