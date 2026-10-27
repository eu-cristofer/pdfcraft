//! Per-document crop-review UI state.

use egui::{Pos2, Rect};
use pdfcraft_engine::Document;

#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct CropReviewView {
    pub selected: Option<String>,
    pub metadata_for: Option<String>,
    pub metadata_caption: String,
    pub metadata_note: String,
    pub creating: bool,
    pub gesture: Option<CropGesture>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct CropGesture {
    pub page: usize,
    pub kind: CropGestureKind,
    pub id: [u64; 2],
    pub from_rect: Rect,
    pub start: Pos2,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum CropGestureKind {
    Move,
    Corner { left: bool, top: bool },
}

impl CropReviewView {
    pub(crate) fn ensure_selected(&mut self, doc: &Document) {
        if self.selected_item(doc).is_some() {
            return;
        }
        if let Some(first) = doc.ordered_crop_review_items().into_iter().next() {
            self.select(first.id.as_str());
        } else {
            self.clear();
        }
    }

    pub fn select(&mut self, id: &str) {
        if self.selected.as_deref() != Some(id) {
            self.selected = Some(id.to_owned());
        }
    }

    pub(crate) fn clear(&mut self) {
        self.selected = None;
        self.metadata_for = None;
        self.metadata_caption.clear();
        self.metadata_note.clear();
        self.creating = false;
        self.gesture = None;
    }

    pub(crate) fn selected_item<'a>(&self, doc: &'a Document) -> Option<&'a pdfcraft_engine::export::CropReviewItem> {
        self.selected.as_deref().and_then(|id| doc.crop_review().get(id))
    }

    pub(crate) fn load_metadata(&mut self, item: &pdfcraft_engine::export::CropReviewItem) {
        self.metadata_for = Some(item.id.clone());
        self.metadata_caption = item.caption.clone();
        self.metadata_note = item.note.clone().unwrap_or_default();
    }
}
