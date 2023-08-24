use bevy::prelude::*;
use bevy_egui::EguiContext;
use egui::{
    style::{Visuals, WidgetVisuals, Widgets},
    Color32, Rounding, Stroke,
};

pub fn set_egui_style(mut query: Query<&mut EguiContext>) {
    for mut ctx in query.iter_mut() {
        ctx.get_mut().set_style(egui::Style {
            visuals: Visuals {
                widgets: Widgets {
                    noninteractive: WidgetVisuals {
                        weak_bg_fill: Color32::from_gray(27),
                        bg_fill: Color32::from_gray(27),
                        bg_stroke: Stroke::new(1.0, Color32::from_gray(60)), // separators, indentation lines
                        fg_stroke: Stroke::new(1.0, Color32::from_gray(140)), // normal text color
                        rounding: Rounding::same(2.0),
                        expansion: 0.0,
                    },
                    inactive: WidgetVisuals {
                        weak_bg_fill: Color32::from_gray(60), // button background
                        bg_fill: Color32::from_gray(60),      // checkbox background
                        bg_stroke: Default::default(),
                        fg_stroke: Stroke::new(1.0, Color32::from_gray(180)), // button text
                        rounding: Rounding::same(2.0),
                        expansion: 0.0,
                    },
                    hovered: WidgetVisuals {
                        weak_bg_fill: Color32::from_gray(70),
                        bg_fill: Color32::from_gray(70),
                        bg_stroke: Stroke::new(1.0, Color32::from_gray(150)), // e.g. hover over window edge or button
                        fg_stroke: Stroke::new(1.5, Color32::from_gray(240)),
                        rounding: Rounding::same(3.0),
                        expansion: 1.0,
                    },
                    active: WidgetVisuals {
                        weak_bg_fill: Color32::from_gray(55),
                        bg_fill: Color32::from_gray(55),
                        bg_stroke: Stroke::new(1.0, Color32::WHITE),
                        fg_stroke: Stroke::new(2.0, Color32::WHITE),
                        rounding: Rounding::same(2.0),
                        expansion: 1.0,
                    },
                    open: WidgetVisuals {
                        weak_bg_fill: Color32::from_gray(27),
                        bg_fill: Color32::from_gray(27),
                        bg_stroke: Stroke::new(1.0, Color32::from_gray(60)),
                        fg_stroke: Stroke::new(1.0, Color32::from_gray(210)),
                        rounding: Rounding::same(2.0),
                        expansion: 0.0,
                    },
                },
                panel_fill: Color32::from_rgb(28, 40, 46),
                window_fill: Color32::from_rgb(28, 40, 46),
                ..default()
            },
            ..default()
        });
    }
}
