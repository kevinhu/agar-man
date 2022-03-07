use indicatif::ProgressBar;
use std::str;
use std::{
    fs::File,
    io::{self, BufRead, BufReader},
    path::Path,
};

static MIN_LENGTH: usize = 3;

fn lines_from_file(filename: impl AsRef<Path>) -> io::Result<Vec<String>> {
    BufReader::new(File::open(filename)?).lines().collect()
}

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

fn filter_line(line: &String, seed: &str) -> bool {
    line.chars().all(char::is_alphanumeric)
        & (line.chars().count() >= MIN_LENGTH)
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
    fn anagram(&self, seed: &str) -> Vec<String> {
        let seed_counter = to_counter(seed);

        let mut results = Vec::new();

        self.anagram_recursive(seed_counter, String::new(), self, &mut results, true);

        return results;
    }

    /** Inner anagram function */
    fn anagram_recursive(
        &self,
        mut seed_counter: [u32; SIZE],
        path: String,
        root: &Trie,
        results: &mut Vec<String>,
        root_call: bool,
    ) -> Vec<String> {
        let mut anagrams = Vec::new();
        if self.end_of_word {
            // if all characters have been used
            if seed_counter == [0; SIZE] {
                // println!("{}", path);
                results.push(path.clone());
            }
            let newpath = path.clone() + " ";
            // redo search from root
            let mut node_anagrams =
                root.anagram_recursive(seed_counter, newpath, root, results, false);
            anagrams.append(&mut node_anagrams);
        }

        let mut inner_loop = |i: usize| {
            let node = self.children[i].as_ref();
            // skip node if it is None
            if !node.is_some() {
                return;
            }
            let letter = to_char(i);
            let count = seed_counter[i];
            if count == 0 {
                return;
            }
            seed_counter[i] -= 1; // decrement the count of the letter in the seed

            let newpath = path.clone() + &letter.to_string();
            // continue search from node
            let mut node_anagrams =
                node.unwrap()
                    .anagram_recursive(seed_counter, newpath, root, results, false);
            anagrams.append(&mut node_anagrams);
            seed_counter[i] = count; // reset counter
        };

        for i in 0..SIZE {
            inner_loop(i);
        }
        if root_call {
            let bar = ProgressBar::new(SIZE as u64);
            bar.inc(0);
            for i in 0..SIZE {
                inner_loop(i);
                bar.inc(1);
            }
        }

        return anagrams;
    }
}

fn main() {
    let seed = "anagram";

    let lines = lines_from_file("./dictionary.txt").expect("Could not load lines");

    println!("Total words: {}", lines.len());

    let filter_line_closure = |line: &String| filter_line(line, seed);
    let processed_lines: Vec<String> = lines // using String as the return type of `to_lowercase`
        .iter()
        .map(process_line)
        .filter(filter_line_closure)
        .collect();

    println!("Filtered words: {}", processed_lines.len());

    let mut trie = Trie::new();
    for line in processed_lines {
        trie.insert(line);
    }

    let anagrams = trie.anagram(seed);

    for anagram in anagrams {
        println!("{}", anagram);
    }
}
