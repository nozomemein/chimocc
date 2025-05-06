use core::panic;
use std::env;
use std::ffi::OsString;
use std::fs::File;
use std::io::Read;
use std::io::Write;
use std::path::Path;

mod lexer;

use crate::lexer::{BinOpToken, TokenKind, TokenStream};

fn main() -> Result<(), std::io::Error> {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        panic!("Please provide a file name as an argument.");
    }

    let (mut input_file, mut output_file) = get_file_name(&args).expect("Failed to get file name");

    let mut input = String::new();

    input_file
        .read_to_string(&mut input)
        .expect("It must be UTF-8");

    writeln!(&mut output_file, ".intel_syntax noprefix")?;
    writeln!(&mut output_file, ".global main")?;
    writeln!(&mut output_file, "main:")?;

    let tokens = lexer::tokenize(input);
    let mut token_stream = TokenStream::new(tokens.into_iter());

    writeln!(
        &mut output_file,
        "  mov rax, {}",
        token_stream.expect_number()
    )?;
    while let Some(token) = token_stream.next() {
        match *token.kind {
            TokenKind::BinOp(BinOpToken::Plus) => writeln!(
                &mut output_file,
                "  add rax, {}",
                token_stream.expect_number()
            )?,
            TokenKind::BinOp(BinOpToken::Minus) => writeln!(
                &mut output_file,
                "  sub rax, {}",
                token_stream.expect_number()
            )?,
            TokenKind::Num(_) => panic!("Unexpected `Num` token: {:?}", token.kind),
            TokenKind::Eof => break,
        }
    }
    writeln!(&mut output_file, "  ret")?;

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
