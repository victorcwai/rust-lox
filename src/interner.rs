// simple interner from https://matklad.github.io/2020/03/22/fast-simple-rust-interner.html
// e.g. "A"+"B"+"A" -> intern "A", "B", "AB", "ABA"
use std::collections::HashMap;

#[derive(Default)]
pub struct Interner {
    map: HashMap<String, u32>,
    vec: Vec<String>,
}

impl Interner {
    pub fn intern(&mut self, name: &str) -> u32 {
        if let Some(&idx) = self.map.get(name) {
            return idx;
        }
        let idx = self.map.len() as u32;
        self.map.insert(name.to_owned(), idx);
        self.vec.push(name.to_owned());

        idx
    }

    pub fn intern_string(&mut self, name: String) -> u32 {
        if let Some(&idx) = self.map.get(&name) {
            return idx;
        }
        let idx = self.map.len() as u32;
        self.map.insert(name.clone(), idx);
        self.vec.push(name);

        idx
    }

    pub fn lookup(&self, idx: u32) -> &str {
        self.vec[idx as usize].as_str()
    }
}
