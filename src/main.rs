mod skiplist;

use skiplist::SkipList;
use crate::skiplist::get_max_level;
use std::cmp::Ordering;

// fn main() {
//     const P: f32 = 0.5;

//     let data = [
//         "a", "e", "w", "d", "q", "u", "y", "b", "n", "c", "t", "m", "f",
//         "z", "g", "o", "s", "h", "v", "i", "j", "p", "k", "r", "x", "l",
//     ];

//     let max_level = get_max_level(data.len(), P);
//     println!("max level: {}", max_level);

//     let mut skip_list = SkipList::<String, ()>::new(max_level, 0.5, Ord::cmp);

//     for d in data {
//         skip_list.insert(d.to_string(), ());
//     }
//     println!("{:?}", skip_list.search(&"a".to_string()));
//     println!("{:?}", skip_list.search(&"z".to_string()));

// }


fn int_comparator(a: &i32, b: &i32) -> Ordering {
    a.cmp(b)
}

fn main() {
    // Create a skip list similar to the C version
    let max_level = get_max_level(100, 0.5);
    println!("max level: {}", max_level);
    
    let mut skip_list = SkipList::new(max_level, 0.5, int_comparator);
    
    // Test insertions
    println!("\n=== Testing Insertions ===");
    for i in [3, 6, 7, 9, 12, 19, 17, 26, 21, 25, 21] {
        let result = skip_list.insert(i, format!("value_{}", i));
        println!("Inserted {}: {:?}", i, result);
    }
    
    println!("Skip list size: {}", skip_list.len());
    
    // Test search
    println!("\n=== Testing Search ===");
    for i in [3, 5, 7, 12, 30] {
        match skip_list.search(&i) {
            Some(data) => println!("Found {}: {}", i, data),
            None => println!("Key {} not found", i),
        }
    }
    
    // Test random access (indexed access)
    println!("\n=== Testing Random Access ===");
    for i in 0..skip_list.len() {
        if let (Some(key), Some(data)) = (skip_list.key_at(i), skip_list.data_at(i)) {
            println!("Index {}: key={}, data={}", i, key, data);
        }
    }
    
    // Test update (insert existing key)
    println!("\n=== Testing Update ===");
    let old_value = skip_list.insert(7, "updated_value_7".to_string());
    println!("Updated key 7, old value: {:?}", old_value);
    
    if let Some(data) = skip_list.search(&7) {
        println!("New value for key 7: {}", data);
    }
    
    // Display the skip list structure
    println!("\n=== Skip List Structure ===");
    skip_list.display_list(Some(|key: &i32, data: &String| {
        print!("{:>6}", key);
    }));
    
    // Test removal
    println!("\n=== Testing Removal ===");
    for key in [7, 15, 12] {
        match skip_list.remove(&key) {
            Some(data) => println!("Removed {}: {}", key, data),
            None => println!("Key {} not found for removal", key),
        }
    }
    
    println!("Skip list size after removals: {}", skip_list.len());
    
    // Display after removals
    println!("\n=== Skip List After Removals ===");
    skip_list.display_list(Some(|key: &i32, data: &String| {
        print!("{:>6}", key);
    }));
    
    // Test edge cases
    println!("\n=== Testing Edge Cases ===");
    
    // Test empty operations - need to specify the data type explicitly
    let mut empty_list: SkipList<i32, String> = SkipList::new(4, 0.5, int_comparator);
    println!("Empty list search for 5: {:?}", empty_list.search(&5));
    println!("Empty list remove 5: {:?}", empty_list.remove(&5));
    println!("Empty list key_at(0): {:?}", empty_list.key_at(0));
    
    // Test bounds
    println!("Out of bounds access key_at({}): {:?}", skip_list.len(), skip_list.key_at(skip_list.len()));
    println!("Out of bounds access data_at({}): {:?}", skip_list.len(), skip_list.data_at(skip_list.len()));

    println!("NOT Out of bounds access key_at({}): {:?}", skip_list.len() - 1, skip_list.key_at(skip_list.len() - 1));
    println!("NOT Out of bounds access data_at({}): {:?}", skip_list.len() - 1, skip_list.data_at(skip_list.len() - 1));
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_basic_operations() {
        let mut skip_list = SkipList::new(16, 0.5, int_comparator);
        
        // Test insertion and search
        assert_eq!(skip_list.insert(5, "five".to_string()), None);
        assert_eq!(skip_list.insert(3, "three".to_string()), None);
        assert_eq!(skip_list.insert(7, "seven".to_string()), None);
        
        assert_eq!(skip_list.search(&5), Some("five".to_string()));
        assert_eq!(skip_list.search(&3), Some("three".to_string()));
        assert_eq!(skip_list.search(&7), Some("seven".to_string()));
        assert_eq!(skip_list.search(&1), None);
        
        assert_eq!(skip_list.len(), 3);
    }
    
    #[test]
    fn test_update() {
        let mut skip_list = SkipList::new(16, 0.5, int_comparator);
        
        skip_list.insert(5, "five".to_string());
        let old = skip_list.insert(5, "FIVE".to_string());
        
        assert_eq!(old, Some("five".to_string()));
        assert_eq!(skip_list.search(&5), Some("FIVE".to_string()));
        assert_eq!(skip_list.len(), 1);
    }
    
    #[test]
    fn test_removal() {
        let mut skip_list = SkipList::new(16, 0.5, int_comparator);
        
        for i in [1, 3, 5, 7, 9] {
            skip_list.insert(i, format!("val_{}", i));
        }
        
        assert_eq!(skip_list.remove(&5), Some("val_5".to_string()));
        assert_eq!(skip_list.remove(&5), None);
        assert_eq!(skip_list.search(&5), None);
        assert_eq!(skip_list.len(), 4);
        
        // Test removing first and last elements
        assert_eq!(skip_list.remove(&1), Some("val_1".to_string()));
        assert_eq!(skip_list.remove(&9), Some("val_9".to_string()));
        assert_eq!(skip_list.len(), 2);
    }
    
    #[test]
    fn test_indexed_access() {
        let mut skip_list = SkipList::new(16, 0.5, int_comparator);
        
        let keys = [3, 1, 4, 1, 5, 9, 2, 6];
        for (i, &key) in keys.iter().enumerate() {
            skip_list.insert(key, i);
        }
        
        // Keys should be sorted: [1, 2, 3, 4, 5, 6, 9] (duplicate 1 updated)
        let expected_keys = [1, 2, 3, 4, 5, 6, 9];
        
        for (i, &expected_key) in expected_keys.iter().enumerate() {
            assert_eq!(skip_list.key_at(i), Some(expected_key));
        }
        
        assert_eq!(skip_list.key_at(expected_keys.len()), None);
    }
}