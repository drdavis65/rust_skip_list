use std::cell::RefCell;
use std::cmp::Ordering;
use std::fmt::Display;
use std::rc::Rc;

pub trait Comparator<K> {
    fn compare(&self, a: &K, b: &K) -> Ordering;
}

pub struct Link<K, V> {
    pub width: usize,
    pub node: Option<Rc<RefCell<SkipNode<K, V>>>>,
}

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
    pub head: Rc<RefCell<SkipNode<K, V>>>,

    pub comparator: Box<dyn Comparator<K>>,
    pub key_destructor: Option<fn(K)>,
    rng: LcgRng,
}

struct LcgRng {
    state: u32,
}

impl LcgRng {
    fn new(seed: u32) -> Self {
        Self { state: seed }
    }

    fn next(&mut self) -> u32 {
        self.state = self.state.wrapping_mul(1103515245).wrapping_add(12345);
        (self.state / 65536) % 32768
    }

    fn next_f32(&mut self) -> f32 {
        self.next() as f32 / 32767.0
    }
}

impl<K, V> SkipList<K, V> {
    fn init_head(max_level: u16) -> Rc<RefCell<SkipNode<K, V>>> {
        let mut forward = Vec::with_capacity(max_level as usize);
        for _ in 0..max_level {
            forward.push(Link {
                width: 0,
                node: None,
            });
        }
        Rc::new(RefCell::new(SkipNode {
            forward,
            key: None,
            data: None,
        }))
    }
}

impl<K, V> SkipList<K, V> {
    pub fn new(
        comparator: Box<dyn Comparator<K>>,
        key_destructor: Option<fn(K)>,
        p: f32,
        max_level: u16,
    ) -> Self {
        assert!(p > 0.0 && p <= 1.0, "Probability p must be in > 0 and <= 1].");

        SkipList {
            max_level,
            p,
            level: 1,
            width: 0,
            head: Self::init_head(max_level),
            comparator,
            key_destructor,
            rng: LcgRng::new(1),
        }
    }
}

pub fn destroy<K, V>(
    skiplist: &mut SkipList<K, V>,
    mut visitor: Option<impl for<'a> FnMut(&'a K, &'a V)>,
) {
    let mut current = skiplist.head.borrow_mut().forward[0].node.take();

    while let Some(node_rc) = current {
        let mut node = node_rc.borrow_mut();
        if let (Some(ref key), Some(ref data)) = (&node.key, &node.data) {
            if let Some(ref mut visit) = visitor {
                visit(key, data);
            }
        }
        current = node.forward[0].node.take();
    }

    skiplist.width = 0;
    skiplist.level = 1;

    let mut head = skiplist.head.borrow_mut();
    for link in head.forward.iter_mut() {
        link.node = None;
        link.width = 0;
    }
}

pub fn node_at<K, V>(skiplist: &SkipList<K, V>, index: usize) -> Option<Rc<RefCell<SkipNode<K, V>>>> {
    if index >= skiplist.width {
        return None;
    }

    let mut w = index + 1;
    let mut node_rc = skiplist.head.clone();

    for level in (0..skiplist.level).rev() {
        loop {
            let next_opt = {
                let node = node_rc.borrow();
                let link = &node.forward[level as usize];
                if let Some(ref next) = link.node {
                    (link.width, Some(next.clone()))
                } else {
                    (0, None)
                }
            };
            match next_opt {
                (width, Some(next)) if width <= w => {
                    w -= width;
                    node_rc = next;
                    if w == 0 {
                        return Some(node_rc);
                    }
                }
                _ => break,
            }
        }
    }

    None
}

pub fn key_at<K: Clone, V>(skiplist: &SkipList<K, V>, index: usize) -> Option<K> {
    node_at(skiplist, index).and_then(|node| node.borrow().key.clone())
}

pub fn data_at<K, V: Clone>(skiplist: &SkipList<K, V>, index: usize) -> Option<V> {
    node_at(skiplist, index).and_then(|node| node.borrow().data.clone())
}

