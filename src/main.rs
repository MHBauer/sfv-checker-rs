
#[macro_use]
extern crate itertools;

extern crate walkdir;
extern crate crc;

#[macro_use]
extern crate clap;


#[macro_use]
extern crate log;
extern crate env_logger;

mod crc_fast;

use std::env;
use env_logger::{LogBuilder, LogTarget};
use log::LogLevelFilter;

use std::error::Error;
use std::path::Path;
use std::fs::File;
//use std::io::Read;
use std::io::BufReader;
use std::io::BufRead;

use std::fs;

use clap::{Arg, App};

#[derive(Debug)]
struct PathHash {
    path: String,
    hash: String,
}

fn main() {
    let mut builder = LogBuilder::new();
    builder.target(LogTarget::Stdout);
    builder.filter(None, LogLevelFilter::Info);
    if env::var("RUST_LOG").is_ok() {
        builder.parse(&env::var("RUST_LOG").unwrap());
    }
    builder.init().unwrap();

    //env_logger::init().unwrap();
    info!("starting up");

    let matches = App::new("")
        .version(crate_version!())
        .author("Morgan Harold Bauer <bauer.morgan@gmail.com>")
        .about("checks sfv files")
        .arg(Arg::with_name("INPUT")
             .help("Sets the input file to use")
             .required(true).index(1))
        .arg(Arg::with_name("v").short("v").multiple(true)
             .help("Sets the level of verbosity"))
        .arg(Arg::with_name("timing").short("t")
             .help("whether to calculate and print timing and throughput"))
        .get_matches();
    match matches.occurrences_of("v") {
        0 => println!("No verbose info"),
        1 => println!("Some verbose info"),
        2 => println!("Tons of verbose info"),
        3 | _ => println!("Don't be crazy"),
    }


    let input = matches.value_of("INPUT").unwrap();
    //read the file <INPUT> and then hash all the files described within
    info!("Using input file: {}", input);
    let path = Path::new(input);
    let display = path.display();

    let metadata = fs::metadata(&path).unwrap();
    // if it's a file, we were maybe given an sfv to verify or an individual file to hash.
    
    if !metadata.is_dir() {

    let file = match File::open(&path) {
        // The `description` method of `io::Error` returns a string that
        // describes the error
        Err(why) => panic!("couldn't open {}: {}", display, why.description()),
        Ok(file) => file,
    };

        // check for sfv ending.
        // TODO: add flag to force hashing of sfv files

        if input.ends_with(".sfv") {
        
    let mut phs = Vec::new();
    let reader = BufReader::new(file);
    for line in reader.lines() {
        match line {
            Err(why) => {
                panic!("couldn't read {}: {}", display, why.description())
            }
            Ok(line) => {
                let line = line.trim();
                trace!("{} contains \n{:?}", display, line);
                if line.starts_with(";") {
                    // a comment, skip the line
                    trace!("comment {}", line);
                } else if line.is_empty() {
                    trace!("empty line");
                } else {
                    trace!("line {:?}", line);
                    // now parse the line and put it into a dictionary/map?
                    // struct is 'path-string' and 'hash value'
                    // don't check the path until later
                    let mut t = line.rsplitn(2, " ");
                    let hash = t.next().expect("should have got a string");
                    let filename = t.next().expect("maybe not this one");
                    trace!("hash {:?}", hash);
                    trace!("filename {:?}", filename);
                    let ph = PathHash {
                        path: filename.to_string(),
                        hash: hash.to_string(),
                    };
                    phs.push(ph);
                    // TODO alternate strategy, matches "ends with 8 alphanums"
                }
            }
        }
    }
    
    for ph in phs {
        info!("ph {:?}", ph);
        let path = Path::new(ph.path.as_str());

        let hc = crc_file(path);
        info!("hash {:#x} calculated", hc);
        info!("hash   {} stored", ph.hash);

        //      info!("hash {:?}", hash);
    }
        }
    }
    // for each of the shits in the dictionary, calculate the hash and
    // check it against the other thing in the struct
    else
        {
    // hash everything in the directory
    use walkdir::WalkDir;
    for entry in WalkDir::new(".") {
        match entry {
            Ok(entry) => {
                if entry.metadata().unwrap().is_dir() {
                    info!("{}", entry.path().display());
                } else {
                    //println!("{}", entry.path().display());
                    //let path = entry.path();
                    //crc_file(path);
                }
            }
            Err(err) => println!("Error: {}", err),
        }
    }
        }
}

fn crc_file(path: &Path) -> u32 {
    info!("{} ", path.display());
    let mut file = match File::open(path) {
        Err(why) => {
            panic!("couldn't open") // {}: {}", path, why.description()),
        }
        Ok(file) => file,
    };

    let metadata = file.metadata().unwrap();
    //let mut reader = BufReader::new(file);
    let reader = BufReader::with_capacity(1024 * 1024, file);
    //use crc_fast::checksum_ieee_sixteen_byte;
    //let x2 = checksum_ieee_sixteen_byte(c.as_slice());
    use crc_fast::checksum_ieee_sixteen_byte_iterator;
    //use std::fs;

    let x3 = checksum_ieee_sixteen_byte_iterator(reader,
                                                 metadata.len() as usize);
    info!("hash {:#x}", x3);

    let finalhash = x3;

    if finalhash != x3 {
        panic!("crc does not match");
    }
    // TODO log this instead of printing it
    trace!("{} hash {:#x}", path.display(), finalhash);
    finalhash
}
