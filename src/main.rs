mod skiplist;

use skiplist::SkipList;
use crate::skiplist::get_max_level;


fn string_comparator(a: &String, b: &String) -> i32 {
    a.cmp(b) as i32
}

fn main() {
    const P: f32 = 0.5;

    let data = [
        "a", "e", "w", "d", "q", "u", "y", "b", "n", "c", "t", "m", "f",
        "z", "g", "o", "s", "h", "v", "i", "j", "p", "k", "r", "x", "l",
    ];

    let max_level = get_max_level(data.len(), P);
    println!("max level: {}", max_level);

    let mut skip_list = SkipList::<String, ()>::new(string_comparator, P, max_level);

}
