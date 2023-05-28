extern crate js_sys;
extern crate wasm_bindgen;
use itertools::Itertools;
use js_sys::Array;
use rustc_hash::{FxHashMap, FxHasher};
use std::collections::{HashMap, HashSet};
use std::hash::BuildHasherDefault;
use std::str;
use std::time::Instant;
use wasm_bindgen::prelude::*;

// #[global_allocator]
// static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

fn contained(smaller: &str, larger: &str) -> bool {
    // speed up by checking characters first
    let contains_letters = smaller.chars().all(|c| larger.contains(c));

    if !contains_letters {
        return false;
    }

    let smaller_counts = to_counter(smaller);
    let larger_counts = to_counter(larger);

    return counter_contains(&larger_counts, &smaller_counts);
}

fn filter_line(line: &String, seed: &str, min_length: usize, excludes: &HashSet<String>) -> bool {
    line.chars().all(char::is_alphanumeric)
        & (line.len() >= min_length)
        & contained(line, seed)
        & !excludes.contains(&line.to_lowercase())
}

const ALPHA_SIZE: usize = 26; // a-z
const ASCII_OFFSET: usize = 97; // a's ASCII code.
const MAX_WORD_LENGTH: usize = 16;

type EncodedWord = [i8; MAX_WORD_LENGTH];
type Counter = [i8; ALPHA_SIZE];
type WordProduct = u64;

/** Convert an ASCII char into an usize, such that 'a' -> 0, 'b' -> 1, ..., 'z' -> 25. */
fn to_index(c: char) -> usize {
    (c as usize) - ASCII_OFFSET
}

fn to_index_i8(c: char) -> i8 {
    ((c as usize) - ASCII_OFFSET) as i8
}

fn encode_word(word: &str) -> EncodedWord {
    let mut encoded_word = [-1; MAX_WORD_LENGTH];
    for (i, c) in word.chars().enumerate() {
        encoded_word[i] = to_index_i8(c);
    }
    encoded_word
}

fn decode_and_extend_word(encoded_word: &EncodedWord, string: &mut String) {
    for i in 0..encoded_word.len() {
        let c = encoded_word[i];
        if c == -1 {
            break;
        }
        string.push(to_char(c as usize));
    }
    string.push(' ');
}

/** Convert a usize to an ASCII char, such that 0 -> 'a', 1 -> 'b', ..., 25 -> 'z' */
fn to_char(i: usize) -> char {
    (i + ASCII_OFFSET) as u8 as char
}

fn to_counter(s: &str) -> Counter {
    let mut counts: Counter = [0; ALPHA_SIZE];
    for c in s.chars() {
        let i = to_index(c);
        counts[i] += 1;
    }
    counts
}

