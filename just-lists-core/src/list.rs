use crate::list_item::ListItem;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Debug)]
pub struct List {
    list_item_store: HashMap<String, ListItem>,
    top_level_items: Vec<String>,
}

impl List {
    pub fn new() -> List {
        let list = List {
            list_item_store: HashMap::new(),
            top_level_items: Vec::new(),
        };

        list
    }

    pub fn from_string(serialized_string: String) -> List {
        if serialized_string.is_empty() {
            return List::new();
        }

        serde_json::from_str(&serialized_string).unwrap()
    }

    pub fn add_list_item(&mut self, item: ListItem, index: Option<usize>) {
        self.list_item_store.insert(item.id.clone(), item.clone());

        let insertion_index = match index {
            Some(index) => index,
            None => self.top_level_items.len(),
        };

        self.top_level_items.insert(insertion_index, item.id);
    }

    pub fn add_existing_child_list_item(
        &mut self,
        item_id: &String,
        parent_id: &String,
        index: Option<usize>,
    ) -> Result<(), ListItemTaskError> {
        let parent = match self.list_item_store.get_mut(parent_id) {
            Some(parent) => parent,
            None => {
                return Err(ListItemTaskError::ParentIdDoesNotExist);
            }
        };

        if !parent.children.contains(&item_id) {
            let insertion_index = match index {
                Some(index) => index,
                None => parent.children.len(),
            };

            parent.children.insert(insertion_index, item_id.clone());
        } else {
            return Err(ListItemTaskError::ParentAlreadyHasItem);
        }

        Ok(())
    }

    pub fn add_child_list_item(
        &mut self,
        item: ListItem,
        parent_id: &String,
        index: Option<usize>,
    ) -> Result<(), ListItemTaskError> {
        let mut item_does_not_exist: bool = false;

        // check if item already exists
        if !self.list_item_store.contains_key(&item.id) {
            item_does_not_exist = true;
        }

        let item_id = item.id.clone();

        if item_does_not_exist {
            self.list_item_store.insert(item_id.clone(), item);
        }

        self.add_existing_child_list_item(&item_id, parent_id, index)
    }

    pub fn get_top_level_list_items(&self) -> Vec<&ListItem> {
        self.top_level_items
            .iter()
            .map(|i| self.list_item_store.get(i).unwrap())
            .collect()
    }

    pub fn get_children(&self, item: &ListItem) -> Vec<&ListItem> {
        item.children
            .iter()
            .map(|i| self.list_item_store.get(i).unwrap())
            .collect()
    }

    pub fn get_list_item(&self, id: &str) -> Option<&ListItem> {
        self.list_item_store.get(id)
    }

    pub fn get_mut_list_item(&mut self, id: &str) -> Option<&mut ListItem> {
        self.list_item_store.get_mut(id)
    }

    pub fn remove_list_item(&mut self, id: &str) -> Result<(), ListItemDeletionError> {
        if self.list_item_store.contains_key(id) {
            self.list_item_store.remove(id);
        } else {
            return Err(ListItemDeletionError::ItemNotExist);
        }

        for item in self.list_item_store.iter_mut().by_ref() {
            item.1.children.retain(|c| c != id);
        }

        Ok(())
    }

    pub fn remove_child_list_item(
        &mut self,
        child_id: &String,
        parent_id: Option<&String>,
    ) -> Result<(), ListItemDeletionError> {
        if !self.list_item_store.contains_key(child_id) {
            return Err(ListItemDeletionError::ChildDoesNotExist);
        }

        if let Some(parent_id) = parent_id {
            if !self.list_item_store.contains_key(parent_id) {
                return Err(ListItemDeletionError::ParentDoesNotExist);
            }

            self.get_mut_list_item(&parent_id)
                .unwrap()
                .children
                .retain(|c| c != child_id);

            Ok(())
        } else {
            self.top_level_items.retain(|i| i != child_id);
            Ok(())
        }
    }

    pub fn into_string(&self) -> String {
        serde_json::to_string(&self).unwrap()
    }

    pub fn get_index_of_child(
        &self,
        child_id: &String,
        parent_id: &String,
    ) -> Result<usize, ListItemTaskError> {
        if let Some(parent) = self.get_list_item(&parent_id) {
            for c in parent.children.iter().enumerate() {
                if c.1 == child_id {
                    return Ok(c.0);
                }
            }

            Err(ListItemTaskError::ChildIdDoesNotExist)
        } else {
            Err(ListItemTaskError::ParentIdDoesNotExist)
        }
    }

    pub fn get_index_of_top_level_item(
        &self,
        child_id: &String,
    ) -> Result<usize, ListItemTaskError> {
        for c in self.top_level_items.iter().enumerate() {
            if c.1 == child_id {
                return Ok(c.0);
            }
        }

        return Err(ListItemTaskError::ChildIdDoesNotExist);
    }
}

pub enum ListItemTaskError {
    ParentAlreadyHasItem,
    ParentIdDoesNotExist,
    ChildIdDoesNotExist,
}

#[derive(Debug)]
pub enum ListItemDeletionError {
    ItemNotExist,
    ChildDoesNotExist,
    ParentDoesNotExist,
}
