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

fn get_raw_dict(name: &str, resource_dir: &str) -> Vec<u8> {
    let dict_path: PathBuf = [resource_dir, name].iter().collect();
    let mut f = File::open(&dict_path).expect("no file found");
    let metadata = metadata(&dict_path).expect("unable to read metadata");
    let mut buffer = vec![0; metadata.len() as usize];
    f.read_exact(&mut buffer).expect("buffer overflow");
    buffer
}

fn last_word(prefix: &[u8], spaces: &[usize]) -> String {
    let mut i = 0;
    if !spaces.is_empty() {
        i = spaces[spaces.len() - 1];
    }
    let new_cur_prefix = prefix[i..prefix.len()].to_vec();
    return String::from(str::from_utf8(&new_cur_prefix).unwrap());
}

fn anagrams(
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

        let mut result = Vec::new();
        result.reserve(spaces.len() + 1);
        let mut i = 0;
        for space in spaces {
            let part = String::from(str::from_utf8(&prefix[i..*space]).unwrap());
            result.push(part);
            i = *space;
        }
        let part = String::from(str::from_utf8(&prefix[i..prefix.len()]).unwrap());
        result.push(part);

        result.sort();

        let mut hasher = DefaultHasher::new();
        for r in result.iter() {
            r.hash(&mut hasher);
        }
        let hash = hasher.finish();
        if output.contains_key(&hash) {
            return;
        }

        println!("{:?}", result);

        output.insert(hasher.finish(), result);
        return;
    }

    let mut rest = Vec::new();
    rest.reserve(input.len() - 1);

    let mut cur = prefix.to_vec();
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
        anagrams(max_spaces, &rest, &cur, spaces, trie, output);

        if trie.contains(&key) {
            // Current prefix is a known word. Add a space and continue.
            let mut new_spaces = spaces.to_vec();
            new_spaces.push(cur.len());
            if spaces.len() < max_spaces {
                anagrams(max_spaces, &rest, &cur, &new_spaces, trie, output);
            }
        }

        cur.pop();
    }
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
            Arg::with_name("INPUT")
                .help("Input sentence from which anagrams are tentatively found")
                .required(true)
                .index(1),
        )
        .get_matches();

    let res_dir = matches.value_of("resource-dir").unwrap();
    let input = matches.value_of("INPUT").unwrap().to_lowercase();
    let lang = matches.value_of("language").unwrap().to_lowercase();

    let txtfile = format!("{}.txt", lang);

    let raw_dict_fr = get_raw_dict(&txtfile, res_dir);
    let mut trie = PatriciaSet::new();

    let mut i = 0;
    let mut start = 0;
    while i < raw_dict_fr.len() {
        if raw_dict_fr[i] == b'\n' {
            let u8_word = &raw_dict_fr[start..i];
            let word = str::from_utf8(u8_word).unwrap();
            let ascii_word: String = word.nfd().filter(char::is_ascii).collect();
            trie.insert(ascii_word.to_lowercase());
            i += 1;
            start = i;
            continue;
        }
        i += 1;
    }

    let input_vec = input.replace(" ", "").as_bytes().to_vec();
    let mut outputs = HashMap::new();
    anagrams(
        input_vec.len() / SPACES_FACTOR + 1,
        &input_vec,
        &Vec::new(),
        &Vec::new(),
        &trie,
        &mut outputs,
    );

    let mut results = HashSet::new();
    for (_, result) in outputs {
        results.insert(Vec::from_iter(result).join(" "));
    }

    let mut ordered_results = Vec::new();
    for result in results {
        ordered_results.push(result);
    }
    ordered_results.sort_by_key(|s| Reverse(s.matches(' ').count()));

    for result in ordered_results {
        println!("{}", result);
    }
}
