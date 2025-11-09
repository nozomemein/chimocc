use std::ffi::OsString;
use std::fs::File;

use std::path::Path;

pub fn get_file_name(args: &[String]) -> Result<(File, File), std::io::Error> {
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
