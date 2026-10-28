//! Crop review UI: the Crops panel plus the stored-crop overlay on the page canvas.

use egui::{Color32, CornerRadius, Pos2, Rect, Sense, Stroke, Vec2};
use pdfcraft_engine::export::CropPdfRect;
use pdfcraft_engine::{
    Document,
    export::{CropReviewItem, ExportItemKind},
};

use crate::icons;
use crate::theme::{self, Tokens};
use crate::{
    canvas::{DocView, PageXform},
    comments,
    page_pointer::PagePointer,
    panels::Nav,
    widgets,
};

const HANDLE_SIZE: f32 = 10.0;
const HANDLE_HIT_RADIUS: f32 = 8.0;
const MIN_SIDE: f32 = 6.0;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum PanelEdit {
    UpdateMetadata { id: String, caption: String, note: Option<String> },
    Delete { id: String },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum HeaderAction {
    Export,
    ToggleHidden,
}

pub(crate) fn header_controls(ui: &mut egui::Ui, hidden: bool) -> Option<HeaderAction> {
    let mut action = None;
    let more = icons::button(ui, "ellipsis", 26.0, false, tl!("More options"));
    egui::Popup::menu(&more).show(|ui| {
        ui.set_min_width(220.0);
        if ui.button(tl!("Export Crop Review Package")).clicked() {
            action = Some(HeaderAction::Export);
            ui.close();
        }
        ui.separator();
        let label = if hidden { tl!("Show all crops") } else { tl!("Hide all crops") };
        if ui.button(label).clicked() {
            action = Some(HeaderAction::ToggleHidden);
            ui.close();
        }
    });
    action
}

pub(crate) fn show(ui: &mut egui::Ui, t: &Tokens, doc: &Document, view: &mut DocView, nav: &mut Option<Nav>) -> Option<PanelEdit> {
    let mut edit = None;
    ui.horizontal(|ui| {
        let label = if view.crop_review.creating { tl!("Stop drawing") } else { tl!("Draw crop") };
        if ui.add(egui::Button::new(label)).clicked() {
            if !view.crop_review.creating {
                view.crop_review.hidden = false;
            }
            view.crop_review.creating = !view.crop_review.creating;
        }
    });
    ui.add_space(8.0);

    let items = doc.ordered_crop_review_items();
    if items.is_empty() {
        crate::panels::empty(ui, t, "crop", "This document has no crops.");
        return None;
    }

    view.crop_review.ensure_selected(doc);

    if view.crop_review.hidden {
        ui.label(egui::RichText::new(tl!("Crop rectangles are hidden on the page.")).small().color(t.text_faint));
        ui.add_space(8.0);
    }

    for item in items {
        let selected = view.crop_review.selected.as_deref() == Some(item.id.as_str());
        if row(ui, t, item, selected) {
            view.crop_review.select(item.id.as_str());
            *nav = Some(Nav::Flash(item.page_number.saturating_sub(1), user_rect(item)));
        }
        ui.add_space(2.0);
    }

    if let Some(item) = view.crop_review.selected_item(doc) {
        if view.crop_review.metadata_for.as_deref() != Some(item.id.as_str()) {
            view.crop_review.load_metadata(item);
        }
        ui.add_space(10.0);
        edit = detail(ui, t, view, item);
    }

    edit
}

pub(crate) fn page_input(
    resp: &egui::Response,
    doc: &Document,
    xf: &PageXform,
    ptr: &PagePointer,
    page: usize,
    view: &mut DocView,
    interactive: bool,
) -> bool {
    if view.crop_review.creating {
        return crate::zoom_snap::marquee_input(resp, xf, ptr, page, &mut view.marquee, &mut view.marquee_done);
    }
    if view.crop_review.hidden {
        return false;
    }
    if !interactive {
        return false;
    }

    if let Some(gesture) = view.crop_review.gesture
        && gesture.page == page
    {
        if resp.drag_stopped() || !ptr.primary_down {
            if let Some(updated) = commit_gesture(doc, xf, ptr.pos.unwrap_or(gesture.start), gesture) {
                let selected = updated.id.clone();
                view.pending_action = Some(crate::canvas::ViewAction::UpsertCropReviewItem(updated));
                view.crop_review.select(&selected);
            }
            view.crop_review.gesture = None;
        }
        return true;
    }

    let Some(pointer) = ptr.pos.filter(|p| xf.rect.contains(*p)) else { return false };
    let items = page_items(doc, page).map(|item| (item, xf.user_rect(&doc.info, page, user_rect(item)))).collect::<Vec<_>>();
    let handle_hit = items.iter().filter_map(|(item, rect)| handle_hit(*rect, pointer).map(|kind| (*item, *rect, kind))).next_back();
    let rect_hit = items.iter().rfind(|(_, rect)| rect.expand(4.0).contains(pointer)).copied();

    if resp.drag_started()
        && let Some(start) = ptr.origin.filter(|p| xf.rect.contains(*p))
    {
        if let Some((item, rect, kind)) = handle_hit {
            view.crop_review.select(item.id.as_str());
            view.crop_review.gesture = Some(crate::crop_review_ui::CropGesture { page, kind, id: item_id(item), from_rect: rect, start });
            return true;
        }
        if let Some((item, rect)) = rect_hit {
            view.crop_review.select(item.id.as_str());
            view.crop_review.gesture = Some(crate::crop_review_ui::CropGesture {
                page,
                kind: crate::crop_review_ui::CropGestureKind::Move,
                id: item_id(item),
                from_rect: rect,
                start,
            });
            return true;
        }
    }

    if resp.clicked()
        && let Some((item, _)) = rect_hit
    {
        view.crop_review.select(item.id.as_str());
        return true;
    }

    (handle_hit.is_some() || rect_hit.is_some()) && resp.is_pointer_button_down_on()
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
    if view.crop_review.hidden && !view.crop_review.creating {
        return;
    }
    for item in page_items(doc, page) {
        let base = xf.user_rect(&doc.info, page, user_rect(item));
        let selected = view.crop_review.selected.as_deref() == Some(item.id.as_str());
        let rect = preview_rect(view, item, base, pointer);
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
        if selected {
            for handle in handles(rect) {
                painter.rect_filled(handle, CornerRadius::same(2), Color32::WHITE);
                painter.rect_stroke(handle, CornerRadius::same(2), Stroke::new(1.0, Color32::from_rgb(0x14, 0x73, 0xE6)), egui::StrokeKind::Middle);
            }
        }
    }

    if view.crop_review.creating {
        crate::zoom_snap::paint_marquee(_ui, xf, pointer, page, view.marquee);
    }
}

pub(crate) fn delete_key(ctx: &egui::Context, doc: &Document, view: &mut DocView) -> bool {
    if ctx.egui_wants_keyboard_input() || view.crop_review.creating || view.crop_review.gesture.is_some() {
        return false;
    }
    let Some(id) = view.crop_review.selected.clone() else { return false };
    if !ctx.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::Delete) || i.consume_key(egui::Modifiers::NONE, egui::Key::Backspace)) {
        return false;
    }
    delete_selected_crop(doc, view, &id);
    true
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