pub fn search<K: Clone, V: Clone>(skiplist: &SkipList<K, V>, key: &K) -> Option<V> {
    let mut node_rc = skiplist.head.clone();

    for level in (0..skiplist.level).rev() {
        loop {
            let (next_opt, cmp) = {
                let node = node_rc.borrow();
                if let Some(ref next) = node.forward[level as usize].node {
                    let cmp = skiplist
                        .comparator
                        .compare(next.borrow().key.as_ref().unwrap(), key);
                    (Some(next.clone()), cmp)
                } else {
                    (None, Ordering::Equal)
                }
            };
            match next_opt {
                Some(next) if cmp == Ordering::Less => {
                    node_rc = next;
                }
                _ => break,
            }
        }
    }

    if let Some(next) = node_rc.borrow().forward[0].node.clone() {
        if skiplist
            .comparator
            .compare(next.borrow().key.as_ref().unwrap(), key)
            == Ordering::Equal
        {
            return next.borrow().data.clone();
        }
    }

    None
}

pub fn insert<K: Clone, V: Clone>(
    skiplist: &mut SkipList<K, V>,
    key: K,
    value: V,
) -> Option<V> {
    let mut update: Vec<Rc<RefCell<SkipNode<K, V>>>> =
        vec![skiplist.head.clone(); skiplist.max_level as usize];
    let mut update_width = vec![0; skiplist.max_level as usize];

    let mut node_rc = skiplist.head.clone();
    let mut w = 0usize;

    for i in (0..skiplist.level).rev() {
        loop {
            let next_opt = {
                let node = node_rc.borrow();
                let link = &node.forward[i as usize];
                if let Some(ref next) = link.node {
                    let cmp = skiplist
                        .comparator
                        .compare(next.borrow().key.as_ref().unwrap(), &key);
                    (cmp, Some(next.clone()), link.width)
                } else {
                    (Ordering::Equal, None, 0)
                }
            };
            match next_opt {
                (Ordering::Less, Some(next), width) => {
                    w += width;
                    node_rc = next;
                }
                _ => break,
            }
        }
        update[i as usize] = node_rc.clone();
        update_width[i as usize] = w;
    }

    if let Some(next) = node_rc.borrow().forward[0].node.clone() {
        if skiplist
            .comparator
            .compare(next.borrow().key.as_ref().unwrap(), &key)
            == Ordering::Equal
        {
            return next.borrow_mut().data.replace(value);
        }
    }

    let level = random_level(skiplist);
    if level > skiplist.level {
        for i in skiplist.level..level {
            update[i as usize] = skiplist.head.clone();
            update_width[i as usize] = 0;
            skiplist.head.borrow_mut().forward[i as usize].width = skiplist.width + 1;
        }
        skiplist.level = level;
    }

    let mut fwd: Vec<Link<K, V>> = Vec::with_capacity(level as usize);
    for _ in 0..level {
        fwd.push(Link { width: 0, node: None });
    }
    let new_node = Rc::new(RefCell::new(SkipNode {
        forward: fwd,
        key: Some(key),
        data: Some(value),
    }));

    for i in 0..level as usize {
        let width = {
            let mut prev = update[i].borrow_mut();
            let width = if prev.forward[i].node.is_some() {
                prev.forward[i]
                    .width
                    .saturating_sub(update_width[0].saturating_sub(update_width[i]))
            } else {
                0
            };
            let next = prev.forward[i].node.take();
            prev.forward[i].node = Some(new_node.clone());
            prev.forward[i].width = update_width[0] - update_width[i] + 1;

            let mut nn = new_node.borrow_mut();
            nn.forward[i].node = next;
            nn.forward[i].width = width;
            width
        };
        let _ = width; // suppress unused variable warning
    }

    skiplist.width += 1;

    None
}

