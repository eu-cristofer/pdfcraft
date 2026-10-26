//! Crop review panel in the real shell (egui_kittest).

use chrono::{TimeZone, Utc};
use egui::accesskit::Role;
use egui_kittest::Harness;
use egui_kittest::kittest::Queryable;
use pdfcraft_engine::export::{CropPdfRect, CropReviewItem, ExportItemKind};
use pdfcraft_ui_egui::PdfCraftApp;

use crate::support;

const FIXTURE: &[u8] = b"%PDF-1.7
1 0 obj << /Type /Catalog /Pages 2 0 R >> endobj
2 0 obj << /Type /Pages /Kids [3 0 R 4 0 R] /Count 2 >> endobj
3 0 obj << /Type /Page /Parent 2 0 R /MediaBox [0 0 300 400] >> endobj
4 0 obj << /Type /Page /Parent 2 0 R /MediaBox [0 0 300 400] >> endobj
trailer << /Root 1 0 R >>
%%EOF";

const WIDTH: f32 = 1400.0;

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
