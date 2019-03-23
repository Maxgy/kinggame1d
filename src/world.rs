use std::collections::HashMap;

use serde_derive::{Deserialize, Serialize};

use crate::{
    entities::{Item, Room},
    types::{CmdResult, ItemMap, WorldError},
};

// Represents a world for the player to explore that consists of a grid of Rooms.
// A World is a graph data structure that encapsulates a collection of Room nodes.
#[derive(Serialize, Deserialize)]
pub struct World {
    curr_room: String,
    rooms: HashMap<String, Box<Room>>,
}

impl World {
    pub fn curr_room(&self) -> String {
        self.curr_room.clone()
    }

    pub fn rooms_mut(&mut self) -> &mut HashMap<String, Box<Room>> {
        &mut self.rooms
    }

    // displays description of the current Room
    pub fn look(&self) -> Result<String, WorldError> {
        if let Some(room) = self.rooms.get(&self.curr_room) {
            Ok(room.desc())
        } else {
            Err(WorldError::NoRoom)
        }
    }

    pub fn inspect(&self, name: &str) -> Option<String> {
        if let Some(room) = self.rooms.get(&self.curr_room) {
            if let Some(item) = room.items().get(name) {
                Some(item.inspection().to_string())
            } else if let Some(item) = room.paths().get(name) {
                Some(item.inspection().to_string())
            } else if let Some(enemy) = room.enemies().get(name) {
                Some(enemy.inspection().to_string())
            } else {
                None
            }
        } else {
            None
        }
    }

    // changes the current Room to the target of the current Room's chosen path
    pub fn move_room(&mut self, direction: &str) -> Result<String, WorldError> {
        if let Some(room) = self.rooms.get(&self.curr_room) {
            if let Some(new_room) = room.paths().get(direction) {
                if new_room.is_locked() == Some(true) {
                    Ok("The way is locked.".to_string())
                } else if new_room.is_closed() == Some(true) {
                    Ok("The way is closed.".to_string())
                } else {
                    self.curr_room = new_room.name().to_string();
                    Ok(self.look()?)
                }
            } else {
                Ok("You cannot go that way.".to_string())
            }
        } else {
            Err(WorldError::NoRoom)
        }
    }

    pub fn open_path(&mut self, path: &str) -> Result<String, WorldError> {
        if let Some(room) = self.rooms.get_mut(&self.curr_room) {
            if let Some(p) = room.paths_mut().get_mut(path) {
                if p.is_closed() == Some(true) {
                    p.open();
                    Ok("Opened.".to_string())
                } else {
                    Ok(format!("The {} is already opened.", path))
                }
            } else {
                Ok(format!("There is no \"{}\".", path))
            }
        } else {
            Err(WorldError::NoRoom)
        }
    }

    pub fn close_path(&mut self, path: &str) -> Result<String, WorldError> {
        if let Some(room) = self.rooms.get_mut(&self.curr_room) {
            if let Some(p) = room.paths_mut().get_mut(path) {
                if p.is_closed() == Some(true) {
                    Ok(format!("The {} is already closed.", path))
                } else {
                    p.close();
                    Ok("Closed.".to_string())
                }
            } else {
                Ok(format!("There is no \"{}\".", path))
            }
        } else {
            Err(WorldError::NoRoom)
        }
    }

    // let an Enemy in the current Room take damage
    pub fn harm_enemy(
        &mut self,
        enemy: &str,
        weapon: &str,
        damage: Option<i32>,
    ) -> Result<CmdResult, WorldError> {
        if let Some(room) = self.rooms.get_mut(&self.curr_room) {
            if let Some(nme) = room.enemies_mut().get_mut(enemy) {
                if let Some(dmg) = damage {
                    nme.get_hit(dmg);
                    if nme.hp() > 0 {
                        Ok(CmdResult::new(
                            true,
                            format!(
                                "You hit the {} with your {} for {} damage.",
                                enemy, weapon, dmg,
                            ),
                        ))
                    } else {
                        let mut res = format!(
                            "You hit the {} with your {} for {} damage. It is dead.\n",
                            enemy, weapon, dmg
                        );
                        if !nme.loot().is_empty() {
                            res.push_str("It dropped:\n");
                            for x in nme.loot().iter() {
                                res.push_str(&format!(" {},", x.1.name()));
                            }
                        }
                        Ok(CmdResult::new(true, res))
                    }
                } else {
                    Ok(CmdResult::new(
                        false,
                        format!("You do not have the \"{}\". {:?} ", weapon, damage),
                    ))
                }
            } else {
                Ok(CmdResult::new(
                    false,
                    format!("There is no \"{}\" here.", enemy),
                ))
            }
        } else {
            Err(WorldError::NoRoom)
        }
    }

    // move an Item out of the current Room
    pub fn give(&mut self, name: &str) -> Option<Box<Item>> {
        if let Some(room) = self.rooms.get_mut(&self.curr_room) {
            room.items_mut().remove(name)
        } else {
            None
        }
    }

    pub fn give_from(&mut self, item: &str, container: &str) -> Option<Box<Item>> {
        if let Some(room) = self.rooms.get_mut(&self.curr_room) {
            if let Some(cont) = room.items_mut().get_mut(container) {
                if let Some(ref mut contents) = cont.contents_mut() {
                    contents.remove(item)
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            None
        }
    }

    pub fn give_all(&mut self) -> ItemMap {
        if let Some(room) = self.rooms.get_mut(&self.curr_room) {
            let items = room.items().clone();
            room.items_mut().clear();
            room.items_mut().shrink_to_fit();
            items
        } else {
            HashMap::new()
        }
    }

    // insert an Item into the current Room
    pub fn insert(
        &mut self,
        cmd: &str,
        name: &str,
        item: Option<Box<Item>>,
    ) -> Result<String, WorldError> {
        if let Some(room) = self.rooms.get_mut(&self.curr_room) {
            if let Some(obj) = item {
                room.items_mut().insert(obj.name().to_string(), obj);
                match cmd {
                    "throw" => Ok(format!("You throw the {} across the room.", name)),
                    _ => Ok("Dropped.".to_string()),
                }
            } else {
                Ok(format!("You do not have the \"{}\".", name))
            }
        } else {
            Err(WorldError::NoRoom)
        }
    }

    // insert an Item into a container Item in the current Room
    pub fn insert_into(
        &mut self,
        name: &str,
        container: &str,
        item: Option<Box<Item>>,
    ) -> Result<String, WorldError> {
        if let Some(room) = self.rooms.get_mut(&self.curr_room) {
            if let Some(obj) = item {
                if let Some(cont) = room.items_mut().get_mut(container) {
                    if let Some(ref mut contents) = cont.contents_mut() {
                        contents.insert(obj.name().to_string(), obj);
                        Ok("Placed.".to_string())
                    } else {
                        Ok("You can not put anything in there.".to_string())
                    }
                } else {
                    Ok(format!("There is no \"{}\" here.", container))
                }
            } else {
                Ok(format!("You do not have the \"{}\".", name))
            }
        } else {
            Err(WorldError::NoRoom)
        }
    }
}