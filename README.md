# anagramme

Find anagrams of a sentense in different languages.

## Build and Run Instructions

To build run:

```
cargo build
```

Then run the program:

```
./target/debug/anagramme "eleven two" -r ./res -l en
```

With:

* `eleven two` is the input sentense from which anagrams are found
* `-r ./res` is the path to the resource directory where the dictionaires are stored
* `-l en` instruct to use the english dictionary.
