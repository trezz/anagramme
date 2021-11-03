extern crate clap;
extern crate unicode_normalization;

use clap::{App, Arg};
use patricia_tree::PatriciaSet;
use std::cmp::Reverse;
use std::collections::hash_map::DefaultHasher;
use std::collections::{HashMap, HashSet};
use std::fs::{metadata, File};
use std::hash::{Hash, Hasher};
use std::io::Read;
use std::path::PathBuf;
use std::str;
use unicode_normalization::UnicodeNormalization;

const SPACES_FACTOR: usize = 6;

// Opens dictionary file name 'name.txt' and located in 'resource_dir' and returns
// its full content as a Vec<u8>.
fn get_raw_dict(name: &str, resource_dir: &str) -> Vec<u8> {
    let dict_path: PathBuf = [resource_dir, name].iter().collect();
    let mut f = File::open(&dict_path).expect("no file found");
    let metadata = metadata(&dict_path).expect("unable to read metadata");
    let mut buffer = vec![0; metadata.len() as usize];
    f.read_exact(&mut buffer).expect("buffer overflow");
    buffer
}

// Returns the last word of the given sentence, which it the characters of prefix
// starting after the position of the last space in 'spaces'.
fn last_word(prefix: &[u8], spaces: &[usize]) -> String {
    let mut i = 0;
    if !spaces.is_empty() {
        i = spaces[spaces.len() - 1];
    }
    let new_cur_prefix = prefix[i..prefix.len()].to_vec();
    return String::from(str::from_utf8(&new_cur_prefix).unwrap());
}

// Splits the given 'letters' with spaces at positions given by 'spaces' and return
// the split words in a Vec<String>.
fn get_sentence(letters: &[u8], spaces: &[usize]) -> Vec<String> {
    let mut result = Vec::new();
    result.reserve(spaces.len() + 1);
    let mut i = 0;
    for space in spaces {
        result.push(String::from(str::from_utf8(&letters[i..*space]).unwrap()));
        i = *space;
    }
    result.push(String::from(str::from_utf8(&letters[i..]).unwrap()));
    result.sort();

    result
}

// Returns the hash sum of the elements of the given vec.
fn compute_hash(v: &[String]) -> u64 {
    let mut hasher = DefaultHasher::new();
    for item in v.iter() {
        item.hash(&mut hasher);
    }

    hasher.finish()
}

// Finds anagrams of 'input' and fill 'output' with found ones.
// At most 'max_spaces' are authorized in found anagrams.
fn anagrams(
    max_spaces: usize,
    input: &[u8],
    trie: &PatriciaSet,
    output: &mut HashMap<u64, Vec<String>>,
) {
    anagrams_rec(max_spaces, input, &Vec::new(), &Vec::new(), trie, output);
}

fn anagrams_rec(
    max_spaces: usize,
    input: &[u8],
    prefix: &[u8],
    spaces: &[usize],
    trie: &PatriciaSet,
    output: &mut HashMap<u64, Vec<String>>,
) {
    if input.is_empty() {
        let key = last_word(prefix, spaces);
        if !trie.contains(&key) {
            return;
        }
        let sentence = get_sentence(prefix, spaces);
        let hash = compute_hash(&sentence);
        if output.contains_key(&hash) {
            return;
        }
        println!("--> {}", sentence.join(" "));
        output.insert(hash, sentence);
        return;
    }

    let mut rest = Vec::new();
    let mut cur = prefix.to_vec();
    rest.reserve(input.len() - 1);
    cur.reserve(input.len());

    for (i, c) in input.iter().enumerate() {
        rest.clear();
        for (j, r) in input.iter().enumerate() {
            if j != i {
                rest.push(*r);
            }
        }
        cur.push(*c);

        let key = last_word(&cur, spaces);
        if trie.iter_prefix(key.as_bytes()).take(1).count() == 0 {
            // Backtrack as the current prefix isn't starting any valid word.
            cur.pop();
            continue;
        }

        // Try with a longer prefix.
        anagrams_rec(max_spaces, &rest, &cur, spaces, trie, output);
        if trie.contains(&key) && spaces.len() < max_spaces {
            // Current prefix is a known word. Add a space and continue.
            let new_spaces = spaces
                .iter()
                .copied()
                .chain([cur.len()])
                .collect::<Vec<_>>();
            anagrams_rec(max_spaces, &rest, &cur, &new_spaces, trie, output);
        }
        cur.pop();
    }
}

// Creates a Patricia trie from a dictionary raw content.
fn trie_from_dict(dict: &[u8]) -> PatriciaSet {
    let mut trie = PatriciaSet::new();

    let mut start = 0;
    for (i, c) in dict.iter().enumerate() {
        if *c == b'\n' {
            let u8_word = &dict[start..i];
            let word = str::from_utf8(u8_word).unwrap();
            let ascii_word: String = word.nfd().filter(char::is_ascii).collect();
            trie.insert(ascii_word.to_lowercase());
            start = i + 1;
        }
    }

    trie
}

// Builds the input sentence as a Vec<u8>, removing the letters from 'hint'.
fn make_input_vec(input: &str, hint: &str) -> Vec<u8> {
    let mut v = Vec::new();
    let mut tmp_hint_chars = hint.as_bytes().to_vec();

    for c in input.replace(" ", "").as_bytes().to_vec() {
        let mut keep = true;
        for (i, h) in tmp_hint_chars.iter().enumerate() {
            if *h == c {
                tmp_hint_chars.remove(i);
                keep = false;
                break;
            }
        }
        if keep {
            v.push(c);
        }
    }

    v
}

fn main() {
    let matches = App::new("Anagramme")
        .author("Vincent C.")
        .arg(
            Arg::with_name("resource-dir")
                .short("r")
                .long("resource-dir")
                .value_name("DIR")
                .required(true)
                .help("Path to the resource directory holding dictionary files")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("language")
                .short("l")
                .long("language")
                .value_name("LANG")
                .required(true)
                .help("Language prefix (e.g. fr for french)")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("hint")
                .long("hint")
                .value_name("HINT")
                .required(false)
                .help("Hint word or short sentense that is part of an anagram of the input")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("INPUT")
                .help("Input sentence from which anagrams are tentatively found")
                .required(true)
                .index(1),
        )
        .get_matches();

    let res_dir = matches.value_of("resource-dir").unwrap();
    let input = matches.value_of("INPUT").unwrap().to_ascii_lowercase();
    let lang = matches.value_of("language").unwrap().to_ascii_lowercase();
    let hint = matches
        .value_of("hint")
        .or(Some(""))
        .unwrap()
        .to_ascii_lowercase();

    let txtfile = format!("{}.txt", lang);

    let trie = trie_from_dict(&get_raw_dict(&txtfile, res_dir));
    let in_vec = make_input_vec(&input, &hint);

    let max_spaces = in_vec.len() / SPACES_FACTOR + 1;

    let mut out = HashMap::new();
    anagrams(max_spaces, &in_vec, &trie, &mut out);

    let mut unique_results = HashSet::new();
    for (_, result) in out {
        unique_results.insert(Vec::from_iter(result).join(" "));
    }

    let mut ordered_results = Vec::new();
    for result in unique_results {
        ordered_results.push(result);
    }
    ordered_results.sort_by_key(|s| Reverse(s.matches(' ').count()));

    for result in ordered_results {
        println!("{} {}", hint, result);
    }
}
