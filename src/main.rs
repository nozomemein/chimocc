use core::panic;
use std::env;
use std::ffi::OsString;
use std::fs::File;
use std::io::Read;
use std::io::Write;
use std::path::Path;

fn main() -> Result<(), std::io::Error> {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        panic!("Please provide a file name as an argument.");
    }

    let input_file_path = Path::new(&args[1]);
    let mut input_file = File::open(input_file_path).expect("Failed to open the file");

    let mut buffer = OsString::with_capacity(input_file_path.as_os_str().len());
    buffer.push(
        input_file_path
            .file_stem()
            .expect("Failed to get file stem"),
    );
    buffer.push(".s");

    let output_file_path = Path::new(buffer.as_os_str());
    let mut output_file = File::create(output_file_path).expect("Failed to create the file");

    let mut input = String::new();

    input_file
        .read_to_string(&mut input)
        .expect("It must be UTF-8");

    writeln!(&mut output_file, ".intel_syntax noprefix")?;
    writeln!(&mut output_file, ".global main")?;
    writeln!(&mut output_file, "main:")?;
    let return_code = input
        .trim_end_matches("\n")
        .parse::<usize>()
        .expect("Only numbers are allowed");
    writeln!(&mut output_file, "    mov rax, {}", return_code)?;
    writeln!(&mut output_file, "    ret")?;

    Ok(())
}
