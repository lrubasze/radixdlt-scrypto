use rand::{Rng, RngCore};
use rand_chacha::rand_core::SeedableRng;
use rand_chacha::ChaCha8Rng;

use std::{time::Instant, u128};

use clap::{arg, value_parser, Command};
use log::{debug, info, Level, LevelFilter, Log, Metadata, Record};

const MIN_LEN: usize = 0;
const MAX_LEN: usize = 1024;

static CONSOLE_LOGGER: ConsoleLogger = ConsoleLogger;

struct ConsoleLogger;

impl Log for ConsoleLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= Level::Trace
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            //println!("{} - {}", record.level(), record.args());
            println!("{}", record.args());
        }
    }

    fn flush(&self) {}
}

struct DataFuzzer {
    rng: ChaCha8Rng,
    max_steps: u32,
    max_duration: u128,
    max_len: usize,
}

impl DataFuzzer {
    fn new() -> Self {
        let rng = ChaCha8Rng::seed_from_u64(1234);
        Self {
            rng,
            max_steps: 0,
            max_duration: 0,
            max_len: MAX_LEN,
        }
    }

    fn should_stop(&self, step: u32, duration: u128) -> bool {
        if self.max_steps > 0 && step >= self.max_steps {
            return true;
        }

        if self.max_duration > 0 && duration >= self.max_duration {
            return true;
        }
        false
    }

    fn set_max_len(&mut self, len: usize) {
        self.max_len = len;
    }

    fn set_max_steps(&mut self, steps: u32) {
        self.max_steps = steps;
    }

    fn set_duration(&mut self, duration: u128) {
        self.max_duration = duration * 1000;
    }

    fn get_rand_vector(&mut self) -> Vec<u8> {
        let mut len = 0;
        if self.max_len > 0 {
            len = self.rng.gen_range(MIN_LEN..self.max_len);
        }
        let mut vec = vec![0; len];
        if len > 0 {
            self.rng.fill_bytes(vec.as_mut_slice());
        }
        vec
    }
}

fn fuzz_init() -> DataFuzzer {
    let matches = Command::new("SimpleFuzzer")
        .about("Simple data fuzzer")
        .arg(
            arg!(--iterations <VALUE> "Number of iterations")
                .required(false)
                .value_parser(value_parser!(u32)),
        )
        .arg(
            arg!(--duration <VALUE> "Number of seconds to fuzz")
                .required(false)
                .value_parser(value_parser!(u128)),
        )
        .arg(
            arg!(--"max-data-len" <VALUE> "Maximal data len")
                .required(false)
                .value_parser(value_parser!(usize)),
        )
        .arg(arg!(-v --verbose ... "Verbose").required(false))
        .get_matches();

    log::set_logger(&CONSOLE_LOGGER).unwrap();

    let mut fuzzer = DataFuzzer::new();

    if let Some(val) = matches.get_one::<u32>("iterations") {
        fuzzer.set_max_steps(*val);
    }
    if let Some(val) = matches.get_one::<u128>("duration") {
        fuzzer.set_duration(*val);
    }
    if let Some(val) = matches.get_one::<usize>("max-data-len") {
        fuzzer.set_max_len(*val);
    }
    match matches.get_count("verbose") {
        0 => log::set_max_level(LevelFilter::Info),
        1 => log::set_max_level(LevelFilter::Debug),
        _ => log::set_max_level(LevelFilter::Trace),
    }
    fuzzer
}

pub fn fuzz<F>(closure: F)
where
    F: Fn(&[u8]),
{
    let mut fuzzer = fuzz_init();

    let mut cnt = 0_u32;
    let start = Instant::now();
    while !fuzzer.should_stop(cnt, start.elapsed().as_millis()) {
        let data = fuzzer.get_rand_vector();
        debug!(
            "step= {} duration= {} ms data len={}",
            cnt,
            start.elapsed().as_millis(),
            data.len()
        );
        closure(&data);

        cnt += 1;
    }
    info!("Done {} runs in {} s ", cnt, start.elapsed().as_secs());
}
