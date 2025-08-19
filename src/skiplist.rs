use std::rc::Rc;
use std::cell::RefCell;
use std::cmp::Ordering;
use libc::{rand,srand};

#[derive(Clone)]
struct Link<K, D> {
    width: usize,
    node: Option<Rc<RefCell<SkipNode<K, D>>>>,
}

struct SkipNode<K, D> {
    forward: Vec<Link<K, D>>,
    key: Option<K>,
    data: Option<D>,
}

pub struct SkipList<K, D> {
    max_level: u16,
    p: f32,
    level: u16,
    width: usize,
    head: Rc<RefCell<SkipNode<K, D>>>,
    comparator: fn(&K, &K) -> std::cmp::Ordering,
}

pub fn get_max_level(n: usize, p: f32) -> u16 {
    assert!((0.0..=1.0).contains(&p));
    let level = ((n as f32).ln() / (1.0 / p).ln()) as u16;
    level.max(1)
}

impl<K, D> SkipList<K, D> {
    pub fn new(
        max_level: u16,
        p: f32,
        comparator: fn(&K, &K) -> Ordering,
    ) -> Self {
        let mut forward = Vec::with_capacity(max_level as usize);
        for _ in 0..max_level {
            forward.push(Link {
                width: 0,
                node: None,
            });
        }

        unsafe {
            libc::srand(42);
        }

        let head = Rc::new(RefCell::new(SkipNode {
            forward,
            key: None,
            data: None,
        }));

        SkipList {
            max_level,
            p,
            level: 1, // Start with level 1 like C version
            width: 0,
            head,
            comparator,
        }
    }
    
    fn random_level(&self) -> usize {
        let mut lvl= 1;
        let mut rnd: f32 = unsafe { libc::rand() as f32 / libc::RAND_MAX as f32 };
        while rnd < self.p && lvl < self.max_level - 1 {
            lvl += 1;
            rnd = unsafe { libc::rand() as f32 / libc::RAND_MAX as f32 };
        }
        lvl as usize
    }
}

impl<K: Clone, D: Clone> SkipList<K, D> {
    pub fn search(&self, key: &K) -> Option<D> {
        let mut current = self.head.clone();

        for i in (0..self.level as usize).rev() {
            loop {
                let next_node_rc = {
                    let current_borrowed = current.borrow();
                    current_borrowed.forward[i].node.clone()
                };
                
                if let Some(next_rc) = next_node_rc {
                    let next_node = next_rc.borrow();
                    if let Some(next_key) = next_node.key.as_ref() {
                        match (self.comparator)(next_key, key) {
                            Ordering::Less => {
                                drop(next_node);
                                current = next_rc;
                            }
                            _ => break,
                        }
                    } else {
                        break;
                    }
                } else {
                    break;
                }
            }
        }

        // Check the next node at level 0
        let current_borrowed = current.borrow();
        if let Some(next_rc) = current_borrowed.forward[0].node.as_ref() {
            let next_node = next_rc.borrow();
            if let Some(next_key) = next_node.key.as_ref() {
                if (self.comparator)(next_key, key) == Ordering::Equal {
                    return next_node.data.clone();
                }
            }
        }
        None
    }

