use rand::Rng;
use std::cmp::Ordering;
use std::fmt::Display;

pub trait Comparator<K> {
    fn compare(&self, a: &K, b: &K) -> Ordering;
}

#[derive(Clone)]
pub struct Link<K, V> {
    pub width: usize,
    pub node: Option<Box<SkipNode<K, V>>>,
}

#[derive(Clone)]
pub struct SkipNode<K, V> {
    pub forward: Vec<Link<K, V>>,
    pub key: Option<K>,
    pub data: Option<V>,
}

pub struct SkipList<K, V> {
    pub max_level: u16,
    pub p: f32,
    pub level: u16,
    pub width: usize,
    pub head: Box<SkipNode<K, V>>,

    pub comparator: Box<dyn Comparator<K>>,
    pub key_destructor: Option<fn(K)>,
}

impl<K, V> SkipList<K, V> {
    fn init_head(max_level: u16) -> Box<SkipNode<K, V>> {
        let mut forward = Vec::with_capacity(max_level as usize);
        for _ in 0..max_level {
            forward.push(Link {
                width: 0,
                node: None,
            });
        }

        Box::new(SkipNode {
            forward,
            key: None,
            data: None,
        })
    }
}

impl<K, V> SkipList<K, V> {
    pub fn new(
        comparator: Box<dyn Comparator<K>>,
        key_destructor: Option<fn(K)>,
        p: f32,
        max_level: u16,
    ) -> Self {
        assert!(
            p > 0.0 && p <= 1.0,
            "Probability p must be in > 0 and <= 1]."
        );

        SkipList {
            max_level,
            p,
            level: 1,
            width: 0,
            head: Self::init_head(max_level),
            comparator,
            key_destructor,
        }
    }
}

pub fn destroy<K, V>(
    skiplist: &mut SkipList<K, V>,
    mut visitor: Option<impl for<'a> FnMut(&'a K, &'a V)>,
) {
    let mut current = skiplist.head.forward[0].node.take();

    while let Some(mut node) = current {
        if let (Some(ref key), Some(ref data)) = (&node.key, &node.data) {
            if let Some(ref mut visit) = visitor {
                visit(key, data);
            }
        }

        current = node.forward[0].node.take();
    }

    skiplist.width = 0;
    skiplist.level = 1;

    for link in skiplist.head.forward.iter_mut() {
        link.node = None;
        link.width = 0;
    }
}

pub fn node_at<K, V>(skiplist: &SkipList<K, V>, index: usize) -> Option<&SkipNode<K, V>> {
    if index >= skiplist.width {
        return None;
    }

    let mut w = index + 1;
    let mut node: &SkipNode<K, V> = &skiplist.head;

    for level in (0..skiplist.level).rev() {
        while let Some(next) = node.forward[level as usize].node.as_ref() {
            let width = node.forward[level as usize].width;
            if width > w {
                break;
            }
            w -= width;
            node = next;
            if w == 0 {
                return Some(node);
            }
        }
    }

    None
}

pub fn key_at<K: Clone, V>(skiplist: &SkipList<K, V>, index: usize) -> Option<K> {
    node_at(skiplist, index).and_then(|node| node.key.as_ref().cloned())
}

pub fn data_at<K, V: Clone>(skiplist: &SkipList<K, V>, index: usize) -> Option<V> {
    node_at(skiplist, index).and_then(|node| node.data.as_ref().cloned())
}

pub fn search<'a, K, V>(skiplist: &'a SkipList<K, V>, key: &'a K) -> Option<&'a V> {
    let mut node: &SkipNode<K, V> = &skiplist.head;

    for level in (0..skiplist.level).rev() {
        loop {
            let next_link = &node.forward[level as usize];
            match next_link.node.as_deref() {
                Some(next_node) => {
                    match skiplist
                        .comparator
                        .compare(next_node.key.as_ref().unwrap(), key)
                    {
                        Ordering::Less => node = next_node,
                        _ => break,
                    }
                }
                None => break,
            }
        }
    }

    if let Some(next) = node.forward[0].node.as_deref() {
        if skiplist.comparator.compare(next.key.as_ref().unwrap(), key) == Ordering::Equal {
            return next.data.as_ref();
        }
    }

    None
}

