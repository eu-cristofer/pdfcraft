//! The Crops panel: a display-only list of crop-review items in engine-defined canonical order.

use egui::Sense;
use pdfcraft_engine::{
    Document,
    export::{CropReviewItem, ExportItemKind},
};

use crate::icons;
use crate::theme::{self, Tokens};

pub(crate) fn show(ui: &mut egui::Ui, t: &Tokens, doc: &Document) {
    let items = doc.ordered_crop_review_items();
    if items.is_empty() {
        crate::panels::empty(ui, t, "crop", "This document has no crops.");
        return;
    }

    for item in items {
        row(ui, t, item);
        ui.add_space(2.0);
    }
}

fn row(ui: &mut egui::Ui, t: &Tokens, item: &CropReviewItem) {
    let primary = primary_label(item);
    let secondary = secondary_label(item);
    let label = format!("{primary} {secondary}");
    let (rect, resp) = ui.allocate_exact_size(egui::vec2(ui.available_width(), 44.0), Sense::hover());
    resp.widget_info(|| egui::WidgetInfo::labeled(egui::Role::ListItem, true, label.clone()));
    let press = theme::Press::track(ui, &resp);
    press.wash(ui, rect, 6, false);
    let body = rect.translate(press.offset());

    icons::paint(ui, egui::Rect::from_min_size(body.min + egui::vec2(4.0, 10.0), egui::vec2(18.0, 18.0)), "crop", 16.0, t.icon);
    ui.scope_builder(egui::UiBuilder::new().max_rect(egui::Rect::from_min_max(body.min + egui::vec2(30.0, 5.0), body.max)), |ui| {
        ui.vertical(|ui| {
            ui.spacing_mut().item_spacing.y = 1.0;
            ui.add(egui::Label::new(egui::RichText::new(&primary).font(theme::medium(13.0))).truncate().selectable(false));
            ui.add(egui::Label::new(egui::RichText::new(&secondary).font(theme::regular(11.0)).color(t.text_faint)).truncate().selectable(false));
        });
    });
}

fn primary_label(item: &CropReviewItem) -> String {
    let caption = item.caption.trim();
    if caption.is_empty() { format!("Page {}", item.page_number) } else { caption.to_owned() }
}

fn secondary_label(item: &CropReviewItem) -> String {
    format!("{} · Page {}", kind_label(item.kind), item.page_number)
}

fn kind_label(kind: ExportItemKind) -> &'static str {
    match kind {
        ExportItemKind::Snapshot => "Snapshot",
    }
}