    pub fn insert(&mut self, key: K, data: D) -> Option<D> {
        let mut update: Vec<Rc<RefCell<SkipNode<K, D>>>> = vec![self.head.clone(); self.max_level as usize];
        let mut update_width: Vec<usize> = vec![0; self.max_level as usize];
        let mut current = self.head.clone();

        // Search for insertion point, following C implementation logic
        for i in (0..self.level).rev() {
            let mut width_sum = 0;

            loop {
                let next_info = {
                    let current_borrowed = current.borrow();
                    let link = &current_borrowed.forward[i as usize];
                    if let Some(node) = link.node.as_ref() {
                        Some((node.clone(), link.width))
                    } else {
                        None
                    }
                };

                if let Some((next_rc, width)) = next_info {
                    let next_node = next_rc.borrow();
                    if let Some(next_key) = next_node.key.as_ref() {
                        if (self.comparator)(next_key, &key) == Ordering::Less {
                            drop(next_node);
                            width_sum += width;
                            current = next_rc;
                            continue;
                        }
                    }
                }
                break;
            }
            update[i as usize] = current.clone();
            update_width[i as usize] = width_sum;
        }

        // Check if key already exists
        {
            let current_borrowed = current.borrow();
            if let Some(next_rc) = current_borrowed.forward[0].node.as_ref() {
                let mut next_node = next_rc.borrow_mut();
                if let Some(next_key) = next_node.key.as_ref() {
                    if (self.comparator)(next_key, &key) == Ordering::Equal {
                        let old_data = next_node.data.replace(data);
                        return old_data;
                    }
                }
            }
        }
        // drop(current_borrowed);

        let node_level = self.random_level();

        // Update level if necessary
        if node_level > self.level as usize {
            for i in self.level as usize..node_level {
                update[i] = self.head.clone();
                update_width[i] = 0;
                // Initialize head's forward links for new levels
                self.head.borrow_mut().forward[i] = Link {
                    width: 0,
                    node: None,
                };
            }
            self.level = node_level as u16;
        }

        // Create new node
        let mut new_forward = Vec::with_capacity(node_level);
        for _ in 0..node_level {
            new_forward.push(Link {
                width: 0,
                node: None,
            });
        }

        let new_node = Rc::new(RefCell::new(SkipNode {
            forward: new_forward,
            key: Some(key),
            data: Some(data),
        }));

        // Insert new node - following C implementation logic
        for i in 0..node_level {
            let old_link = {
                let mut upd = update[i].borrow_mut();
                std::mem::replace(&mut upd.forward[i], Link {
                    width: 0,
                    node: Some(new_node.clone()),
                })
            };

            // Set new node's forward link
            new_node.borrow_mut().forward[i] = old_link;

            // Update widths following C logic
            if i > 0 {
                let width_before = update_width[i - 1] + {
                    let upd_prev = update[i - 1].borrow();
                    upd_prev.forward[i - 1].width
                };

                let new_node_width = if new_node.borrow().forward[i].width > 0 {
                    new_node.borrow().forward[i].width + 1 - width_before
                } else {
                    0
                };

                new_node.borrow_mut().forward[i].width = new_node_width;
                update[i].borrow_mut().forward[i].width = width_before;
            } else {
                // Level 0
                let old_width = new_node.borrow().forward[i].width;
                new_node.borrow_mut().forward[i].width = old_width;
                update[i].borrow_mut().forward[i].width = 1;
            }
        }

        // Update widths of levels above the new node
        for i in node_level..self.level as usize {
            let mut upd = update[i].borrow_mut();
            if upd.forward[i].node.is_some() {
                upd.forward[i].width += 1;
            } else {
                break;
            }
        }

        self.width += 1;
        None
    }

    pub fn remove(&mut self, key: &K) -> Option<D> {
        let mut update: Vec<Rc<RefCell<SkipNode<K, D>>>> = vec![self.head.clone(); self.level as usize];
        let mut current = self.head.clone();

        // Find the node to remove
        for i in (0..self.level).rev() {
            loop {
                let next_node_rc = {
                    let current_borrowed = current.borrow();
                    current_borrowed.forward[i as usize].node.clone()
                };

                if let Some(next_rc) = next_node_rc {
                    let next_node = next_rc.borrow();
                    if let Some(next_key) = next_node.key.as_ref() {
                        if (self.comparator)(next_key, key) == Ordering::Less {
                            drop(next_node);
                            current = next_rc;
                            continue;
                        }
                    }
                }
                break;
            }
            update[i as usize] = current.clone();
        }

        // Get the node to remove
        let target_node = {
            let current_borrowed = current.borrow();
            current_borrowed.forward[0].node.clone()
        };

        let target_node = match target_node {
            Some(node) => node,
            None => return None,
        };

        // Verify it's the right node
        let target_borrowed = target_node.borrow();
        if let Some(target_key) = target_borrowed.key.as_ref() {
            if (self.comparator)(target_key, key) != Ordering::Equal {
                return None;
            }
        } else {
            return None;
        }

        let old_data = target_borrowed.data.clone();
        let target_forward = target_borrowed.forward.clone();
        drop(target_borrowed);

        // Update the skip list structure
        for i in 0..self.level as usize {
            let mut upd = update[i].borrow_mut();
            if let Some(upd_next) = upd.forward[i].node.as_ref() {
                if Rc::ptr_eq(upd_next, &target_node) {
                    upd.forward[i] = target_forward[i].clone();
                    
                    if target_forward[i].width > 0 {
                        upd.forward[i].width += target_forward[i].width - 1;
                    } else {
                        upd.forward[i].width = 0;
                    }
                } else {
                    if upd.forward[i].width > 0 {
                        upd.forward[i].width -= 1;
                    }
                }
            }
        }

        // Update level if necessary
        while self.level > 1 {
            let head_borrowed = self.head.borrow();
            if head_borrowed.forward[self.level as usize - 1].node.is_none() {
                drop(head_borrowed);
                self.level -= 1;
            } else {
                break;
            }
        }

        self.width -= 1;
        old_data
    }

