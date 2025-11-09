use core::panic;
use std::env;

use std::io::BufWriter;
use std::io::Read;
use std::io::Write;

mod analyzer;
mod generator;
mod lexer;
mod parser;
mod utils;

use generator::Generator;

use crate::lexer::TokenStream;

fn main() -> Result<(), std::io::Error> {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        panic!("Please provide a file name as an argument.");
    }

    let (mut input_file, output_file) =
        utils::get_file_name(&args).expect("Failed to get file name");

    let mut input = String::new();

    input_file
        .read_to_string(&mut input)
        .expect("It must be UTF-8");

    let tokens = lexer::Lexer::new(&input).tokenize();
    let mut token_stream = TokenStream::new(tokens.into_iter(), &input);

    let parser = parser::Parser::new();
    let program = parser.parse_program(&mut token_stream);

    let program = analyzer::Analyzer::down_program(program);

    let mut buf_writer = BufWriter::new(output_file);
    Generator::gen_head(&mut buf_writer, program)?;

    buf_writer.flush()?;

    Ok(())
}
