//! Crop review panel in the real shell (egui_kittest).

use chrono::{TimeZone, Utc};
use egui::accesskit::Role;
use egui::{Pos2, Rect, pos2};
use egui_kittest::Harness;
use egui_kittest::kittest::Queryable;
use pdfcraft_engine::export::{CropPdfRect, CropReviewItem, ExportItemKind};
use pdfcraft_ui_egui::PdfCraftApp;

use crate::support;

const FIXTURE: &[u8] = b"%PDF-1.7
1 0 obj << /Type /Catalog /Pages 2 0 R >> endobj
2 0 obj << /Type /Pages /Kids [3 0 R 4 0 R] /Count 2 >> endobj
3 0 obj << /Type /Page /Parent 2 0 R /MediaBox [0 0 300 400] /Contents 5 0 R /Resources << /Font << /F1 7 0 R >> >> >> endobj
4 0 obj << /Type /Page /Parent 2 0 R /MediaBox [0 0 300 400] /Contents 6 0 R /Resources << /Font << /F1 7 0 R >> >> >> endobj
5 0 obj << /Length 43 >> stream
BT /F1 16 Tf 20 320 Td (Crop review page 1) Tj ET
endstream endobj
6 0 obj << /Length 43 >> stream
BT /F1 16 Tf 20 320 Td (Crop review page 2) Tj ET
endstream endobj
7 0 obj << /Type /Font /Subtype /Type1 /BaseFont /Helvetica >> endobj
trailer << /Root 1 0 R >>
%%EOF";

const WIDTH: f32 = 1400.0;

fn temp_dir(name: &str) -> std::path::PathBuf {
    static NEXT: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
    let n = NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    let dir = std::env::temp_dir().join(format!("pdfcraft-crop-ui-{name}-{}-{n}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn harness(setup: impl FnOnce(&mut PdfCraftApp) + 'static) -> Harness<'static, PdfCraftApp> {
    let mut h = Harness::builder().with_size(egui::vec2(WIDTH, 900.0)).build_eframe(move |_cc| {
        let mut app = PdfCraftApp::new();
        app.set_option("language", "en").unwrap();
        app.open_bytes("doc.pdf", None, FIXTURE.to_vec()).unwrap();
        app.set_option("left", "closed").unwrap();
        setup(&mut app);
        app
    });
    h.run_steps(6);
    support::settle(&mut h);
    h
}

fn stamp(seconds: i64) -> chrono::DateTime<Utc> {
    Utc.timestamp_opt(seconds, 0).single().expect("timestamp")
}

fn item(id: &str, page_number: usize, rect_pdf: CropPdfRect, caption: &str, seconds: i64) -> CropReviewItem {
    CropReviewItem {
        id: id.to_string(),
        kind: ExportItemKind::Snapshot,
        page_number,
        rect_pdf,
        caption: caption.to_string(),
        note: None,
        ocr_text: None,
        ocr_extracted_at: None,
        created_at: stamp(seconds),
        updated_at: stamp(seconds),
    }
}

fn seed(app: &mut PdfCraftApp, items: impl IntoIterator<Item = CropReviewItem>) {
    let id = app.active_ids().expect("active document").1;
    let store = app.session.crop_review_mut(id).expect("crop review store");
    for item in items {
        store.upsert(item).expect("valid crop");
    }
}

fn crop_button(h: &Harness<'static, PdfCraftApp>) -> egui::Rect {
    h.get_by_role_and_label(Role::Button, "Crops").rect()
}

fn crop_row(h: &Harness<'static, PdfCraftApp>, label: &str) -> egui::Rect {
    h.get_by_role_and_label(Role::ListItem, label).rect()
}

fn field<'a>(h: &'a Harness<'static, PdfCraftApp>, label: &'a str) -> egui_kittest::Node<'a> {
    h.query_all_by(|node| matches!(node.role(), Role::TextInput | Role::MultilineTextInput) && node.label().as_deref() == Some(label))
        .next()
        .unwrap_or_else(|| panic!("no field {label:?}"))
}

