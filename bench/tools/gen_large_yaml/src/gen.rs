#![allow(clippy::too_many_arguments)]

use std::sync::LazyLock;

use rand::{distr::Alphanumeric, rngs::SmallRng, RngExt};

static LIPSUM_WORDS: LazyLock<Vec<&'static str>> = LazyLock::new(|| {
    lipsum::LOREM_IPSUM
        .split_whitespace()
        .chain(lipsum::LIBER_PRIMUS.split_whitespace())
        .collect()
});

/// Generate lipsum words using the outer generator's random state.
fn lipsum_words(rng: &SmallRng, nwords: usize) -> String {
    let mut rng = rng.clone();
    let mut output = String::new();

    for index in 0..nwords {
        if index > 0 {
            output.push(' ');
        }
        output.push_str(LIPSUM_WORDS[rng.random_range(0..LIPSUM_WORDS.len())]);
    }

    output
}

/// Generate a string with hexadecimal digits of the specified length.
pub fn hex_string(rng: &mut SmallRng, len: usize) -> String {
    const DIGITS: &[u8] = b"0123456789abcdef";
    string_from_set(rng, len, len + 1, DIGITS)
}

/// Generate an e-mail address.
pub fn email(rng: &mut SmallRng, len_lo: usize, len_hi: usize) -> String {
    const CHARSET: &[u8] = b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ-_.0123456789";
    format!(
        "{}@example.com",
        string_from_set(rng, len_lo, len_hi, CHARSET)
    )
}

/// Generate a random URL.
pub fn url(
    rng: &mut SmallRng,
    scheme: &str,
    n_paths_lo: usize,
    n_paths_hi: usize,
    path_len_lo: usize,
    path_len_hi: usize,
    extension: Option<&str>,
) -> String {
    let mut string = format!("{scheme}://example.com");
    for _ in 0..rng.random_range(n_paths_lo..n_paths_hi) {
        string.push('/');
        string.push_str(&alnum_string(rng, path_len_lo, path_len_hi));
    }
    if let Some(extension) = extension {
        string.push('.');
        string.push_str(extension);
    }
    string
}

/// Generate a random integer.
pub fn integer(rng: &mut SmallRng, lo: i64, hi: i64) -> i64 {
    rng.random_range(lo..hi)
}

/// Generate an alphanumeric string with a length between `lo_len` and `hi_len`.
pub fn alnum_string(rng: &mut SmallRng, lo_len: usize, hi_len: usize) -> String {
    let len = rng.random_range(lo_len..hi_len);
    rng.sample_iter(Alphanumeric)
        .take(len)
        .map(char::from)
        .collect()
}

/// Generate a string with hexadecimal digits of the specified length.
pub fn string_from_set(rng: &mut SmallRng, len_lo: usize, len_hi: usize, set: &[u8]) -> String {
    (0..rng.random_range(len_lo..len_hi))
        .map(|_| set[rng.random_range(0..set.len())] as char)
        .collect()
}

/// Generate a lipsum paragraph.
pub fn paragraph(
    rng: &mut SmallRng,
    lines_lo: usize,
    lines_hi: usize,
    wps_lo: usize,
    wps_hi: usize,
    line_maxcol: usize,
) -> Vec<String> {
    let mut ret = Vec::new();
    let nlines = rng.random_range(lines_lo..lines_hi);

    while ret.len() < nlines {
        let words_in_sentence = rng.random_range(wps_lo..wps_hi);
        let mut sentence = lipsum_words(rng, words_in_sentence);

        if let Some(last_line) = ret.pop() {
            sentence = format!("{last_line} {sentence}");
        }

        while sentence.len() > line_maxcol {
            let last_space_idx = line_maxcol
                - sentence[0..line_maxcol]
                    .chars()
                    .rev()
                    .position(char::is_whitespace)
                    .unwrap();
            ret.push(sentence[0..last_space_idx].to_string());
            sentence = sentence[last_space_idx + 1..].to_string();
        }
        if !sentence.is_empty() {
            ret.push(sentence);
        }
    }

    ret
}

/// Generate a full name.
pub fn full_name(rng: &mut SmallRng, len_lo: usize, len_hi: usize) -> String {
    format!(
        "{} {}",
        name(rng, len_lo, len_hi),
        name(rng, len_lo, len_hi)
    )
}

/// Generate a name.
pub fn name(rng: &mut SmallRng, len_lo: usize, len_hi: usize) -> String {
    const UPPER: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ";
    const LOWER: &[u8] = b"abcdefghijklmnopqrstuvwxyz";

    let len = rng.random_range(len_lo..len_hi);
    let mut ret = String::new();
    ret.push(UPPER[rng.random_range(0..UPPER.len())] as char);
    ret.push_str(string_from_set(rng, len, len + 1, LOWER).as_str());

    ret
}

/// Generate a set of words.
pub fn words(rng: &mut SmallRng, words_lo: usize, words_hi: usize) -> String {
    let nwords = rng.random_range(words_lo..words_hi);
    lipsum_words(rng, nwords).replace(|c| "-\'\",*:".contains(c), "")
}

/// Generate a lipsum text.
///
/// Texts are composed of some paragraphs and empty lines between them.
pub fn text(
    rng: &mut SmallRng,
    paragraphs_lo: usize,
    paragraphs_hi: usize,
    lines_lo: usize,
    lines_hi: usize,
    wps_lo: usize,
    wps_hi: usize,
    line_maxcol: usize,
) -> Vec<String> {
    let mut ret = Vec::new();
    let mut first = true;

    for _ in 0..rng.random_range(paragraphs_lo..paragraphs_hi) {
        if first {
            first = false;
        } else {
            ret.push(String::new());
        }

        ret.extend(paragraph(
            rng,
            lines_lo,
            lines_hi,
            wps_lo,
            wps_hi,
            line_maxcol,
        ));
    }

    ret
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use rand::SeedableRng;

    use super::*;

    #[test]
    fn equal_length_word_sequences_vary() {
        let mut rng = SmallRng::seed_from_u64(0);
        let samples = (0..32)
            .map(|_| words(&mut rng, 10, 11))
            .collect::<HashSet<_>>();

        assert!(samples.len() > 1);
    }
}
