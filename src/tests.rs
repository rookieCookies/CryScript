#![allow(unused)]

use rand::{Rng, distributions::Alphanumeric};

use crate::utils::Position;
const TEST_DEPTH : usize = 1_000;

#[test]
fn test_position_line_advance() {
    assert!(Position::construct(&"This is a test\nWho could have guessed am I right?\nHonestly I don't even know why I'm doing this but fun am I right?".to_string()).line == 2);
    assert!(Position::construct(&"Well a second static test is also pretty nice innit?\nA lil unnecessary but ehh\nP.P\n\nReeee".to_string()).line == 4);
    
    for _ in 0..TEST_DEPTH {
        let mut data = String::new();
        let line_count = rand::thread_rng().gen_range(3..20);
        for _ in 0_u32..line_count {
            data.push_str(generate_random_string().as_str());
            data.push('\n')
        }
        assert!(Position::construct(&data).line == line_count.try_into().unwrap())
    }
}

#[test]
fn test_position_column_advance() {
    assert!(Position::construct(&"This is a test\nWho could have guessed am I right?\nHonestly I don't even know why I'm doing this but fun am I right?".to_string()).column == 65);
    assert!(Position::construct(&"Well a second static test is also pretty nice innit?\nA lil unnecessary but ehh\nP.P\n\nReeee".to_string()).column == 5);
    
    for _ in 0..TEST_DEPTH {
        let mut data = String::new();
        let mut last_line = String::new();
        for _ in 0_u32..rand::thread_rng().gen_range(3..100) {
            data.push('\n');
            last_line = generate_random_string();
            data.push_str(last_line.as_str());
        }
        assert!(Position::construct(&data).column == last_line.len())
    }
}

#[test]
fn test_position_line_retreat() {
}

#[test]
fn test_position_column_retreat() {
}

fn generate_random_string() -> String {
    rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(rand::thread_rng().gen_range(0..100000))
        .map(char::from)
        .collect()
}