fn crop_center(h: &Harness<'static, PdfCraftApp>, page_number: usize, rect: CropPdfRect) -> Pos2 {
    let page = h.state().views[0].page_screen_rect(page_number - 1).expect("page on screen");
    let x = ((rect.left + rect.right) * 0.5) / 300.0 * page.width();
    let y = (400.0 - (rect.bottom + rect.top) * 0.5) / 400.0 * page.height();
    pos2(page.left() + x, page.top() + y)
}

fn click(h: &mut Harness<'static, PdfCraftApp>, p: Pos2) {
    h.hover_at(p);
    h.run_steps(1);
    h.drag_at(p);
    h.run_steps(1);
    h.drop_at(p);
    h.run_steps(3);
}

fn drag(h: &mut Harness<'static, PdfCraftApp>, from: Pos2, to: Pos2) {
    h.hover_at(from);
    h.run_steps(1);
    h.drag_at(from);
    h.run_steps(1);
    for k in 1..=4 {
        h.hover_at(from + (to - from) * (k as f32 / 4.0));
        h.run_steps(1);
    }
    h.drop_at(to);
    h.run_steps(4);
}

fn crop_corner(h: &Harness<'static, PdfCraftApp>, page_number: usize, rect: CropPdfRect, right: bool, top: bool) -> Pos2 {
    let page = h.state().views[0].page_screen_rect(page_number - 1).expect("page on screen");
    let x = if right { rect.right } else { rect.left } / 300.0 * page.width();
    let y_pdf = if top { rect.top } else { rect.bottom };
    let y = (400.0 - y_pdf) / 400.0 * page.height();
    pos2(page.left() + x, page.top() + y)
}

#[test]
fn crops_panel_is_a_right_side_panel_and_lists_rows_in_canonical_order() {
    let h = harness(|app| {
        seed(
            app,
            [
                item("page-2", 2, CropPdfRect { left: 20.0, bottom: 300.0, right: 80.0, top: 360.0 }, "Third", 30),
                item("page-1-lower", 1, CropPdfRect { left: 20.0, bottom: 200.0, right: 80.0, top: 260.0 }, "Second", 20),
                item("page-1-top", 1, CropPdfRect { left: 20.0, bottom: 320.0, right: 80.0, top: 380.0 }, "First", 10),
            ],
        );
        app.set_option("panel", "crops").unwrap();
    });

    let rail = crop_button(&h);
    let first = crop_row(&h, "First Snapshot · Page 1");
    let second = crop_row(&h, "Second Snapshot · Page 1");
    let third = crop_row(&h, "Third Snapshot · Page 2");

    assert!(first.right() < rail.left() && first.left() > WIDTH / 2.0, "{first:?} {rail:?}");
    assert!(first.top() < second.top() && second.top() < third.top(), "{first:?} {second:?} {third:?}");
}

#[test]
fn crops_panel_prefers_caption_and_falls_back_to_page_number() {
    let h = harness(|app| {
        seed(
            app,
            [
                item("captioned", 1, CropPdfRect { left: 24.0, bottom: 280.0, right: 84.0, top: 360.0 }, "Named crop", 10),
                item("fallback", 2, CropPdfRect { left: 30.0, bottom: 250.0, right: 90.0, top: 330.0 }, "   ", 20),
            ],
        );
        app.set_option("panel", "crops").unwrap();
    });

    crop_row(&h, "Named crop Snapshot · Page 1");
    crop_row(&h, "Page 2 Snapshot · Page 2");
}

#[test]
fn crops_panel_secondary_line_shows_kind_and_page_number() {
    let h = harness(|app| {
        seed(
            app,
            [
                item("first", 1, CropPdfRect { left: 18.0, bottom: 300.0, right: 78.0, top: 372.0 }, "Alpha", 10),
                item("second", 2, CropPdfRect { left: 24.0, bottom: 240.0, right: 84.0, top: 312.0 }, "", 20),
            ],
        );
        app.set_option("panel", "crops").unwrap();
    });

    crop_row(&h, "Alpha Snapshot · Page 1");
    crop_row(&h, "Page 2 Snapshot · Page 2");
}

