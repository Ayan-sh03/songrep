use std::cmp::{max, min};
use std::{collections::HashMap, fs};
#[derive(Debug, Clone)]
struct Song {
    id: u32,
    lyrics: String,
    title: String,
}
#[derive(Debug)]
struct Index {
    songs: Vec<Song>,
    exact: HashMap<String, Vec<(u32, usize, usize)>>, //word -> song_id,line_num,pos
}

fn main() {
    // Load Songs
    let path: String = ("./lyrics/yearwise_dataset/2016").to_string();
    let songs = load_songs(path).expect("Failed to load");

    println!("Songs Loaded");
    let index = create_index(songs).expect("Failed to index");
    // let query = "beraham duao se nafrat karunga";
    let query = "tiri ankhon ki namaken";
    // let query = "Poem";
    let songs = search(&index, query);

    println!("{:?}", songs);

    println!("===============================================");
    let mut ranked_songs: Vec<(usize, &Song)> = songs
        .iter()
        .map(|song| (song.lyrics.matches(query).count(), song))
        .collect();

    // Sort by frequency descending (b compares to a)
    ranked_songs.sort_by(|a, b| b.0.cmp(&a.0));

    for (count, song) in ranked_songs {
        println!("Frequency: {}, Song: {}", count, song.title);
    }
}

fn load_songs(path: String) -> Result<Vec<Song>, Box<dyn std::error::Error>> {
    let mut songs: Vec<Song> = Vec::new();
    let mut id_counter: u32 = 0;
    for entry in fs::read_dir(path)? {
        let entry = entry?;
        let path = entry.path();

        if !path.is_dir() && path.extension().and_then(|s| s.to_str()) == Some("txt") {
            let file_name = path
                .file_name()
                .and_then(|f| f.to_str())
                .unwrap_or("Unknown");
            println!("File name: {} ", file_name);
            let content = fs::read_to_string(&path)?;
            println!("Content: {} bytes", content.len());
            let file_name_stripped = extract_between(&file_name)
                .map(|s| s.trim())
                .map(|s| s.to_string())
                .unwrap_or_else(|| "Unknown".to_string());
            let song = Song {
                id: id_counter,
                lyrics: content,
                title: file_name_stripped,
            };

            id_counter += 1;

            songs.push(song);
        }
    }
    // println!("{:?}", songs);
    // for entri in &songs {
    //     println!("{:?}", entri.title)
    // }
    Ok(songs)
}

fn create_index(songs: Vec<Song>) -> Result<Index, Box<dyn std::error::Error>> {
    let mut exact: HashMap<String, Vec<(u32, usize, usize)>> = HashMap::new();
    for entry in &songs {
        // split song lyrics in line number and words
        let title_with_lyrics = format!("{} {} \n", entry.title, entry.lyrics);
        let lines: Vec<&str> = title_with_lyrics.split('\n').collect();

        // println!("{}", lines.len());
        // // println!("{}", lines)
        for (index, line) in lines.iter().enumerate() {
            // collect words per line
            let words: Vec<&str> = line.split(' ').collect();

            for (word_index, word) in words.iter().enumerate() {
                let entry = (entry.id, index, word_index);
                exact
                    .entry(word.to_lowercase())
                    .or_insert(Vec::new())
                    .push(entry)
            }
        }
    }
    let index = Index {
        exact: exact,
        songs: songs,
    };
    Ok(index)
}

