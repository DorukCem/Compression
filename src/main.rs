use std::{
    collections::HashMap, env::current_exe, fs::File, io::{self, Read, Write}, path::Path, u128
};

use bitvec::{order::Lsb0, view::BitView};
use itertools::Itertools;

fn main() {
    let file_name = "az.txt";
    let path = Path::new("data").join(file_name);
    let file = File::open(path).expect("Could not open file");
    let metadata = file.metadata().expect("Can not read metadata");
    huffman(file);
}

struct HuffmanNode {
    ch: Option<char>,
    freq: usize,
    left: Option<Box<HuffmanNode>>,
    right: Option<Box<HuffmanNode>>,
}

impl HuffmanNode {
    fn new(ch: Option<char>, freq: usize) -> Self {
        HuffmanNode {
            ch: ch,
            freq: freq,
            left: None,
            right: None,
        }
    }

    fn combine(l_node: HuffmanNode, r_node: HuffmanNode) -> HuffmanNode {
        HuffmanNode {
            ch: None,
            freq: l_node.freq + r_node.freq,
            left: Some(Box::new(l_node)),
            right: Some(Box::new(r_node)),
        }
    }
}

fn print_codes(dict: &HashMap<char, u64>) {
    for (k, v) in dict.iter() {
        print!("{}: {:b}   ", k, v);
    }
    io::stdout().flush().unwrap();
}

fn huffman(mut file: File) {
    let mut contents = String::new();
    file.read_to_string(&mut contents).unwrap();

    let letter_freqs = contents.chars().counts();

    let tree: HuffmanNode = create_tree(letter_freqs);
    let dict = assign_codes(tree);
    print_codes(&dict);
    let (encoded_message, code_count) = encode_message(contents, &dict);
    let encoded_table = encode_table(&dict);
}

fn encode_table(dict: &HashMap<char, u64>) -> Vec<u8> {
    todo!()
}

fn encode_message(contents: String, dict: &HashMap<char, u64>) -> (Vec<u8>, u128) {
    let mut bytes = Vec::new();
    let mut current_byte: u8 = 0;
    let mut idx = 0;
    let mut code_count = 0u128;

    for letter in contents.chars() {
        let code = dict.get(&letter).unwrap();
        let code_len = 64 - code.leading_zeros() - 1;
        code_count += 1;

        for i in 0..=code_len {
            let most_signficant_digit = 1 << code_len - i;
            let left_bit = ((code & most_signficant_digit) > 0) as u8;
            current_byte = (current_byte << 1) | left_bit;
            // println!("code: {:b} current byte: {:08b}  left bit: {:b}", code , current_byte, left_bit);

            idx += 1;
            if idx == 8 {
                idx = 0;
                bytes.push(current_byte);
                current_byte = 0;
            }
        }
    }

    if current_byte > 0 {
        current_byte = current_byte << (8-idx);
        bytes.push(current_byte);
    }

    return (bytes, code_count);
}

fn assign_codes(tree: HuffmanNode) -> HashMap<char, u64> {
    let mut map = HashMap::new();
    recursive_assign_codes(tree, 0, &mut map);
    return map;
}

fn recursive_assign_codes(node: HuffmanNode, current: u64, map: &mut HashMap<char, u64>) {
    if let Some(ch) = node.ch {
        map.insert(ch, current);
    } else {
        if let Some(left) = node.left {
            recursive_assign_codes(*left, current << 1, map);
        }
        if let Some(right) = node.right {
            recursive_assign_codes(*right, (current << 1) | 1, map);
        }
    }
}

fn create_tree(letter_freqs: std::collections::HashMap<char, usize>) -> HuffmanNode {
    let mut nodes = letter_freqs
        .into_iter()
        .map(|(k, v)| HuffmanNode::new(Some(k), v))
        .collect_vec();

    while nodes.len() > 1 {
        nodes.sort_unstable_by(|a, b| b.freq.cmp(&a.freq));
        let a = nodes.pop().unwrap();
        let b = nodes.pop().unwrap();
        let comb = if a.freq <= b.freq {
            HuffmanNode::combine(a, b)
        } else {
            HuffmanNode::combine(b, a)
        };
        nodes.push(comb);
    }

    return nodes.pop().unwrap();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_assign_codes() {
        let freqs = HashMap::from([
            ('E', 120),
            ('D', 42),
            ('L', 42),
            ('U', 37),
            ('C', 32),
            ('M', 24),
            ('K', 7),
            ('Z', 2),
        ]);

        let tree: HuffmanNode = create_tree(freqs.clone());
        let dict = assign_codes(tree);

        let expected = HashMap::from([
            ('E', 0b0),
            ('D', 0b101),
            ('L', 0b110),
            ('U', 0b100),
            ('C', 0b1110),
            ('M', 0b11111),
            ('K', 0b111101),
            ('Z', 0b111100),
        ]);

        assert_eq!(expected, dict)
    }

    #[test]
    fn test_encode() {
        let codes = HashMap::from([
            ('E', 0b0),
            ('D', 0b101),
            ('L', 0b110),
            ('U', 0b100),
            ('C', 0b1110),
            ('M', 0b11111),
            ('K', 0b111101),
            ('Z', 0b111100),
        ]);

        let message = "LUCKDUCKD".to_owned();
        let (encoded_message, count) = encode_message(message, &codes);
        println!("{count}");
        for byte in encoded_message.iter() {
            println!("{:#010b}", byte)
        }
        let expected = Vec::from([0b110_100_11, 0b10_111101_, 0b101_100_11, 0b10_111101, 0b101_00000]);

        assert_eq!(encoded_message, expected)
    }
}
