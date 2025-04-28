use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

fn build_map_from_file <P: AsRef<Path>>(
    filename: P,
    target_counts: &[u8; 26]
) -> std::io::Result<HashMap<[u8; 26], Vec<String>>> {
    let file = File::open(filename)?;
    let reader = BufReader::new(file);
    let mut map: HashMap<[u8; 26], Vec<String>> = HashMap::new();

    for line in reader.lines() {
        let word = line?;
        if word.is_empty() { continue; }

        let counts = get_letter_counts(&word);

        if !fits_inside(target_counts, &counts) { continue; }

        map.entry(counts)
            .or_insert_with(Vec::new)
            .push(word)
    }

    Ok(map)
}

fn fits_inside(target_counts: &[u8; 26], counts: &[u8; 26]) -> bool {
    for i in 0..26 {
        if counts[i] > target_counts[i] {
            return false;
        }
    }
    true
}

fn get_letter_counts(word: &str) -> [u8; 26] {
    let mut counts = [0u8; 26];
    for c in word.chars() {
        if c.is_ascii_alphabetic() {
            let idx = (c.to_ascii_lowercase() as u8 - b'a') as usize;
            counts[idx] += 1;
        }
    }
    counts
}



fn main() -> std::io::Result<()> {
    let default_file = "/home/josh/.local/bin/words.txt";
    let input = "abracadabra";
    let target = get_letter_counts(input);

    // Use the ? operator to handle potential errors
    let wordmap = build_map_from_file(default_file, &target)?;

    // Iterate through key-value pairs
    for (key, val) in wordmap.iter() {
        // Print the letters that make up this key
        print_key(key);

        // Print the words for this combination
        println!(": {:?}", val);
    }

    Ok(())
}

// Helper function to print the letter counts as a readable string
fn print_key(counts: &[u8; 26]) {
    print!("Letters: ");
    for (i, &count) in counts.iter().enumerate() {
        let letter = (b'a' + i as u8) as char;
        for _ in 0..count {
            print!("{}", letter);
        }
    }
    print!(" ");
}
