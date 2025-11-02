pub mod list;
pub mod list_item;

use crate::list::List;
use crate::list_item::ListItem;

pub fn get_sample_list() -> List {
    let mut list = List::new();

    let top_item_1 = ListItem::new("First item".to_string());
    let top_item_2 = ListItem::new("Second item".to_string());

    let parent_1_id = top_item_1.id.clone();
    let parent_2_id = top_item_2.id.clone();

    list.add_list_item(top_item_1);
    list.add_list_item(top_item_2);

    let _ = list.add_child_list_item(ListItem::new("First child item".to_string()), &parent_1_id);
    let _ = list.add_child_list_item(ListItem::new("Second child item".to_string()), &parent_2_id);

    list
}

#[cfg(test)]
mod tests {
    use core::panic;

    use crate::list::List;
    use crate::list_item::ListItem;

    #[test]
    fn can_get_new_list() {
        let _ = List::new();
    }

    #[test]
    fn can_add_item_to_list() {
        let mut list = List::new();
        list.add_list_item(ListItem::new("First list item".to_string()));

        let mut list_length = list.get_top_level_list_items().len();
        assert!(
            list_length == 1,
            "List item length was not 1. Actual: {list_length}"
        );

        list.add_list_item(ListItem::new("Second item".to_string()));

        list_length = list.get_top_level_list_items().len();
        assert!(
            list_length == 2,
            "List item length was not 2. Actual: {list_length}"
        );
    }

    #[test]
    fn can_serialize_and_deserialize() {
        let list = super::get_sample_list();

        let serialized = list.into_string();
        let deserialized: List = List::from_string(serialized);

        println!("{:?}", deserialized);
    }

    #[test]
    fn can_remove_list_items() {
        let mut list = super::get_sample_list();

        let top_level_items = list.get_top_level_list_items();

        let item_1_id = top_level_items.first().unwrap().id.clone();
        let item_2_id = top_level_items.last().unwrap().id.clone();

        match list.remove_list_item(&item_1_id) {
            Ok(_value) => (),
            Err(error) => panic!(
                "Expected to successfully remove top level item. Actual:{:?}",
                error
            ),
        };

        let first_child_of_2_id = list
            .get_list_item(
                list.get_list_item(&item_2_id)
                    .unwrap()
                    .children
                    .first()
                    .unwrap(),
            )
            .unwrap()
            .id
            .clone();

        match list.remove_list_item(&first_child_of_2_id) {
            Ok(_value) => (),
            Err(error) => panic!(
                "Expected to successfully remove child level item. Actual:{:?}",
                error
            ),
        }
    }
}
