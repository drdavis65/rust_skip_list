use rand::Rng;

struct Link<K, D> {
    width: usize,
    node: Option<Box<SkipNode<K, D>>>, // Use Box to avoid recursive type size issues
}

struct SkipNode<K, D> {
    forward: Vec<Link<K, D>>, // You forgot the type parameters here
    key: Option<K>,
    data: Option<D>,
}

pub struct SkipList<K, D> {
    max_level: u16,
    p: f32,
    level: u16,
    head: Box<SkipNode<K, D>>, // Usually boxed to avoid large stack allocations
    comparator: fn(&K, &K) -> i32, // Compare by reference, not move!
}

pub fn get_max_level(n: usize, p: f32) -> u16 {
    assert!((0.0..=1.0).contains(&p));
    let level = ((n as f32).ln() / (1.0 / p).ln()) as u16;
    level.max(1)
}



impl<K, D> SkipList<K, D> {
    pub fn new(
        comparator: fn(&K, &K) -> i32,
        p: f32,
        max_level: u16,
    ) -> Self {
        assert!((0.0..=1.0).contains(&p));

        let mut head = SkipNode {
            forward: Vec::with_capacity(max_level as usize),
            key: None,
            data: None,
        };

        for _ in 0..max_level {
            head.forward.push(Link {
                width: 0,
                node: None,
            });
        }

        Self {
            max_level,
            p,
            level: 1,
            head: Box::new(head),
            comparator,
        }
    }

    pub fn random_level(&mut self) -> u16 {
        let mut rng = rand::thread_rng();
        let mut rnd: f32 = rng.gen(); // generates float in [0.0, 1.0)
        let mut level: u16 = 1;

        while rnd < self.p && level < self.max_level - 1 {
            level += 1;
            rnd = rng.gen();
        }

        level
    }

    }

    pub fn insert(&mut self, key: K, data: D) {
        let mut x = &mut *self.head;

        let mut update: Vec<*mut SkipNode<K, D>> = vec![std::ptr::null_mut(); self.max_level as usize];
        let mut update_width: Vec<usize> = vec![0; self.max_level as usize];

        for i in (0..self.level).rev() {
            let mut width_sum = 0;

            while let Some(ref mut next_node) = x.forward[i].node {
                if (self.comparator)(&next_node.key.as_ref().unwrap(), &key) < 0 {
                    width_sum += 1;
                    x = &mut *next_node;
                } else {
                    break;
                }
            }
            update[i] = x as *mut _;
            update_width[i] = width_sum;
        }

        if let Some(ref mut next_node) = x.forward[0].node {
            if (self.comparator)(&next_node.key.as_ref().unwrap(), &key) == 0 {
                let old = next_node.data.replace(data);
                return old;
            }
        }

        let level = random_level();

        assert!(level < self.max_level);

        if level > self.level {
            for i in self.level..level as usize{
                update[i] = &mut *self.head as *mut _;
                update_width[i] = 0;

                self.head.forward[i].node = None;
                self.head.forward[i].width = 0;
            }
            self.level = level;
        }

        let mut new_node = Box::new(SkipNode {
            key: Some(key),
            data: Some(data),
            forward: Vec::with_capacity(level as usize),
        });

        // TODO... starting at jrsl.h line 318

    }
}


