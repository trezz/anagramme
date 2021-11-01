extern crate clap;
extern crate unicode_normalization;

use clap::{App, Arg};
use patricia_tree::PatriciaSet;
use std::collections::hash_map::DefaultHasher;
use std::collections::{HashMap, HashSet};
use std::fs::{metadata, File};
use std::hash::{Hash, Hasher};
use std::io::Read;
use std::str;
use unicode_normalization::UnicodeNormalization;

fn get_raw_dict(name: &str, resource_dir: &str) -> Vec<u8> {
    let dict_path_elts = vec![resource_dir.to_string(), name.to_string()];
    let dict_path = dict_path_elts.join("/");
    let mut f = File::open(&dict_path).expect("no file found");
    let metadata = metadata(&dict_path).expect("unable to read metadata");
    let mut buffer = vec![0; metadata.len() as usize];
    f.read(&mut buffer).expect("buffer overflow");
    return buffer;
}

fn last_word(prefix: &Vec<u8>, spaces: &Vec<usize>) -> String {
    let mut i = 0;
    if spaces.len() > 0 {
        i = spaces[spaces.len() - 1];
    }
    let new_cur_prefix = prefix[i..prefix.len()].to_vec();
    return String::from(str::from_utf8(&new_cur_prefix).unwrap());
}

fn anagrams(
    input_size: usize,
    input: &Vec<u8>,
    prefix: &Vec<u8>,
    spaces: &Vec<usize>,
    trie: &PatriciaSet,
    output: &mut HashMap<u64, HashSet<String>>,
) {
    if input.len() == 0 {
        let key = last_word(&prefix, &spaces);
        if !trie.contains(&key) {
            return;
        }

        let mut hasher = DefaultHasher::new();
        let mut result = HashSet::new();
        let mut i = 0;
        for space in spaces {
            let part = String::from(str::from_utf8(&prefix[i..*space]).unwrap());
            part.hash(&mut hasher);
            result.insert(part);
            i = *space;
        }
        let part = String::from(str::from_utf8(&prefix[i..prefix.len()]).unwrap());
        part.hash(&mut hasher);
        let hash = hasher.finish();
        if output.contains_key(&hash) {
            return;
        }

        print!("-");
        result.insert(part);
        output.insert(hasher.finish(), result);
        return;
    }

    let mut rest = Vec::new();
    rest.reserve(input.len() - 1);

    let mut cur = prefix.to_vec();
    cur.reserve(input.len());

    let mut i = 0;
    while i < input.len() {
        rest.clear();
        let mut j = 0;
        while j < input.len() {
            if j != i {
                rest.push(input[j]);
            }
            j += 1;
        }

        cur.push(input[i]);
        i += 1;

        let key = last_word(&cur, &spaces);
        let mut prefixes = trie.iter_prefix(&key.as_bytes()).take(1);
        match prefixes.next() {
            Some(_) => {}
            None => {
                cur.pop();
                continue;
            }
        }

        if trie.contains(&key) {
            // Current prefix is a known word. Add a space and continue.
            let mut new_spaces = spaces.to_vec();
            new_spaces.push(cur.len());
            if spaces.len() < input_size / 4 {
                anagrams(input_size, &rest, &cur, &new_spaces, trie, output);
            }
        }

        // Try also with a longer prefix.
        anagrams(input_size, &rest, &cur, spaces, trie, output);

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
            Arg::with_name("INPUT")
                .help("Input sentence from which anagrams are tentatively found")
                .required(true)
                .index(1),
        )
        .get_matches();

    let res_dir = matches.value_of("resource-dir").unwrap();
    let input = matches.value_of("INPUT").unwrap().to_lowercase();

    let raw_dict_fr = get_raw_dict("fr.txt", res_dir);
    let mut trie = PatriciaSet::new();

    let mut i = 0;
    let mut start = 0;
    while i < raw_dict_fr.len() {
        if raw_dict_fr[i] == '\n' as u8 {
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

    let mut input_vec = input.as_bytes().to_vec();
    input_vec.sort();
    let mut outputs = HashMap::new();
    anagrams(
        input_vec.len(),
        &input_vec,
        &Vec::new(),
        &Vec::new(),
        &trie,
        &mut outputs,
    );

    let mut results = HashSet::new();
    for (_, result) in outputs {
        let mut tmp = Vec::from_iter(result);
        tmp.sort();
        let s = tmp.join(" ");
        results.insert(s);
    }

    let mut ordered_results = Vec::new();
    for result in results {
        ordered_results.push(result);
    }
    ordered_results.sort_by(|a, b| b.len().cmp(&a.len()));

    for result in ordered_results {
        println!("{}", result);
    }
}
