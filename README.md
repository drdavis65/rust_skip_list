**C vs. Rust SkipList Comparison**
- Original C implementation from https://github.com/Garfield1002/jrsl
- Fix in jrsl.h search function (previously was not utilizing Log(n) run time and searching all levels)

**Benchmark**
- Inserts: 1000000
- Updates: 500000
- Removes: 750000 (hits: 393282, misses: 356718)
- Searches: 1000000 (hits: 303000, misses: 697000)
- Access by index:  (final length: 606718, checksum: 430643106620)