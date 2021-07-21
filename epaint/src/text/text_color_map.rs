use crate::color::Color32;
use std::collections::BTreeMap;

/// Used to assign colors to different parts of a text.
#[derive(Clone, Debug, PartialEq)]
pub struct TextColorMap {
    color_map: BTreeMap<usize, Color32>,
}

impl Default for TextColorMap {
    fn default() -> Self {
        Self::new()
    }
}

impl TextColorMap {
    pub fn new() -> Self {
        TextColorMap {
            color_map: BTreeMap::new(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.color_map.is_empty()
    }

    pub fn add_color_change_at_index(&mut self, idx: usize, color: Color32) {
        self.color_map.insert(idx, color);
    }

    pub fn adjust(&mut self, adjust_color: &impl Fn(&mut Color32)) {
        for color in self.color_map.iter_mut() {
            adjust_color(color.1);
        }
    }

    pub fn color_change_at_index(&self, idx: usize) -> Option<&Color32> {
        self.color_map.get(&idx)
    }
}
