pub mod nomap;

// use std::collections::hash_map::DefaultHasher;
// use std::hash::{Hash, Hasher};

// #[derive(Default)]
// pub struct NaiveHasher(Vec<u32>);

// impl NaiveHasher {
//     pub fn add(&mut self, x: u32) {
//         self.0.push(x);
//     }

//     pub fn set(&mut self, x: Vec<u32>) {
//         self.0 = x;
//     }

//     pub fn get(&self) -> u32 {
//         let mut hasher = DefaultHasher::new();
//         self.0.hash(&mut hasher);
//         hasher.finish() as u32
//     }

//     pub fn hash(x: &[u32]) -> u32 {
//         let h = Self(x.to_vec());
//         h.get()
//     }
// }