fn search(index: &Index, query: &str) -> Vec<Song> {
    println!("--- 🔎 Search Started: \"{}\" ---", query);

    let words: Vec<&str> = query.split_whitespace().collect();
    if words.is_empty() {
        println!("⚠️ Search query is empty.");
        return vec![];
    }

    let mut song_ids: Option<Vec<u32>> = None;

    for word in &words {
        let norm = normalize(word);

        // Extract unique IDs for the current word
        let hits = index.exact.get(&norm);

        // if hits are empty , then check with levenshtein
        // go acrsoo all the keys and hit with min edit distance 1 , that is tuum->tum

        let mut ids: Vec<u32> = index
            .exact
            .get(&norm)
            .map(|hits| hits.iter().map(|(id, _, _)| *id).collect())
            .unwrap_or_default();

        // 2. ALWAYS check for fuzzy matches (even if exact exists)
        // or only if exact hits are low
        if ids.len() < 5 {
            // Example: if we found few or no exact matches
            for indexed_word in index.exact.keys() {
                if (indexed_word.len() as i32 - norm.len() as i32).abs() <= 2 {
                    let dist = levenshtein(&norm, indexed_word);
                    // distance 1 catches ankhon -> aankhon
                    if dist <= 1 {
                        if let Some(hits) = index.exact.get(indexed_word) {
                            for (id, _, _) in hits {
                                ids.push(*id);
                            }
                        }
                    }
                }
            }
        }
        ids.sort();
        ids.dedup();
        println!("📖 Word: '{}' -> Found in {} unique songs", norm, ids.len());

        song_ids = match song_ids {
            None => {
                println!(
                    "📌 First word: establishing baseline with {} songs",
                    ids.len()
                );
                Some(ids)
            }
            Some(existing) => {
                let prev_count = existing.len();
                let intersection: Vec<u32> =
                    existing.into_iter().filter(|id| ids.contains(id)).collect();
                println!(
                    "📉 Filtering: {} songs matched previous words, now {} match both (Intersection)",
                    prev_count,
                    intersection.len()
                );
                Some(intersection)
            }
        }
    }

    let final_ids = song_ids.unwrap_or_default();

    // Retrieve full Song objects
    let results: Vec<Song> = index
        .songs
        .iter()
        .filter(|s| final_ids.contains(&s.id))
        .cloned()
        .collect();

    println!(
        "✅ Search Complete. Found {} songs matching all terms.",
        results.len()
    );
    results
}

fn levenshtein(str1: &str, str2: &str) -> usize {
    if str1.is_empty() || str2.is_empty() {
        return std::cmp::max(str1.chars().count(), str2.chars().count());
    }
    let s1: Vec<char> = str1.chars().collect();
    let s2: Vec<char> = str2.chars().collect();
    if str1 == str2 {
        return 0;
    }

    let rows = s1.len() + 1;
    let cols = s2.len() + 1;

    // let mut diff = Vec<Vec<usize>>::new();

    // create matrix of length str+1 and str1+1
    // next at 0 row and 0 col , fil with what it would take to convert ""->empty string into that char, it would be 1 eidt
    // so fill it with indexes , next traverse from whole matrix , where zeroth of string is 1 in matrix

    let mut diff = vec![vec![0; cols]; rows];

    diff[0][0] = 0;
    for i in 1..cols {
        diff[0][i] = i;
    }

    for i in 1..rows {
        diff[i][0] = i;
    }

    for i in 1..rows {
        for j in 1..cols {
            if s1[i - 1] == s2[j - 1] {
                diff[i][j] = diff[i - 1][j - 1];
            } else if s1[i - 1] != s2[j - 1] {
                diff[i][j] = min(diff[i - 1][j - 1], min(diff[i - 1][j], diff[i][j - 1])) + 1;
            }
        }
    }

    return diff[rows - 1][cols - 1];
}

