use cry_script::run;

fn main() {
    let file_path = String::from("script.cry");
    // print!("File path: ");
    // stdout().flush().unwrap();
    // stdin().read_line(&mut file_path).unwrap();
    // if let Some('\n') = file_path.chars().next_back() {
    //     file_path.pop();
    // }
    // if let Some('\r') = file_path.chars().next_back() {
    //     file_path.pop();
    // }
    // let mut sample_size = String::new();
    // print!("Sample size: ");
    // stdout().flush().unwrap();
    // stdin().read_line(&mut sample_size).unwrap();
    // if let Some('\n') = sample_size.chars().next_back() {
    //     sample_size.pop();
    // }
    // if let Some('\r') = sample_size.chars().next_back() {
    //     sample_size.pop();
    // }
    // let mut system = System::new_all();
    // let sample_size = 1; //sample_size.parse().unwrap();

    // let mut average_time = 0;
    // let mut highest_value = 0;
    // let mut lowest_value = u128::MAX;

    // for _ in 0..sample_size {
    //     let time = run(file_path.as_str());
    //     average_time += time / sample_size;
    //     if time > highest_value {
    //         highest_value = time;
    //         // println!("new highest {}", highest_value)
    //     }
    //     if time < lowest_value {
    //         lowest_value = time;
    //         // println!("new lowest {}", lowest_value)
    //     }
    //     // system.refresh_all();
    //     // println!("mem usage: {}", system.process(sysinfo::get_current_pid().unwrap()).unwrap().memory());
    //     // println!("virtual mem usage: {}\n\n", system.process(sysinfo::get_current_pid().unwrap()).unwrap().virtual_memory())
    // }
    // println!(
    //     "Average time was {} in {} samples",
    //     average_time, sample_size
    // );
    // println!("Highest time was {}", highest_value);
    // println!("Lowest time was {}", lowest_value);
    println!(
        "\nExecuted in {}ms",
        run(file_path.as_str()) as f64 / 1_000_000_f64,
    )
}