fn detail(ui: &mut egui::Ui, t: &Tokens, view: &mut DocView, item: &CropReviewItem) -> Option<PanelEdit> {
    let mut edit = None;
    let mut open_menu = false;
    let mut more_rect = None;
    ui.group(|ui| {
        ui.set_width(ui.available_width());
        ui.spacing_mut().item_spacing.y = 8.0;
        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                ui.label(egui::RichText::new(primary_label(item)).font(theme::semibold(15.5)));
                ui.label(egui::RichText::new(secondary_label(item)).font(theme::regular(11.0)).color(t.text_faint));
            });
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Min), |ui| {
                let more = icons::button(ui, "ellipsis", 22.0, false, tl!("More"));
                open_menu = more.clicked();
                more_rect = Some(more.rect);
            });
        });

        let caption_label = ui.label(tl!("Caption"));
        let caption = ui
            .add(widgets::line(&mut view.crop_review.metadata_caption).desired_width(f32::INFINITY))
            .labelled_by(caption_label.id)
            .accessible_name(tl!("Caption"));
        if caption.changed() {
            edit = Some(pending_metadata_edit(view, item.id.as_str()));
        }

        let note_label = ui.label(tl!("Note"));
        let note = ui
            .add(egui::TextEdit::multiline(&mut view.crop_review.metadata_note).desired_width(f32::INFINITY).desired_rows(4))
            .labelled_by(note_label.id)
            .accessible_name(tl!("Note"));
        if note.changed() {
            edit = Some(pending_metadata_edit(view, item.id.as_str()));
        }

        let rect = item.rect_pdf;
        ui.label(
            egui::RichText::new(crate::i18n::fmt(tl!("Kind: {kind}"), &[("kind", kind_label(item.kind))]))
                .font(theme::regular(11.0))
                .color(t.text_faint),
        );
        ui.label(
            egui::RichText::new(crate::i18n::fmt(tl!("Page: {page}"), &[("page", &item.page_number.to_string())]))
                .font(theme::regular(11.0))
                .color(t.text_faint),
        );
        ui.label(egui::RichText::new(rect_label(rect)).font(theme::regular(11.0)).color(t.text_faint));
    });

    let menu_id = ui.scope_id().with(("crop-menu", item.id.as_str()));
    if open_menu {
        egui::Popup::open_id(ui.ctx(), menu_id);
    }
    let anchor = more_rect.map_or(egui::PopupAnchor::PointerFixed, egui::PopupAnchor::ParentRect);
    egui::Popup::new(menu_id, ui.ctx().clone(), anchor, ui.layer_id())
        .open_memory(None)
        .kind(egui::PopupKind::Menu)
        .layout(egui::Layout::top_down_justified(egui::Align::Min))
        .show(|ui| {
            ui.set_min_width(160.0);
            if ui.button(tl!("Delete")).clicked() {
                edit = Some(PanelEdit::Delete { id: item.id.clone() });
                ui.close();
            }
        });

    edit
}