#[test]
fn selecting_from_the_crops_panel_selects_the_overlay_item_and_updates_metadata() {
    let rect = CropPdfRect { left: 24.0, bottom: 280.0, right: 84.0, top: 360.0 };
    let mut h = harness(move |app| {
        seed(app, [item("captioned", 1, rect, "Named crop", 10)]);
        {
            let id = app.active_ids().expect("active document").1;
            let store = app.session.crop_review_mut(id).expect("crop review store");
            let mut crop = store.get("captioned").expect("seeded crop").clone();
            crop.note = Some("First note".into());
            store.upsert(crop).expect("updated crop");
        }
        app.set_option("panel", "crops").unwrap();
    });

    crop_row(&h, "Named crop Snapshot · Page 1");
    h.get_by_role_and_label(Role::ListItem, "Named crop Snapshot · Page 1").click();
    h.run_steps(3);

    assert_eq!(h.state().views[0].selected_crop(), Some("captioned"));
    assert!(h.state().views[0].flash.is_some(), "selecting from the list flashes the overlay rectangle");

    assert_eq!(field(&h, "Caption").value().as_deref(), Some("Named crop"));
    assert_eq!(field(&h, "Note").value().as_deref(), Some("First note"));
}

#[test]
fn selecting_from_the_overlay_selects_the_crops_row_and_updates_metadata() {
    let rect = CropPdfRect { left: 24.0, bottom: 280.0, right: 84.0, top: 360.0 };
    let mut h = harness(move |app| {
        seed(app, [item("overlay-hit", 1, rect, "Overlay crop", 10)]);
        {
            let id = app.active_ids().expect("active document").1;
            let store = app.session.crop_review_mut(id).expect("crop review store");
            let mut crop = store.get("overlay-hit").expect("seeded crop").clone();
            crop.note = Some("Overlay note".into());
            store.upsert(crop).expect("updated crop");
        }
        app.set_option("panel", "crops").unwrap();
    });

    let center = crop_center(&h, 1, rect);
    click(&mut h, center);

    assert_eq!(h.state().views[0].selected_crop(), Some("overlay-hit"));
    crop_row(&h, "Overlay crop Snapshot · Page 1");

    assert_eq!(field(&h, "Caption").value().as_deref(), Some("Overlay crop"));
    assert_eq!(field(&h, "Note").value().as_deref(), Some("Overlay note"));
}

#[test]
fn metadata_edits_update_the_selected_crop_without_oscillating_selection_state() {
    let rect = CropPdfRect { left: 24.0, bottom: 280.0, right: 84.0, top: 360.0 };
    let mut h = harness(move |app| {
        seed(app, [item("editable", 1, rect, "Before", 10)]);
        {
            let id = app.active_ids().expect("active document").1;
            let store = app.session.crop_review_mut(id).expect("crop review store");
            let mut crop = store.get("editable").expect("seeded crop").clone();
            crop.note = Some("Old note".into());
            store.upsert(crop).expect("updated crop");
        }
        app.set_option("panel", "crops").unwrap();
        app.views[0].select_crop("editable");
    });

    let caption = field(&h, "Caption");
    caption.click();
    h.key_press(egui::Key::End);
    caption.type_text(" after");
    h.run_steps(2);

    let note = field(&h, "Note");
    note.click();
    h.key_press(egui::Key::End);
    note.type_text(" updated");
    h.run_steps(2);

    let id = h.state().active_ids().expect("active document").1;
    let doc = h.state().session.get(id).expect("document");
    let item = doc.crop_review().get("editable").expect("updated crop");
    assert_eq!(item.caption, "Before after");
    assert_eq!(item.note.as_deref(), Some("Old note updated"));
    assert_eq!(h.state().views[0].selected_crop(), Some("editable"));
    assert_eq!(field(&h, "Caption").value().as_deref(), Some("Before after"));
    assert_eq!(field(&h, "Note").value().as_deref(), Some("Old note updated"));
}

