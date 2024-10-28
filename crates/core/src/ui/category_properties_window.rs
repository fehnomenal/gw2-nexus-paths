use std::iter::once;

use egui::{Context, Window};

use crate::markers::MarkerCategoryTreeNode;

use super::{
    utils::{trail_color_selector, trail_width_selector},
    UiActions,
};

pub struct CategoryPropertiesWindow<'a, A: UiActions> {
    pub actions: A,
    pub current_category_node: Option<MarkerCategoryTreeNode<'a>>,
}

impl<'a, A: UiActions> CategoryPropertiesWindow<'a, A> {
    pub fn render(&mut self, ctx: &Context) {
        let mut open = self.current_category_node.is_some();

        Window::new("Category properties")
            .open(&mut open)
            .auto_sized()
            .show(ctx, |ui| {
                if let Some(node) = &self.current_category_node {
                    let mut path = once(node.data().label.to_owned())
                        .into_iter()
                        .chain(node.ancestors().map_while(|n| {
                            let a = n.data().label.to_owned();

                            if a == "" {
                                None
                            } else {
                                Some(a)
                            }
                        }))
                        .collect::<Vec<_>>();

                    path.reverse();

                    ui.label(path.join(" > "));

                    trail_color_selector(&self.actions, ui, "Route color:", node, true);
                    trail_width_selector(&self.actions, ui, "Route width:", node, true);
                }
            });

        if !open {
            self.current_category_node = None;
        }
    }
}