fn page_items<'a>(doc: &'a Document, page: usize) -> impl Iterator<Item = &'a CropReviewItem> + 'a {
    doc.crop_review().items().iter().filter(move |item| item.page_number == page + 1)
}

fn preview_rect(view: &DocView, item: &CropReviewItem, base: Rect, pointer: Option<Pos2>) -> Rect {
    if let Some(gesture) = view.crop_review.gesture
        && gesture.id == item_id(item)
        && let Some(pointer) = pointer
    {
        return dragged_rect(gesture.from_rect, gesture.kind, pointer - gesture.start);
    }
    base
}

fn item_id(item: &CropReviewItem) -> [u64; 2] {
    hash_id(item.id.as_bytes())
}

fn hash_id(bytes: &[u8]) -> [u64; 2] {
    use std::hash::Hasher;
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    hasher.write(bytes);
    let first = hasher.finish();
    let mut second = std::collections::hash_map::DefaultHasher::new();
    second.write_u64(first.rotate_left(13));
    second.write(bytes);
    [first, second.finish()]
}

fn handles(r: Rect) -> [Rect; 4] {
    [
        Rect::from_center_size(r.left_top(), Vec2::splat(HANDLE_SIZE)),
        Rect::from_center_size(r.right_top(), Vec2::splat(HANDLE_SIZE)),
        Rect::from_center_size(r.left_bottom(), Vec2::splat(HANDLE_SIZE)),
        Rect::from_center_size(r.right_bottom(), Vec2::splat(HANDLE_SIZE)),
    ]
}

fn handle_hit(rect: Rect, pointer: Pos2) -> Option<crate::crop_review_ui::CropGestureKind> {
    [
        (rect.left_top(), crate::crop_review_ui::CropGestureKind::Corner { left: true, top: true }),
        (rect.right_top(), crate::crop_review_ui::CropGestureKind::Corner { left: false, top: true }),
        (rect.left_bottom(), crate::crop_review_ui::CropGestureKind::Corner { left: true, top: false }),
        (rect.right_bottom(), crate::crop_review_ui::CropGestureKind::Corner { left: false, top: false }),
    ]
    .into_iter()
    .find_map(|(corner, kind)| (corner.distance(pointer) <= HANDLE_HIT_RADIUS).then_some(kind))
}

