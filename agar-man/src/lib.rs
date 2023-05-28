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

// #[global_allocator]
// static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

fn process_line(line: &String) -> String {
    line.to_lowercase()
}

fn contained(smaller: &str, larger: &str) -> bool {
    // speed up by checking characters first
    let contains_letters = smaller.chars().all(|c| larger.contains(c));

    if !contains_letters {
        return false;
    }

    let smaller_counts = to_counter(smaller);
    let larger_counts = to_counter(larger);

    let zero: usize = 0;

    return counter_contains(&larger_counts, &smaller_counts);
}

fn filter_line(line: &String, seed: &str, min_length: usize) -> bool {
    line.chars().all(char::is_alphanumeric) & (line.len() >= min_length) & contained(line, seed)
}

fn filter_grid_word(word: &String, grid_letters: &Vec<char>, min_length: usize) -> bool {
    word.chars().all(char::is_alphanumeric)
        & (word.chars().count() >= min_length)
        & contained(word, &grid_letters.iter().collect::<String>())
}

const ALPHA_SIZE: usize = 26; // a-z
const ASCII_OFFSET: usize = 97; // a's ASCII code.

type Counter = [u8; ALPHA_SIZE];
type WordProduct = u64;

/** Convert an ASCII char into an usize, such that 'a' -> 0, 'b' -> 1, ..., 'z' -> 25. */
fn to_index(c: char) -> usize {
    (c as usize) - ASCII_OFFSET
}

/** Convert a usize to an ASCII char, such that 0 -> 'a', 1 -> 'b', ..., 25 -> 'z' */
fn to_char(i: usize) -> char {
    (i + ASCII_OFFSET) as u8 as char
}

fn array_to_string(array: &Vec<usize>) -> String {
    let mut s = String::new();
    for i in 0..array.len() {
        s.push_str(&to_char(array[i]).to_string());
    }
    s
}

fn compare_char_array(a: &Vec<usize>, b: &Vec<usize>) -> bool {
    for i in 0..cmp::min(a.len(), b.len()) {
        if a[i] < b[i] {
            return true;
        } else if a[i] > b[i] {
            return false;
        }
    }
    false
}

fn to_counter(s: &str) -> Counter {
    let mut counts = [0; ALPHA_SIZE];
    for c in s.chars() {
        let i = to_index(c);
        counts[i] += 1;
    }
    counts
}

fn to_counter_indexed(s: &str, indices: &[usize; ALPHA_SIZE]) -> Counter {
    let mut counts = [0; ALPHA_SIZE];
    for c in s.chars() {
        let i = indices[to_index(c)];
        counts[i] += 1;
    }
    counts
}

fn add_counters(a: &mut Counter, b: &Counter) {
    for i in 0..ALPHA_SIZE {
        a[i] += b[i];
    }
}

fn subtract_counters(a: &mut Counter, b: &Counter) {
    for i in 0..ALPHA_SIZE {
        a[i] -= b[i];
    }
}

fn counter_contains(a: &Counter, b: &Counter) -> bool {
    for i in 0..ALPHA_SIZE {
        if a[i] < b[i] {
            return false;
        }
    }
    true
}

fn word_to_bitmask(s: &str) -> u32 {
    let mut bitmask: u32 = 0;
    for c in s.chars() {
        let i = to_index(c);
        bitmask |= 1 << i;
    }
    bitmask
}

fn counter_to_bitmask(counter: &Counter) -> u32 {
    let mut bitmask: u32 = 0;
    for i in 0..ALPHA_SIZE {
        if counter[i] != 0 {
            bitmask |= 1 << i;
        }
    }
    bitmask
}

fn bitmask_contains(a: u32, b: u32) -> bool {
    a & b == b
}

#[derive(Default)]
struct Trie {
    children: [Option<Box<Trie>>; ALPHA_SIZE], // See https://doc.rust-lang.org/book/ch15-01-box.html#enabling-recursive-types-with-boxes
    end_of_word: bool,
}

impl Trie {
    fn new() -> Self {
        Default::default()
    }

    /** Insert a word. */
    fn insert(&mut self, word: String) {
        let mut node = self;
        for i in word.chars().map(to_index) {
            node = node.children[i].get_or_insert(Box::new(Trie::new()));
        }
        node.end_of_word = true;
    }

