use core::panic;
use std::env;
use std::ffi::OsString;
use std::fs::File;
use std::io::BufWriter;
use std::io::Read;
use std::io::Write;
use std::path::Path;

mod analyzer;
mod generator;
mod lexer;
mod parser;

use generator::Generator;

use crate::lexer::TokenStream;

fn main() -> Result<(), std::io::Error> {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        panic!("Please provide a file name as an argument.");
    }

    let (mut input_file, output_file) = get_file_name(&args).expect("Failed to get file name");

    let mut input = String::new();

    input_file
        .read_to_string(&mut input)
        .expect("It must be UTF-8");

    let tokens = lexer::Lexer::new(&input).tokenize();
    let mut token_stream = TokenStream::new(tokens.into_iter(), &input);

    let parser = parser::Parser::new();
    let expr = parser.parse_expr(&mut token_stream);

    let expr = analyzer::Analyzer::down_expr(expr);

    let mut buf_writer = BufWriter::new(output_file);
    Generator::gen_head(&mut buf_writer, expr)?;

    buf_writer.flush()?;

    Ok(())
}

fn get_file_name(args: &[String]) -> Result<(File, File), std::io::Error> {
    if args.len() < 2 {
        panic!("Please provide a file name as an argument.");
    }

    let input_file_path = Path::new(&args[1]);
    let input_file = File::open(input_file_path).expect("Failed to open the file");

    let mut buffer = OsString::with_capacity(input_file_path.as_os_str().len());
    buffer.push(
        input_file_path
            .file_stem()
            .expect("Failed to get file stem"),
    );
    buffer.push(".s");

    let output_file_path = Path::new(buffer.as_os_str());
    let output_file = File::create(output_file_path).expect("Failed to create the file");

    Ok((input_file, output_file))
}
