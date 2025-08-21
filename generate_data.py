#!/usr/bin/env python3
"""
Generate benchmark data for SkipList comparison between Rust and C.
Uses the same xorshift64* RNG algorithm as the benchmark code.
"""

import struct

class XorShift64Star:
    """Same xorshift64* RNG as used in the benchmark"""
    
    def __init__(self, seed):
        self.state = seed if seed != 0 else 1
    
    def next_u64(self):
        x = self.state
        x ^= x >> 12
        x ^= x << 25
        x ^= x >> 27
        x &= 0xFFFFFFFFFFFFFFFF  # Keep it 64-bit
        self.state = x
        return (x * 0x2545F4914F6CDD1D) & 0xFFFFFFFFFFFFFFFF
    
    def next_usize(self, bound):
        if bound <= 1:
            return 0
        return self.next_u64() % bound
    
    def next_f64(self):
        # 53 random bits -> [0,1)
        return float(self.next_u64() >> 11) * (1.0 / float(1 << 53))
    
    def rand_char(self):
        return chr(ord('a') + (self.next_u64() % 26))
    
    def shuffle(self, array):
        """Fisher-Yates shuffle"""
        for i in range(len(array) - 1, 0, -1):
            j = self.next_usize(i + 1)
            array[i], array[j] = array[j], array[i]

# Configuration (customized per user requirements)
N = 1_000_000
SEED = 0xDEADBEEFCAFEBABE
UPDATE_COUNT = 500_000
REMOVE_COUNT = 750_000
REMOVE_HITS = 500_000  # Exact count hitting existing keys
REMOVE_MISSES = 250_000  # Exact count missing
SEARCH_COUNT = 1_000_000
SEARCH_HIT_RATE = 0.50  # 50% of searches hit existing keys

def generate_data():
    print("Generating benchmark data...")
    rng = XorShift64Star(SEED)
    
    # Generate insert keys (0..N-1)
    insert_keys = list(range(N))
    
    # Generate random data for inserts
    insert_data = [rng.rand_char() for _ in range(N)]
    
    # Shuffle keys for random order (matching ORDER_RANDOM)
    order = list(range(N))
    rng.shuffle(order)
    # Apply shuffle to keys to match the Rust behavior
    shuffled_keys = [insert_keys[order[i]] for i in range(N)]
    
    # Generate update data
    update_indices = [rng.next_usize(N) for _ in range(UPDATE_COUNT)]
    update_data = [rng.rand_char() for _ in range(UPDATE_COUNT)]
    
    # Generate remove data (exact counts)
    remove_is_hit = []
    remove_indices = []
    remove_miss_keys = []
    
    # Generate exactly REMOVE_HITS hits followed by REMOVE_MISSES misses
    for _ in range(REMOVE_HITS):
        remove_is_hit.append(True)
        remove_indices.append(rng.next_usize(N))
        remove_miss_keys.append(0)  # Unused for hits
    
    for _ in range(REMOVE_MISSES):
        remove_is_hit.append(False)
        remove_indices.append(0)  # Unused for misses
        # Generate miss key outside the domain
        miss_key = N + (rng.next_u64() % N) + 1
        remove_miss_keys.append(miss_key)
    
    # Generate search data
    search_is_hit = []
    search_indices = []
    search_miss_keys = []
    
    for _ in range(SEARCH_COUNT):
        is_hit = rng.next_f64() < SEARCH_HIT_RATE
        search_is_hit.append(is_hit)
        if is_hit:
            search_indices.append(rng.next_usize(N))
            search_miss_keys.append(0)  # Unused
        else:
            search_indices.append(0)  # Unused
            # Generate miss key outside the domain  
            miss_key = N + (rng.next_u64() % N) + 1
            search_miss_keys.append(miss_key)
    
    return {
        'insert_keys': shuffled_keys,
        'insert_data': insert_data,
        'update_indices': update_indices,
        'update_data': update_data,
        'remove_is_hit': remove_is_hit,
        'remove_indices': remove_indices,
        'remove_miss_keys': remove_miss_keys,
        'search_is_hit': search_is_hit,
        'search_indices': search_indices,
        'search_miss_keys': search_miss_keys,
        'n': N,
        'updates': UPDATE_COUNT,
        'removes': REMOVE_COUNT,
        'searches': SEARCH_COUNT
    }

