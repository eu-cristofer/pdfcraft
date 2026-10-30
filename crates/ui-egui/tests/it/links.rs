//! Help links: the Help button is one click away everywhere; Help menu, About dialog and home
//! screen open the issue tracker, this app's repository and the upstream attribution link.

use egui::accesskit::Role;
use egui_kittest::Harness;
use egui_kittest::kittest::Queryable;
use pdfcraft_engine::links;
use pdfcraft_ui_egui::{Dialog, PdfCraftApp};

fn harness(setup: impl FnOnce(&mut PdfCraftApp) + 'static) -> Harness<'static, PdfCraftApp> {
    let mut h = Harness::builder().with_size(egui::vec2(1400.0, 900.0)).build_eframe(move |_cc| {
        let mut app = PdfCraftApp::new();
        app.set_option("language", "en").unwrap();
        setup(&mut app);
        app
    });
    h.run_steps(4);
    h
}

#[test]
fn help_button_in_the_top_bar_opens_the_issue_tracker() {
    let mut h = harness(|_| {});
    h.get_by_role_and_label(Role::Button, "Help").click();
    h.run_steps(2);
    assert_eq!(h.state().last_opened_url.as_deref(), Some(links::HELP));
    assert!(links::HELP.contains("eu-cristofer/pdfcraft/issues"));
}

#[test]
fn home_screen_links() {
    for l in links::LINKS {
        let mut h = harness(|_| {});
        h.get_by_label("Help and feedback");
        h.get_by_label(l.label).click();
        h.run_steps(2);
        assert_eq!(h.state().last_opened_url.as_deref(), Some(l.url), "{}", l.label);
    }
}

#[test]
fn home_community_card_closes_for_this_run() {
    let mut h = harness(|_| {});
    h.get_by_label("Help and feedback");
    h.get_by_label("Close").click();
    h.run_steps(2);
    assert!(h.state().community_card_closed);
    assert!(h.query_by_label("Help and feedback").is_none());
    assert!(h.query_by_label("Get help or report a problem").is_none());
    // Not remembered: a fresh app shows the card again.
    assert!(!PdfCraftApp::new().community_card_closed);
}

#[test]
fn about_dialog_shows_the_brand_and_links() {
    let pdf = b"%PDF-1.7\n1 0 obj << /Type /Catalog /Pages 2 0 R >> endobj\n2 0 obj << /Type /Pages /Kids [3 0 R] /Count 1 >> endobj\n3 0 obj << /Type /Page /Parent 2 0 R /MediaBox [0 0 200 200] >> endobj\ntrailer << /Root 1 0 R >>\n%%EOF";
    // With a document open, so the home screen's own links are not on screen.
    let mut h = harness(move |app| {
        app.open_bytes("one.pdf", None, pdf.to_vec()).unwrap();
        app.dialog = Some(Dialog::About);
    });
    assert!(h.query_all_by_label(pdfcraft_ui_egui::branding::APP_NAME).count() >= 1, "the suite mark (alt text) or name");
    assert!(h.query_by_label(pdfcraft_ui_egui::branding::BASED_ON).is_some());
    assert!(h.query_by_label("ArtCraft").is_none(), "no ArtCraft mark in About");
    h.get_by_label("Get help or report a problem").click();
    h.run_steps(2);
    assert_eq!(h.state().last_opened_url.as_deref(), Some(links::HELP));
}

#[test]
fn help_commands_open_each_link() {
    for l in links::LINKS {
        let mut h = harness(|_| {});
        assert!(h.state_mut().execute(l.command), "{}", l.command);
        assert_eq!(h.state().last_opened_url.as_deref(), Some(l.url));
        assert_eq!(pdfcraft_engine::commands::command(l.command).unwrap().menu, Some("Help"));
    }
}

#[test]
fn no_link_points_at_artcraft_or_discord() {
    for l in links::LINKS {
        let url = l.url.to_lowercase();
        assert!(!url.contains("artcraft") && !url.contains("discord"), "{}", l.url);
    }
}
