# songrep (song-grep)

A tiny Rust CLI that indexes a directory of song lyrics and searches them with exact and fuzzy matching.

## How it works

- Loads `.txt` lyric files from `lyrics/yearwise_dataset/<year>`
- Builds an inverted index of word -> (song, line, position)
- Searches a multi-word query with AND semantics, falling back to Levenshtein distance <= 1 for typo tolerance
- Ranks results by match frequency

## Usage

```bash
cargo run
```

The input path and query live in `src/main.rs` (no CLI args yet).

## Test

```bash
cargo test
```

## License

MIT