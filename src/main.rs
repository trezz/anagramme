extern crate clap;
extern crate unicode_normalization;

use clap::{App, Arg};
use patricia_tree::PatriciaSet;
use std::collections::HashSet;
use std::fs::{metadata, File};
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
    input: Vec<u8>,
    prefix: Vec<u8>,
    spaces: Vec<usize>,
    trie: &PatriciaSet,
    output: &mut Vec<HashSet<String>>,
) {
    if input.len() == 0 {
        let key = last_word(&prefix, &spaces);
        if !trie.contains(&key) {
            return;
        }
        let mut result = HashSet::new();
        let mut i = 0;
        for space in &spaces {
            result.insert(String::from(str::from_utf8(&prefix[i..*space]).unwrap()));
            i = *space;
        }
        result.insert(String::from(
            str::from_utf8(&prefix[i..prefix.len()]).unwrap(),
        ));

        println!("--> {:?}", result);
        output.push(result);
        return;
    }

    let mut i = 0;
    while i < input.len() {
        let mut rest = input[0..i].to_vec();
        if i + 1 < input.len() {
            rest.append(&mut input[i + 1..input.len()].to_vec());
        }

        let mut new_prefix = prefix.to_vec();
        new_prefix.push(input[i]);

        let key = last_word(&new_prefix, &spaces);
        if trie.contains(&key) {
            // Current prefix is a known word. Add a space and continue.
            let mut new_spaces = spaces.to_vec();
            new_spaces.push(new_prefix.len());
            anagrams(rest.to_vec(), new_prefix.to_vec(), new_spaces, trie, output);
        }

        i += 1;

        // Try also with a longer prefix.
        anagrams(rest, new_prefix, spaces.to_vec(), trie, output);
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
    let mut outputs = Vec::new();
    anagrams(input_vec, Vec::new(), Vec::new(), &trie, &mut outputs);

    let mut results = HashSet::new();
    for result in outputs {
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
