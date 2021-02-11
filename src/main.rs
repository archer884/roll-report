use std::{
    fs::OpenOptions,
    io::{self, BufReader},
};

use clap::Clap;
use hashbrown::HashMap;
use io::BufRead;
use rollstat::{Entry, ParseEntryError};

#[derive(Clap, Clone, Debug)]
struct Opts {
    path: String,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let opts = Opts::parse();
    let mut map: HashMap<i32, Vec<i32>> = HashMap::new();

    for entry in read_entries(&opts.path)? {
        let entry = entry??;
        map.entry(entry.max).or_default().extend(entry.values);
    }

    let mut mapping: Vec<_> = map.into_iter().collect();
    mapping.sort_unstable_by_key(|x| x.0);

    for (max, values) in mapping {
        println!("{}: {:.03}", max, average(&values));
    }

    Ok(())
}

fn average(values: &[i32]) -> f64 {
    let sum: i32 = values.iter().copied().sum();
    sum as f64 / values.len() as f64
}

fn read_entries(
    path: &str,
) -> io::Result<impl Iterator<Item = io::Result<Result<Entry, ParseEntryError>>>> {
    let read = OpenOptions::new().read(true).create(false).open(path)?;
    let read = BufReader::new(read);
    Ok(read.lines().map(|x| x.map(|x| x.parse())))
}
