use std::{
    cmp::Ordering,
    collections::{btree_map::Keys, HashMap},
    env::{self, current_exe},
    fs::{self, File},
    io::{self, BufWriter, Read, Write},
    iter::zip,
    path::Path,
    u128,
};

use bitvec::{order::Lsb0, store::BitStore, view::BitView};
use itertools::Itertools;

fn main() {
    let file_name = "indiana.txt";
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
            freq: freq,
            ch: ch,
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

impl Ord for HuffmanNode {
    fn cmp(&self, other: &Self) -> Ordering {
        (self.freq, self.ch.unwrap_or_default()).cmp(&(other.freq, other.ch.unwrap_or_default()))
    }
}

impl PartialOrd for HuffmanNode {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for HuffmanNode {
    fn eq(&self, other: &Self) -> bool {
        (self.freq, self.ch) == (other.freq, other.ch)
    }
}

impl Eq for HuffmanNode {}

fn print_codes(dict: &HashMap<char, u64>) {
    for (k, v) in dict.iter() {
        print!("{}: {:b}   ", k, v);
    }
    print!("\n");
    io::stdout().flush().unwrap();
}

#[derive(PartialEq, Debug, Clone)]
struct Table {
    len_table: u32,
    chars: Vec<char>,
    codes: Vec<u64>,
}

#[derive(PartialEq, Debug, Clone)]
struct HuffmanData {
    message: Vec<u8>,
    extra_bits: u8,
    table: Table,
}

fn huffman(mut file: File) {
    let mut contents = String::new();
    file.read_to_string(&mut contents).unwrap();

    let letter_freqs = contents.chars().counts();
    println!("{:?}", letter_freqs);
    let tree: HuffmanNode = create_tree(letter_freqs);
    let dict = assign_codes(tree);
    print_codes(&dict);
    let (encoded_message, extra_bits) = encode_message(contents.clone(), &dict);
    let encoded_table = encode_table(&dict);
    let data = HuffmanData {
        message: encoded_message,
        extra_bits: extra_bits,
        table: encoded_table,
    };

    create_compressed_file(data.clone());
    let decompressed_data = read_compressed_file();

    assert_eq!(
        data.table.len_table, decompressed_data.table.len_table,
        "table length"
    );
    assert_eq!(
        data.table.chars, decompressed_data.table.chars,
        "table chars"
    );
    assert_eq!(
        data.table.codes, decompressed_data.table.codes,
        "table codes"
    );
    assert_eq!(data.extra_bits, decompressed_data.extra_bits, "extra bits");
    assert_eq!(data.message, decompressed_data.message, "message");

    let decompressed_message = decode_huffman_data(decompressed_data);
    assert_eq!(contents, decompressed_message);
}

fn decode_huffman_data(data: HuffmanData) -> String {
    let table = data.table;
    let hmap = zip(table.codes, table.chars).collect::<HashMap<_, _>>();
    let message = data.message;
    let extra_bits_length = data.extra_bits;
    // println!("{extra_bits_length}");
    let mut current_code = 0;
    let mut result = String::new();

    for byte in message {
        let mut current_byte = byte.reverse_bits();
        // print!("{:08b}  ", current_byte);
        for _ in 0..8 {
            let bit = (current_byte & 1) as u64;
            current_code = (current_code << 1) | bit;
            
            if let Some(ch) = hmap.get(&current_code) {
                result.push(*ch);
                current_code = 0;
            }
            current_byte >>= 1;
        }
    }
    result.truncate(result.len() - extra_bits_length as usize);
    
    // print!("\n");
    // io::stdout().flush();
    return result;
}

fn create_compressed_file(data: HuffmanData) {
    let mut file = File::create("compressed-data/indiana.txt").expect("cannot create file");
    let mut br = BufWriter::new(file);
    br.write_all(&data.table.len_table.to_be_bytes());
    br.write_all(&String::from_iter(data.table.chars).as_bytes());
    let codes: Vec<u8> = data
        .table
        .codes
        .iter()
        .map(|x| x.to_be_bytes())
        .flatten()
        .collect();
    br.write_all(&codes);
    br.write_all(&data.extra_bits.to_be_bytes());
    br.write_all(&data.message);
}

fn read_compressed_file() -> HuffmanData {
    let bytes = fs::read("compressed-data/indiana.txt").expect("cannot compressed read file");
    // for b in bytes.iter() {
    //     print!("{:08b}   ", b);
    // }
    // io::stdout().flush();

    let len_table = u32::from_be_bytes(bytes[0..4].try_into().unwrap());
    let table_keys_end = 4 + len_table as usize;
    let table_keys = &bytes[4..table_keys_end];
    let table_keys: Vec<char> = table_keys.iter().map(|b| *b as char).collect();
    let table_codes_end = table_keys_end + len_table as usize * 8;
    let table_codes = &bytes[table_keys_end..table_codes_end].to_vec();
    let table_codes: Vec<u64> = table_codes
        .chunks_exact(8)
        .map(|chunk| u64::from_be_bytes(chunk.try_into().unwrap()))
        .collect_vec();

    let extra_bits = u8::from_be(bytes[table_codes_end]);
    let message = &bytes[(table_codes_end + 1) as usize..bytes.len()].to_vec();

    return HuffmanData {
        message: message.clone(),
        extra_bits: extra_bits,
        table: Table {
            len_table: len_table,
            chars: table_keys,
            codes: table_codes,
        },
    };
}

fn encode_table(dict: &HashMap<char, u64>) -> Table {
    let (chars, codes): (Vec<char>, Vec<u64>) = dict.iter().unzip();

    return Table {
        len_table: chars.len() as u32,
        chars: chars,
        codes: codes,
    };
}

fn encode_message(contents: String, dict: &HashMap<char, u64>) -> (Vec<u8>, u8) {
    let mut bytes = Vec::new();
    let mut current_byte: u8 = 0;
    let mut idx = 0;

    for letter in contents.chars() {
        let code = dict.get(&letter).unwrap();
        let code_len = if *code != 0 {
            64 - code.leading_zeros() - 1
        } else {
            0
        };

        for i in 0..=code_len {
            let most_signficant_digit = 1 << code_len - i;
            let left_bit = ((code & most_signficant_digit) > 0) as u8;
            current_byte = (current_byte << 1) | left_bit;

            idx += 1;
            if idx == 8 {
                idx = 0;
                bytes.push(current_byte);
                current_byte = 0;
            }
        }
    }

    let mut extra_bits: u8 = 0;
    if current_byte > 0 {
        current_byte = current_byte << (8 - idx);
        bytes.push(current_byte);
        extra_bits = 8 - idx;
    }

    return (bytes, extra_bits);
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
        nodes.sort_by(|a, b| b.cmp(a));
        let a = nodes.pop().unwrap();
        let b = nodes.pop().unwrap();
        let comb = HuffmanNode::combine(a, b);
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

        let message = "LUCKDUCKDE".to_owned();
        let (encoded_message, count) = encode_message(message, &codes);
        println!("{count}");
        for byte in encoded_message.iter() {
            println!("{:#010b}", byte)
        }
        let expected = Vec::from([
            0b110_100_11,
            0b10_111101_,
            0b101_100_11,
            0b10_111101,
            0b101_0_0000,
        ]);

        assert_eq!(encoded_message, expected)
    }

    #[test]
    fn test_pack_unpack() {
        let data: Vec<u64> = vec![0b11, 0b11001, 0b11010, 0b10];

        let codes: Vec<u8> = data.iter().map(|x| x.to_be_bytes()).flatten().collect();

        for b in codes.iter() {
            print!("{:08b}   ", b);
        }
        io::stdout().flush();

        let decomp = codes
            .chunks_exact(8)
            .map(|chunk| u64::from_be_bytes(chunk.try_into().unwrap()))
            .collect_vec();

        println!("\n--");

        for b in decomp.iter() {
            print!("{:b}   ", b);
        }
        io::stdout().flush();
        assert_eq!(data, decomp);
    }

    #[test]
    fn test_ordering() {
        let d = HuffmanNode::new(Some('D'), 42);
        let l = HuffmanNode::new(Some('L'), 42);
        assert!(d < l);
    }
}