    /** Outer anagram function */
    fn anagram(&self, seed: &String) -> Vec<String> {
        let seed_counter = to_counter(seed);

        let mut results = Vec::new();
        let mut path = Vec::new();

        let mut current_word = Vec::new();
        let mut previous_word = Vec::new();

        self.anagram_recursive(
            seed_counter,
            &mut path,
            self,
            &mut current_word,
            &mut previous_word,
            &mut results,
        );

        return results;
    }

    /** Inner anagram function */
    fn anagram_recursive(
        &self,
        mut seed_counter: Counter,
        path: &mut Vec<usize>,
        root: &Trie,
        current_word: &mut Vec<usize>,
        previous_word: &mut Vec<usize>,
        results: &mut Vec<String>,
    ) {
        if self.end_of_word {
            // if all characters have been used
            if seed_counter == [0; ALPHA_SIZE] {
                results.push(array_to_string(&path));
            }
            path.push(27);
            let old_word = previous_word.clone();
            previous_word.clear();
            previous_word.extend_from_slice(&current_word);
            current_word.clear();
            // redo search from root
            root.anagram_recursive(
                seed_counter,
                path,
                root,
                current_word,
                previous_word,
                results,
            );
            path.pop();
            current_word.clear();
            current_word.extend_from_slice(&previous_word);
            previous_word.clear();
            previous_word.extend_from_slice(&old_word);
        }

        let mut inner_loop = |i: usize| {
            let node = self.children[i].as_ref();
            // skip node if it is None
            if !node.is_some() {
                return;
            }
            let count = seed_counter[i];
            if count == 0 {
                return;
            }
            if compare_char_array(current_word, previous_word) {
                return;
            }
            seed_counter[i] -= 1; // decrement the count of the letter in the seed

            path.push(i);
            current_word.push(i);
            // continue search from node
            node.unwrap().anagram_recursive(
                seed_counter,
                path,
                root,
                current_word,
                previous_word,
                results,
            );
            path.pop(); // pop the letter from the path
            current_word.pop(); // pop the letter from the current word
            seed_counter[i] = count; // reset counter
        };

        for i in 0..ALPHA_SIZE {
            inner_loop(i);
        }
    }
}

pub fn trie_solve(seed: &str, min_length: usize) -> (Vec<String>, Vec<String>) {
    let dictionary = include_str!("dictionary.txt");

    let seed = str::replace(seed, " ", "").to_string();

    let lines: Vec<String> = dictionary.split("\n").map(str::to_string).collect();

    let filter_line_closure = |line: &String| filter_line(line, &seed, min_length);
    let processed_lines: Vec<String> = lines // using String as the return type of `to_lowercase`
        .iter()
        .map(process_line)
        .filter(filter_line_closure)
        .collect();

    let mut trie = Trie::new();
    for line in &processed_lines {
        trie.insert(line.clone());
    }

    let anagrams = trie.anagram(&seed);

    return (anagrams, processed_lines);
}

const GRID_SIZE: usize = 4;
const NUM_NODES: usize = GRID_SIZE * GRID_SIZE;

fn dfs(
    grid: &[[char; GRID_SIZE]; GRID_SIZE],
    adj_matrix: &[[bool; NUM_NODES]; NUM_NODES],
    current_node: usize,
    visited: &mut [bool; NUM_NODES],
    remaining_word: &str,
) -> bool {
    if remaining_word.is_empty() {
        return true;
    }

    visited[current_node] = true;

    let next_char = remaining_word.chars().next().unwrap();
    let next_word = &remaining_word[1..];

    for (neighbor, &connected) in adj_matrix[current_node].iter().enumerate() {
        if !connected || visited[neighbor] {
            continue;
        }

        let neighbor_char = grid[neighbor / GRID_SIZE][neighbor % GRID_SIZE];

        if next_char == neighbor_char {
            if dfs(grid, adj_matrix, neighbor, visited, next_word) {
                return true;
            }
        }
    }

    visited[current_node] = false;
    return false;
}

