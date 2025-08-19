mod skiplist;

use skiplist::SkipList;
use crate::skiplist::get_max_level;
use std::cmp::Ordering;

fn int_comparator(a: &i32, b: &i32) -> Ordering {
    a.cmp(b)
}

fn main() {
    // Create a skip list similar to the C version
    let max_level = get_max_level(11, 0.5);
    println!("max level: {}", max_level);
    
    let mut skip_list = SkipList::new(max_level, 0.5, int_comparator);

    println!("\n=== Testing Insertions ===");
    for i in [(3,'a'), (6, 'b'), (7, 'c'), (12,'d'), (19,'e'), (17,'f'), (26,'g'), (21,'h'), (25,'i'), (21,'k')] {
        let result = skip_list.insert(i.0, i.1);
        println!("Inserted {}:{} -- {:?}", i.0, i.1, result);
    }
    skip_list.display_list(Some(|key: &i32, data: &char| {
        print!("{:>6}:{}", key, data);
    }));
    for key in [6, 9, 19, 26, 25] {
        match skip_list.remove(&key) {
            Some(data) => println!("Removed: {}:{}", key, data),
            None => println!("Unable to remove {}, key not found", key),
        }
    }
        
    skip_list.display_list(Some(|key: &i32, data: &char| {
        print!("{:>6}:{}", key, data);
    }));
}
