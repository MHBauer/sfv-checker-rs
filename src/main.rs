extern crate walkdir;
extern crate crc;
extern crate clap;

use std::path::Path;

use clap::{Arg, App};


fn main() {
    println!("Hello, world!");

    let matches = App::new("")
        .version("0.0.1")
        .author("Morgan Harold Bauer <bauer.morgan@gmail.com>")
        .about("checks sfv files")
        .arg(Arg::with_name("INPUT")
             .help("Sets the input file to use")
             .required(true).index(1))
        .arg(Arg::with_name("v").short("v").multiple(true)
             .help("Sets the level of verbosity"))
        .get_matches();

    //read the file <INPUT> and then hash all the files described within



    // hash everything in the directory
    use walkdir::WalkDir;
    for entry in WalkDir::new(".") {
        match entry {
            Ok(entry) => {
                if entry.metadata().unwrap().is_dir() {
                    //println!("{}", entry.path().display());
                } else {
                    println!("{}", entry.path().display());
                    let path = entry.path();
                    // crc_file(path);
                }
            }
            Err(err) => println!("Error: {}", err),
        }
    }
}

fn crc_file(path: &Path) -> u32 {
    use std::fs::File;
    use std::io::Read;
    let mut file = File::open(path).unwrap();
    let mut c = Vec::new();
    file.read_to_end(&mut c).unwrap();

    use crc::crc32;
    // instead of byte slice buffered reader to bytes() iterator
    let x = crc32::checksum_ieee(c.as_slice());
    // TODO log this instead of printing it
    // println!("{:#x}", x);
    x
}
