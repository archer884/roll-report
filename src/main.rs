use std::{
    fs::OpenOptions,
    io::{self, BufReader},
    num::ParseIntError,
    str::FromStr,
};

use chrono::{DateTime, NaiveDateTime, TimeZone, Utc};
use clap::Clap;
use hashbrown::HashMap;
use io::BufRead;

#[derive(Clap, Clone, Debug)]
struct Opts {
    path: String,
}

struct Entry {
    timestamp: DateTime<Utc>,
    version: String,
    max: i32,
    values: Vec<i32>,
}

impl FromStr for Entry {
    type Err = ParseEntryError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut segments = s.split('|');

        let timestamp = Utc.from_utc_datetime(&NaiveDateTime::parse_from_str(
            segments
                .next()
                .ok_or(ParseEntryError::Format("missing timestamp"))?,
            "%F %R",
        )?);
        let version = segments
            .next()
            .ok_or(ParseEntryError::Format("missing version"))?
            .to_owned();

        let data = segments
            .next()
            .ok_or(ParseEntryError::Format("missing data segment"))?;
        let (left, right) = match data.find(':') {
            Some(idx) => (&data[..idx], &data[idx + 1..]),
            None => return Err(ParseEntryError::Format("bad data segment")),
        };

        let max = left.parse()?;
        let values: Result<Vec<_>, _> = right.split(',').map(|x| x.parse::<i32>()).collect();

        Ok(Entry {
            timestamp,
            version,
            max,
            values: values?,
        })
    }
}

#[derive(Debug, thiserror::Error)]
enum ParseEntryError {
    #[error(transparent)]
    ParseDate(#[from] chrono::ParseError),
    #[error(transparent)]
    ParseInt(#[from] ParseIntError),
    #[error("malformed entry: {0}")]
    Format(&'static str),
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let opts = Opts::parse();
    let mut map: HashMap<i32, Vec<i32>> = HashMap::new();

    for entry in read_entries(&opts.path)? {
        let entry = entry??;
        map.entry(entry.max).or_default().extend(entry.values);
    }

    for (max, values) in map {
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