pub fn insert<K: Clone, V: Clone>(skiplist: &mut SkipList<K, V>, key: K, value: V) -> Option<V> {
    let mut update: Vec<*mut SkipNode<K, V>> =
        vec![std::ptr::null_mut(); skiplist.max_level as usize];
    let mut update_width = vec![0; skiplist.max_level as usize];

    let mut node: *mut SkipNode<K, V> = &mut *skiplist.head;

    let mut w = 0;
    unsafe {
        for i in (0..skiplist.level).rev() {
            while let Some(ref mut next) = (*node).forward[i as usize].node {
                if skiplist
                    .comparator
                    .compare(next.key.as_ref().unwrap(), &key)
                    == Ordering::Less
                {
                    w += (*node).forward[i as usize].width;
                    node = next.as_mut() as *mut _;
                } else {
                    break;
                }
            }
            update[i as usize] = node;
            update_width[i as usize] = w;
        }

        // Check for duplicate key
        if let Some(next) = (*node).forward[0].node.as_mut() {
            if skiplist
                .comparator
                .compare(next.as_ref().key.as_ref().unwrap(), &key)
                == Ordering::Equal
            {
                let old_val = next.data.replace(value);
                return old_val;
            }
        }

        // Insert new node
        let level = random_level(skiplist);
        if level > skiplist.level {
            for i in skiplist.level..level {
                update[i as usize] = &mut *skiplist.head as *mut _;
                update_width[i as usize] = 0;
                skiplist.head.forward[i as usize].width = skiplist.width + 1;
            }
            skiplist.level = level;
        }

        let mut forward = Vec::with_capacity(level as usize);
        for _ in 0..level {
            forward.push(Link {
                width: 0,
                node: None,
            });
        }

        let mut new_node = Box::new(SkipNode {
            forward,
            key: Some(key),
            data: Some(value),
        });

        for i in 0..level {
            let prev = update[i as usize];
            let width = if (*prev).forward[i as usize].node.is_some() {
                (*prev).forward[i as usize]
                    .width
                    .saturating_sub(update_width[0].saturating_sub(update_width[i as usize]))
            } else {
                0
            };

            new_node.forward[i as usize].node = (*prev).forward[i as usize].node.take();
            new_node.forward[i as usize].width = width;

            (*prev).forward[i as usize].node = Some(new_node.clone());
            (*prev).forward[i as usize].width = update_width[0] - update_width[i as usize] + 1;
        }

        skiplist.width += 1;
    }

    None
}

pub fn remove<K: Clone, V>(skiplist: &mut SkipList<K, V>, key: &K) -> Option<V> {
    // Array holding pointers that need their links updated
    let mut update: Vec<*mut SkipNode<K, V>> = vec![std::ptr::null_mut(); skiplist.level as usize];

    let mut x: *mut SkipNode<K, V> = &mut *skiplist.head;

    unsafe {
        // Find path and fill update array
        for i in (0..skiplist.level).rev() {
            while let Some(ref mut next) = (*x).forward[i as usize].node {
                if skiplist.comparator.compare(next.key.as_ref().unwrap(), key) == Ordering::Less {
                    x = next.as_mut();
                } else {
                    break;
                }
            }
            update[i as usize] = x;
        }

        // Advance to possible node
        x = match (*x).forward[0].node.as_mut() {
            Some(node) => node.as_mut(),
            None => return None,
        };

        // Key not found
        if skiplist.comparator.compare((*x).key.as_ref().unwrap(), key) != Ordering::Equal {
            return None;
        }

        // Update links and widths
        for i in 0..skiplist.level {
            let prev = update[i as usize];
            let link = &mut (*prev).forward[i as usize];
            if let Some(ref mut node) = link.node {
                if skiplist.comparator.compare(node.key.as_ref().unwrap(), key) == Ordering::Equal {
                    let (next, width) = {
                        let l = &mut (*x).forward[i as usize];
                        (l.node.take(), l.width)
                    };

                    link.node = next;
                    if width > 0 {
                        link.width += width - 1;
                    } else {
                        link.width = 0;
                    }
                    continue;
                } else {
                    link.width = link.width.saturating_sub(1);
                }
            }
        }

        skiplist.width = skiplist.width.saturating_sub(1);

        // Remove node and capture value
        let mut boxed = Box::from_raw(x);
        let old = boxed.data.take();
        // Drop the node
        drop(boxed);

        // Adjust current level if highest levels become empty
        while skiplist.level > 1
            && skiplist.head.forward[(skiplist.level - 1) as usize]
                .node
                .is_none()
        {
            skiplist.level -= 1;
        }

        old
    }
}

pub fn random_level<K, V>(skiplist: &SkipList<K, V>) -> u16 {
    let mut lvl = 1;
    let mut rng = rand::thread_rng();

    while rng.gen::<f32>() < skiplist.p && lvl < skiplist.max_level {
        lvl += 1;
    }

    lvl
}

pub fn display_list<K: Display + Clone, V>(skiplist: &SkipList<K, V>) {
    println!("SkipList ({} elements):", skiplist.width);
    for i in 0..skiplist.width {
        if let Some(k) = key_at(skiplist, i) {
            print!("{:<4}", k);
        }
    }
    println!();
}