def write_rust_file(data):
    """Write data.rs file"""
    print("Writing data.rs...")
    
    with open('data.rs', 'w') as f:
        f.write("// Auto-generated benchmark data\n")
        f.write("// DO NOT EDIT - regenerate with data_generator.py\n\n")
        
        # Constants
        f.write(f"pub const N: usize = {data['n']};\n")
        f.write(f"pub const UPDATES: usize = {data['updates']};\n")
        f.write(f"pub const REMOVES: usize = {data['removes']};\n")
        f.write(f"pub const SEARCHES: usize = {data['searches']};\n\n")
        
        # Insert data
        f.write("pub const INSERT_KEYS: &[i32] = &[\n")
        for i in range(0, len(data['insert_keys']), 10):
            chunk = data['insert_keys'][i:i+10]
            f.write("    " + ", ".join(map(str, chunk)) + ",\n")
        f.write("];\n\n")
        
        f.write("pub const INSERT_DATA: &[char] = &[\n")
        for i in range(0, len(data['insert_data']), 20):
            chunk = data['insert_data'][i:i+20]
            f.write("    " + ", ".join(f"'{c}'" for c in chunk) + ",\n")
        f.write("];\n\n")
        
        # Update data
        f.write("pub const UPDATE_INDICES: &[usize] = &[\n")
        for i in range(0, len(data['update_indices']), 10):
            chunk = data['update_indices'][i:i+10]
            f.write("    " + ", ".join(map(str, chunk)) + ",\n")
        f.write("];\n\n")
        
        f.write("pub const UPDATE_DATA: &[char] = &[\n")
        for i in range(0, len(data['update_data']), 20):
            chunk = data['update_data'][i:i+20]
            f.write("    " + ", ".join(f"'{c}'" for c in chunk) + ",\n")
        f.write("];\n\n")
        
        # Remove data
        f.write("pub const REMOVE_IS_HIT: &[bool] = &[\n")
        for i in range(0, len(data['remove_is_hit']), 20):
            chunk = data['remove_is_hit'][i:i+20]
            f.write("    " + ", ".join("true" if x else "false" for x in chunk) + ",\n")
        f.write("];\n\n")
        
        f.write("pub const REMOVE_INDICES: &[usize] = &[\n")
        for i in range(0, len(data['remove_indices']), 10):
            chunk = data['remove_indices'][i:i+10]
            f.write("    " + ", ".join(map(str, chunk)) + ",\n")
        f.write("];\n\n")
        
        f.write("pub const REMOVE_MISS_KEYS: &[u64] = &[\n")
        for i in range(0, len(data['remove_miss_keys']), 10):
            chunk = data['remove_miss_keys'][i:i+10]
            f.write("    " + ", ".join(map(str, chunk)) + ",\n")
        f.write("];\n\n")
        
        # Search data
        f.write("pub const SEARCH_IS_HIT: &[bool] = &[\n")
        for i in range(0, len(data['search_is_hit']), 20):
            chunk = data['search_is_hit'][i:i+20]
            f.write("    " + ", ".join("true" if x else "false" for x in chunk) + ",\n")
        f.write("];\n\n")
        
        f.write("pub const SEARCH_INDICES: &[usize] = &[\n")
        for i in range(0, len(data['search_indices']), 10):
            chunk = data['search_indices'][i:i+10]
            f.write("    " + ", ".join(map(str, chunk)) + ",\n")
        f.write("];\n\n")
        
        f.write("pub const SEARCH_MISS_KEYS: &[u64] = &[\n")
        for i in range(0, len(data['search_miss_keys']), 10):
            chunk = data['search_miss_keys'][i:i+10]
            f.write("    " + ", ".join(map(str, chunk)) + ",\n")
        f.write("];\n")

