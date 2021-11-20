// simple interner from https://matklad.github.io/2020/03/22/fast-simple-rust-interner.html
// e.g. "A"+"B"+"A" -> intern "A", "B", "AB", "ABA"
use std::collections::HashMap;

pub type StringObjIdx = u32;
#[derive(Default)]
pub struct Interner {
    map: HashMap<String, u32>,
    vec: Vec<String>,
}

impl Interner {
    pub fn intern(&mut self, name: &str) -> StringObjIdx {
        if let Some(&idx) = self.map.get(name) {
            return idx;
        }
        let idx = self.map.len() as StringObjIdx;
        self.map.insert(name.to_owned(), idx);
        self.vec.push(name.to_owned());

        idx
    }

    pub fn intern_string(&mut self, name: String) -> StringObjIdx {
        if let Some(&idx) = self.map.get(&name) {
            return idx;
        }
        let idx = self.map.len() as StringObjIdx;
        self.map.insert(name.clone(), idx);
        self.vec.push(name);

        idx
    }

    pub fn lookup(&self, idx: StringObjIdx) -> &str {
        self.vec[idx as usize].as_str()
    }
}
