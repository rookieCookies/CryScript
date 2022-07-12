use std::env;

pub const ENV_FILE_NAME : &str = "CRYSCRIPT_FILE_NAME";
pub const ENV_DEV_DEBUG_EXCEPTION : &str = "CRYSCRIPT_DEV_DEBUG_EXCEPTION";
pub const ENV_DEV_DEBUG_LEXER : &str = "CRYSCRIPT_DEV_DEBUG_LEXER";
pub const ENV_DEV_DEBUG_PARSER : &str = "CRYSCRIPT_DEV_DEBUG_PARSER";
pub const ENV_DUMP : &str = "CRYSCRIPT_DEBUG_DUMP";


pub(super) fn register_environment_variables() {
    let args = env::args().map(|x| x).collect::<Vec<String>>();
    if args.len() < 2 {
        println!("Please provide a file path.")
    }
    env::set_var(ENV_FILE_NAME, args[1].as_str());
    env::set_var(ENV_DEV_DEBUG_EXCEPTION, args.contains(&"--dbglangexception".to_string()).to_string());
    env::set_var(ENV_DEV_DEBUG_LEXER, args.contains(&"--dbglanglexer".to_string()).to_string());
    env::set_var(ENV_DEV_DEBUG_PARSER, args.contains(&"--dbglangparser".to_string()).to_string());
    env::set_var(ENV_DUMP, args.contains(&"--dump".to_string()).to_string());
}
