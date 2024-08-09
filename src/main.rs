use bitvec::prelude::*;
use itertools::Itertools;
use std::{collections::HashMap, fmt, fs::File, io::{BufWriter, Read}, path::Path};
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug)]
struct HuffmanData {
    dictionary: HashMap<String, char>,
    message: BitVec,
}

struct Node {
    ch: Option<char>,
    freq: usize,
    left: Option<Box<Node>>,
    right: Option<Box<Node>>,
}
impl Node {
    fn new(ch: Option<char>, freq: usize) -> Self {
        Node {
            ch: ch,
            freq: freq,
            left: None,
            right: None,
        }
    }

    fn combine(l_node: Node, r_node: Node) -> Node {
        Node {
            ch: None,
            freq: l_node.freq + r_node.freq,
            left: Some(Box::new(l_node)),
            right: Some(Box::new(r_node)),
        }
    }

    fn assign_codes(node: Node) -> HashMap<char, String> {
        let mut map: HashMap<char, String> = HashMap::new();
        Self::recursive_assign_codes(node, "".to_owned(), &mut map);
        return map;
    }

    fn recursive_assign_codes(node: Node, current: String, map: &mut HashMap<char, String>) {
        if let Some(ch) = node.ch {
            map.insert(ch, current);
        } else {
            if let Some(left) = node.left {
                Self::recursive_assign_codes(*left, current.clone() + "0", map);
            }
            if let Some(right) = node.right {
                Self::recursive_assign_codes(*right, current.clone() + "1", map);
            }
        }
    }
}

impl fmt::Debug for Node {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "ch: {:#?}, freq: {}, left: {:#?}, right: {:#?}",
            self.ch, self.freq, self.left, self.right
        )
    }
}

fn encode_as_bits(dict: &HashMap<char, String>, message: &String) -> BitVec {
    let mut bv: BitVec = bitvec!();
    for c in message.chars() {
        let code = dict.get(&c).unwrap();
        for binary in code.chars() {
            match binary {
                '0' => bv.push(false),
                '1' => bv.push(true),
                _ => panic!("Encoding should have only 0's and 1's"),
            }
        }
    }

    return bv;
}

fn decode_bits(decode_dict: HashMap<String, char>, bits: BitVec) -> String {
    let mut result : String = String::new();
    let mut current = String::new();
    
    for b in bits {
        if b {
            current += "1"
        } else {
            current += "0"
        }
        if let Some(ch) = decode_dict.get(&current) {
            result.push(*ch);
            current.clear();
        }
    }
    return result;
}

fn huffman(mut file: File) -> HuffmanData {
    let mut contents = String::new();
    file.read_to_string(&mut contents).unwrap();

    let letter_freqs = contents.chars().counts();
    let mut nodes = letter_freqs
        .into_iter()
        .map(|(k, v)| Node::new(Some(k), v))
        .collect_vec();

    while nodes.len() > 1 {
        nodes.sort_unstable_by(|a, b| b.freq.cmp(&a.freq));
        let a = nodes.pop().unwrap();
        let b = nodes.pop().unwrap();
        let comb = if a.freq <= b.freq {
            Node::combine(a, b)
        } else {
            Node::combine(b, a)
        };
        nodes.push(comb);
    }

    let dict = Node::assign_codes(nodes.pop().unwrap());
    let encoded_bits = encode_as_bits(&dict, &contents);
    let decode_dict: HashMap<String, char> = dict.into_iter().map(|(k, v)| (v, k)).collect();

    assert_eq!(decode_bits(decode_dict.clone(), encoded_bits.clone()), contents.clone());

    return HuffmanData { dictionary: decode_dict, message: encoded_bits }

}

fn main() {
    let path = Path::new("data").join("romeo-juliet.txt") ;
    let file = File::open(path).expect("Could not open file");
    let metadata = file.metadata().expect("Can not read metadata");
    let huffman_data: HuffmanData = huffman(file);
    let save_path = Path::new("./compressed-data/").join("romeo-juliet.txt");
    let save_file = File::create(save_path).unwrap();
}
