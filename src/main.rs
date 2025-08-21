mod skiplist;
use skiplist::SkipList;
use crate::skiplist::get_max_level;
mod data;
use data::*;
use std::cmp::Ordering;
use std::time::Instant;

fn int_comparator(a: &i32, b: &i32) -> Ordering { a.cmp(b) }

fn main() {
    let max_level = get_max_level(N, 0.5);
    let mut sl = SkipList::new(max_level, 0.5, int_comparator);
    
    println!("Starting benchmark with N={}", N);

    // Start timing the entire benchmark
    let t0 = Instant::now();

    // ================== INSERTS ==================
    for i in 0..N {
        let _ = sl.insert(INSERT_KEYS[i], INSERT_DATA[i]);
    }

    // ================== UPDATES (REPLACEMENTS) ==================
    for i in 0..UPDATES {
        let key = INSERT_KEYS[UPDATE_INDICES[i]];
        let _prev = sl.insert(key, UPDATE_DATA[i]); // replace value for the key
    }

    // ================== REMOVALS ==================
    let mut remove_hits = 0;
    let mut remove_misses = 0;
    
    for i in 0..REMOVES {
        let key = if REMOVE_IS_HIT[i] == true {
            // Try to remove an existing key
            INSERT_KEYS[REMOVE_INDICES[i]]
        } else {
            // Try to remove a non-existent key (guaranteed miss)
            REMOVE_MISS_KEYS[i] as i32
        };
        
        match sl.remove(&key) {
            Some(_) => remove_hits += 1,
            None => remove_misses += 1,
        }
    }

    // ================== SEARCHES ==================
    let mut search_hits = 0;
    let mut search_misses = 0;
    let size_after_remove = sl.len();
    
    for i in 0..SEARCHES {
        let key = if SEARCH_IS_HIT[i] == true && size_after_remove > 0 {
            // Try to search for an existing key (should find it and return Some(value))
            INSERT_KEYS[SEARCH_INDICES[i]]
        } else {
            // Try to search for a non-existent key (should return None)
            SEARCH_MISS_KEYS[i] as i32
        };
        
        match sl.search(&key) {
            Some(_) => search_hits += 1,    // Found the key
            None => search_misses += 1,     // Didn't find the key
        }
    }

    // ================== INDEX-BY-RANK ==================
    let mut index_sum: u64 = 0;
    let len_now = sl.len();
    
    // Use a simple counter for index operations since we don't have pre-generated ranks
    let idx_ops = N; // Same as the original benchmark
    for i in 0..idx_ops {
        if len_now == 0 { break; }
        let r = i % len_now; // Simple deterministic pattern
        if let (Some(k), Some(d)) = (sl.key_at(r), sl.data_at(r)) {
            index_sum = index_sum.wrapping_add(k as u64).wrapping_add(d as u64);
        }
    }

    let total_time = t0.elapsed();

    // ================== RESULTS ==================
    println!("=== Simplified SkipList Benchmark Results ===");
    println!("Operations completed:");
    println!("  Inserts:  {}", N);
    println!("  Updates:  {}", UPDATES);
    println!("  Removes:  {} (hits: {}, misses: {})", REMOVES, remove_hits, remove_misses);
    println!("  Searches: {} (hits: {}, misses: {})", SEARCHES, search_hits, search_misses);
    println!("  Index:    {} (len_now: {}, checksum: {})", idx_ops, len_now, index_sum);
    println!();
    println!("Total time: {} ms", total_time.as_millis());
    println!("Final skiplist length: {}", sl.len());
}