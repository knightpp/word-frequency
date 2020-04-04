use crossbeam::channel::bounded;
use crossbeam::channel::unbounded;
use itertools::Itertools;
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::thread;
use std::time::Instant;

const FILE_BUFFER: usize = 1024 * 1024 * 100;
const CHUNK_SIZE: usize = 1024;
const BOUNDED_CAP: usize = 1024;

fn split_by_whitespace(s: String) -> Vec<String>{
    s.split_whitespace().map(|word| {
        word
            .to_lowercase()
            .chars()
    
            .filter(|c| c.is_alphabetic())
            .collect::<String>()
    }).collect_vec()
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (s, r) = bounded::<Vec<_>>(BOUNDED_CAP);
    let (sr, rr) = unbounded();
    let file = BufReader::with_capacity(FILE_BUFFER, File::open("enwik8")?);
    let start = Instant::now();
    let iter = file
        .lines()
        .flat_map(|line| {
            split_by_whitespace(line.unwrap())
        })
        .chunks(CHUNK_SIZE);
    let mut handles = Vec::new();
    // TODO: where is bottleneck?
    //for _i in 0..8
    {
        let sr = sr.clone();
        let r = r.clone();
        handles.push(thread::spawn(move || {
            let mut dict = HashMap::new();

            for words in r.into_iter() {
                for word in words {
                    let counter = dict.entry(word).or_insert(0);
                    *counter += 1;
                }
                //println!("Thread {} -- got work", _i);
            }
            sr.send(dict).unwrap();
        }));
    }

    for chunk in &iter {
        s.send(chunk.collect::<Vec<_>>())?;
    }
    drop(s);
    drop(r);
    drop(sr);
    for h in handles {
        h.join().unwrap();
    }
    let elapsed = start.elapsed();

    let mut hm = HashMap::new();
    for dict in rr.iter() {
        for (word, freq) in dict.into_iter() {
            let counter = hm.entry(word).or_insert(0);
            *counter += freq;
        }
    }
    let mut v = hm.into_iter().collect_vec();
    v.sort_unstable_by(|a,b|a.1.cmp(&b.1).reverse());

    for (word, freq) in v.into_iter().take(10){
        println!("'{}' - {}", word, freq);
    }
    println!("Elapsed ms: {}", elapsed.as_millis());

    Ok(())
}
