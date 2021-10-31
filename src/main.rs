extern crate clap;
extern crate unicode_normalization;

use clap::{App, Arg};
use radix_trie::Trie;
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

    buffer
}

fn last_word(prefix: &Vec<u8>, spaces: &Vec<usize>) -> String {
    let mut prefix_start = 0;
    if spaces.len() > 0 {
        prefix_start = spaces[spaces.len() - 1];
    }
    let new_cur_prefix = prefix[prefix_start..prefix.len()].to_vec();
    let key = String::from(str::from_utf8(&new_cur_prefix).unwrap());

    return key;
}

fn permute(
    input: Vec<u8>,
    prefix: Vec<u8>,
    spaces: Vec<usize>,
    trie: &Trie<String, bool>,
    output: &mut Vec<String>,
) {
    if input.len() == 0 {
        let key = last_word(&prefix, &spaces);
        match trie.get(&key) {
            Some(_) => {
                let mut result = prefix;
                let mut i = 0;
                for space in &spaces {
                    result.insert(*space + i, ' ' as u8);
                    i += 1;
                }
                let s = String::from(str::from_utf8(&result).unwrap());
                println!("--> {}", s);
                output.push(s);
            }
            None => {}
        }
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

        let rest_str = last_word(&rest, &Vec::new());
        let new_prefix_str = last_word(&new_prefix, &Vec::new());
        let key = last_word(&new_prefix, &spaces);

        // println!(            "{} prefix={} key={} spaces={:?} rest={}",            i, new_prefix_str, key, spaces, rest_str        );

        match trie.get(&key) {
            Some(_) => {
                // Current prefix is a known word. Add a space and continue.
                let mut new_spaces = spaces.to_vec();
                new_spaces.push(new_prefix.len());
                //println!("--> FOUND!");
                permute(
                    rest.to_vec(),
                    new_prefix.to_vec(),
                    new_spaces.to_vec(),
                    trie,
                    output,
                );
            }
            None => {}
        }

        permute(rest, new_prefix, spaces.to_vec(), trie, output);
        i += 1;
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
    let mut trie = Trie::new();

    let mut i = 0;
    let mut start = 0;
    while i < raw_dict_fr.len() {
        if raw_dict_fr[i] == '\n' as u8 {
            let u8_word = &raw_dict_fr[start..i];
            let word = str::from_utf8(u8_word).unwrap();
            let ascii_word: String = word.nfd().filter(char::is_ascii).collect();
            trie.insert(ascii_word.to_lowercase(), true);
            i += 1;
            start = i;
            continue;
        }
        i += 1;
    }

    let input_vec = input.as_bytes().to_vec();
    let mut outputs = Vec::new();
    permute(input_vec, Vec::new(), Vec::new(), &trie, &mut outputs);

    outputs.sort_by(|a, b| a.matches(' ').count().cmp(&b.matches(' ').count()));

    for result in outputs {
        println!("{}", result);
    }
}
