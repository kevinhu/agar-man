extern crate js_sys;
extern crate wasm_bindgen;
use flate2::read::GzDecoder;
use itertools::Itertools;
use js_sys::Array;
use rustc_hash::{FxHashMap, FxHasher};
use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::hash::BuildHasherDefault;
use std::io::{BufRead, BufReader, Write};
use std::time::Instant;
use std::{cmp, fs, str};
use wasm_bindgen::prelude::*;

fn aggregate_1grams() {
    // let mut lines = Vec::new();

    // list files in ./1grams
    let paths = fs::read_dir("./data/1grams").unwrap();

    let mut total_counts = Vec::new();

    for path in paths {
        // ignore if path is not .gz
        let path = path.unwrap();
        if path.path().extension().unwrap_or_default() != "gz" {
            continue;
        }

        println!("Processing {:?}", path);

        // open file
        let file = File::open(path.path()).unwrap();
        let reader = BufReader::new(GzDecoder::new(file));

        // read lines
        for line in reader.lines() {
            let line = line.unwrap();
            let mut split = line.split("\t");
            let word = split.next().unwrap().to_string();

            let mut total = 0;

            for count in split {
                // format is (year, count, volumes)
                let mut count_split = count.split(',');

                let year = count_split.next().unwrap();
                let count = count_split.next().unwrap();

                // println!("{} {} {}", word, year, count);

                total += count.parse::<u64>().unwrap();
            }

            total_counts.push((word, total));
        }

        // println!("Total counts: {:?}", total_counts);
    }
    let mut out = File::create("./data/total_counts.txt").unwrap();

    // sort by word
    glidesort::sort_by(&mut total_counts, |a, b| a.1.cmp(&b.1));

    // write total counts to file
    for (word, count) in total_counts {
        out.write_all(format!("{}\t{}\n", word, count).as_bytes())
            .unwrap();
    }
}

fn filter_1grams() {
    // let mut lines = Vec::new();

    // read frequencies
    let file = File::open("./data/total_counts.txt").unwrap();
    let reader = BufReader::new(file);

    let mut counts = FxHashMap::default();

    for line in reader.lines() {
        let line = line.unwrap();
        let mut split = line.split("\t");
        let word = split.next().unwrap().to_string();

        // only accept if is all alphabetical
        if !word.chars().all(|c| c.is_ascii_alphabetic()) {
            continue;
        }

        let count = split.next().unwrap().parse::<u64>().unwrap();
        let lowercase = word.to_lowercase();

        // counts.insert(word.to_lowercase(), count);

        if counts.contains_key(&lowercase) {
            let old_count = counts.get(&lowercase).unwrap();
            counts.insert(lowercase, old_count + count);
        } else {
            counts.insert(lowercase, count);
        }
    }

    let mut v = Vec::from_iter(counts);
    glidesort::sort_by(&mut v, |a, b| b.1.cmp(&a.1));

    let mut filtered = File::create("./data/filtered_counts.txt").unwrap();

    for (word, count) in v {
        filtered
            .write_all(format!("{}\t{}\n", word, count).as_bytes())
            .unwrap();
    }
}

fn assign_counts() {
    let dictionary = include_str!("dictionary.txt");

    let mut counts = FxHashMap::default();
    let counts_file = File::open("./data/filtered_counts.txt").unwrap();
    let counts_reader = BufReader::new(counts_file);

    for line in counts_reader.lines() {
        let line = line.unwrap();
        let mut split = line.split("\t");
        let word = split.next().unwrap().to_string();
        let count = split.next().unwrap().parse::<u64>().unwrap();

        counts.insert(word, count);
    }

    let mut dictionary_counts = Vec::new();

    for line in dictionary.lines() {
        // println!("{}", line);
        let line = line.to_lowercase();

        let freq = *counts.get(&line).unwrap_or(&40) as f64 / 40.0;

        dictionary_counts.push((line, (freq.log2() * 100.0) as u32));
    }

    glidesort::sort_by(&mut dictionary_counts, |a, b| {
        b.1.partial_cmp(&a.1).unwrap()
    });
    let mut out = File::create("./src/dictionary_counts.txt").unwrap();

    for (word, count) in dictionary_counts {
        out.write_all(format!("{}\t{}\n", word, count).as_bytes())
            .unwrap();
    }
}
