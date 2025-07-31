use rand::Rng;
use std::cmp::Ordering;

pub trait Comparator<K> {
    fn compare(&self, a: &K, b: &K) -> Ordering;
}

pub struct Link<K, V>{
    pub width: usize,
    pub node: Options<Box<SkipNode<K, V>>,
}

pub struct SkipNode<K, V>{
    pub forward: Vec<Link<K, V>>,
    pub key: K,
    pub data: V,
}

pub struct SkipList {
    pub max_level: u16,
    pub p: f32,
    pub level: u16,
    pub width: usize,
    pub head: Option<Box<SkipNode<K, V>>>,

    pub comparator: Box<dyn Comparator<K>>,
    pub key_destructor: Option<fn(K)>,
}

impl SkipList