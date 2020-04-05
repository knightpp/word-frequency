use crossbeam::channel::bounded;
use crossbeam::channel::unbounded;
use itertools::Itertools;
use regex::Regex;
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::thread;
use std::time::Instant;

const FILE_BUFFER: usize = 1024 * 1024 * 100;
const BOUNDED_CAP: usize = 1024;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (s, r) = bounded::<String>(BOUNDED_CAP);
    let (sr, rr) = unbounded();
    let file = BufReader::with_capacity(FILE_BUFFER, File::open("enwik8")?);
    let start = Instant::now();
    let iter = file.lines();

    let mut handles = Vec::new();

    for _i in 0..8 {
        let sr = sr.clone();
        let r = r.clone();
        handles.push(thread::spawn(move || {
            let word_regex = Regex::new("(?i)[a-z']+").unwrap();
            let mut dict = HashMap::new();

            for line in r.iter() {
                word_regex
                    .find_iter(&line)
                    .map(|m| m.as_str())
                    .for_each(|word| {
                        if dict.contains_key(word){
                            *dict.get_mut(word).unwrap() += 1;
                        }else{
                            dict.insert(word.to_string(), 1);
                        }
                    });
                //println!("Thread {} -- got work", _i);
            }
            sr.send(dict).unwrap();
        }));
    }

    for line in iter {
        s.send(line.unwrap())?;
    }

    drop(s);
    drop(r);
    drop(sr);
    for h in handles {
        h.join().unwrap();
    }

    let mut hm = HashMap::new();
    for dict in rr.iter() {
        for (word, freq) in dict.into_iter() {
            let counter = hm.entry(word).or_insert(0);
            *counter += freq;
        }
    }
    let mut v = hm.into_iter().collect_vec();
    v.sort_unstable_by(|a, b| a.1.cmp(&b.1).reverse());

    for (word, freq) in v.into_iter().take(10) {
        println!("'{}' - {}", word, freq);
    }
    let elapsed = start.elapsed();
    println!("Elapsed ms: {}", elapsed.as_millis());

    Ok(())
}
