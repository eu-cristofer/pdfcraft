//! The Metadata panel for the currently selected crop-review item.

use pdfcraft_engine::{
    Document,
    export::ExportItemKind,
};

use crate::{
    canvas::DocView,
    theme::{self, Tokens},
    widgets,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct MetadataEdit {
    pub id: String,
    pub caption: String,
    pub note: Option<String>,
}

pub(crate) fn show(ui: &mut egui::Ui, t: &Tokens, doc: &Document, view: &mut DocView) -> Option<MetadataEdit> {
    if doc.crop_review().items().is_empty() {
        crate::panels::empty(ui, t, "crop", "This document has no crops.");
        return None;
    }

    view.crop_review.ensure_selected(doc);

    let Some(item) = view.crop_review.selected_item(doc) else {
        crate::panels::empty(ui, t, "tag", "Select a crop in the Crops panel or on the page.");
        return None;
    };

    if view.crop_review.metadata_for.as_deref() != Some(item.id.as_str()) {
        view.crop_review.load_metadata(item);
    }

    let mut edit = None;
    ui.vertical(|ui| {
        ui.spacing_mut().item_spacing.y = 8.0;

        ui.label(egui::RichText::new(crate::crops_panel::primary_label(item)).font(theme::semibold(15.5)));
        ui.label(egui::RichText::new(crate::crops_panel::secondary_label(item)).font(theme::regular(11.0)).color(t.text_faint));
        ui.add_space(6.0);

        let caption_label = ui.label(tl!("Caption"));
        let caption = ui
            .add(widgets::line(&mut view.crop_review.metadata_caption).desired_width(f32::INFINITY))
            .labelled_by(caption_label.id)
            .accessible_name(tl!("Caption"));
        if caption.changed() {
            edit = Some(pending_edit(view, item.id.as_str()));
        }

        let note_label = ui.label(tl!("Note"));
        let note = ui
            .add(egui::TextEdit::multiline(&mut view.crop_review.metadata_note).desired_width(f32::INFINITY).desired_rows(4))
            .labelled_by(note_label.id)
            .accessible_name(tl!("Note"));
        if note.changed() {
            edit = Some(pending_edit(view, item.id.as_str()));
        }

        ui.add_space(6.0);
        ui.label(egui::RichText::new(crate::i18n::fmt(tl!("Kind: {kind}"), &[("kind", kind_label(item.kind))])).font(theme::regular(11.0)).color(t.text_faint));
        ui.label(egui::RichText::new(crate::i18n::fmt(tl!("Page: {page}"), &[("page", &item.page_number.to_string())])).font(theme::regular(11.0)).color(t.text_faint));
    });

    edit
}

fn pending_edit(view: &DocView, id: &str) -> MetadataEdit {
    MetadataEdit {
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
