use std::time::Instant;
use std::{cmp, str};
use wasm_bindgen::prelude::*;
use web_sys::console;

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

    for i in zero..SIZE {
        if smaller_counts[i] > larger_counts[i] {
            return false;
        }
    }
    true
}

fn filter_line(line: &String, seed: &String, min_length: usize) -> bool {
    line.chars().all(char::is_alphanumeric)
        & (line.chars().count() >= min_length)
        & contained(line, seed)
}

const SIZE: usize = 26; // a-z
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

/** Convert string into an array of character counts where a[0] is the count of 'a', a[1] is 'b', etc. */
fn to_counter(s: &str) -> [u32; SIZE] {
    let mut counts = [0; SIZE];
    for c in s.chars() {
        let i = to_index(c);
        counts[i] += 1;
    }
    counts
}

#[derive(Default)]
struct Trie {
    children: [Option<Box<Trie>>; SIZE], // See https://doc.rust-lang.org/book/ch15-01-box.html#enabling-recursive-types-with-boxes
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
        mut seed_counter: [u32; SIZE],
        path: &mut Vec<usize>,
        root: &Trie,
        current_word: &mut Vec<usize>,
        previous_word: &mut Vec<usize>,
        results: &mut Vec<String>,
    ) -> Vec<Vec<usize>> {
        let mut anagrams = Vec::new();
        if self.end_of_word {
            // if all characters have been used
            if seed_counter == [0; SIZE] {
                results.push(array_to_string(&path));
            }
            path.push(27);
            let old_word = previous_word.clone();
            previous_word.clear();
            previous_word.extend_from_slice(&current_word);
            current_word.clear();
            // redo search from root
            let mut node_anagrams = root.anagram_recursive(
                seed_counter,
                path,
                root,
                current_word,
                previous_word,
                results,
            );
            anagrams.append(&mut node_anagrams);
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
            let mut node_anagrams = node.unwrap().anagram_recursive(
                seed_counter,
                path,
                root,
                current_word,
                previous_word,
                results,
            );
            anagrams.append(&mut node_anagrams);
            path.pop(); // pop the letter from the path
            current_word.pop(); // pop the letter from the current word
            seed_counter[i] = count; // reset counter
        };

        for i in 0..SIZE {
            inner_loop(i);
        }

        return anagrams;
    }
}

pub fn generate() -> Vec<String> {
    let dictionary = include_str!("dictionary.txt");

    let min_length = 5;
    let seed = "misunderstanding";
    let seed = str::replace(seed, " ", "").to_string();

    let lines: Vec<String> = dictionary.split("\n").map(str::to_string).collect();

    let filter_line_closure = |line: &String| filter_line(line, &seed, min_length);
    let processed_lines: Vec<String> = lines // using String as the return type of `to_lowercase`
        .iter()
        .map(process_line)
        .filter(filter_line_closure)
        .collect();

    let mut trie = Trie::new();
    for line in processed_lines {
        trie.insert(line);
    }

    let anagrams = trie.anagram(&seed);

    return anagrams;
}

#[wasm_bindgen]
pub fn js_generate() {
    console_error_panic_hook::set_once();
    let anagrams = generate();
    console::log_2(&"Anagrams:".into(), &anagrams.len().into());
}

#[allow(dead_code)]
fn main() {
    let start = Instant::now();
    let anagrams = generate();
    let duration = start.elapsed();
    println!("Time elapsed: {:?}", duration);
    println!("Anagrams: {}", anagrams.len());
    // for anagram in anagrams {
    //     println!("{}", anagram);
    // }
}
