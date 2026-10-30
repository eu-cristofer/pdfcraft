use std::sync::{Arc, Mutex};

use chrono::{DateTime, Utc};
use pdfcraft_engine::DocId;
use pdfcraft_engine::export::{CropReviewOcrMode, CropReviewStore};

use crate::PdfCraftApp;

#[derive(Debug, Default)]
pub struct CropReviewOcrProgress {
    pub result: Option<Result<CropReviewOcrResult, String>>,
}

#[derive(Clone, Debug)]
pub struct CropReviewOcrRun {
    pub doc: DocId,
    pub crop_id: String,
    pub generation: u64,
    pub updated_at: DateTime<Utc>,
    pub progress: Arc<Mutex<CropReviewOcrProgress>>,
}

#[derive(Clone, Debug)]
pub struct CropReviewOcrResult {
    pub crop_id: String,
    pub text: Option<String>,
    pub extracted_at: DateTime<Utc>,
}

impl PdfCraftApp {
    pub(crate) fn queue_crop_review_ocr(&mut self, doc: DocId, crop_id: &str) {
        if !pdfcraft_engine::ocr::available() {
            return;
        }
        let Some(document) = self.session.get(doc) else { return };
        let Some(crop) = document.crop_review().get(crop_id) else { return };
        if !crop.ocr_mode.allows_ocr() || crop.ocr_text.as_deref().is_some_and(|text| !text.trim().is_empty()) {
            return;
        }
        let generation = document.edit_generation();
        let updated_at = crop.updated_at;
        if self
            .crop_review_ocr_run
            .as_ref()
            .is_some_and(|run| run.doc == doc && run.crop_id == crop_id && run.generation == generation && run.updated_at == updated_at)
        {
            return;
        }
        let mut items = match document.crop_review_snapshot_export_items() {
            Ok(items) => items,
            Err(_) => return,
        };
        let Some(index) = items.iter().position(|item| item.id == crop_id) else { return };
        for (position, item) in items.iter_mut().enumerate() {
            item.allow_ocr_fill = position == index && item.allow_ocr_fill;
            if position != index {
                item.ocr = Some(pdfcraft_engine::export::SnapshotExportOcr { text: String::new() });
            }
        }
        let source = document.export_source();
        let progress = Arc::new(Mutex::new(CropReviewOcrProgress::default()));
        let out = progress.clone();
        let crop_id = crop_id.to_owned();
        let run_crop_id = crop_id.clone();
        let mut work = move || {
            let result = pdfcraft_engine::guard(|| {
                pdfcraft_engine::export::populate_snapshot_export_ocr(&source, &mut items).map(|()| CropReviewOcrResult {
                    crop_id: run_crop_id.clone(),
                    text: items.get(index).and_then(|item| item.ocr.as_ref().map(|ocr| ocr.text.clone())).filter(|text| !text.trim().is_empty()),
                    extracted_at: Utc::now(),
                })
            })
            .unwrap_or_else(Err);
            if let Ok(mut slot) = out.lock() {
                slot.result = Some(result);
            }
        };
        #[cfg(not(target_arch = "wasm32"))]
        if self.run_inline {
            work();
        } else {
            std::thread::Builder::new().name("pdfcraft-crop-review-ocr".into()).spawn(work).ok();
        }
        #[cfg(target_arch = "wasm32")]
        work();
        self.crop_review_ocr_run = Some(CropReviewOcrRun { doc, crop_id, generation, updated_at, progress });
        self.poll_crop_review_ocr();
    }

    pub(crate) fn invalidate_crop_review_ocr(&mut self, doc: DocId, crop_id: &str) {
        if let Ok(store) = self.session.crop_review_mut(doc)
            && let Some(mut item) = store.get(crop_id).cloned()
            && item.ocr_mode != CropReviewOcrMode::Skip
            && (item.ocr_text.is_some() || item.ocr_extracted_at.is_some())
        {
            item.ocr_text = None;
            item.ocr_extracted_at = None;
            let _ = store.upsert(item);
        }
    }

    pub(crate) fn invalidate_crop_review_ocr_store(store: &mut CropReviewStore) {
        let ids = store
            .items()
            .iter()
            .filter(|item| item.ocr_mode != CropReviewOcrMode::Skip && (item.ocr_text.is_some() || item.ocr_extracted_at.is_some()))
            .map(|item| item.id.clone())
            .collect::<Vec<_>>();
        for id in ids {
            if let Some(mut item) = store.get(&id).cloned() {
                item.ocr_text = None;
                item.ocr_extracted_at = None;
                let _ = store.upsert(item);
            }
        }
    }

    pub(crate) fn poll_crop_review_ocr(&mut self) {
        let Some(run) = self.crop_review_ocr_run.clone() else { return };
        let result = match run.progress.lock() {
            Ok(mut progress) => progress.result.take(),
            Err(error) => {
                self.crop_review_ocr_run = None;
                self.notify_fmt("Crop OCR failed: {e}", &[("e", &error.to_string())]);
                return;
            }
        };
        let Some(result) = result else {
            if let Some(ctx) = &self.ctx {
                ctx.request_repaint_after(std::time::Duration::from_millis(200));
            }
            return;
        };
        self.crop_review_ocr_run = None;
        let Some(document) = self.session.get(run.doc) else { return };
        let Some(current) = document.crop_review().get(&run.crop_id) else { return };
        if document.edit_generation() != run.generation || current.updated_at != run.updated_at {
            return;
        }
        match result {
            Ok(found) => {
                let Ok(store) = self.session.crop_review_mut(run.doc) else { return };
                let Some(mut item) = store.get(&found.crop_id).cloned() else { return };
                if !item.ocr_mode.allows_ocr() {
                    return;
                }
                item.ocr_text = found.text;
                item.ocr_extracted_at = item.ocr_text.as_ref().map(|_| found.extracted_at);
                let _ = store.upsert(item);
            }
            Err(error) => self.notify_fmt("Crop OCR failed: {e}", &[("e", &error)]),
        }
    }
}
