extern crate crc32fast;
extern crate sha2;

extern crate walkdir;

#[macro_use]
extern crate clap;

#[macro_use]
extern crate log;
extern crate env_logger;

use env_logger::{LogBuilder, LogTarget};
use log::LogLevelFilter;
use std::env;

use std::time::Instant;

use std::error::Error;
use std::fs::File;
use std::io::BufRead;
use std::io::BufReader;
use std::path::Path;
mod crc_fast;

use std::fs;

use clap::{App, Arg};

#[derive(Debug)]
struct PathHash {
    path: String,
    hash: u32,
}

impl PathHash {
    fn check(&self, _enable_timing: bool) -> bool {
        info!("ph {:?}", self);
        let path = Path::new(&self.path);

        let metadata = fs::metadata(&path).unwrap();
        let bytes = metadata.len();
        let hs = self.hash;
        extern crate time;
        let start = time::precise_time_ns();
        let starti = Instant::now();
        let hc = hash_file(path, "sfv");
        let end = time::precise_time_ns();
        let duration = (end - start) as f64;
        let duration = duration / 1_000_000_000.0;
        let bpns = bytes as f64 / duration;
        let bps = bpns;

        let endi = starti.elapsed();
        let time: f64 = endi.as_secs() as f64
            + endi.subsec_nanos() as f64 / 1_000_000_000.0;
        eprintln!(
            "{} bytes in {} seconds at {} bytes/sec",
            bytes, duration, bps
        );
        eprintln!("hashing took {}", time);
        eprintln!("mb/s is {}", (bytes as f64 / (1024.0 * 1024.0)) / time);

        info!("hash {:#x} calculated", hc);
        info!("hash {:#x} stored", hs);
        hc == hs
    }
}

fn main() {
    let matches = App::new("")
        .version(crate_version!())
        .author("Morgan Harold Bauer <me@mhbauer.com>")
        .about("checks sfv files")
        .arg(
            Arg::with_name("INPUT")
                .help("Sets the input file to use")
                .required(true)
                .index(1),
        )
        .arg(
            Arg::with_name("v")
                .short("v")
                .multiple(true)
                .help("Sets the level of verbosity"),
        )
        .arg(
            Arg::with_name("alg")
                .long("alg")
                .default_value("sfv")
                .takes_value(true)
                .help("Sets the algorithm to use for hashing"),
        )
        .arg(
            Arg::with_name("timing")
                .short("t")
                .default_value("true")
                .help("whether to calculate and print timing and throughput"),
        )
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

    let algorithm = matches.value_of("alg").unwrap();
    info!("using algorithm {}", algorithm);

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

        // find the base of the file and
        // let _base = Path::new(path).canonicalize().unwrap().parent().unwrap();

        // check for sfv ending.
        // TODO: add flag to force hashing of sfv files
        if input.ends_with(".sfv") {
            let mut phs = Vec::new();
            let reader = BufReader::new(file);
            for line in reader.lines() {
                match line {
                    Err(why) => {
                        panic!(
                            "couldn't read {}: {}",
                            display,
                            why.description()
                        )
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
                            //let filename =
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
                } else {
                    println!("bad hash for {}", ph.path)
                }
            }
        } else
        // it's not an sfv file, but an individual file and we should hash it and go away.
        {
            // we have file
            let hash = hash_file(path, algorithm);
            println!("{} hash {:#x}", path.display(), hash);
        }
    } else {
        // hash everything in the directory
        // with a --check flag, look for sfv in each directory, and run the normal checking flow.
        // with no flag, hash all the files in the directory
        use walkdir::WalkDir;
        for entry in WalkDir::new(path) {
            match entry {
                Ok(entry) => {
                    if entry.metadata().unwrap().is_dir() {
                        info!("{}", entry.path().display());
                    } else {
                        let path = entry.path();
                        let hash = hash_file(path, algorithm);
                        let metadata = path.metadata().unwrap();
                        println!("{} {:08X}", path.display(), hash);
                    }
                }
                Err(err) => println!("Error: {}", err),
            }
        }
    }
}

//extern crate indicatif;

fn hash_file(path: &Path, _hasher: &str) -> u32 {
    trace!("{} ", path.display());
    let file = match File::open(path) {
        Err(why) => {
            panic!("couldn't open {}: {}", path.display(), why.description());
        }
        Ok(file) => file,
    };

    //let metadata = file.metadata().unwrap();
    let mut reader = BufReader::with_capacity(1024 * 1024, file);

    const ONE_MEG: usize = 1024 * 1024;

    let mut current = vec![0; ONE_MEG];

    let mut num_bytes_read = ONE_MEG;
    // progress bar
    //use indicatif::ProgressBar;

    //  let bar = ProgressBar::new(metadata.len());

    let mut output = 0;
    use std::io::Read;

    if _hasher == "sfv" {
        use crc32fast::Hasher;
        let mut hasher = Hasher::new();

        while num_bytes_read == ONE_MEG {
            num_bytes_read = reader.read(&mut current[..]).unwrap();
            trace!("{}", num_bytes_read);
            hasher.update(&current[..num_bytes_read]);
            //    bar.inc(num_bytes_read as u64);
        }
        let checksum = hasher.finalize();
        output = checksum;
    }

    if _hasher == "sha256" {
        use sha2::{Digest, Sha256, Sha512};
        let mut hasher = Sha256::new();

        while num_bytes_read == ONE_MEG {
            num_bytes_read = reader.read(&mut current[..]).unwrap();
            trace!("{}", num_bytes_read);
            hasher.update(&current[..num_bytes_read]);
            //    bar.inc(num_bytes_read as u64);
        }
        let checksum = hasher.finalize();
        let (int_bytes, _) = checksum.split_at(std::mem::size_of::<u32>());
        use std::convert::TryInto;
        output = u32::from_ne_bytes(int_bytes.try_into().unwrap());
    }

    //bar.finish_and_clear();

    trace!("{} hash {:#x}", path.display(), output);
    output
}
