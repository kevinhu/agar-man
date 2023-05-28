
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

pub fn trie_solve(
    seed: &str,
    min_length: usize,
    excludes: &HashSet<String>,
) -> (Vec<String>, Vec<String>) {
    let dictionary = include_str!("dictionary.txt");

    let seed = str::replace(seed, " ", "").to_string();

    let lines: Vec<String> = dictionary.split("\n").map(str::to_string).collect();

    let filter_line_closure = |line: &String| filter_line(line, &seed, min_length, excludes);
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
