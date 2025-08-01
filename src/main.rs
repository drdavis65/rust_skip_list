mod skiplist;
use skiplist::*;
use std::cmp::Ordering;

struct StringComparator;

impl Comparator<String> for StringComparator {
    fn compare(&self, a: &String, b: &String) -> Ordering {
        a.cmp(b)
    }
}

fn main() {
    const P: f32 = 0.5;

    let data = [
        "a", "e", "w", "d", "q", "u", "y", "b", "n", "c", "t", "m", "f", "z",
        "g", "o", "s", "h", "v", "i", "j", "p", "k", "r", "x", "l",
    ];

    let max_level = 16;
    let mut skiplist = SkipList::<String, Box<i32>>::new(
        Box::new(StringComparator),
        None,
        P,
        max_level,
    );

    println!("\nEmpty skip list");
    display_list(&skiplist);

    println!("\nInserting elements");
    for &key in &data {
        insert(&mut skiplist, key.to_string(), Box::new(0));
    }
    display_list(&skiplist);

    println!("\nInserting an element which is already in the list");
    if let Some(old) = insert(&mut skiplist, data[0].to_string(), Box::new(0)) {
        drop(old);
    }
    display_list(&skiplist);

    println!("\nRemoving all the vowels");
    for &key in &["a", "e", "i", "o", "u", "skip_list"] {
        if let Some(val) = remove(&mut skiplist, &key.to_string()) {
            drop(val);
        }
    }
    display_list(&skiplist);

    println!("\nSearching for elements");
    println!(
        "Is `a` in the skip list ? {}",
        if search(&skiplist, &"a".to_string()).is_some() {
            1
        } else {
            0
        }
    );
    println!(
        "Is `f` in the skip list ? {}",
        if search(&skiplist, &"f".to_string()).is_some() {
            1
        } else {
            0
        }
    );

    println!("\nRandom Access \n");
    let keys = [5, 10, 15, 50];
    for idx in keys.iter() {
        let label = match key_at(&skiplist, *idx) {
            Some(k) => k,
            None => "(null)".to_string(),
        };
        println!(
            "The {}th element of the skip list is {}",
            idx,
            label
        );
    }
}