def write_c_file(data):
    """Write data.h file"""
    print("Writing data.h...")
    
    with open('data.h', 'w') as f:
        f.write("/* Auto-generated benchmark data */\n")
        f.write("/* DO NOT EDIT - regenerate with data_generator.py */\n\n")
        f.write("#ifndef DATA_H\n#define DATA_H\n\n")
        f.write("#include <stdint.h>\n#include <stddef.h>\n\n")
        
        # Constants
        f.write(f"#define N {data['n']}\n")
        f.write(f"#define UPDATES {data['updates']}\n")
        f.write(f"#define REMOVES {data['removes']}\n")
        f.write(f"#define SEARCHES {data['searches']}\n\n")
        
        # Insert data
        f.write("static const int INSERT_KEYS[N] = {\n")
        for i in range(0, len(data['insert_keys']), 10):
            chunk = data['insert_keys'][i:i+10]
            f.write("    " + ", ".join(map(str, chunk)) + ",\n")
        f.write("};\n\n")
        
        f.write("static const char INSERT_DATA[N] = {\n")
        for i in range(0, len(data['insert_data']), 20):
            chunk = data['insert_data'][i:i+20]
            f.write("    " + ", ".join(f"'{c}'" for c in chunk) + ",\n")
        f.write("};\n\n")
        
        # Update data
        f.write("static const size_t UPDATE_INDICES[UPDATES] = {\n")
        for i in range(0, len(data['update_indices']), 10):
            chunk = data['update_indices'][i:i+10]
            f.write("    " + ", ".join(map(str, chunk)) + ",\n")
        f.write("};\n\n")
        
        f.write("static const char UPDATE_DATA[UPDATES] = {\n")
        for i in range(0, len(data['update_data']), 20):
            chunk = data['update_data'][i:i+20]
            f.write("    " + ", ".join(f"'{c}'" for c in chunk) + ",\n")
        f.write("};\n\n")
        
        # Remove data
        f.write("static const int REMOVE_IS_HIT[REMOVES] = {\n")
        for i in range(0, len(data['remove_is_hit']), 20):
            chunk = data['remove_is_hit'][i:i+20]
            f.write("    " + ", ".join("1" if x else "0" for x in chunk) + ",\n")
        f.write("};\n\n")
        
        f.write("static const size_t REMOVE_INDICES[REMOVES] = {\n")
        for i in range(0, len(data['remove_indices']), 10):
            chunk = data['remove_indices'][i:i+10]
            f.write("    " + ", ".join(map(str, chunk)) + ",\n")
        f.write("};\n\n")
        
        f.write("static const uint64_t REMOVE_MISS_KEYS[REMOVES] = {\n")
        for i in range(0, len(data['remove_miss_keys']), 8):
            chunk = data['remove_miss_keys'][i:i+8]
            f.write("    " + ", ".join(f"{x}ULL" for x in chunk) + ",\n")
        f.write("};\n\n")
        
        # Search data
        f.write("static const int SEARCH_IS_HIT[SEARCHES] = {\n")
        for i in range(0, len(data['search_is_hit']), 20):
            chunk = data['search_is_hit'][i:i+20]
            f.write("    " + ", ".join("1" if x else "0" for x in chunk) + ",\n")
        f.write("};\n\n")
        
        f.write("static const size_t SEARCH_INDICES[SEARCHES] = {\n")
        for i in range(0, len(data['search_indices']), 10):
            chunk = data['search_indices'][i:i+10]
            f.write("    " + ", ".join(map(str, chunk)) + ",\n")
        f.write("};\n\n")
        
        f.write("static const uint64_t SEARCH_MISS_KEYS[SEARCHES] = {\n")
        for i in range(0, len(data['search_miss_keys']), 8):
            chunk = data['search_miss_keys'][i:i+8]
            f.write("    " + ", ".join(f"{x}ULL" for x in chunk) + ",\n")
        f.write("};\n\n")
        
        f.write("#endif /* DATA_H */\n")

def main():
    print("Starting benchmark data generation...")
    print(f"Configuration: N={N:,}, seed=0x{SEED:016x}")
    print(f"Updates: {UPDATE_COUNT:,}")
    print(f"Removes: {REMOVE_COUNT:,} ({REMOVE_HITS:,} hits, {REMOVE_MISSES:,} misses)")
    print(f"Searches: {SEARCH_COUNT:,} ({SEARCH_HIT_RATE:.0%} hit rate)")
    print()
    
    data = generate_data()
    
    write_rust_file(data)
    write_c_file(data)
    
    print()
    print("Generated files:")
    print("  data.rs  - Rust constants")
    print("  data.h   - C constants")
    print()
    print("Statistics:")
    print(f"  Insert keys: {len(data['insert_keys']):,}")
    print(f"  Insert data: {len(data['insert_data']):,}")
    print(f"  Update operations: {len(data['update_indices']):,}")
    print(f"  Remove operations: {len(data['remove_is_hit']):,}")
    print(f"    - Hits: {REMOVE_HITS:,}")
    print(f"    - Misses: {REMOVE_MISSES:,}")
    print(f"  Search operations: {len(data['search_is_hit']):,}")
    print(f"    - Expected hits: ~{int(SEARCH_COUNT * SEARCH_HIT_RATE):,}")
    print(f"    - Expected misses: ~{int(SEARCH_COUNT * (1 - SEARCH_HIT_RATE)):,}")
    print()
    print("Done! You can now use these files in your Rust and C benchmarks.")

if __name__ == "__main__":
    main()