#[test]
fn creating_a_crop_rectangle_appears_in_overlay_crops_and_metadata() {
    let mut h = harness(|app| app.set_option("panel", "crops").unwrap());
    h.state_mut().set_option("quick", "crop-review").unwrap();
    h.state_mut().views[0].set_crop_review_creating(true);
    h.state_mut().views[0].marquee_done = Some((0, Rect::from_min_max(pos2(10.0, 10.0), pos2(40.0, 40.0)), [20.0, 40.0, 90.0, 85.0]));
    h.run_steps(4);

    let id = h.state().active_ids().expect("active document").1;
    let doc = h.state().session.get(id).expect("document");
    assert_eq!(doc.crop_review().items().len(), 1);
    let created = &doc.crop_review().items()[0];
    assert_eq!(h.state().views[0].selected_crop(), Some(created.id.as_str()));

    crop_row(&h, "Page 1 Snapshot · Page 1");
    assert_eq!(field(&h, "Caption").value().as_deref(), Some(""));
}

#[test]
fn deleting_a_selected_crop_removes_it_and_selects_the_next_crop() {
    let mut h = harness(|app| {
        seed(
            app,
            [
                item("crop-0001", 1, CropPdfRect { left: 24.0, bottom: 280.0, right: 84.0, top: 360.0 }, "First", 10),
                item("crop-0002", 2, CropPdfRect { left: 30.0, bottom: 240.0, right: 90.0, top: 320.0 }, "Second", 20),
            ],
        );
        app.set_option("panel", "crops").unwrap();
        app.views[0].select_crop("crop-0001");
    });

    h.get_by_label("More").click();
    h.run_steps(2);
    h.get_by_label("Delete").click();
    h.run_steps(3);

    let id = h.state().active_ids().expect("active document").1;
    let doc = h.state().session.get(id).expect("document");
    assert!(doc.crop_review().get("crop-0001").is_none());
    assert_eq!(h.state().views[0].selected_crop(), Some("crop-0002"));
    assert_eq!(field(&h, "Caption").value().as_deref(), Some("Second"));
}

#[test]
fn delete_key_removes_the_selected_crop() {
    let mut h = harness(|app| {
        seed(app, [item("crop-0001", 1, CropPdfRect { left: 24.0, bottom: 280.0, right: 84.0, top: 360.0 }, "First", 10)]);
        app.set_option("panel", "crops").unwrap();
        app.views[0].select_crop("crop-0001");
    });

    h.key_press(egui::Key::Delete);
    h.run_steps(3);

    let id = h.state().active_ids().expect("active document").1;
    let doc = h.state().session.get(id).expect("document");
    assert!(doc.crop_review().items().is_empty());
    assert_eq!(h.state().views[0].selected_crop(), None);
}

#[test]
fn dragging_a_selected_crop_moves_its_rectangle() {
    let rect = CropPdfRect { left: 24.0, bottom: 280.0, right: 84.0, top: 360.0 };
    let mut h = harness(move |app| {
        seed(app, [item("move", 1, rect, "Move me", 10)]);
        app.set_option("panel", "crops").unwrap();
        app.views[0].select_crop("move");
    });

    let from = crop_center(&h, 1, rect);
    let to = from + egui::vec2(24.0, 20.0);
    drag(&mut h, from, to);

    let id = h.state().active_ids().expect("active document").1;
    let doc = h.state().session.get(id).expect("document");
    let moved = doc.crop_review().get("move").expect("moved crop");
    assert!(moved.rect_pdf.left > rect.left, "left should move right");
    assert!(moved.rect_pdf.top < rect.top, "top should move down on screen drag");
    assert_eq!(h.state().views[0].selected_crop(), Some("move"));
}

#[test]
fn dragging_a_corner_handle_resizes_the_selected_crop() {
    let rect = CropPdfRect { left: 24.0, bottom: 280.0, right: 84.0, top: 360.0 };
    let mut h = harness(move |app| {
        seed(app, [item("resize", 1, rect, "Resize me", 10)]);
        app.set_option("panel", "crops").unwrap();
        app.views[0].select_crop("resize");
    });

    let from = crop_corner(&h, 1, rect, true, true);
    let to = from + egui::vec2(28.0, -24.0);
    drag(&mut h, from, to);

    let id = h.state().active_ids().expect("active document").1;
    let doc = h.state().session.get(id).expect("document");
    let resized = doc.crop_review().get("resize").expect("resized crop");
    assert!(resized.rect_pdf.right > rect.right, "right edge should expand");
    assert!(resized.rect_pdf.top > rect.top, "top edge should expand upward");
    assert_eq!(h.state().views[0].selected_crop(), Some("resize"));
}