fn dragged_rect(from: Rect, kind: crate::crop_review_ui::CropGestureKind, delta: Vec2) -> Rect {
    match kind {
        crate::crop_review_ui::CropGestureKind::Move => from.translate(delta),
        crate::crop_review_ui::CropGestureKind::Corner { left, top } => {
            let handle = (if left { -1 } else { 1 }, if top { -1 } else { 1 });
            let resized = comments::resized(from, handle, delta, None);
            if resized.width() >= MIN_SIDE && resized.height() >= MIN_SIDE { resized } else { from }
        }
    }
}

fn commit_gesture(doc: &Document, xf: &PageXform, pointer: Pos2, gesture: crate::crop_review_ui::CropGesture) -> Option<CropReviewItem> {
    let id = find_item_id(doc, gesture.id)?;
    let item = doc.crop_review().get(id)?.clone();
    let rect = dragged_rect(gesture.from_rect, gesture.kind, pointer - gesture.start).intersect(xf.rect);
    if rect.width() < MIN_SIDE || rect.height() < MIN_SIDE {
        return None;
    }
    let user = screen_rect_to_user(xf, &doc.info, gesture.page, rect);
    let mut updated = item;
    updated.rect_pdf = CropPdfRect::from_user_rect(user).ok()?;
    updated.updated_at = chrono::Utc::now();
    Some(updated)
}

fn screen_rect_to_user(xf: &PageXform, info: &pdfcraft_render::DocInfo, page: usize, rect: Rect) -> [f32; 4] {
    let (a, b) = (xf.screen_to_view(rect.min), xf.screen_to_view(rect.max));
    info.pages[page].view_rect_to_user([a.0, a.1, b.0, b.1])
}

fn find_item_id(doc: &Document, id: [u64; 2]) -> Option<&str> {
    doc.crop_review().items().iter().find(|item| item_id(item) == id).map(|item| item.id.as_str())
}

fn delete_selected_crop(doc: &Document, view: &mut DocView, id: &str) {
    if doc.crop_review().get(id).is_some() {
        view.pending_action = Some(crate::canvas::ViewAction::DeleteCropReviewItem(id.to_owned()));
    }
}

pub(crate) fn primary_label(item: &CropReviewItem) -> String {
    let caption = item.caption.trim();
    if caption.is_empty() { format!("Page {}", item.page_number) } else { caption.to_owned() }
}

pub(crate) fn secondary_label(item: &CropReviewItem) -> String {
    format!("{} · Page {}", kind_label(item.kind), item.page_number)
}

fn pending_metadata_edit(view: &DocView, id: &str) -> PanelEdit {
    PanelEdit::UpdateMetadata {
        id: id.to_owned(),
        caption: view.crop_review.metadata_caption.clone(),
        note: Some(view.crop_review.metadata_note.clone()).filter(|text| !text.trim().is_empty()),
    }
}

fn kind_label(kind: ExportItemKind) -> &'static str {
    match kind {
        ExportItemKind::Snapshot => "Snapshot",
    }
}

fn rect_label(rect: CropPdfRect) -> String {
    crate::i18n::fmt(
        tl!("Area: {left}, {bottom}, {right}, {top}"),
        &[
            ("left", &format_coord(rect.left)),
            ("bottom", &format_coord(rect.bottom)),
            ("right", &format_coord(rect.right)),
            ("top", &format_coord(rect.top)),
        ],
    )
}

fn format_coord(value: f32) -> String {
    format!("{value:.1}")
}

fn user_rect(item: &CropReviewItem) -> [f32; 4] {
    [item.rect_pdf.left, item.rect_pdf.bottom, item.rect_pdf.right, item.rect_pdf.top]
}