    fn node_at(&self, index: usize) -> Option<Rc<RefCell<SkipNode<K, D>>>> {
        if index >= self.width {
            return None;
        }

        let mut remaining_width = index + 1; // +1 because of head node
        let mut current = self.head.clone();

        for i in (0..self.level as usize).rev() {
            loop {
                let current_borrowed = current.borrow();
                let link = &current_borrowed.forward[i];
                
                if let Some(next_node) = link.node.as_ref() {
                    if link.width <= remaining_width {
                        remaining_width -= link.width;
                        let next = next_node.clone();
                        drop(current_borrowed);
                        current = next;
                        
                        if remaining_width == 0 {
                            return Some(current);
                        }
                    } else {
                        break;
                    }
                } else {
                    break;
                }
            }
        }
        None
    }

    pub fn key_at(&self, index: usize) -> Option<K> {
        self.node_at(index)
            .and_then(|node| node.borrow().key.clone())
    }

    pub fn data_at(&self, index: usize) -> Option<D> {
        self.node_at(index)
            .and_then(|node| node.borrow().data.clone())
    }

    pub fn display_list(&self, label_printer: Option<fn(&K, &D)>) {
        for level in (0..self.level).rev() {
            // Print widths
            let mut current = self.head.clone();
            loop {
                let current_borrowed = current.borrow();
                let link = &current_borrowed.forward[level as usize];
                
                if link.width > 0 {
                    let width_str = link.width.to_string();
                    let padding = link.width * 6;
                    print!("{:^width$}", width_str, width = padding.saturating_sub(1));
                }
                
                if let Some(next) = link.node.clone() {
                    drop(current_borrowed);
                    current = next;
                } else {
                    break;
                }
            }
            println!();

            // Print arrows
            current = self.head.clone();
            loop {
                let current_borrowed = current.borrow();
                let link = &current_borrowed.forward[level as usize];
                
                if link.width > 0 {
                    let arrow_width = link.width * 6 - 3;
                    print!("o{:->width$}> ", "", width = arrow_width);
                } else {
                    print!("x ");
                }

                if let Some(next) = link.node.clone() {
                    drop(current_borrowed);
                    current = next;
                } else {
                    break;
                }
            }
            println!(" Level {}", level);
        }

        // Print labels if provided
        if let Some(printer) = label_printer {
            let current_borrowed = self.head.borrow();
            if let Some(first_node) = current_borrowed.forward[0].node.as_ref() {
                //print!("      ");
                let mut current = first_node.clone();
                drop(current_borrowed);
                
                loop {
                    let (key, data, next) = {
                        let current_borrowed = current.borrow();
                        (
                            current_borrowed.key.clone(),
                            current_borrowed.data.clone(),
                            current_borrowed.forward[0].node.clone()
                        )
                    };
                    
                    if let (Some(key), Some(data)) = (key.as_ref(), data.as_ref()) {
                        printer(key, data);
                    }
                    
                    if let Some(next_node) = next {
                        current = next_node;
                    } else {
                        break;
                    }
                }
                println!();
            }
        }
    }

    pub fn len(&self) -> usize {
        self.width
    }

    // pub fn is_empty(&self) -> bool {
    //     self.width == 0
    // }
}
