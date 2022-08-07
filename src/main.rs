use std::collections::HashMap;
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

        if word_buf.len() == WORD_SIZE + 1
            && (&word_buf[..WORD_SIZE])
                .chars()
                .all(|c| c.is_ascii_alphabetic())
        {
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

fn main() -> anyhow::Result<()> {
    let words = read_words()?;
    eprintln!("words {}", words.len());

    let mut groups: HashMap<u32, Vec<String>> = HashMap::new();
    let mut bitsets: Vec<u32> = Vec::new();

    for word in words {
        let bitset = make_bitset(&word);
        if bitset.count_ones() != 5 {
            continue;
        }
        if !groups.contains_key(&bitset) {
            bitsets.push(bitset);
        }
        let v = groups.entry(bitset).or_default();
        v.push(word);
    }
    eprintln!("bitsets {}", bitsets.len());

    let mut indices: [usize; WORD_SIZE] = [0; WORD_SIZE];
    let mut i: usize = 0;
    let mut j: usize = 0;
    let mut acc: u32 = 0;

    loop {
        if indices[0] >= bitsets.len() {
            break;
        } else if i == 5 {
            let missing = bitset_to_letter(!acc & !0xfc00_0000);
            print_words(indices, &bitsets, &groups, missing);
            i -= 1;
            j = indices[i];
            acc ^= bitsets[j];
        } else if j == bitsets.len() {
            i -= 1;
            j = indices[i];
            acc ^= bitsets[j];
        } else if (acc | bitsets[j]).count_ones() == ((i + 1) * WORD_SIZE) as u32 {
            acc |= bitsets[j];
            indices[i] = j;
            i += 1;
        }
        j += 1;
    }

    return Ok(());
}

fn print_words(
    indices: [usize; WORD_SIZE],
    bitsets: &[u32],
    groups: &HashMap<u32, Vec<String>>,
    missing: char,
) {
    for a in groups.get(&bitsets[indices[0]]).unwrap() {
        for b in groups.get(&bitsets[indices[1]]).unwrap() {
            for c in groups.get(&bitsets[indices[2]]).unwrap() {
                for d in groups.get(&bitsets[indices[3]]).unwrap() {
                    for e in groups.get(&bitsets[indices[4]]).unwrap() {
                        println!("{} {} {} {} {} {}", a, b, c, d, e, missing);
                    }
                }
            }
        }
    }
}
