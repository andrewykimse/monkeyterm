use rand::seq::SliceRandom;
use rand::thread_rng;

/// Built-in word lists
const COMMON_200: &[&str] = &[
    "the", "be", "to", "of", "and", "a", "in", "that", "have", "it",
    "for", "not", "on", "with", "he", "as", "you", "do", "at", "this",
    "but", "his", "by", "from", "they", "we", "say", "her", "she", "or",
    "an", "will", "my", "one", "all", "would", "there", "their", "what",
    "so", "up", "out", "if", "about", "who", "get", "which", "go", "me",
    "when", "make", "can", "like", "time", "no", "just", "him", "know",
    "take", "people", "into", "year", "your", "good", "some", "could",
    "them", "see", "other", "than", "then", "now", "look", "only", "come",
    "its", "over", "think", "also", "back", "after", "use", "two", "how",
    "our", "work", "first", "well", "way", "even", "new", "want", "because",
    "any", "these", "give", "day", "most", "us", "great", "between", "need",
    "large", "often", "hand", "high", "place", "hold", "turn", "were", "main",
    "move", "live", "where", "much", "before", "line", "right", "too", "mean",
    "old", "any", "same", "tell", "boy", "follow", "came", "want", "show",
    "also", "around", "form", "small", "set", "put", "end", "does", "another",
    "well", "large", "need", "big", "go", "home", "us", "try", "ask", "those",
    "start", "very", "light", "open", "seem", "together", "next", "white",
    "children", "begin", "got", "walk", "example", "ease", "paper", "group",
    "always", "music", "those", "both", "mark", "often", "letter", "until",
];

const PROGRAMMING: &[&str] = &[
    "function", "variable", "return", "const", "let", "mut", "struct",
    "enum", "match", "if", "else", "loop", "while", "for", "impl", "trait",
    "pub", "mod", "use", "crate", "type", "where", "async", "await", "move",
    "closure", "iterator", "vector", "hashmap", "result", "option", "error",
    "string", "slice", "reference", "ownership", "borrow", "lifetime", "generic",
    "trait", "interface", "class", "object", "method", "field", "module",
    "package", "import", "export", "default", "static", "dynamic", "stack",
    "heap", "pointer", "null", "boolean", "integer", "float", "array", "list",
    "tuple", "map", "set", "queue", "stack", "tree", "graph", "node", "edge",
    "index", "key", "value", "hash", "cache", "thread", "mutex", "channel",
    "future", "promise", "callback", "event", "stream", "buffer", "socket",
    "server", "client", "request", "response", "header", "body", "token",
    "parse", "format", "serialize", "deserialize", "encode", "decode",
    "compile", "runtime", "debug", "release", "test", "bench", "docs",
];

const PANGRAMS: &[&str] = &[
    "The quick brown fox jumps over the lazy dog",
    "Pack my box with five dozen liquor jugs",
    "How vexingly quick daft zebras jump",
    "The five boxing wizards jump quickly",
    "Sphinx of black quartz judge my vow",
    "Jackdaws love my big sphinx of quartz",
    "The job requires extra pluck and zeal from every young wage earner",
    "A quivering Texas zombie fought republic linked jewelry",
];

#[derive(Debug, Clone, PartialEq)]
pub enum WordList {
    Common200,
    Programming,
}

impl WordList {
    pub fn words(&self) -> &'static [&'static str] {
        match self {
            WordList::Common200 => COMMON_200,
            WordList::Programming => PROGRAMMING,
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            WordList::Common200 => "english",
            WordList::Programming => "code",
        }
    }
}

pub fn generate_word_list(word_list: &WordList, count: usize) -> Vec<String> {
    let mut rng = thread_rng();
    let words = word_list.words();
    let mut chosen: Vec<String> = words
        .choose_multiple(&mut rng, count.min(words.len()))
        .map(|s| s.to_string())
        .collect();

    // If we need more words than available, sample with replacement
    if count > words.len() {
        while chosen.len() < count {
            let extra: Vec<String> = words
                .choose_multiple(&mut rng, count - chosen.len())
                .map(|s| s.to_string())
                .collect();
            chosen.extend(extra);
        }
    }

    chosen.truncate(count);
    chosen
}

pub fn get_quote() -> String {
    let mut rng = thread_rng();
    PANGRAMS.choose(&mut rng).unwrap().to_string()
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generate_word_list_returns_exact_count() {
        let words = generate_word_list(&WordList::Common200, 25);
        assert_eq!(words.len(), 25);
    }

    #[test]
    fn generate_word_list_zero_returns_empty() {
        let words = generate_word_list(&WordList::Common200, 0);
        assert_eq!(words.len(), 0);
    }

    #[test]
    fn generate_word_list_more_than_available_fills_to_count() {
        // Common200 has ~200 unique words; request 500 → should still give 500
        let words = generate_word_list(&WordList::Common200, 500);
        assert_eq!(words.len(), 500);
    }

    #[test]
    fn generate_word_list_all_non_empty_strings() {
        let words = generate_word_list(&WordList::Common200, 50);
        for w in &words {
            assert!(!w.is_empty(), "generated an empty word");
        }
    }

    #[test]
    fn generate_word_list_programming_uses_programming_words() {
        let words = generate_word_list(&WordList::Programming, 30);
        assert_eq!(words.len(), 30);
        // Programming words should all be ASCII
        for w in &words {
            assert!(w.is_ascii(), "programming word '{w}' is not ASCII");
        }
    }

    #[test]
    fn word_list_names() {
        assert_eq!(WordList::Common200.name(), "english");
        assert_eq!(WordList::Programming.name(), "code");
    }

    #[test]
    fn get_quote_is_nonempty_multiword_string() {
        let quote = get_quote();
        assert!(!quote.is_empty());
        assert!(quote.contains(' '), "quote should contain at least two words");
    }

    #[test]
    fn get_quote_is_from_known_set() {
        // Every returned quote must exist in PANGRAMS
        for _ in 0..20 {
            let q = get_quote();
            assert!(
                PANGRAMS.contains(&q.as_str()),
                "get_quote returned unknown string: {q:?}"
            );
        }
    }
}
