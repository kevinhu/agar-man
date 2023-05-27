extern crate js_sys;
extern crate wasm_bindgen;
use itertools::Itertools;
use js_sys::Array;
use nohash_hasher::{IntMap, IntSet, NoHashHasher};
use rug::integer::IntegerExt64;
use rug::Integer;
use std::collections::{BTreeSet, HashMap, HashSet};
use std::hash::BuildHasherDefault;
use std::ops::Bound::Included;
use std::time::Instant;
use std::u64::MAX;
use std::{cmp, str};
use wasm_bindgen::prelude::*;
use rustc_hash::{FxHashMap, FxHasher};

#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

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

fn filter_line(line: &String, seed: &String, min_length: usize) -> bool {
    line.chars().all(char::is_alphanumeric)
        & (line.chars().count() >= min_length)
        & contained(line, seed)
}

fn filter_grid_word(word: &String, grid_letters: &Vec<char>, min_length: usize) -> bool {
    word.chars().all(char::is_alphanumeric)
        & (word.chars().count() >= min_length)
        & contained(word, &grid_letters.iter().collect::<String>())
}

const ALPHA_SIZE: usize = 26; // a-z
const ASCII_OFFSET: usize = 97; // a's ASCII code.

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

pub fn generate(seed: String, min_length: usize) -> (Vec<String>, Vec<String>) {
    let dictionary = include_str!("dictionary.txt");

    let seed = str::replace(seed.as_str(), " ", "").to_string();

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

#[wasm_bindgen(getter_with_clone)]
pub struct ResultsStruct {
    // pub value: String, // This won't work. See working example below.
    pub anagrams: js_sys::Array,
    pub partials: js_sys::Array,
}

#[wasm_bindgen]
pub fn js_generate(seed: String, min_length: usize) -> ResultsStruct {
    console_error_panic_hook::set_once();
    let (anagrams, partials) = generate(seed.into(), min_length.into());

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

fn solve_anagrams() {
    let start = Instant::now();
    let (anagrams, partials) = generate("misunderstanding".to_string(), 4);
    let duration = start.elapsed();
    println!("Time elapsed: {:?}", duration);
    println!("Anagrams: {}", anagrams.len());
    println!("Partials: {}", partials.len());
    // for anagram in anagrams {
    //     println!("{}", anagram);
    // }
}

const PRIMES: [u64; 26] = [
    2, 3, 5, 7, 11, 13, 17, 19, 23, 29, 31, 37, 41, 43, 47, 53, 59, 61, 67, 71, 73, 79, 83, 89, 97,
    101,
];

// recursively find anagrams given a target product
fn find_anagrams(
    target: &Integer,
    target_length: usize,
    target_counter: &mut Counter,
    products_by_length: &Vec<BTreeSet<u64>>,
    min_word_length: usize,
    max_num_words: usize,
    path: &mut Vec<u64>,
    max: u64,
    found_anagrams: &mut Vec<Vec<u64>>,
    product_to_bitmask: &IntMap<u64, u32>,
    product_to_counter: &IntMap<u64, Counter>,
) {
    if target_length == 0 {
        found_anagrams.push(path.clone());
        return;
    }

    // if there are no possible words, return
    if target_length < min_word_length {
        return;
    }

    // if we have reached the max number of words but still need more letters, return
    if path.len() == max_num_words && target_length != 0 {
        return;
    }

    let target_bitmask = counter_to_bitmask(target_counter);

    // otherwise, try to find a factor of the target
    for i in 2..=target_length {
        for product in products_by_length[i].range((Included(&1), Included(&max))) {
            if !bitmask_contains(target_bitmask, product_to_bitmask[&product]) {
                continue;
            }

            let product_counter = &product_to_counter[&product];

            if !counter_contains(target_counter, product_counter) {
                continue;
            }

            if target.is_divisible_u64(*product) {
                path.push(product.clone());
                subtract_counters(target_counter, product_counter);
                find_anagrams(
                    &target.clone().div_exact_u64(*product),
                    target_length - i,
                    target_counter,
                    products_by_length,
                    min_word_length,
                    max_num_words,
                    path,
                    *product,
                    found_anagrams,
                    product_to_bitmask,
                    product_to_counter,
                );
                add_counters(target_counter, product_counter);
                path.pop();
            }
        }
    }
}

fn factored_anagram_solve() {
    let target = "misunderstanding".to_string();
    let min_word_length = 3;
    let max_num_words = 10;

    let dictionary = include_str!("dictionary.txt");
    let lines: Vec<String> = dictionary.split("\n").map(str::to_string).collect();

    println!("Dictionary size: {}", lines.len());

    let filter_line_closure = |line: &String| filter_line(line, &target, min_word_length);
    let filtered_lines: Vec<String> = lines // using String as the return type of `to_lowercase`
        .iter()
        .map(process_line)
        .filter(filter_line_closure)
        .collect();

    println!("Filtered words: {}", filtered_lines.len());

    let mut letter_frequencies = [0; ALPHA_SIZE];
    for line in &filtered_lines {
        // lowercase
        let line = line.to_lowercase();
        for c in line.chars() {
            let i = to_index(c);
            letter_frequencies[i] += 1;
        }
    }

    println!("Letter frequencies: {:?}", letter_frequencies);

    // sort primes by character frequency
    // argsort letter_frequencies
    let mut sorted_indices: Vec<usize> = (0..ALPHA_SIZE).collect();
    sorted_indices.sort_by(|a, b| letter_frequencies[*b].cmp(&letter_frequencies[*a]));

    println!("Sorted indices: {:?}", sorted_indices);

    // assign primes to letters
    let mut letter_primes = [0; ALPHA_SIZE];
    for (i, j) in sorted_indices.iter().enumerate() {
        letter_primes[*j] = PRIMES[i];
    }

    println!("Letter primes: {:?}", letter_primes);

    // hashmap of products to words
    let mut products_to_words: HashMap<u64, Vec<String>> = HashMap::new();
    let mut products_by_length: Vec<BTreeSet<u64>> = vec![BTreeSet::new(); target.len() + 1];

    let mut product_to_bitmask = IntMap::<u64, u32>::with_capacity_and_hasher(
        1000000,
        BuildHasherDefault::<NoHashHasher<u64>>::default(),
    );
    let mut product_to_counter = IntMap::<u64, Counter>::with_capacity_and_hasher(
        1000000,
        BuildHasherDefault::<NoHashHasher<u64>>::default(),
    );

    let mut max_product = 1;
    let mut max_word = "".to_string();

    for line in &filtered_lines {
        // lowercase
        let line = line.to_lowercase();
        let mut product: u64 = 1;
        let length = line.len();
        for c in line.chars() {
            let i = to_index(c);
            product *= letter_primes[i];
        }

        if product > max_product {
            max_product = product;
            max_word = line.clone();
        }

        products_by_length[length].insert(product);
        product_to_bitmask
            .entry(product)
            .or_insert(word_to_bitmask(&line));
        product_to_counter
            .entry(product)
            .or_insert(to_counter(&line));

        if let Some(words) = products_to_words.get_mut(&product) {
            words.push(line);
        } else {
            products_to_words.insert(product, vec![line]);
        }
    }

    println!("Max product: {}", max_product);
    println!("Max word: {}", max_word);

    // print max product

    let mut target_product: Integer = Integer::from(1);
    for c in target.chars() {
        let i = to_index(c);
        target_product *= letter_primes[i];
    }

    let mut found_anagrams: Vec<Vec<u64>> = Vec::new();
    let mut path: Vec<u64> = Vec::with_capacity(target.len());

    find_anagrams(
        &target_product,
        target.len(),
        &mut to_counter(&target),
        &products_by_length,
        min_word_length,
        max_num_words,
        &mut path,
        target_product.to_u64().unwrap_or(MAX),
        &mut found_anagrams,
        &product_to_bitmask,
        &product_to_counter,
    );

    println!("Found anagrams: {}", found_anagrams.len());

    // let duration = start.elapsed();
    // println!("Time elapsed: {:?}", duration);

    let mut all_combos = Vec::new();

    // convert anagrams to words, with all possible combinations
    for anagram in found_anagrams {
        let mut anagram_words: Vec<Vec<String>> = Vec::new();
        for product in anagram {
            if let Some(words) = products_to_words.get(&product) {
                anagram_words.push(words.clone());
            }
        }
        let combinations = anagram_words.iter().multi_cartesian_product();

        for combination in combinations {
            let mut anagram = String::new();
            for word in combination {
                anagram.push_str(word);
                anagram.push(' ');
            }
            // println!("{}", anagram);
            all_combos.push(anagram);
        }
    }

    println!("All combinations: {}", all_combos.len());

}

type Counter = [u8; ALPHA_SIZE];

#[derive(Debug, Default)]
struct CounterNode {
    product: u64,
    children: Vec<(u8, CounterNode)>,
}

impl CounterNode {
    fn new() -> Self {
        Default::default()
    }

    fn insert(&mut self, counter: &Counter, counter_product: &u64, index: usize) {
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
        result_products: &mut Vec<u64>,
        // min_product: &u64,
    ) {
        // if self.product >= *min_product {
            result_products.push(self.product);
        // }

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
    target_product: &Integer,
    target_length: usize,
    target_counter: &mut Counter,
    product_to_length: &HashMap<u64, usize, BuildHasherDefault<FxHasher>>,
    min_word_length: &usize,
    max_num_words: &usize,
    path: &mut Vec<u64>,
    found_anagrams: &mut Vec<Vec<u64>>,
    product_to_counter:&HashMap<u64, Counter, BuildHasherDefault<FxHasher>>,
    counter_root: &CounterNode,
    min_product: u64,
    cache: &mut HashMap<Integer, Vec<u64>, BuildHasherDefault<FxHasher>>,
) {

    let mut products = Vec::new();

    match cache.get(&target_product) {
        None => {
            counter_root.retrieve_anagrams(target_counter, 0, &mut products);
            glidesort::sort(&mut products);
            products.reverse();
            cache.insert(target_product.clone(), products.clone());
        }
        
        Some(p) => {
            products.extend(p);
        }
    };

    for product in products {
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
                &target_product.clone().div_exact_u64(product),
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

fn counter_solve() {
    let target = "misunderstanding".to_string();
    let min_word_length = 3;
    let max_num_words = 10;

    let dictionary = include_str!("dictionary.txt");
    let lines: Vec<String> = dictionary.split("\n").map(str::to_string).collect();

    println!("Dictionary size: {}", lines.len());

    let filter_line_closure = |line: &String| filter_line(line, &target, min_word_length);
    let filtered_lines: Vec<String> = lines // using String as the return type of `to_lowercase`
        .iter()
        .map(process_line)
        .filter(filter_line_closure)
        .collect();

    println!("Filtered words: {}", filtered_lines.len());

    let mut letter_frequencies = [0; ALPHA_SIZE];
    for line in &filtered_lines {
        // lowercase
        let line = line.to_lowercase();
        for c in line.chars() {
            let i = to_index(c);
            letter_frequencies[i] += 1;
        }
    }

    println!("Letter frequencies: {:?}", letter_frequencies);

    // sort primes by character frequency
    // argsort letter_frequencies
    let mut index_map: [usize; ALPHA_SIZE] = [
        0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24,
        25,
    ];

    let target_counts = to_counter(&target);
    println!("Target counts: {:?}", target_counts);

    // argsort target_counts
    let mut sorted_indices: Vec<usize> = (0..ALPHA_SIZE).collect();
    sorted_indices.sort_by(|a, b| letter_frequencies[*b].cmp(&letter_frequencies[*a]));

    // map indices to sorted indices
    for (i, j) in sorted_indices.iter().enumerate() {
        index_map[*j] = i;
    }

    println!("Ordered indices: {:?}", index_map);
    let ordered_target_counts = to_counter_indexed(&target, &index_map);
    println!("Ordered target counts: {:?}", ordered_target_counts);

    println!("Sorted indices: {:?}", index_map);

    // assign primes to letters
    let mut letter_primes = [0; ALPHA_SIZE];
    for (i, j) in sorted_indices.iter().enumerate() {
        letter_primes[*j] = PRIMES[i];
    }

    println!("Letter primes: {:?}", letter_primes);

    // hashmap of products to words
    let mut products_to_words: HashMap<u64, Vec<String>, BuildHasherDefault<FxHasher>> = FxHashMap::default();
    let mut product_to_length = FxHashMap::default();
    let mut product_to_counter = FxHashMap::default();

    let mut root = CounterNode::new();

    for line in &filtered_lines {
        // lowercase
        let line = line.to_lowercase();
        let mut product: u64 = 1;
        let length = line.len();
        for c in line.chars() {
            let i = to_index(c);
            product *= letter_primes[i];
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

    println!("Products: {:?}", product_to_counter.len());

    let mut target_counter = to_counter_indexed(&target, &index_map);

    println!("Target counter: {:?}", target_counter);

    let mut result_products = Vec::new();

    root.retrieve_anagrams(&target_counter, 0, &mut result_products);

    println!("Result products: {:?}", result_products.len());

    let mut found_anagrams = Vec::new();

    let mut target_product: Integer = Integer::from(1);
    for c in target.chars() {
        let i = to_index(c);
        target_product *= letter_primes[i];
    }

    let mut cache = FxHashMap::default();

    find_anagrams_counter(
        &target_product,
        target.len(),
        &mut target_counter,
        &product_to_length,
        &min_word_length,
        &max_num_words,
        &mut Vec::with_capacity(target.len()),
        &mut found_anagrams,
        &product_to_counter,
        &root,
        2,
        &mut cache
    );

    println!("Found anagrams: {:?}", found_anagrams.len());
}

#[allow(dead_code)]
fn main() {
    // let start = Instant::now();
    // solve_anagrams();
    // let duration = start.elapsed();
    // println!("Time elapsed: {:?}", duration);

    let start = Instant::now();
    factored_anagram_solve();
    let duration = start.elapsed();
    println!("Time elapsed: {:?}", duration);

    let start = Instant::now();
    counter_solve();
    let duration = start.elapsed();
    println!("Time elapsed: {:?}", duration);
}
