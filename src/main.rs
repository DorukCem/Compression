use std::{collections::HashMap, fmt, fs::File, io::Read};

use itertools::Itertools;

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
        println!("{:?} {:?}", l_node.ch, r_node.ch);
        Node {
            ch: None,
            freq: l_node.freq + r_node.freq,
            left: Some(Box::new(l_node)),
            right: Some(Box::new(r_node)),
        }
    }

    fn assign_codes(node: Node) -> HashMap<char, String>{
        let mut map: HashMap<char, String> = HashMap::new();
        Self::recursive_assign_codes(node, "".to_owned(), &mut map);
        println!("{:#?}", map);
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

fn dictionary_encode(dict : HashMap<char, String>, message: String)-> String{
    message.chars().map(|x|  dict.get(&x).unwrap()).join("")
}

fn huffman(mut file: File) {
    let mut contents = String::new();
    file.read_to_string(&mut contents).unwrap();

    let letter_freqs = contents.chars().counts();
    println!("{:?}", &letter_freqs);
    let mut nodes = letter_freqs
        .into_iter()
        .map(|(k, v)| Node::new(Some(k), v))
        .collect_vec();

    while nodes.len() > 1 {
        nodes.sort_unstable_by(|a, b| b.freq.cmp(&a.freq));
        let a = nodes.pop().unwrap();
        let b = nodes.pop().unwrap();
        let comb = if a.freq >= b.freq {
            Node::combine(a, b)
        } else {
            Node::combine(b, a)
        };
        nodes.push(comb);
    }

    let dict = Node::assign_codes(nodes.pop().unwrap());
    let encoded_string = dictionary_encode(dict, contents);
    println!("{}", encoded_string);
}

fn main() {
    let file = File::open("data/az.txt").expect("Could not open file");
    // let metadata = file.metadata().expect("Can not read metadata");
    huffman(file);
}