fn to_counter_indexed(s: &str, indices: &[usize; ALPHA_SIZE]) -> Counter {
    let mut counts: Counter = [0; ALPHA_SIZE];
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

fn is_partial_anagram(a: &str, b: &str) -> bool {
    let mut a_counts = to_counter(a);
    let b_counts = to_counter(b);
    return counter_contains(&a_counts, &b_counts);
}

const PRIMES: [u64; 26] = [
    2, 3, 5, 7, 11, 13, 17, 19, 23, 29, 31, 37, 41, 43, 47, 53, 59, 61, 67, 71, 73, 79, 83, 89, 97,
    101,
];

#[derive(Debug, Default)]
struct CounterNode {
    product: WordProduct,
    children: Vec<(i8, CounterNode)>,
}

impl CounterNode {
    fn new() -> Self {
        Default::default()
    }

    fn insert(&mut self, counter: &Counter, counter_product: &WordProduct, index: usize) {
        let remaining_sum = counter.iter().skip(index).sum::<i8>();

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
    excludes: &HashSet<String>,
    includes: &Vec<String>,
    top_n: usize,
) -> (Vec<String>, Vec<String>) {
    // filter out all non-abecedarian characters
    let mut target = target
        .chars()
        .filter(|c| c.is_ascii_alphabetic())
        .collect::<String>()
        .to_lowercase();

    for included in includes {
        if !is_partial_anagram(&target, included) {
            return (Vec::new(), Vec::new());
        }

        for c in included.chars() {
            target = target.replacen(&c.to_string(), "", 1);
        }
    };

    let dictionary = include_str!("dictionary_counts.txt");

    let mut word_counts = Vec::new();
    for line in dictionary.split("\n").take(top_n) {
        if line.len() == 0 {
            continue;
        }
        let mut split = line.split("\t");
        let word = split.next().unwrap().to_lowercase();
        let count = split.next().unwrap().parse::<u32>().unwrap();
        word_counts.push((word.to_string(), count));
    }

    let filtered_lines = word_counts // using String as the return type of `to_lowercase`
        .iter()
        .filter(|(word, _)| filter_line(word, &target, min_length, excludes))
        .map(|(word, _)| word.clone())
        .collect::<Vec<String>>();

    let counts_map = FxHashMap::from_iter(
        word_counts
            .iter()
            .map(|(word, count)| (encode_word(word), count)),
    );

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
    let mut product_to_words: HashMap<WordProduct, Vec<EncodedWord>, BuildHasherDefault<FxHasher>> =
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

        if let Some(words) = product_to_words.get_mut(&product) {
            words.push(encode_word(&line));
        } else {
            product_to_words.insert(product, vec![encode_word(&line)]);
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
        &(max_num_words - includes.len()),
        &mut Vec::with_capacity(target.len()),
        &mut found_anagrams,
        &product_to_counter,
        &root,
        2,
        &mut cache,
    );

    // convert to strings, expanding each product to all possible words
    let mut found_anagrams_strings = Vec::new();

    for anagram in &found_anagrams {
        let mut anagram_strings = Vec::new();

        for product in anagram {
            let words = product_to_words.get(product).unwrap();
            anagram_strings.push(words.clone());
        }
        // take the cartesian product of the words
        let expanded = anagram_strings.iter().multi_cartesian_product();

        for product in expanded {
            let mut string = String::new();

            for included in includes {
                string.push_str(included);
                string.push(' ');
            }

            let mut count_avg = 0.0;
            let mut num_words = 0;
            for word in product {
                let word_count = counts_map.get(word).unwrap();
                decode_and_extend_word(word, &mut string);
                count_avg += **word_count as f32;
                num_words += 1;
            }
            count_avg /= num_words as f32;

            string.pop();
            found_anagrams_strings.push((string, count_avg));
        }
    }

    glidesort::sort_by(&mut found_anagrams_strings, |a, b| b.1.total_cmp(&a.1));

    let found_anagrams_strings = found_anagrams_strings
        .into_iter()
        .map(|(string, _)| string)
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
pub fn js_generate(
    seed: String,
    min_length: usize,
    max_num_words: usize,
    excludes: String,
    includes: String,
    top_n: usize,
) -> ResultsStruct {
    console_error_panic_hook::set_once();
    let excludes = excludes
        .trim()
        .split(",")
        .map(|x| x.trim().to_lowercase())
        .filter(|x| !x.is_empty())
        .collect::<HashSet<_>>();
    let includes = includes
        .trim()
        .split(",")
        .map(|x| x.trim().to_lowercase())
        .filter(|x| !x.is_empty())
        .collect::<Vec<_>>();

    let (anagrams, partials) = counter_solve(
        &seed,
        min_length,
        max_num_words,
        &excludes,
        &includes,
        top_n,
    );

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

#[allow(dead_code)]
fn main() {
    // aggregate_1grams();
    // filter_1grams();
    // assign_counts();

    let target = "village technologies";
    let min_length = 2;
    let max_words = 10;

    let start = Instant::now();
    let results = counter_solve(
        target,
        min_length,
        max_words,
        &HashSet::default(),
        &vec!["the".to_string(), "ai".to_string()],
        200_000,
    );
    println!("Anagrams: {:?}", results.0.len());
    let duration = start.elapsed();
    println!("Time elapsed: {:?}", duration);
}
