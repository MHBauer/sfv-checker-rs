
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
    hash: u32,
}

impl PathHash {
    fn check(&self, enable_timing: bool) -> bool {
        info!("ph {:?}", self);
        let path = Path::new(&self.path);
        //let hs = self.hash.parse::<u32>().unwrap();
        let hs = self.hash;
        let hc = crc_file(path);
        info!("hash {:#x} calculated", hc);
        info!("hash {:#x} stored", hs);
        hc == hs
    }
}

fn main() {

    let matches = App::new("")
        .version(crate_version!())
        .author("Morgan Harold Bauer <bauer.morgan@gmail.com>")
        .about("checks sfv files")
        .arg(Arg::with_name("INPUT")
             .help("Sets the input file to use")
             .required(true).index(1))
        .arg(Arg::with_name("v").short("v").multiple(true)
             .help("Sets the level of verbosity"))
        .arg(Arg::with_name("timing").short("t").default_value("true")
             .help("whether to calculate and print timing and throughput"))
        .get_matches();

    let mut builder = LogBuilder::new();
    builder.target(LogTarget::Stdout);

    match matches.occurrences_of("v") {
        0 => {
            builder.filter(None, LogLevelFilter::Error);
        }
        1 => {
            builder.filter(None, LogLevelFilter::Warn);
        }
        2 => {
            builder.filter(None, LogLevelFilter::Info);
        }
        3 => {
            builder.filter(None, LogLevelFilter::Debug);
        }
        _ => {
            builder.filter(None, LogLevelFilter::Trace);
        }
    }

    if env::var("RUST_LOG").is_ok() {
        builder.parse(&env::var("RUST_LOG").unwrap());
    }
    builder.init().unwrap();

    info!("starting up");
    let enable_timing = value_t!(matches, "timing", bool).unwrap_or(false);

    let input = matches.value_of("INPUT").unwrap();
    //read the file <INPUT> and then hash all the files described
    // within
    info!("Using input file: {}", input);
    let path = Path::new(input);
    let display = path.display();

    let metadata = fs::metadata(&path).unwrap();
    // if it's a file, we were maybe given an sfv to verify or an
    // individual file to hash.

    if !metadata.is_dir() {

        let file = match File::open(&path) {
            // The `description` method of `io::Error` returns a
            // string that describes the error
            Err(why) => {
                panic!("couldn't open {}: {}", display, why.description())
            }
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
                        panic!("couldn't read {}: {}",
                               display,
                               why.description())
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
                            // TODO alternate strategy, matches "ends with 8 alphanums"
                            let mut t = line.rsplitn(2, " ");
                            let hash =
                                t.next().expect("should have got a string");
                            let filename =
                                t.next().expect("maybe not this one");
                            trace!("hash {:?}", hash);
                            trace!("filename {:?}", filename);

                            match u32::from_str_radix(&hash, 16) {
                                // if we get a bad looking hash skip it
                                Err(_) => {
                                    warn!("couldn't parse the hash value {}, skipping file {}",
                                          hash,
                                          filename)
                                }
                                Ok(hs) => {
                                    let ph = PathHash {
                                        path: filename.to_string(),
                                        hash: hs,
                                    };
                                    phs.push(ph);
                                }
                            }
                        }
                    }
                }
            }

            for ph in phs {
                if ph.check(enable_timing) {
                    println!("good hash for {}", ph.path)
                }
                else {
                    println!("bad hash for {}", ph.path)
                }
            }
        }
    } else {
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
    trace!("{} ", path.display());
    let mut file = match File::open(path) {
        Err(why) => {
            panic!("couldn't open") // {}: {}", path, why.description()),
        }
        Ok(file) => file,
    };

    let metadata = file.metadata().unwrap();
    let reader = BufReader::with_capacity(1024 * 1024, file);
    use crc_fast::checksum_ieee_sixteen_byte_iterator;
    let hc = checksum_ieee_sixteen_byte_iterator(reader,
                                                 metadata.len() as usize);
    trace!("{} hash {:#x}", path.display(), hc);
    hc
}
