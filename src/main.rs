use std::fs::File;
use std::path::Path;
use std::collections::HashMap;
use std::io::{self, Read};
use std::env;
#[cfg(windows)]
use std::io::{BufRead, BufReader};

// Helper function to print the letter counts as a readable string
fn _print_key(counts: &[u8; 26]) {
    print!("Letters: ");
    for (i, &count) in counts.iter().enumerate() {
        let letter = (b'a' + i as u8) as char;
        for _ in 0..count {
            print!("{}", letter);
        }
    }
    print!(" ");
}


struct WordGroup {
    counts: [u8; 26],
    words: Vec<String>,
    len: usize,
}

struct RepeatedGroup<'a> {
    group: &'a WordGroup,
    reps: usize,
}

fn fits_inside(
    target_counts: &[u8; 26],
    counts: &[u8; 26]
) -> bool {
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

/// Count only ASCII letters (ignoring apostrophes or others)
fn get_letter_counts_bytes(word: &[u8]) -> [u8; 26] {
    let mut counts = [0u8; 26];
    for &b in word {
        match b {
            b'a'..=b'z' => counts[(b - b'a') as usize] += 1,
            b'A'..=b'Z' => counts[(b - b'A') as usize] += 1,
            _           => {}
        }
    }
    counts
}

#[cfg(windows)]
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
           .or_default()
           .push(word)
    }

    Ok(map)
}

#[cfg(not(windows))]
fn build_map_from_file <P: AsRef<Path>>(
    filename: P,
    target_counts: &[u8; 26]
) -> std::io::Result<HashMap<[u8; 26], Vec<String>>> {
    let file = File::open(filename)?;
    let mmap = unsafe { memmap2::MmapOptions::new().map(&file)? };
    let data: &[u8] = &mmap;

    let mut map: HashMap<[u8; 26], Vec<String>> = HashMap::new();

    for line in data.split(|&b| b == b'\n') {
        if line.is_empty() { continue; }
        let counts = get_letter_counts_bytes(line);
        if !fits_inside(target_counts, &counts) { continue; }
        let word = std::str::from_utf8(line).unwrap();
        map.entry(counts).or_default().push(word.to_string());
    }

    Ok(map)
}

fn build_word_groups_from_map(
    map: HashMap<[u8; 26], Vec<String>>,
) -> Vec<WordGroup> {
    map.into_iter()
        .map(|(counts, words)| WordGroup {
            len: counts.iter().map(|&c| c as usize).sum(),
            counts,
            words,
        })
        .collect()
}

fn find_anagrams<'a>(
    target: &mut [u8; 26],
    length: usize,
    input_buffers: &mut [Vec<&'a WordGroup>],
    combo: &mut Vec<RepeatedGroup<'a>>,
    solution_buffer: &mut String,

) {
    if length == 0 {
        expand_solution(combo, solution_buffer);
        return;
    }

    let (current_slice, rest_buffers) = input_buffers.split_at_mut(1);
    let remaining: &[&WordGroup] = &current_slice[0]; 

    for (i, &wg) in remaining.iter().enumerate() {
        // subtract wordgroup's letters from target
        (0..26).for_each(|j| { target[j] -= wg.counts[j]; });
        // add group to our running combo (increment reps if last group is the same)
        if combo.last().is_some_and(|last| std::ptr::eq(last.group, wg)) {
            combo.last_mut().unwrap().reps += 1;
        } else {
            combo.push(RepeatedGroup { group: wg, reps: 1 })
        }


        {
            // clear & refill next_buf from remaining[i..]
            let next_buf: &mut Vec<&WordGroup> = &mut rest_buffers[0];
            next_buf.clear();
            for &wg2 in &remaining[i..] {
                if fits_inside(target, &wg2.counts) {
                    next_buf.push(wg2);
                }
            }
        }

        find_anagrams(
            target,
            length - wg.len,
            rest_buffers,
            combo,
            solution_buffer,
        );

        // remove group from our running combo (decrement reps if last group has more than 1 rep)
        if combo.last_mut().unwrap().reps > 1 {
            combo.last_mut().unwrap().reps -= 1;
        } else {
            combo.pop();
        }
        // add wordgroup's letters back to target
        (0..26).for_each(|j| { target[j] += wg.counts[j]; });
    }

}