fn solve_grid() {
    let mut grid_strs = ["vgne", "optm", "alia", "irrh"];
    let mut grid = [[' '; GRID_SIZE]; GRID_SIZE];
    for (i, row) in grid_strs.iter().enumerate() {
        for (j, c) in row.chars().enumerate() {
            grid[i][j] = c;
        }
    }

    let mut graph = [[false; NUM_NODES]; NUM_NODES];

    // fill in graph
    for i in 0..NUM_NODES {
        let row = i / GRID_SIZE;
        let col = i % GRID_SIZE;

        // check horizontal
        if col > 0 {
            let left = i - 1;
            graph[i][left] = true;
            graph[left][i] = true;
        }

        // check vertical
        if row > 0 {
            let up = i - GRID_SIZE;
            graph[i][up] = true;
            graph[up][i] = true;
        }

        // check diagonal
        if row > 0 && col > 0 {
            let up_left = i - GRID_SIZE - 1;
            graph[i][up_left] = true;
            graph[up_left][i] = true;
        }

        if row > 0 && col < GRID_SIZE - 1 {
            let up_right = i - GRID_SIZE + 1;
            graph[i][up_right] = true;
            graph[up_right][i] = true;
        }
    }

    // // print out graph
    // for i in 0..NUM_NODES {
    //     let letter = grid[i / GRID_SIZE][i % GRID_SIZE];
    //     print!("{}: ", letter);

    //     for j in 0..NUM_NODES {
    //         // print!("{}", graph[i][j] as u8);

    //         if graph[i][j] {
    //             let next_letter = grid[j / GRID_SIZE][j % GRID_SIZE];
    //             print!("{} ", next_letter);
    //         }
    //     }
    //     println!();
    // }

    let dictionary = include_str!("dictionary.txt");
    let lines: Vec<String> = dictionary.split("\n").map(str::to_string).collect();

    let grid_chars = grid
        .iter()
        .flat_map(|row| row.iter())
        .map(|c| *c)
        .collect::<Vec<char>>();

    let filter_line_closure = |word: &String| filter_grid_word(word, &grid_chars, 3);

    let processed_lines: Vec<String> = lines // using String as the return type of `to_lowercase`
        .iter()
        .map(process_line)
        .filter(filter_line_closure)
        .collect();

    let words_set = processed_lines.iter().cloned().collect::<HashSet<String>>();

    let mut results: HashSet<String> = HashSet::new();

    // check if each word in the dictionary is a valid path
    for word in &words_set {
        let mut found = false;
        // println!("Checking {}", word);

        // check if there is a path in the graph that spells out the word
        for i in 0..NUM_NODES {
            let mut visited = [false; NUM_NODES];
            if dfs(&grid, &graph, i, &mut visited, word) {
                found = true;
                break;
            }
        }

        if found {
            results.insert(word.clone());
        }
    }

    // println!("Results: {}", results.len());
    // for result in results {
    //     println!("{}", result);
    // }
    // sort by length
    let mut results_vec = results.into_iter().collect::<Vec<String>>();
    results_vec.sort_by(|a, b| a.len().cmp(&b.len()));

    for result in results_vec {
        println!("{}", result);
    }
}

const PRIMES: [u64; 26] = [
    2, 3, 5, 7, 11, 13, 17, 19, 23, 29, 31, 37, 41, 43, 47, 53, 59, 61, 67, 71, 73, 79, 83, 89, 97,
    101,
];

#[derive(Debug, Default)]
struct CounterNode {
    product: WordProduct,
    children: Vec<(u8, CounterNode)>,
}

impl CounterNode {
    fn new() -> Self {
        Default::default()
    }

    fn insert(&mut self, counter: &Counter, counter_product: &WordProduct, index: usize) {
        let remaining_sum = counter.iter().skip(index).sum::<u8>();

        if remaining_sum == 0 {
            if self.product != 0 {
                panic!("Duplicate product");
            }
            self.product = *counter_product;
            return;
        }

        let count = counter[index];

        let mut exists = false;

        for (j, child) in &mut self.children {
            if *j == count {
                child.insert(counter, counter_product, index + 1);
                exists = true;
                break;
            }
        }

        if !exists {
            let mut new = CounterNode::new();
            new.insert(counter, counter_product, index + 1);
            self.children.push((count, new));
        }
    }

    fn sort(&mut self) {
        self.children.sort_by(|a, b| a.0.cmp(&b.0));
        for (_, child) in &mut self.children {
            child.sort();
        }
    }

    fn retrieve_anagrams(
        &self,
        target_counter: &Counter,
        index: usize,
        result_products: &mut Vec<WordProduct>,
    ) {
        if self.product != 0 {
            result_products.push(self.product);
        }

        for (child_count, child) in &self.children {
            if *child_count <= target_counter[index] {
                child.retrieve_anagrams(target_counter, index + 1, result_products);
            } else {
                // we can stop here because the children are sorted
                break;
            }
        }
    }
}