#[test]
fn snapshot_tool_still_copies_pixels_and_does_not_open_crop_review_flow() {
    let mut h = harness(|app| {
        app.system_clipboard = false;
    });
    assert!(h.state_mut().execute("edit.snapshot"));
    h.state_mut().views[0].marquee_done = Some((0, Rect::from_min_max(pos2(10.0, 10.0), pos2(40.0, 40.0)), [20.0, 40.0, 90.0, 85.0]));
    h.run_steps(4);

    assert!(h.state().last_snapshot.is_some(), "snapshot pixels were copied");
    let id = h.state().active_ids().expect("active document").1;
    let doc = h.state().session.get(id).expect("document");
    assert!(doc.crop_review().items().is_empty(), "snapshot did not create crop-review items");
    assert!(h.state().export_draft.snapshot_package.is_some(), "snapshot kept its export dialog flow");
}

#[test]
fn created_crop_rectangles_are_stored_in_pdf_coordinates_not_screen_space() {
    let mut h = harness(|_app| {});
    h.state_mut().set_option("quick", "crop-review").unwrap();
    h.state_mut().views[0].set_crop_review_creating(true);
    h.state_mut().views[0].marquee_done = Some((0, Rect::from_min_max(pos2(10.0, 10.0), pos2(40.0, 40.0)), [20.0, 40.0, 90.0, 85.0]));
    h.run_steps(4);

    let id = h.state().active_ids().expect("active document").1;
    let doc = h.state().session.get(id).expect("document");
    let item = doc.crop_review().items().first().expect("created crop");
    assert!(item.rect_pdf.left > 0.0);
    assert!(item.rect_pdf.bottom > 0.0);
    assert_ne!(item.rect_pdf.left, 20.0, "stored x is not the screen-space/view-space x directly");
    assert_ne!(item.rect_pdf.bottom, 35.0, "stored y is not the screen-space/view-space y directly");
}

#[test]
fn crop_review_project_save_and_reopen_restores_workspace_state() {
    let dir = temp_dir("project");
    let project = dir.join("review-project");
    let mut h = harness(|app| {
        seed(
            app,
            [
                item("crop-0002", 2, CropPdfRect { left: 30.0, bottom: 250.0, right: 90.0, top: 330.0 }, "Résumé", 20),
                item("crop-0001", 1, CropPdfRect { left: 24.0, bottom: 280.0, right: 84.0, top: 360.0 }, "第二", 10),
            ],
        );
        let id = app.active_ids().expect("active document").1;
        let store = app.session.crop_review_mut(id).expect("crop review store");
        let mut first = store.get("crop-0001").expect("first crop").clone();
        first.note = Some("ação".into());
        store.upsert(first).expect("updated first crop");
        let mut second = store.get("crop-0002").expect("second crop").clone();
        second.note = Some("ocr 日本語".into());
        store.upsert(second).expect("updated second crop");
    });

    h.state_mut().save_crop_review_project(project.clone()).unwrap();
    h.state_mut().open_crop_review_project(&project).unwrap();
    h.run_steps(4);

    let id = h.state().active_ids().expect("active document").1;
    let doc = h.state().session.get(id).expect("document");
    assert_eq!(doc.crop_review().items().len(), 2);
    assert_eq!(doc.ordered_crop_review_items().iter().map(|item| item.id.as_str()).collect::<Vec<_>>(), vec!["crop-0001", "crop-0002"]);
    crop_row(&h, "第二 Snapshot · Page 1");
    crop_row(&h, "Résumé Snapshot · Page 2");

    h.state_mut().set_option("panel", "crops").unwrap();
    h.state_mut().views[0].select_crop("crop-0001");
    h.run_steps(3);
    assert_eq!(field(&h, "Caption").value().as_deref(), Some("第二"));
    assert_eq!(field(&h, "Note").value().as_deref(), Some("ação"));

    let _ = std::fs::remove_dir_all(dir);
}