fn extract_between(s: &str) -> Option<&str> {
    let parts: Vec<&str> = s.split("#_#").collect();
    if parts.len() >= 3 {
        Some(parts[1])
    } else {
        None
    }
}
fn normalize(word: &str) -> String {
    word.to_lowercase()
        .trim()
        .chars()
        .filter(|c| c.is_alphanumeric())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::{self, File};
    use std::io::Write;

    #[test]
    fn test_load_songs() {
        let test_dir = "./test_lyrics_tmp";
        fs::create_dir_all(test_dir).unwrap();

        let file_path = format!("{}/prefix#_#Test Song#_#suffix.txt", test_dir);
        let mut file = File::create(&file_path).unwrap();
        writeln!(file, "Sample lyrics content").unwrap();

        let result = load_songs(test_dir.to_string());

        assert!(result.is_ok());

        fs::remove_dir_all(test_dir).unwrap();
    }

    #[test]
    fn test_extract_between_normal() {
        assert_eq!(
            extract_between("prefix#_#Song Title#_#suffix"),
            Some("Song Title")
        );
    }

    #[test]
    fn test_extract_between_missing_delimiter() {
        assert_eq!(extract_between("prefix suffix"), None);
    }

    #[test]
    fn test_extract_between_only_one_delimiter() {
        assert_eq!(extract_between("prefix#_#suffix"), None);
    }

    #[test]
    fn test_extract_between_multiple_delimiters() {
        assert_eq!(extract_between("a#_#b#_#c#_#d"), Some("b"));
    }

    #[test]
    fn test_normalize_basic() {
        assert_eq!(normalize("Hello"), "hello");
    }

    #[test]
    fn test_normalize_with_punctuation() {
        assert_eq!(normalize("Hello, World!"), "helloworld");
    }

    #[test]
    fn test_normalize_with_spaces() {
        assert_eq!(normalize("  hello  "), "hello");
    }

    #[test]
    fn test_normalize_with_mixed_case() {
        assert_eq!(normalize("HeLLo WoRLD"), "helloworld");
    }

    #[test]
    fn test_normalize_with_numbers() {
        assert_eq!(normalize("Hello123"), "hello123");
    }

    #[test]
    fn test_normalize_empty() {
        assert_eq!(normalize(""), "");
    }

    #[test]
    fn test_create_index_basic() {
        let songs = vec![Song {
            id: 0,
            title: "Test Song".to_string(),
            lyrics: "hello world\nhello again".to_string(),
        }];
        let index = create_index(songs).unwrap();

        assert!(!index.exact.is_empty());
        assert!(index.exact.contains_key("hello"));
        assert!(index.exact.contains_key("world"));
    }

    #[test]
    fn test_create_index_empty_songs() {
        let songs: Vec<Song> = vec![];
        let index = create_index(songs).unwrap();

        assert!(index.exact.is_empty());
        assert!(index.songs.is_empty());
    }

    #[test]
    fn test_create_index_word_positions() {
        let songs = vec![Song {
            id: 0,
            title: "Test".to_string(),
            lyrics: "hello world hello".to_string(),
        }];
        let index = create_index(songs).unwrap();

        let hello_hits = index.exact.get("hello");
        assert!(hello_hits.is_some());
        assert_eq!(hello_hits.unwrap().len(), 2);
    }

    #[test]
    fn test_search_single_word() {
        let songs = vec![
            Song {
                id: 0,
                title: "Song A".to_string(),
                lyrics: "hello world".to_string(),
            },
            Song {
                id: 1,
                title: "Song B".to_string(),
                lyrics: "goodbye world".to_string(),
            },
        ];
        let index = create_index(songs).unwrap();
        let results = search(&index, "world");

        assert_eq!(results.len(), 2);
    }

    #[test]
    fn test_search_multiple_words_and() {
        let songs = vec![
            Song {
                id: 0,
                title: "Song A".to_string(),
                lyrics: "hello beautiful world".to_string(),
            },
            Song {
                id: 1,
                title: "Song B".to_string(),
                lyrics: "hello world".to_string(),
            },
        ];
        let index = create_index(songs).unwrap();
        let results = search(&index, "hello world");

        assert_eq!(results.len(), 2);
    }

    #[test]
    fn test_search_no_results() {
        let songs = vec![Song {
            id: 0,
            title: "Song A".to_string(),
            lyrics: "hello world".to_string(),
        }];
        let index = create_index(songs).unwrap();
        let results = search(&index, "nonexistent");

        assert_eq!(results.len(), 0);
    }

    #[test]
    fn test_search_empty_query() {
        let songs = vec![Song {
            id: 0,
            title: "Song A".to_string(),
            lyrics: "hello world".to_string(),
        }];
        let index = create_index(songs).unwrap();
        let results = search(&index, "");

        assert_eq!(results.len(), 0);
    }

    #[test]
    fn test_search_case_insensitive() {
        let songs = vec![Song {
            id: 0,
            title: "Song A".to_string(),
            lyrics: "Hello World".to_string(),
        }];
        let index = create_index(songs).unwrap();
        let results = search(&index, "HELLO");

        assert_eq!(results.len(), 1);
    }

    #[test]
    fn test_identical() {
        assert_eq!(levenshtein("tum", "tum"), 0);
        assert_eq!(levenshtein("", ""), 0);
    }

    #[test]
    fn test_empty() {
        assert_eq!(levenshtein("", "tum"), 3);
        assert_eq!(levenshtein("pyaar", ""), 5);
    }

    #[test]
    fn test_one_edit() {
        assert_eq!(levenshtein("tum", "tuum"), 1); // insert
        assert_eq!(levenshtein("tum", "tu"), 1); // delete
        assert_eq!(levenshtein("tum", "tun"), 1); // replace
    }

    #[test]
    fn test_multiple_edits() {
        assert_eq!(levenshtein("pyaar", "piyar"), 2);
        assert_eq!(levenshtein("abc", "xyz"), 3);
        assert_eq!(levenshtein("kitten", "sitting"), 3);
    }
}
