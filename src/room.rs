use std::collections::HashMap;

use item::Item;

/// A node found within a World that is connected by paths
pub struct Room {
    name: String,
    desc: String,
    pub paths: HashMap<String, String>,
    pub items: HashMap<String, Box<Item>>,
}

impl Room {
    pub fn new(name: &str, desc: &str, items: HashMap<String, Box<Item>>) -> Room {
        Room {
            name: name.to_owned(),
            desc: desc.to_owned(),
            paths: HashMap::new(),
            items: items,
        }
    }
    pub fn name(&self) -> String {
        self.name.clone()
    }
    /// compiles all descriptions in the Room for printing
    pub fn desc(&self) -> String {
        let mut desc = format!("{}\n{}\n", self.name, self.desc);
        for x in self.items.iter() {
            desc.push_str(&x.1.desc());
        }
        desc
    }
    /// add path directive to another Room
    pub fn add_path(&mut self, dir: &str, room: &String, desc: &str) {
        self.paths.insert(dir.to_owned(), room.clone());
        self.desc.push_str(format!("\n{}", desc).as_str());
    }
}