// recursively find anagrams given a target product
fn find_anagrams_counter(
    target_length: usize,
    target_counter: &mut Counter,
    product_to_length: &HashMap<WordProduct, usize, BuildHasherDefault<FxHasher>>,
    min_word_length: &usize,
    max_num_words: &usize,
    path: &mut Vec<WordProduct>,
    found_anagrams: &mut Vec<Vec<WordProduct>>,
    product_to_counter: &HashMap<WordProduct, Counter, BuildHasherDefault<FxHasher>>,
    counter_root: &CounterNode,
    min_product: WordProduct,
    cache: &mut HashMap<Counter, Vec<WordProduct>, BuildHasherDefault<FxHasher>>,
) {
    let mut products = Vec::new();

    match cache.get(target_counter) {
        None => {
            counter_root.retrieve_anagrams(target_counter, 0, &mut products);
            glidesort::sort(&mut products);
            products.reverse();
            cache.insert(target_counter.clone(), products.clone());
        }

        Some(p) => {
            products.extend(p);
        }
    };

    for product in products {
        // products are sorted in descending order, so we can stop if the product is too small
        if product < min_product {
            break;
        }

        let product_counter = product_to_counter.get(&product).unwrap();
        let product_length = product_to_length.get(&product).unwrap();

        if product_length < min_word_length {
            continue;
        }

        let new_target_length = target_length - product_length;

        if new_target_length == 0 {
            path.push(product.clone());
            found_anagrams.push(path.clone());
            path.pop();
        } else if new_target_length < *min_word_length || path.len() == *max_num_words - 1 {
            continue;
        } else {
            path.push(product.clone());

            subtract_counters(target_counter, product_counter);
            find_anagrams_counter(
                new_target_length,
                target_counter,
                product_to_length,
                min_word_length,
                max_num_words,
                path,
                found_anagrams,
                product_to_counter,
                counter_root,
                product,
                cache,
            );
            add_counters(target_counter, product_counter);
            path.pop();
        }
    }
}