pub fn remove<K: Clone, V>(skiplist: &mut SkipList<K, V>, key: &K) -> Option<V> {
    let mut update: Vec<Rc<RefCell<SkipNode<K, V>>>> =
        vec![skiplist.head.clone(); skiplist.level as usize];

    let mut x = skiplist.head.clone();

    for i in (0..skiplist.level).rev() {
        loop {
            let next_opt = {
                let node = x.borrow();
                if let Some(ref next) = node.forward[i as usize].node {
                    let cmp = skiplist
                        .comparator
                        .compare(next.borrow().key.as_ref().unwrap(), key);
                    (cmp, Some(next.clone()))
                } else {
                    (Ordering::Equal, None)
                }
            };
            match next_opt {
                (Ordering::Less, Some(next)) => x = next,
                _ => break,
            }
        }
        update[i as usize] = x.clone();
    }

    let target = match x.borrow().forward[0].node.clone() {
        Some(node) => node,
        None => return None,
    };

    if skiplist
        .comparator
        .compare(target.borrow().key.as_ref().unwrap(), key)
        != Ordering::Equal
    {
        return None;
    }

    for i in 0..skiplist.level as usize {
        let mut prev = update[i].borrow_mut();
        if let Some(ref next) = prev.forward[i].node {
            if Rc::ptr_eq(next, &target) {
                let (next_node, width) = {
                    let mut t = target.borrow_mut();
                    let l = t.forward.get_mut(i).map(|l| (l.node.take(), l.width));
                    l.unwrap_or((None, 0))
                };
                prev.forward[i].node = next_node;
                if width > 0 {
                    prev.forward[i].width += width - 1;
                } else {
                    prev.forward[i].width = 0;
                }
            } else {
                prev.forward[i].width = prev.forward[i].width.saturating_sub(1);
            }
        }
    }

    skiplist.width = skiplist.width.saturating_sub(1);

    while skiplist.level > 1
        && skiplist.head.borrow().forward[(skiplist.level - 1) as usize]
            .node
            .is_none()
    {
        skiplist.level -= 1;
    }

    let old = target.borrow_mut().data.take();
    old
}

pub fn random_level<K, V>(skiplist: &mut SkipList<K, V>) -> u16 {
    let mut lvl = 1u16;
    while skiplist.rng.next_f32() < skiplist.p && lvl < skiplist.max_level {
        lvl += 1;
    }
    lvl
}

pub fn display_list<K: Display + Clone, V>(skiplist: &SkipList<K, V>) {
    for i in (0..skiplist.level).rev() {
        let mut node = skiplist.head.clone();
        loop {
            let (width, next) = {
                let node_b = node.borrow();
                let link = &node_b.forward[i as usize];
                (link.width, link.node.clone())
            };
            if let Some(n) = next {
                if width > 0 {
                    let s = width.to_string();
                    let pad = width * 6 - 1;
                    let left = (pad - s.len()) / 2;
                    let right = pad - left - s.len();
                    print!("{}{}{}", " ".repeat(left), s, " ".repeat(right));
                }
                node = n;
            } else {
                break;
            }
        }
        println!();

        node = skiplist.head.clone();
        loop {
            let (width, next) = {
                let node_b = node.borrow();
                let link = &node_b.forward[i as usize];
                (link.width, link.node.clone())
            };
            if let Some(n) = next {
                if width > 0 {
                    print!("o{:-<width$}> ", "", width = width * 6 - 3);
                } else {
                    print!("x ");
                    break;
                }
                node = n;
            } else {
                print!("x ");
                break;
            }
        }
        println!(" Level {} ", i);
    }

    if let Some(node) = skiplist.head.borrow().forward[0].node.clone() {
        print!("      ");
        let mut n = Some(node);
        while let Some(rc) = n {
            if let Some(ref k) = rc.borrow().key {
                print!("{:<6}", k);
            }
            n = rc.borrow().forward[0].node.clone();
        }
    }
    println!();
}
