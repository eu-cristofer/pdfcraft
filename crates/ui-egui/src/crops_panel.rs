//! Crop review UI: the Crops panel plus the stored-crop overlay on the page canvas.

use egui::{Color32, CornerRadius, Pos2, Sense, Stroke};
use pdfcraft_engine::{
    Document,
    export::{CropReviewItem, ExportItemKind},
};

use crate::icons;
use crate::theme::{self, Tokens};
use crate::{
    canvas::{DocView, PageXform},
    page_pointer::PagePointer,
    panels::Nav,
};

pub(crate) fn show(ui: &mut egui::Ui, t: &Tokens, doc: &Document, view: &mut DocView, nav: &mut Option<Nav>) {
    ui.horizontal(|ui| {
        let label = if view.crop_review.creating { tl!("Stop drawing") } else { tl!("Draw crop") };
        if ui.add(egui::Button::new(label)).clicked() {
            view.crop_review.creating = !view.crop_review.creating;
        }
    });
    ui.add_space(8.0);

    let items = doc.ordered_crop_review_items();
    if items.is_empty() {
        crate::panels::empty(ui, t, "crop", "This document has no crops.");
        return;
    }

    view.crop_review.ensure_selected(doc);

    for item in items {
        let selected = view.crop_review.selected.as_deref() == Some(item.id.as_str());
        if row(ui, t, item, selected) {
            view.crop_review.select(item.id.as_str());
            *nav = Some(Nav::Flash(item.page_number.saturating_sub(1), user_rect(item)));
        }
        ui.add_space(2.0);
    }
}

pub(crate) fn page_input(resp: &egui::Response, doc: &Document, xf: &PageXform, ptr: &PagePointer, page: usize, view: &mut DocView) -> bool {
    if view.crop_review.creating {
        return crate::zoom_snap::marquee_input(resp, xf, ptr, page, &mut view.marquee, &mut view.marquee_done);
    }

    let Some(pointer) = ptr.pos.filter(|p| xf.rect.contains(*p)) else { return false };
    let hit = page_items(doc, page).filter(|item| xf.user_rect(&doc.info, page, user_rect(item)).expand(4.0).contains(pointer)).last();

    if resp.clicked()
        && let Some(item) = hit
    {
        view.crop_review.select(item.id.as_str());
        return true;
    }

    hit.is_some() && resp.is_pointer_button_down_on()
}

pub(crate) fn paint_page(
    _ui: &egui::Ui,
    painter: &egui::Painter,
    doc: &Document,
    xf: &PageXform,
    page: usize,
    view: &DocView,
    pointer: Option<Pos2>,
) {
    for item in page_items(doc, page) {
        let rect = xf.user_rect(&doc.info, page, user_rect(item));
        let selected = view.crop_review.selected.as_deref() == Some(item.id.as_str());
        let hovered = pointer.is_some_and(|p| rect.expand(4.0).contains(p));
        let fill = if selected {
            Color32::from_rgba_unmultiplied(0x14, 0x73, 0xE6, 28)
        } else if hovered {
            Color32::from_rgba_unmultiplied(0x14, 0x73, 0xE6, 14)
        } else {
            Color32::TRANSPARENT
        };
        let stroke = if selected {
            Stroke::new(2.0, Color32::from_rgb(0x14, 0x73, 0xE6))
        } else if hovered {
            Stroke::new(1.5, Color32::from_rgb(0x3A, 0x7B, 0xF0))
        } else {
            Stroke::new(1.0, Color32::from_gray(0x8A))
        };
        painter.rect_filled(rect, CornerRadius::same(4), fill);
        painter.rect_stroke(rect, CornerRadius::same(4), stroke, egui::StrokeKind::Middle);
    }

    if view.crop_review.creating {
        crate::zoom_snap::paint_marquee(_ui, xf, pointer, page, view.marquee);
    }
}

fn row(ui: &mut egui::Ui, t: &Tokens, item: &CropReviewItem, selected: bool) -> bool {
    let primary = primary_label(item);
    let secondary = secondary_label(item);
    let label = format!("{primary} {secondary}");
    let (rect, resp) = ui.allocate_exact_size(egui::vec2(ui.available_width(), 44.0), Sense::click());
    resp.widget_info(|| egui::WidgetInfo::selected(egui::Role::ListItem, true, selected, label.clone()));
    let press = theme::Press::track(ui, &resp);
    if selected {
        ui.painter().rect_filled(rect, CornerRadius::same(6), t.hover.gamma_multiply(0.7));
    }
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

    theme::hand(resp).clicked()
}

fn page_items<'a>(doc: &'a Document, page: usize) -> impl Iterator<Item = &'a CropReviewItem> + 'a {
    doc.crop_review().items().iter().filter(move |item| item.page_number == page + 1)
}

pub(crate) fn primary_label(item: &CropReviewItem) -> String {
    let caption = item.caption.trim();
    if caption.is_empty() { format!("Page {}", item.page_number) } else { caption.to_owned() }
}

pub(crate) fn secondary_label(item: &CropReviewItem) -> String {
    format!("{} · Page {}", kind_label(item.kind), item.page_number)
}

fn kind_label(kind: ExportItemKind) -> &'static str {
    match kind {
        ExportItemKind::Snapshot => "Snapshot",
    }
}

fn user_rect(item: &CropReviewItem) -> [f32; 4] {
    [item.rect_pdf.left, item.rect_pdf.bottom, item.rect_pdf.right, item.rect_pdf.top]
}
