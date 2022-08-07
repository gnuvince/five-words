use std::io::{self, BufRead, BufReader};

const WORD_SIZE: usize = 5;

fn read_words() -> anyhow::Result<Vec<String>> {
    let stdin = io::stdin();
    let stdin = stdin.lock();
    let mut stdin = BufReader::new(stdin);

    let mut word_buf: String = String::with_capacity(4096);

    let mut words: Vec<String> = Vec::new();

    loop {
        word_buf.clear();
        let n = stdin.read_line(&mut word_buf)?;
        if n == 0 {
            break;
        }

        if word_buf.len() == WORD_SIZE + 1 {
            words.push(word_buf.trim().to_ascii_lowercase());
        }
    }
    return Ok(words);
}

fn make_bitset(word: &str) -> u32 {
    let mut bitset: u32 = 0;
    for c in 'a'..='z' {
        if word.contains(c) {
            let i = c as u32 - 'a' as u32;
            let bit = 1 << i;
            bitset |= bit;
        }
    }
    return bitset;
}

#[test]
fn test_make_bitset() {
    assert_eq!(make_bitset(""), 0);
    assert_eq!(make_bitset("a"), 1);
    assert_eq!(make_bitset("b"), 2);
    assert_eq!(make_bitset("c"), 4);
    assert_eq!(make_bitset("cab"), 7);
}

fn bitset_to_letter(b: u32) -> char {
    let lz = (b.leading_zeros() - 5) as u8;
    let pos = 26 - lz;
    return (pos + b'a') as char;
}

#[test]
fn test_bitset_to_letter() {
    assert_eq!(bitset_to_letter(1 << 0), 'a');
    assert_eq!(bitset_to_letter(1 << 1), 'b');
    assert_eq!(bitset_to_letter(1 << 2), 'c');
    assert_eq!(bitset_to_letter(1 << 23), 'x');
    assert_eq!(bitset_to_letter(1 << 24), 'y');
    assert_eq!(bitset_to_letter(1 << 25), 'z');
}

fn make_bitsets(words: &[String]) -> Vec<u32> {
    let mut bitsets: Vec<u32> = Vec::new();
    for word in words {
        bitsets.push(make_bitset(word));
    }
    return bitsets;
}

fn main() -> anyhow::Result<()> {
    let mut words = read_words()?;
    let mut words_bitsets = make_bitsets(&words);

    assert_eq!(words.len(), words_bitsets.len());

    // Keep only the words that have `WORD_SIZE` distinct letters.
    {
        let mut i: usize = 0;
        let mut j: usize = words.len() - 1;

        while i < j {
            if words_bitsets[i].count_ones() != WORD_SIZE as u32 {
                words.swap(i, j);
                words_bitsets.swap(i, j);
                j -= 1;
            } else {
                i += 1;
            }
        }

        words.truncate(j);
        words_bitsets.truncate(j);
    }

    let mut indices: [usize; WORD_SIZE] = [0; WORD_SIZE];
    let mut i: usize = 0;
    let mut j: usize = 0;
    let mut acc: u32 = 0;

    while i < WORD_SIZE {
        if j == words_bitsets.len() {
            i -= 1;
            j = indices[i];
            acc ^= words_bitsets[j];
        } else if (acc | words_bitsets[j]).count_ones() == ((i + 1) * WORD_SIZE) as u32 {
            acc |= words_bitsets[j];
            indices[i] = j;
            i += 1;
        }
        j += 1;
    }

    for i in indices {
        print!("{} ", words[i]);
    }
    let missing = bitset_to_letter(!acc & !0xfc00_0000);
    println!("{}", missing);

    return Ok(());
}