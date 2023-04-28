use serde_json::Value;

use crate::layout::layout_json::ItemEnum;
use crate::layout::layout_item::Item;
use crate::error::{Error, LayoutErr};

pub struct Row {
    items: Vec<Item>,
}

fn reemit_layout_error(e: Error, count: usize) -> Error {
    match e {
        Error::LayoutError(e ) => Error::LayoutError(LayoutErr {
            message: e.message,
            row: None,
            item: Some(count as u64)
        }),
        _ => e
    }
}

impl Row {
    pub fn from_str(data: &str) -> Self {
        let mut item_list = Vec::new();
        let mut previous_char = '\0';
        let mut current = String::new();
        for c in data.to_string().chars() {
            if (c == '{' || c == '}') && previous_char != '\\' {
                item_list.push(Item::from_str(&current));
                current = String::new();
                previous_char = '\0';
            } else {
                current += &c.to_string();
                previous_char = c;
            }
        }
        if !current.is_empty() {
            item_list.push(Item::from_str(&current));
        }
        Row { items: item_list }
    }

    pub fn from_vec(data: Vec<ItemEnum>) -> Self {
        let mut items: Vec<Item> = Vec::new();
        for (_count, item) in data.iter().enumerate() {
            items.push(Item::new(item.clone()));
        }
        Row { items }
    }

    pub fn to_string(
        &self,
        data: &Value,
        variable_color: String,
        text_color: String,
        unit_color: String,
        variable_bg_color: String,
        text_bg_color: String,
        unit_bg_color: String,
        metric: bool,
    ) -> crate::Result<String> {
        let mut s = String::new();
        for (count, i) in self.items.iter().enumerate() {
            s += &*i.to_string(
                data,
                variable_color.clone(),
                text_color.clone(),
                unit_color.clone(),
                variable_bg_color.clone(),
                text_bg_color.clone(),
                unit_bg_color.clone(),
                metric,
            ).map_err(|e| reemit_layout_error(e, count))?;
        }
        Ok(s)
    }
}
