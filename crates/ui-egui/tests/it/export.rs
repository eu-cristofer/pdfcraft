//! Export a PDF ▸ Image and Text from the real shell (egui_kittest).

use egui::accesskit::Role;
use egui_kittest::Harness;
use egui_kittest::kittest::{NodeT, Queryable};
use pdfcraft_ui_egui::PdfCraftApp;

const FIXTURE: &[u8] = b"%PDF-1.7
1 0 obj << /Type /Catalog /Pages 2 0 R >> endobj
2 0 obj << /Type /Pages /Kids [3 0 R 4 0 R] /Count 2 /MediaBox [0 0 200 100] >> endobj
3 0 obj << /Type /Page /Parent 2 0 R >> endobj
4 0 obj << /Type /Page /Parent 2 0 R >> endobj
trailer << /Root 1 0 R >>
%%EOF";

fn dir(name: &str) -> std::path::PathBuf {
    static NEXT: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
    let n = NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    let dir = std::env::temp_dir().join(format!("pdfcraft-export-ui-{name}-{}-{n}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn harness(dir: std::path::PathBuf) -> Harness<'static, PdfCraftApp> {
    let d = dir.clone();
    let mut h = Harness::builder().with_size(egui::vec2(1400.0, 900.0)).build_eframe(move |_cc| {
        let mut app = PdfCraftApp::new();
        app.set_option("language", "en").unwrap();
        app.open_bytes("doc.pdf", None, FIXTURE.to_vec()).unwrap();
        app.export_dir_override = Some(d.to_string_lossy().into_owned());
        app.system_clipboard = false;
        app
    });
    h.run_steps(6);
    h
}

fn at(h: &Harness<'static, PdfCraftApp>, x: f32, y: f32) -> egui::Pos2 {
    let r = h.state().views[0].page_screen_rect(0).expect("page 1 on screen");
    egui::pos2(r.left() + x / 200.0 * r.width(), r.top() + (100.0 - y) / 100.0 * r.height())
}

fn drag(h: &mut Harness<'static, PdfCraftApp>, from: egui::Pos2, to: egui::Pos2) {
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

#[test]
fn export_dialogs_write_images_and_text() {
    let dir = dir("basic");
    let mut h = harness(dir.clone());
    assert!(h.state_mut().execute("export.image"));
    h.run_steps(2);
    h.get_by_label("Export to Image");
    h.get_by_label("Export").click();
    h.run_steps(4);
    assert!(dir.join("doc_page_1.png").exists() && dir.join("doc_page_2.png").exists());
    h.get_by_label_contains("Exported 2 images");
    assert!(h.state_mut().execute("export.text"));
    h.run_steps(2);
    h.get_by_label("Export").click();
    h.run_steps(4);
    assert!(dir.join("doc.txt").exists());
    // Export all images: this document has none, and says so.
    assert!(h.state_mut().execute("export.all_images"));
    h.run_steps(2);
    h.get_by_label("Export All Images");
    h.get_by_label("Export").click();
    h.run_steps(4);
    h.get_by_label_contains("Exported 0 images");
}

#[test]
fn snapshot_marquee_opens_package_export_and_writes_engine_package() {
    let dir = dir("snapshot");
    let mut h = harness(dir.clone());

    assert!(h.state_mut().execute("edit.snapshot"));
    h.run_steps(1);
    let from = at(&h, 20.0, 80.0);
    let to = at(&h, 90.0, 35.0);
    drag(&mut h, from, to);

    h.get_by_label("Export Snapshot Package");
    assert!(h.get_by_label("Export").accesskit_node().is_disabled(), "caption is required");

    h.get_by_role_and_label(Role::TextInput, "Caption").click();
    h.get_by_role_and_label(Role::TextInput, "Caption").type_text("Desk rectangle");
    h.run_steps(1);
    h.get_by_role_and_label(Role::TextInput, "Note").click();
    h.get_by_role_and_label(Role::TextInput, "Note").type_text("Typed from the desktop flow.");
    h.run_steps(1);

    h.get_by_label("Export").click();
    h.run_steps(6);

    if !pdfcraft_engine::ocr::available() {
        assert!(!dir.join("doc_snapshot_package").exists());
        return;
    }

    let package = dir.join("doc_snapshot_package");
    assert!(package.is_dir());
    assert!(package.join("manifest.json").is_file());
    assert!(package.join("report.tex").is_file());
    assert!(package.join("source.pdf").is_file());
    assert!(package.join("evidence/item_1.png").is_file());

    let manifest = std::fs::read_to_string(package.join("manifest.json")).unwrap();
    assert!(manifest.contains("\"caption\": \"Desk rectangle\""));
    assert!(manifest.contains("\"note\": \"Typed from the desktop flow.\""));

    let report = std::fs::read_to_string(package.join("report.tex")).unwrap();
    assert!(report.contains("\\label{fig:item_1}"));
    assert!(report.contains("Desk rectangle"));
    if pdfcraft_engine::ocr::available() {
        assert!(report.contains("\\paragraph{OCR.}"), "{report}");
    }
}
