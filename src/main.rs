extern crate core;

mod lexer;
mod parser;
mod runtime;

use crate::runtime::run;
use anyhow::{bail, Context, Result};
use std::{env, fs};

fn main() -> Result<()> {
    env_logger::init();
    let args: Vec<String> = env::args().collect();

    // Check if an argument is provided
    if args.len() < 2 {
        bail!("Usage: bina <filename>");
    }

    // Read the file specified in the first argument
    let filename = &args[1];
    let contents = fs::read_to_string(filename).context("Error reading input file")?;
    let tokens = lexer::parse(&contents)?;
    //dbg!(&tokens);
    let parsed = parser::parse_input(tokens)?;
    //dbg!(&parsed);
    run(parsed)?;
    Ok(())
}