/// Expand one *primitive* solution (the `combo` of `RepeatedGroup`s) into all real anagram sentences.
/// Uses `buffer` to accumulate one sentence at a time and prints each when complete.
fn expand_solution(
    combo: &[RepeatedGroup<'_>],
    buffer: &mut String,
) {
    // Base case: no more groups ⇒ print what’s in buffer (joined with spaces)
    if combo.is_empty() {
        if !buffer.is_empty() {
            println!("{}", buffer);
        }
        return;
    }

    let RepeatedGroup { group: wg, reps } = &combo[0];
    choose_words(&wg.words, *reps, buffer, &combo[1..]);
}

/// A helper function for choosing combinations of `words` with `reps` repetitions
fn choose_words(
    words: &[String],
    reps: usize,
    buffer: &mut String,
    rest: &[RepeatedGroup<'_>],
) {
    if reps == 0 {
        return expand_solution(rest, buffer);
    }
    for (i, word) in words.iter().enumerate() {
        let start = buffer.len();
        buffer.push_str(word);
        buffer.push(' ');
        choose_words(&words[i..], reps - 1, buffer, rest);
        buffer.truncate(start);
    }
}


// Returns (wordlist_path, input_string)
fn parse_args() -> (String, String) {
    // Default wordlist and empty inputs
    let mut wordlist = "/home/josh/.local/bin/words.txt".to_string();
    let mut inputs: Vec<String> = Vec::new();

    // Skip argv[0]
    let mut args = env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "-f" | "--file" => {
                // Next argument must be the filename
                wordlist = args
                    .next()
                    .unwrap_or_else(|| {
                        eprintln!("-f/--file requires a path");
                        std::process::exit(1);
                    });
            }
            s if s.starts_with('-') => {
                eprintln!("Unknown option: {}", s);
                std::process::exit(1);
            }
            other => {
                // Positional input
                inputs.push(other.to_owned());
            }
        }
    }

    // If the user provided words on the command line, join them.
    // Otherwise, read everything from stdin into one string.
    let input_string = if !inputs.is_empty() {
        inputs.join(" ")
    } else {
        let mut buf = String::new();
        io::stdin()
            .read_to_string(&mut buf)
            .expect("Failed to read from stdin");
        buf.trim_end().to_string()
    };

    (wordlist, input_string)
}

fn main() -> std::io::Result<()> {
    let (wordlist_path, input) = parse_args();
    let mut target_counts = get_letter_counts(&input);
    let length = target_counts.iter().map(|&c| c as usize).sum();


    // let wordmap = build_map_from_file(wordlist_path, &target_counts)?;
    let wordmap = build_map_from_file(wordlist_path, &target_counts)?;
    if wordmap.is_empty() { return Ok(()); }
    let mut wordgroups: Vec<WordGroup> = build_word_groups_from_map(wordmap);
    wordgroups.sort_by(|a, b| b.len.cmp(&a.len));

    // max_depth = length (or a tighter estimate)
    let max_depth = length;
    let mut input_buffers: Vec<Vec<&WordGroup>> = Vec::with_capacity(max_depth + 1);
    for _ in 0..=max_depth {
        input_buffers.push(Vec::with_capacity(length));
    }

    // Worst‐case depth is `length` (one letter per group).
    let mut combo_buffer: Vec<RepeatedGroup> = Vec::with_capacity(length);

    // Worst‐case you print every letter as its own word + a space ⇒ ~2*length chars.
    let mut solution_buffer = String::with_capacity(length);

    // load the very first level with all your groups:
    input_buffers[0].extend(wordgroups.iter());

    find_anagrams(
        &mut target_counts,
        length,
        &mut input_buffers,
        &mut combo_buffer,
        &mut solution_buffer
    );

    Ok(())
}

