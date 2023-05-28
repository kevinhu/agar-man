fn filter_grid_word(word: &String, grid_letters: &Vec<char>, min_length: usize) -> bool {
    word.chars().all(char::is_alphanumeric)
        & (word.chars().count() >= min_length)
        & contained(word, &grid_letters.iter().collect::<String>())
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
