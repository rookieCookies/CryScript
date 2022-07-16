use std::env;

pub const ENV_FILE_NAME : &str = "CRYSCRIPT_FILE_NAME";
pub const ENV_DEV_DEBUG_EXCEPTION : &str = "CRYSCRIPT_DEV_DEBUG_EXCEPTION";
pub const ENV_DEV_DEBUG_LEXER : &str = "CRYSCRIPT_DEV_DEBUG_LEXER";
pub const ENV_DEV_DEBUG_PARSER : &str = "CRYSCRIPT_DEV_DEBUG_PARSER";
pub const ENV_DEV_DEBUG_INTERPRETER : &str = "CRYSCRIPT_DEV_DEBUG_INTERPRETER";
pub const ENV_DUMP : &str = "CRYSCRIPT_DEBUG_DUMP";
pub const ENV_NO_STD : &str = "CRYSCRIPT_NO_STD";


pub(super) fn register_environment_variables() {
    let args = env::args().map(|x| x).collect::<Vec<String>>();
    env::set_var(ENV_FILE_NAME, if args.len() < 2 {
        use std::io::{stdin,stdout,Write};
        let mut s=String::new();
        print!("File: ");
        let _=stdout().flush();
        stdin().read_line(&mut s).expect("Did not enter a correct string");
        if let Some('\n')=s.chars().next_back() {
            s.pop();
        }
        if let Some('\r')=s.chars().next_back() {
            s.pop();
        }
        s
    } else {
        args[1].clone()
    });
    env::set_var(ENV_DEV_DEBUG_EXCEPTION, args.contains(&"--dbglangexception".to_string()).to_string());
    env::set_var(ENV_DEV_DEBUG_LEXER, args.contains(&"--dbglanglexer".to_string()).to_string());
    env::set_var(ENV_DEV_DEBUG_PARSER, args.contains(&"--dbglangparser".to_string()).to_string());
    env::set_var(ENV_DEV_DEBUG_INTERPRETER, args.contains(&"--dbglanginterpreter".to_string()).to_string());
    env::set_var(ENV_DUMP, args.contains(&"--dump".to_string()).to_string());
    env::set_var(ENV_NO_STD, args.contains(&"--nostd".to_string()).to_string());
}
