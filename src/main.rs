mod skiplist;
use skiplist::*;

struct StringComparator;

impl Comparator<String> for StringComparator {
    fn compare(&self, a: &String, b: &String) -> std::cmp::Ordering {
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

    println!("\n\nEmpty skip list:");
    display_list(&skiplist);

    // Insert elements
    println!("\n\nInserting elements:");
    for &key in &data {
        insert(&mut skiplist, key.to_string(), Box::new(0));
    }
    display_list(&skiplist);

    // Insert duplicate key
    println!("\n\nInserting duplicate key 'a':");
    if let Some(old) = insert(&mut skiplist, "a".to_string(), Box::new(42)) {
        println!("Replaced old value.");
        drop(old);
    }
    display_list(&skiplist);

    // Remove vowels
    println!("\n\nRemoving all the vowels:");
    for &key in &["a", "e", "i", "o", "u", "skip_list"] {
        if let Some(val) = remove(&mut skiplist, &key.to_string()) {
            println!("Removed key '{}'", key);
            drop(val);
        }
    }
    display_list(&skiplist);

    // Search
    println!("\n\nSearching:");
    println!(
        "Is 'a' in the skip list? {}",
        search(&skiplist, &"a".to_string()).is_some()
    );
    println!(
        "Is 'f' in the skip list? {}",
        search(&skiplist, &"f".to_string()).is_some()
    );

    // Random Access
    println!("\n\nRandom Access:");
    for i in [5, 10, 15, 50] {
        match key_at(&skiplist, i) {
            Some(k) => println!("{}th element: {}", i, k),
            None => println!("{}th element: None", i),
        }
    }

    // Destroy
    println!("\n\nDestroying skip list:");
    destroy(&mut skiplist, Some(|k: &String, _v: &Box<i32>| {
        println!("Freeing key: {}", k);
    }));

}
