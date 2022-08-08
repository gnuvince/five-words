use std::collections::HashMap;
use std::io::{self, BufRead, BufReader};
use std::sync::mpsc;
use std::sync::Arc;

const WORD_SIZE: usize = 5;
const OTHER_WORDS: usize = 4;

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
    for c in word.chars() {
        let i = c as u32 - 'a' as u32;
        let bit = 1 << i;
        bitset |= bit;
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
    let pool = threadpool::ThreadPool::new(8);

    let words = read_words()?;
    eprintln!("words {}", words.len());

    let mut groups: HashMap<u32, Vec<String>> = HashMap::new();
    let mut bitsets: Vec<u32> = Vec::new();

    for word in words {
        let bitset = make_bitset(&word);
        if bitset.count_ones() != WORD_SIZE as u32 {
            continue;
        }
        if !groups.contains_key(&bitset) {
            bitsets.push(bitset);
        }
        let v = groups.entry(bitset).or_default();
        v.push(word);
    }
    let shared_groups = Arc::new(groups); // For threadpool
    eprintln!("bitsets {}", bitsets.len());

    let t = std::time::Instant::now();
    let mut non_conflicting_bitsets: Vec<(u32, Vec<u32>)> = Vec::new();
    for i in 0..bitsets.len() {
        let mut v: Vec<u32> = Vec::new();
        for j in i + 1..bitsets.len() {
            if bitsets[i] & bitsets[j] == 0 {
                v.push(bitsets[j]);
            }
        }
        non_conflicting_bitsets.push((bitsets[i], v));
    }
    eprintln!("time to build non_conflicting_bitsets {:?}", t.elapsed());

    let (tx, rx) = mpsc::channel::<Vec<String>>();
    for (k, v) in non_conflicting_bitsets {
        let shared_groups = shared_groups.clone();
        let tx = tx.clone();
        pool.execute(move || {
            let mut indices: [usize; OTHER_WORDS] = [0; OTHER_WORDS];
            let mut i: usize = 0;
            let mut j: usize = 0;
            let mut acc: u32 = k;

            loop {
                if i == OTHER_WORDS {
                    // Successfully found a set of five words; write them out, send them
                    // to the main thread, and backtrack to find more candidate.
                    let missing = bitset_to_letter(!(acc | 0xfc00_0000));
                    let words = gen_words(k, indices, &v, &shared_groups, missing);
                    tx.send(words).expect("send");
                    i -= 1;
                    j = indices[i];
                    acc ^= v[j]
                } else if j == v.len() {
                    // Reached the end of the word list without finding a set of 5;
                    // exit the loop if we exhausted all possibilities, or else backtrack.
                    if i == 0 {
                        break;
                    }
                    i -= 1;
                    j = indices[i];
                    acc ^= v[j];
                } else if acc & v[j] == 0 {
                    // Found a match (empty intersection between acc and the current word);
                    // add it to the set.
                    acc |= v[j];
                    indices[i] = j;
                    i += 1;
                }
                j += 1;
            }
        });
    }
    pool.join();

    drop(tx);

    for v in rx.into_iter() {
        for m in v {
            println!("{}", m);
        }
    }

    return Ok(());
}

fn gen_words(
    key: u32,
    indices: [usize; OTHER_WORDS],
    bitsets: &[u32],
    groups: &HashMap<u32, Vec<String>>,
    missing: char,
) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for a in groups.get(&key).unwrap() {
        for b in groups.get(&bitsets[indices[0]]).unwrap() {
            for c in groups.get(&bitsets[indices[1]]).unwrap() {
                for d in groups.get(&bitsets[indices[2]]).unwrap() {
                    for e in groups.get(&bitsets[indices[3]]).unwrap() {
                        out.push(format!("{} {} {} {} {} {}", a, b, c, d, e, missing));
                    }
                }
            }
        }
    }
    return out;
}
