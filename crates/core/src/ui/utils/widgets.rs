use std::iter::once;

use egui::{Slider, Ui};
use log_err::LogErrOption;

use crate::{
    markers::MarkerCategoryTreeNode,
    settings::{TrailColor, TrailWidth},
    ui::UiActions,
};

pub fn trail_color_selector<A: UiActions>(
    actions: &A,
    ui: &mut Ui,
    label: &str,
    category_node: &MarkerCategoryTreeNode,
    show_reset: bool,
) {
    ui.horizontal(|ui| {
        ui.label(label);

        let mut color = once(category_node.data())
            .chain(category_node.ancestors().map(|n| n.data()))
            .filter_map(|n| *n.trail_color.borrow())
            .next()
            .log_expect("the root always has the default color")
            .0;

        let resp = ui.color_edit_button_srgb(&mut color);
        if resp.changed() {
            *category_node.data().trail_color.borrow_mut() = Some(TrailColor(color));

            actions.update_active_marker_categories();
            actions.save_settings();
        }

        if show_reset {
            if ui
                .small_button("🗑")
                .on_hover_text("Reset to parent's value")
                .clicked()
            {
                *category_node.data().trail_color.borrow_mut() = None;

                actions.update_active_marker_categories();
                actions.save_settings();
            }
        }
    });
}

pub fn trail_width_selector<A: UiActions>(
    actions: &A,
    ui: &mut Ui,
    label: &str,
    category_node: &MarkerCategoryTreeNode,
    show_reset: bool,
) {
    ui.horizontal(|ui| {
        ui.label(label);

        let mut width = once(category_node.data())
            .chain(category_node.ancestors().map(|n| n.data()))
            .filter_map(|n| *n.trail_width.borrow())
            .next()
            .log_expect("the root always has the default width")
            .0;

        let resp = ui.add(Slider::new(&mut width, TrailWidth::MIN..=TrailWidth::MAX));
        if resp.changed() {
            *category_node.data().trail_width.borrow_mut() = Some(TrailWidth(width));

            actions.update_active_marker_categories();
            actions.save_settings();
        }

        if show_reset {
            if ui
                .small_button("🗑")
                .on_hover_text("Reset to parent's value")
                .clicked()
            {
                *category_node.data().trail_width.borrow_mut() = None;

                actions.update_active_marker_categories();
                actions.save_settings();
            }
        }
    });
}