fn counter_solve(
    target: &str,
    min_length: usize,
    max_num_words: usize,
) -> (Vec<String>, Vec<String>) {
    // filter out all non-abecedarian characters
    let target = target
        .chars()
        .filter(|c| c.is_ascii_alphabetic())
        .collect::<String>()
        .to_lowercase();

    let dictionary = include_str!("dictionary_counts.txt");

    let mut word_counts = Vec::new();
    for line in dictionary.split("\n") {
        if line.len() == 0 {
            continue;
        }
        let mut split = line.split("\t");
        let word = split.next().unwrap();
        let count = split.next().unwrap().parse::<u32>().unwrap();
        word_counts.push((word.to_string(), count));
    }

    let filtered_lines = word_counts // using String as the return type of `to_lowercase`
        .iter()
        .filter(|(word,count)| filter_line(word, &target, min_length))
        .map(|(word,count)| word.clone())
        .collect::<Vec<String>>();

    let counts_map = FxHashMap::from_iter(word_counts);

    let mut letter_frequencies = [0; ALPHA_SIZE];
    for line in &filtered_lines {
        // lowercase
        let line = line.to_lowercase();
        for c in line.chars() {
            let i = to_index(c);
            letter_frequencies[i] += 1;
        }
    }

    // sort primes by character frequency
    // argsort letter_frequencies
    let mut index_map: [usize; ALPHA_SIZE] = [
        0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24,
        25,
    ];

    // argsort target_counts
    let mut sorted_indices: Vec<usize> = (0..ALPHA_SIZE).collect();
    sorted_indices.sort_by(|a, b| letter_frequencies[*b].cmp(&letter_frequencies[*a]));

    // map indices to sorted indices
    for (i, j) in sorted_indices.iter().enumerate() {
        index_map[*j] = i;
    }

    // assign primes to letters
    let mut letter_primes = [0; ALPHA_SIZE];
    for (i, j) in sorted_indices.iter().enumerate() {
        letter_primes[*j] = PRIMES[i];
    }

    // hashmap of products to words
    let mut products_to_words: HashMap<WordProduct, Vec<String>, BuildHasherDefault<FxHasher>> =
        FxHashMap::default();
    let mut product_to_length = FxHashMap::default();
    let mut product_to_counter = FxHashMap::default();

    let mut root = CounterNode::new();

    for line in &filtered_lines {
        // lowercase
        let line = line.to_lowercase();
        let mut product: WordProduct = 1;
        let length = line.len();
        for c in line.chars() {
            let i = to_index(c);
            product *= letter_primes[i] as WordProduct;
        }

        let product_counter = to_counter_indexed(&line, &index_map);

        // products_by_length[length].insert(product);
        product_to_length.insert(product, length);
        product_to_counter.entry(product).or_insert(product_counter);

        if let Some(words) = products_to_words.get_mut(&product) {
            words.push(line);
        } else {
            products_to_words.insert(product, vec![line]);
        }
    }

    for (product, counter) in &product_to_counter {
        root.insert(counter, product, 0);
    }
    root.sort();

    let mut target_counter = to_counter_indexed(&target, &index_map);
    let mut found_anagrams = Vec::new();
    let mut cache = FxHashMap::default();

    find_anagrams_counter(
        target.len(),
        &mut target_counter,
        &product_to_length,
        &min_length,
        &max_num_words,
        &mut Vec::with_capacity(target.len()),
        &mut found_anagrams,
        &product_to_counter,
        &root,
        2,
        &mut cache,
    );

    found_anagrams.sort_by(|a, b| b.len().cmp(&a.len()));

    // convert to strings, expanding each product to all possible words
    let mut found_anagrams_strings = Vec::new();

    for anagram in &found_anagrams {
        let mut anagram_strings = Vec::new();
        for product in anagram {
            let words = products_to_words.get(product).unwrap();
            anagram_strings.push(words.clone());
        }
        // take the cartesian product of the words
        let expanded = anagram_strings.iter().multi_cartesian_product();

        for mut product in expanded {
            let mut string = String::new();
            let mut count_avg = 0.0;
            let mut num_words = 0;
            // sort words alphabetically
            glidesort::sort(&mut product);
            for word in product {
                string.push_str(word);
                let word_count = counts_map.get(word).unwrap();
                count_avg += *word_count as f64;
                num_words += 1;

                string.push(' ');
            }
            count_avg /= num_words as f64;

            string.pop();
            found_anagrams_strings.push((string, count_avg));
        }
    }

    // glidesort::sort(&mut found_anagrams_strings);
    glidesort::sort_by(&mut found_anagrams_strings, |a, b| {
        b.1.total_cmp(&a.1)
    });

    let found_anagrams_strings = found_anagrams_strings
        .into_iter()
        .map(|(string, count_avg)| string)
        .collect::<Vec<String>>();

    return (found_anagrams_strings, filtered_lines);
}

#[wasm_bindgen(getter_with_clone)]
pub struct ResultsStruct {
    // pub value: String, // This won't work. See working example below.
    pub anagrams: js_sys::Array,
    pub partials: js_sys::Array,
}

#[wasm_bindgen]
pub fn js_generate(seed: String, min_length: usize, max_num_words: usize) -> ResultsStruct {
    console_error_panic_hook::set_once();
    let (anagrams, partials) = counter_solve(&seed, min_length, max_num_words);

    let anagrams_js = Array::new_with_length(anagrams.len() as u32);
    for i in 0..anagrams_js.length() {
        let s = JsValue::from_str(anagrams[i as usize].as_str());
        anagrams_js.set(i, s);
    }

    let partials_js = Array::new_with_length(partials.len() as u32);
    for i in 0..partials_js.length() {
        let s = JsValue::from_str(partials[i as usize].as_str());
        partials_js.set(i, s);
    }

    return ResultsStruct {
        anagrams: anagrams_js,
        partials: partials_js,
    };
}

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
        filtered.write_all(format!("{}\t{}\n", word, count).as_bytes())
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

    glidesort::sort_by(&mut dictionary_counts, |a, b| b.1.partial_cmp(&a.1).unwrap());
    let mut out = File::create("./src/dictionary_counts.txt").unwrap();

    for (word, count) in dictionary_counts {
        out.write_all(format!("{}\t{}\n", word, count).as_bytes())
            .unwrap();
    }
}

#[allow(dead_code)]
fn main() {
    // aggregate_1grams();
    // filter_1grams();
    // assign_counts();

    let target = "misunderstanding";
    let min_length = 4;
    let max_words = 5;

    let start = Instant::now();
    counter_solve(target, min_length, max_words);
    let duration = start.elapsed();
    println!("Time elapsed: {:?}", duration);
}
