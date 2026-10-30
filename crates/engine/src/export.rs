//! Export a PDF ▸ Image and Text (execution plan M10.3, the first formats).
//!
//! [`Exporter`] renders pages to PNG, JPEG or TIFF at a resolution and extracts reading-order
//! text, from a
//! document's working file (so unsaved edits and hidden layers are respected, as on screen).

use std::cmp::Ordering;
use std::path::{Component, Path, PathBuf};
use std::sync::Arc;

use chrono::{DateTime, Utc};
use pdfcraft_render::{PageRenderer, RenderRequest, RequestKind, Tile};
use serde::{Deserialize, Serialize};

use crate::js::words_with;
use crate::{Document, comment_kind};

/// Renders and extracts pages of one document state.
pub struct Exporter {
    renderer: PageRenderer,
    pages: usize,
    /// Each page's displayed size in points, for [`Exporter::dpi_used`].
    sizes: Vec<(f32, f32)>,
}

/// What an export needs from a document, as plain values that can move to a worker thread.
#[derive(Clone)]
pub struct ExportSource {
    pub bytes: std::sync::Arc<Vec<u8>>,
    pub config: pdfcraft_render::RenderConfig,
    pub pages: usize,
    /// Each page's displayed size in points (after `/Rotate` and `/UserUnit`).
    pub sizes: Vec<(f32, f32)>,
}

/// A snapshot rectangle in the export contract's displayed-page coordinate space.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ExportRect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl ExportRect {
    fn validate(self) -> Result<Self, String> {
        if !self.x.is_finite() || !self.y.is_finite() {
            return Err("snapshot rectangle coordinates must be finite".into());
        }
        if self.x < 0.0 || self.y < 0.0 {
            return Err("snapshot rectangle coordinates must be non-negative".into());
        }
        if !self.width.is_finite() || !self.height.is_finite() || self.width <= 0.0 || self.height <= 0.0 {
            return Err("snapshot rectangle size must be finite and positive".into());
        }
        Ok(self)
    }
}

/// Explicit user-typed metadata for one rectangle-driven snapshot export item.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SnapshotExportMetadata {
    pub caption: String,
    pub note: Option<String>,
}

/// OCR derived from the same selected portion as the snapshot rectangle.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SnapshotExportOcr {
    pub text: String,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CropReviewOcrMode {
    #[default]
    Auto,
    Include,
    Skip,
}

impl CropReviewOcrMode {
    pub fn allows_ocr(self) -> bool {
        !matches!(self, Self::Skip)
    }
}

/// Raw rectangle-driven snapshot export data before canonical ordering, IDs and image paths are assigned.
#[derive(Clone, Debug, PartialEq)]
pub struct SnapshotExportItemInput {
    pub order: usize,
    /// 1-based page number in displayed-page space.
    pub page: usize,
    pub rect: ExportRect,
    pub metadata: SnapshotExportMetadata,
    /// OCR for this selected portion, when a later caller has already derived it.
    pub ocr: Option<SnapshotExportOcr>,
    /// Whether missing OCR may be derived later for this item.
    pub allow_ocr_fill: bool,
}

/// The export package item kind. v1 only emits snapshots, but later package artifacts consume
/// this shared ordered item model directly.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ExportItemKind {
    Snapshot,
}

impl ExportItemKind {
    pub fn as_str(self) -> &'static str {
        match self {
            ExportItemKind::Snapshot => "snapshot",
        }
    }
}

/// One crop rectangle persisted in PDF/document coordinate space.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CropPdfRect {
    pub left: f32,
    pub bottom: f32,
    pub right: f32,
    pub top: f32,
}

impl CropPdfRect {
    pub fn validate(self) -> Result<Self, String> {
        if !self.left.is_finite() || !self.bottom.is_finite() || !self.right.is_finite() || !self.top.is_finite() {
            return Err("crop rectangle coordinates must be finite".into());
        }
        if self.right <= self.left || self.top <= self.bottom {
            return Err("crop rectangle must have positive width and height".into());
        }
        Ok(self)
    }

    pub fn from_user_rect(rect: [f32; 4]) -> Result<Self, String> {
        CropPdfRect { left: rect[0].min(rect[2]), bottom: rect[1].min(rect[3]), right: rect[0].max(rect[2]), top: rect[1].max(rect[3]) }.validate()
    }

    pub fn user_rect(self) -> [f32; 4] {
        [self.left, self.bottom, self.right, self.top]
    }

    pub fn to_export_rect(self, page: &pdfcraft_render::PageInfo) -> Result<ExportRect, String> {
        let rect = self.validate()?;
        let corners = [
            page.user_to_view(rect.left, rect.bottom),
            page.user_to_view(rect.left, rect.top),
            page.user_to_view(rect.right, rect.bottom),
            page.user_to_view(rect.right, rect.top),
        ];
        let mut left = f32::INFINITY;
        let mut top = f32::INFINITY;
        let mut right = f32::NEG_INFINITY;
        let mut bottom = f32::NEG_INFINITY;
        for [x, y] in corners {
            left = left.min(x);
            top = top.min(y);
            right = right.max(x);
            bottom = bottom.max(y);
        }
        ExportRect { x: left, y: top, width: right - left, height: bottom - top }.validate()
    }
}

/// One persistent crop-review item owned by the engine/document rather than the UI.
#[derive(Clone, Debug, PartialEq)]
pub struct CropReviewItem {
    pub id: String,
    pub kind: ExportItemKind,
    pub page_number: usize,
    pub rect_pdf: CropPdfRect,
    pub caption: String,
    pub note: Option<String>,
    pub ocr_mode: CropReviewOcrMode,
    pub ocr_text: Option<String>,
    pub ocr_extracted_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl CropReviewItem {
    pub fn validate(&self) -> Result<(), String> {
        if self.id.trim().is_empty() {
            return Err("crop id must not be empty".into());
        }
        if self.page_number == 0 {
            return Err("crop page number must be 1-based".into());
        }
        self.rect_pdf.validate()?;
        if self.ocr_extracted_at.is_some() && self.ocr_text.is_none() {
            return Err("crop OCR timestamp requires OCR text".into());
        }
        if self.updated_at < self.created_at {
            return Err("crop updated_at must not be earlier than created_at".into());
        }
        Ok(())
    }

    pub fn metadata(&self) -> SnapshotExportMetadata {
        SnapshotExportMetadata { caption: self.caption.clone(), note: self.note.clone() }
    }

    pub fn ocr(&self) -> Option<SnapshotExportOcr> {
        self.ocr_text.as_ref().map(|text| SnapshotExportOcr { text: text.clone() })
    }

    pub fn export_rect(&self, page: &pdfcraft_render::PageInfo) -> Result<ExportRect, String> {
        self.rect_pdf.to_export_rect(page)
    }

    pub fn export_item_input(&self, order: usize, page: &pdfcraft_render::PageInfo) -> Result<SnapshotExportItemInput, String> {
        self.validate()?;
        Ok(SnapshotExportItemInput {
            order,
            page: self.page_number,
            rect: self.export_rect(page)?,
            metadata: self.metadata(),
            ocr: self.ocr(),
            allow_ocr_fill: self.ocr_mode.allows_ocr(),
        })
    }
}

impl Document {
    pub fn crop_review_snapshot_export_items(&self) -> Result<Vec<SnapshotExportItem>, String> {
        let mut items = Vec::new();
        for (order, crop) in self.crop_review.ordered_items(&self.info.pages).into_iter().enumerate() {
            let page_index =
                crop.page_number.checked_sub(1).ok_or_else(|| format!("crop {} has invalid page number {}", crop.id, crop.page_number))?;
            let page = self.info.pages.get(page_index).ok_or_else(|| format!("crop {} refers to missing page {}", crop.id, crop.page_number))?;
            items.push(crop.export_item_input(order, page)?);
        }
        normalize_snapshot_export_items(self.info.pages.len(), items)
    }

    pub fn crop_review_snapshot_export_package_input(&self) -> Result<SnapshotExportPackageInput, String> {
        Ok(SnapshotExportPackageInput {
            items: self.crop_review_snapshot_export_items()?,
            render_source: self.export_source(),
            source_pdf_bytes: self.bytes.clone(),
            title: self.info.title.clone().unwrap_or_else(|| self.name.trim_end_matches(".pdf").to_string()),
            include_word_report: false,
        })
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct CropReviewStore {
    items: Vec<CropReviewItem>,
}

impl CropReviewStore {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn items(&self) -> &[CropReviewItem] {
        &self.items
    }

    pub fn get(&self, id: &str) -> Option<&CropReviewItem> {
        self.items.iter().find(|item| item.id == id)
    }

    pub fn upsert(&mut self, item: CropReviewItem) -> Result<Option<CropReviewItem>, String> {
        item.validate()?;
        if let Some(existing) = self.items.iter_mut().find(|existing| existing.id == item.id) {
            return Ok(Some(std::mem::replace(existing, item)));
        }
        self.items.push(item);
        Ok(None)
    }

    pub fn remove(&mut self, id: &str) -> Option<CropReviewItem> {
        let index = self.items.iter().position(|item| item.id == id)?;
        Some(self.items.remove(index))
    }

    pub fn ordered_items<'a>(&'a self, pages: &[pdfcraft_render::PageInfo]) -> Vec<&'a CropReviewItem> {
        let mut items = self.items.iter().collect::<Vec<_>>();
        items.sort_by(|left, right| compare_crop_review_items(left, right, pages));
        items
    }

    pub fn ordered_ids<'a>(&'a self, pages: &[pdfcraft_render::PageInfo]) -> Vec<&'a str> {
        self.ordered_items(pages).into_iter().map(|item| item.id.as_str()).collect()
    }
}

const CROP_REVIEW_PROJECT_SCHEMA_VERSION: u32 = 1;
const CROP_REVIEW_PROJECT_JSON_PATH: &str = "project.json";
const CROP_REVIEW_PROJECT_SOURCE_PDF_PATH: &str = "source.pdf";

#[derive(Clone, Debug, PartialEq)]
pub struct CropReviewProject {
    pub source_pdf_bytes: Arc<Vec<u8>>,
    pub crops: Vec<CropReviewItem>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
struct CropReviewProjectJson {
    schema_version: u32,
    source_pdf: String,
    crops: Vec<CropReviewProjectCropJson>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
struct CropReviewProjectCropJson {
    id: String,
    kind: String,
    page_number: usize,
    rect_pdf: CropReviewProjectRectJson,
    caption: String,
    note: Option<String>,
    #[serde(default)]
    ocr_mode: CropReviewOcrMode,
    ocr_text: Option<String>,
    ocr_extracted_at: Option<DateTime<Utc>>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
struct CropReviewProjectRectJson {
    left: f32,
    bottom: f32,
    right: f32,
    top: f32,
}

impl CropReviewProjectJson {
    fn from_project(project: &CropReviewProject) -> Self {
        Self {
            schema_version: CROP_REVIEW_PROJECT_SCHEMA_VERSION,
            source_pdf: CROP_REVIEW_PROJECT_SOURCE_PDF_PATH.to_owned(),
            crops: project.crops.iter().map(CropReviewProjectCropJson::from_item).collect(),
        }
    }

    fn validate(&self) -> Result<(), String> {
        if self.schema_version != CROP_REVIEW_PROJECT_SCHEMA_VERSION {
            return Err(format!("crop review project schema version {} is not supported", self.schema_version));
        }
        if self.source_pdf != CROP_REVIEW_PROJECT_SOURCE_PDF_PATH {
            return Err(format!("crop review project source_pdf must be {}, got {}", CROP_REVIEW_PROJECT_SOURCE_PDF_PATH, self.source_pdf));
        }
        for crop in &self.crops {
            crop.validate()?;
        }
        Ok(())
    }
}

pub const EMBEDDED_CROP_REVIEW_ATTACHMENT_NAME: &str = "pdfcraft-crop-review.json";

pub fn crop_review_store_json(crops: &[CropReviewItem]) -> Result<String, String> {
    let project = CropReviewProject { source_pdf_bytes: Arc::new(Vec::new()), crops: crops.to_vec() };
    let json = CropReviewProjectJson {
        schema_version: CROP_REVIEW_PROJECT_SCHEMA_VERSION,
        source_pdf: String::new(),
        crops: project.crops.iter().map(CropReviewProjectCropJson::from_item).collect(),
    };
    serde_json::to_string_pretty(&json)
        .map(|mut out| {
            out.push('\n');
            out
        })
        .map_err(|e| format!("failed to serialize crop review JSON: {e}"))
}

pub fn crop_review_store_from_json(json_text: &str) -> Result<CropReviewStore, String> {
    let json: CropReviewProjectJson = serde_json::from_str(json_text).map_err(|e| format!("crop review JSON is not valid: {e}"))?;
    if json.schema_version != CROP_REVIEW_PROJECT_SCHEMA_VERSION {
        return Err(format!("crop review schema version {} is not supported", json.schema_version));
    }
    let mut store = CropReviewStore::new();
    for crop in json.crops {
        let item = crop.into_item()?;
        store.upsert(item)?;
    }
    Ok(store)
}

impl CropReviewProjectCropJson {
    fn from_item(item: &CropReviewItem) -> Self {
        Self {
            id: item.id.clone(),
            kind: item.kind.as_str().to_owned(),
            page_number: item.page_number,
            rect_pdf: CropReviewProjectRectJson {
                left: item.rect_pdf.left,
                bottom: item.rect_pdf.bottom,
                right: item.rect_pdf.right,
                top: item.rect_pdf.top,
            },
            caption: item.caption.clone(),
            note: item.note.clone(),
            ocr_mode: item.ocr_mode,
            ocr_text: item.ocr_text.clone(),
            ocr_extracted_at: item.ocr_extracted_at,
            created_at: item.created_at,
            updated_at: item.updated_at,
        }
    }

    fn into_item(self) -> Result<CropReviewItem, String> {
        let kind = match self.kind.as_str() {
            "snapshot" => ExportItemKind::Snapshot,
            other => return Err(format!("crop kind {other} is not supported")),
        };
        let item = CropReviewItem {
            id: self.id,
            kind,
            page_number: self.page_number,
            rect_pdf: CropPdfRect { left: self.rect_pdf.left, bottom: self.rect_pdf.bottom, right: self.rect_pdf.right, top: self.rect_pdf.top },
            caption: self.caption,
            note: self.note,
            ocr_mode: self.ocr_mode,
            ocr_text: self.ocr_text,
            ocr_extracted_at: self.ocr_extracted_at,
            created_at: self.created_at,
            updated_at: self.updated_at,
        };
        item.validate()?;
        Ok(item)
    }

    fn validate(&self) -> Result<(), String> {
        let kind = match self.kind.as_str() {
            "snapshot" => ExportItemKind::Snapshot,
            other => return Err(format!("crop kind {other} is not supported")),
        };
        CropReviewItem {
            id: self.id.clone(),
            kind,
            page_number: self.page_number,
            rect_pdf: CropPdfRect { left: self.rect_pdf.left, bottom: self.rect_pdf.bottom, right: self.rect_pdf.right, top: self.rect_pdf.top },
            caption: self.caption.clone(),
            note: self.note.clone(),
            ocr_mode: self.ocr_mode,
            ocr_text: self.ocr_text.clone(),
            ocr_extracted_at: self.ocr_extracted_at,
            created_at: self.created_at,
            updated_at: self.updated_at,
        }
        .validate()
    }
}

fn compare_crop_review_items(left: &CropReviewItem, right: &CropReviewItem, pages: &[pdfcraft_render::PageInfo]) -> Ordering {
    left.page_number
        .cmp(&right.page_number)
        .then_with(|| compare_f32(crop_review_sort_key(left, pages).0, crop_review_sort_key(right, pages).0))
        .then_with(|| compare_f32(crop_review_sort_key(left, pages).1, crop_review_sort_key(right, pages).1))
        .then_with(|| left.id.cmp(&right.id))
}

fn crop_review_sort_key(item: &CropReviewItem, pages: &[pdfcraft_render::PageInfo]) -> (f32, f32) {
    let Some(page) = item.page_number.checked_sub(1).and_then(|index| pages.get(index)) else {
        return (-item.rect_pdf.top, item.rect_pdf.left);
    };
    match item.rect_pdf.to_export_rect(page) {
        Ok(rect) => (rect.y, rect.x),
        Err(_) => (-item.rect_pdf.top, item.rect_pdf.left),
    }
}

fn compare_f32(left: f32, right: f32) -> Ordering {
    left.partial_cmp(&right).unwrap_or(Ordering::Equal)
}

/// One normalized item in the canonical export order.
#[derive(Clone, Debug, PartialEq)]
pub struct SnapshotExportItem {
    pub id: String,
    pub kind: ExportItemKind,
    /// 1-based page number in displayed-page space.
    pub page: usize,
    pub rect: ExportRect,
    pub image_path: String,
    pub metadata: SnapshotExportMetadata,
    /// OCR for this selected portion, kept out of `manifest.json` in v1.
    pub ocr: Option<SnapshotExportOcr>,
    /// Whether missing OCR may be derived later for this item.
    pub allow_ocr_fill: bool,
}

/// End-to-end inputs for writing one snapshot export package.
#[derive(Clone)]
pub struct SnapshotExportPackageInput {
    pub items: Vec<SnapshotExportItem>,
    /// Display-equivalent bytes/config for crop rendering.
    pub render_source: ExportSource,
    /// Working-file bytes for `source.pdf`.
    pub source_pdf_bytes: Arc<Vec<u8>>,
    /// Title used by derived convenience artifacts such as `report.docx`.
    pub title: String,
    /// Whether the package should also include a derived Word report.
    pub include_word_report: bool,
}

/// A selected comment in the engine-facing 1-based page/index shape shared with other comment
/// addressing entry points.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct SelectedCommentTarget {
    /// 1-based page number.
    pub page: usize,
    /// 1-based annotation index within the page's `/Annots` array.
    pub index: usize,
}

impl Document {
    /// What an export reads, for exporting on another thread. These are the displayed
    /// bytes, so they may carry missing comment appearances PdfCraft drew. Signature
    /// artwork stays original; viewer validation badges are not document content and
    /// do not enter image/text exports or Print as Image. Saving uses the working file.
    pub fn export_source(&self) -> ExportSource {
        ExportSource {
            bytes: self.display.clone(),
            config: self.config.clone(),
            pages: self.info.pages.len(),
            sizes: self.info.pages.iter().map(|p| (p.width, p.height)).collect(),
        }
    }
}

fn zero_based_selected_comment_target(target: SelectedCommentTarget) -> Result<(usize, usize), String> {
    if target.page == 0 || target.index == 0 {
        return Err("selected comment targets use 1-based page and index".into());
    }
    Ok((target.page - 1, target.index - 1))
}

fn resolve_selected_comment_annotation_in<'a>(
    pages: &[pdfcraft_render::PageInfo],
    annotations: &'a [pdfcraft_render::Annotation],
    target: SelectedCommentTarget,
) -> Result<&'a pdfcraft_render::Annotation, String> {
    let (page, index) = zero_based_selected_comment_target(target)?;
    if page >= pages.len() {
        return Err(format!("page {} does not exist", target.page));
    }
    annotations
        .iter()
        .find(|annotation| annotation.page == page && annotation.index == index)
        .ok_or_else(|| format!("page {} has no comment {}", target.page, target.index))
}

/// Resolves one selected 1-based page/index target to the current inspected annotation.
pub fn resolve_selected_comment_annotation(doc: &Document, target: SelectedCommentTarget) -> Result<&pdfcraft_render::Annotation, String> {
    resolve_selected_comment_annotation_in(&doc.info.pages, &doc.info.annotations, target)
}

fn selected_comment_note(contents: Option<&str>) -> Option<String> {
    contents.map(|text| text.trim_matches('\0').trim().to_string()).filter(|text| !text.is_empty())
}

fn selected_comment_caption(annotation: &pdfcraft_render::Annotation) -> Result<String, String> {
    let kind = comment_kind(annotation);
    match kind {
        "Rectangle" | "Callout" | "Text Box" => Ok(kind.to_owned()),
        _ => Err(format!("page {} comment {} has unsupported selected-rectangle kind {}", annotation.page + 1, annotation.index + 1, kind)),
    }
}

fn selected_comment_metadata(annotation: &pdfcraft_render::Annotation) -> Result<SnapshotExportMetadata, String> {
    Ok(SnapshotExportMetadata { caption: selected_comment_caption(annotation)?, note: selected_comment_note(annotation.contents.as_deref()) })
}

fn selected_comment_export_rect(page: &pdfcraft_render::PageInfo, annotation: &pdfcraft_render::Annotation) -> Result<ExportRect, String> {
    let shown = annotation.shown_rect(page.rotation);
    let x0 = shown[0].min(shown[2]);
    let y0 = shown[1].min(shown[3]);
    let x1 = shown[0].max(shown[2]);
    let y1 = shown[1].max(shown[3]);
    let corners = [[x0, y0], [x0, y1], [x1, y0], [x1, y1]];

    let mut left = f32::INFINITY;
    let mut top = f32::INFINITY;
    let mut right = f32::NEG_INFINITY;
    let mut bottom = f32::NEG_INFINITY;

    for [x, y] in corners {
        let [view_x, view_y] = page.user_to_view(x, y);
        left = left.min(view_x);
        top = top.min(view_y);
        right = right.max(view_x);
        bottom = bottom.max(view_y);
    }

    ExportRect { x: left, y: top, width: right - left, height: bottom - top }.validate()
}

fn selected_comment_snapshot_export_item_input_from_parts(
    page: &pdfcraft_render::PageInfo,
    annotation: &pdfcraft_render::Annotation,
    order: usize,
) -> Result<SnapshotExportItemInput, String> {
    Ok(SnapshotExportItemInput {
        order,
        page: annotation.page + 1,
        rect: selected_comment_export_rect(page, annotation)?,
        metadata: selected_comment_metadata(annotation)?,
        ocr: None,
        allow_ocr_fill: true,
    })
}

/// Maps one selected comment to one raw snapshot export item input.
pub fn selected_comment_snapshot_export_item_input(doc: &Document, target: SelectedCommentTarget) -> Result<SnapshotExportItemInput, String> {
    let annotation = resolve_selected_comment_annotation(doc, target)?;
    let page = doc.info.pages.get(annotation.page).ok_or_else(|| format!("page {} does not exist", annotation.page + 1))?;
    selected_comment_snapshot_export_item_input_from_parts(page, annotation, 0)
}

fn canonical_selected_comment_targets(targets: &[SelectedCommentTarget]) -> Vec<SelectedCommentTarget> {
    let mut ordered = targets.iter().copied().enumerate().collect::<Vec<_>>();
    ordered.sort_by_key(|(original_index, target)| (target.page, target.index, *original_index));
    ordered.into_iter().map(|(_, target)| target).collect()
}

fn selected_comment_snapshot_export_item_inputs_from_parts(
    pages: &[pdfcraft_render::PageInfo],
    annotations: &[pdfcraft_render::Annotation],
    targets: &[SelectedCommentTarget],
) -> Result<Vec<SnapshotExportItemInput>, String> {
    let ordered_targets = canonical_selected_comment_targets(targets);
    let mut items = Vec::with_capacity(ordered_targets.len());
    for (order, target) in ordered_targets.into_iter().enumerate() {
        let annotation = resolve_selected_comment_annotation_in(pages, annotations, target)?;
        let page = pages.get(annotation.page).ok_or_else(|| format!("page {} does not exist", annotation.page + 1))?;
        items.push(selected_comment_snapshot_export_item_input_from_parts(page, annotation, order)?);
    }
    Ok(items)
}

/// Maps selected comments to canonical raw snapshot export inputs in document order.
pub fn selected_comment_snapshot_export_item_inputs(
    doc: &Document,
    targets: &[SelectedCommentTarget],
) -> Result<Vec<SnapshotExportItemInput>, String> {
    selected_comment_snapshot_export_item_inputs_from_parts(&doc.info.pages, &doc.info.annotations, targets)
}

/// Maps selected comments to normalized snapshot export items with canonical IDs and image paths.
pub fn selected_comment_snapshot_export_items(doc: &Document, targets: &[SelectedCommentTarget]) -> Result<Vec<SnapshotExportItem>, String> {
    normalize_snapshot_export_items(doc.info.pages.len(), selected_comment_snapshot_export_item_inputs(doc, targets)?)
}

/// Export all images: the images `pages` (0-based) use, each once, skipping those under
/// `min_side` pixels on their shorter side. JPEGs come out unchanged, other images as PNG.
pub fn extract_images(src: &ExportSource, pages: &[usize], min_side: u32) -> Result<pdfcraft_create::ImageExport, String> {
    let doc = pdfcraft_cos::Document::open_with_password(src.bytes.clone(), src.config.password.as_deref()).map_err(|e| e.to_string())?;
    Ok(pdfcraft_create::extract_images(&doc, pages, min_side))
}

/// The file name for the `index`-th (1-based) exported image: `<stem>_Page_<n>_Image_<index>.<ext>`.
/// The page number has leading zeros for as many digits as the document's `page_count` has, so the
/// files sort in page order (#2208): page 7 of 120 is `Page_007`.
pub fn image_file_name(stem: &str, image: &pdfcraft_create::ExtractedImage, index: usize, page_count: usize) -> String {
    let width = page_count.max(1).to_string().len();
    format!("{stem}_Page_{:0width$}_Image_{index:04}.{}", image.page + 1, image.extension)
}

fn snapshot_export_item_id(index: usize) -> Result<String, String> {
    let position = index.checked_add(1).ok_or_else(|| "too many export items".to_string())?;
    Ok(format!("item_{position}"))
}

fn snapshot_export_image_path(id: &str) -> Result<String, String> {
    normalize_package_relative_path(&format!("evidence/{id}.png"))
}

fn validate_snapshot_export_page(page_count: usize, page: usize) -> Result<(), String> {
    if page == 0 || page > page_count { Err(format!("page {page} does not exist")) } else { Ok(()) }
}

/// Normalizes one package-internal relative path to a canonical `/`-separated form.
pub fn normalize_package_relative_path(path: &str) -> Result<String, String> {
    if path.is_empty() {
        return Err("package path is empty".into());
    }
    if path.contains('\\') {
        return Err("package path must use '/' separators".into());
    }
    if Path::new(path).is_absolute() {
        return Err("package path must be relative".into());
    }

    let mut parts = Vec::new();
    for component in Path::new(path).components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => return Err("package path cannot escape the package root".into()),
            Component::RootDir | Component::Prefix(_) => return Err("package path must be relative".into()),
            Component::Normal(part) => parts.push(part.to_str().ok_or_else(|| "package path must be valid UTF-8".to_string())?),
        }
    }

    if parts.is_empty() {
        return Err("package path is empty".into());
    }

    Ok(parts.join("/"))
}

/// Builds the canonical ordered list that later manifest and LaTeX generation consume.
pub fn normalize_snapshot_export_items(page_count: usize, items: Vec<SnapshotExportItemInput>) -> Result<Vec<SnapshotExportItem>, String> {
    let mut ordered = items.into_iter().enumerate().collect::<Vec<_>>();
    ordered.sort_by_key(|(original_index, item)| (item.order, *original_index));

    let mut normalized = Vec::with_capacity(ordered.len());
    for (index, (_, item)) in ordered.into_iter().enumerate() {
        validate_snapshot_export_page(page_count, item.page)?;
        let rect = item.rect.validate()?;
        let id = snapshot_export_item_id(index)?;
        let image_path = snapshot_export_image_path(&id)?;
        normalized.push(SnapshotExportItem {
            id,
            kind: ExportItemKind::Snapshot,
            page: item.page,
            rect,
            image_path,
            metadata: item.metadata,
            ocr: item.ocr,
            allow_ocr_fill: item.allow_ocr_fill,
        });
    }
    Ok(normalized)
}

const SNAPSHOT_EXPORT_PACKAGE_KIND: &str = "pdfcraft.snapshot_export";
const SNAPSHOT_EXPORT_MANIFEST_VERSION: &str = "1.1";
const SNAPSHOT_EXPORT_MANIFEST_PATH: &str = "manifest.json";
const SNAPSHOT_EXPORT_REPORT_PATH: &str = "report.tex";
const SNAPSHOT_EXPORT_REPORT_DOCX_PATH: &str = "report.docx";
const SNAPSHOT_EXPORT_MARKER_PATH: &str = "report.marker.json";
const SNAPSHOT_EXPORT_EVIDENCE_DIR: &str = "evidence";
const SNAPSHOT_EXPORT_SOURCE_PDF_PATH: &str = "source.pdf";
const SNAPSHOT_EXPORT_SOURCE_PDF_STATE: &str = "working-copy";
const SNAPSHOT_EXPORT_COORDINATE_PAGE_ORIGIN: &str = "top-left";
const SNAPSHOT_EXPORT_COORDINATE_PAGE_SPACE: &str = "display";
const SNAPSHOT_EXPORT_COORDINATE_UNITS: &str = "pt";
const SNAPSHOT_EXPORT_COORDINATE_PAGE_ROTATION: &str = "already-applied";
const SNAPSHOT_EXPORT_RENDERING_SOURCE: &str = "display-equivalent";
const SNAPSHOT_EXPORT_RENDERING_IMAGE_FORMAT: &str = "png";
const SNAPSHOT_EXPORT_PIXELS_PER_POINT: f32 = 2.0;
const SNAPSHOT_EXPORT_OCR_DPI: f32 = 300.0;

/// One axis of a tile: `start` and `span` in points, scaled once to device pixels at `scale`.
fn snapshot_export_tile_axis(start: f32, span: f32, scale: f32) -> Result<(u32, u32), String> {
    if !start.is_finite() || !span.is_finite() || !scale.is_finite() || scale <= 0.0 {
        return Err("snapshot rectangle coordinates must be finite".into());
    }
    let origin = (f64::from(start) * f64::from(scale)).floor();
    let end = (f64::from(start + span) * f64::from(scale)).ceil();
    if !origin.is_finite() || !end.is_finite() || origin < 0.0 || end <= origin || end > f64::from(u32::MAX) {
        return Err("snapshot image dimensions are out of range".into());
    }
    let origin = origin as u32;
    let end = end as u32;
    Ok((origin, end - origin))
}

fn snapshot_export_render_tile(rect: ExportRect) -> Result<Tile, String> {
    snapshot_export_render_tile_at_scale(rect, SNAPSHOT_EXPORT_PIXELS_PER_POINT)
}

fn snapshot_export_render_tile_at_scale(rect: ExportRect, scale: f32) -> Result<Tile, String> {
    let rect = rect.validate()?;
    let (x, w) = snapshot_export_tile_axis(rect.x, rect.width, scale)?;
    let (y, h) = snapshot_export_tile_axis(rect.y, rect.height, scale)?;
    Ok(Tile { x, y, w, h })
}

fn snapshot_export_image_size(rect: ExportRect) -> Result<(u32, u32), String> {
    let tile = snapshot_export_render_tile(rect)?;
    Ok((tile.w, tile.h))
}

fn validate_snapshot_export_rect_on_page(page_size: (f32, f32), rect: ExportRect) -> Result<(), String> {
    let rect = rect.validate()?;
    let right = rect.x + rect.width;
    let bottom = rect.y + rect.height;
    let epsilon = 0.001;
    if right > page_size.0 + epsilon || bottom > page_size.1 + epsilon {
        return Err(format!(
            "snapshot rectangle {} {} {} {} exceeds page bounds {} {}",
            rect.x, rect.y, rect.width, rect.height, page_size.0, page_size.1
        ));
    }
    Ok(())
}

fn validate_snapshot_export_contract_items(items: &[SnapshotExportItem]) -> Result<Vec<(u32, u32)>, String> {
    if items.is_empty() {
        return Err("snapshot export requires at least one item".into());
    }

    let mut image_sizes = Vec::with_capacity(items.len());
    for (index, item) in items.iter().enumerate() {
        let expected_id = snapshot_export_item_id(index)?;
        if item.id != expected_id {
            return Err(format!("snapshot item {} must have id {expected_id}", index + 1));
        }
        if item.kind != ExportItemKind::Snapshot {
            return Err(format!("snapshot item {} must have kind 'snapshot', got '{}'", item.id, item.kind.as_str()));
        }
        if item.page == 0 {
            return Err(format!("snapshot item {} page must be at least 1", item.id));
        }

        let expected_image_path = snapshot_export_image_path(&item.id)?;
        if item.image_path != expected_image_path {
            return Err(format!("snapshot item {} must have image path {expected_image_path}", item.id));
        }

        image_sizes.push(snapshot_export_image_size(item.rect)?);
    }

    Ok(image_sizes)
}

fn json_string(value: &str) -> Result<String, String> {
    serde_json::to_string(value).map_err(|e| format!("failed to serialize snapshot export JSON string: {e}"))
}

fn json_f32(value: f32) -> Result<String, String> {
    serde_json::to_string(&value).map_err(|e| format!("failed to serialize snapshot export JSON number: {e}"))
}

fn latex_escape_plain_text(text: &str) -> String {
    let mut escaped = String::with_capacity(text.len());
    for ch in text.chars() {
        match ch {
            '\\' => escaped.push_str(r"\textbackslash{}"),
            '{' => escaped.push_str(r"\{"),
            '}' => escaped.push_str(r"\}"),
            '%' => escaped.push_str(r"\%"),
            '&' => escaped.push_str(r"\&"),
            '$' => escaped.push_str(r"\$"),
            '#' => escaped.push_str(r"\#"),
            '_' => escaped.push_str(r"\_"),
            '^' => escaped.push_str(r"\^{}"),
            '~' => escaped.push_str(r"\~{}"),
            '\r' => {}
            _ => escaped.push(ch),
        }
    }
    escaped
}

/// Serializes the canonical v1 `manifest.json` from the normalized ordered item list.
pub fn snapshot_export_manifest_json(items: &[SnapshotExportItem]) -> Result<String, String> {
    let image_sizes = validate_snapshot_export_contract_items(items)?;
    let mut out = String::new();

    out.push_str("{\n");
    out.push_str("  \"package_kind\": ");
    out.push_str(&json_string(SNAPSHOT_EXPORT_PACKAGE_KIND)?);
    out.push_str(",\n");
    out.push_str("  \"manifest_version\": ");
    out.push_str(&json_string(SNAPSHOT_EXPORT_MANIFEST_VERSION)?);
    out.push_str(",\n");
    out.push_str("  \"source_pdf\": {\n");
    out.push_str("    \"path\": ");
    out.push_str(&json_string(SNAPSHOT_EXPORT_SOURCE_PDF_PATH)?);
    out.push_str(",\n");
    out.push_str("    \"state\": ");
    out.push_str(&json_string(SNAPSHOT_EXPORT_SOURCE_PDF_STATE)?);
    out.push('\n');
    out.push_str("  },\n");
    out.push_str("  \"coordinate_space\": {\n");
    out.push_str("    \"page_origin\": ");
    out.push_str(&json_string(SNAPSHOT_EXPORT_COORDINATE_PAGE_ORIGIN)?);
    out.push_str(",\n");
    out.push_str("    \"page_space\": ");
    out.push_str(&json_string(SNAPSHOT_EXPORT_COORDINATE_PAGE_SPACE)?);
    out.push_str(",\n");
    out.push_str("    \"units\": ");
    out.push_str(&json_string(SNAPSHOT_EXPORT_COORDINATE_UNITS)?);
    out.push_str(",\n");
    out.push_str("    \"page_rotation\": ");
    out.push_str(&json_string(SNAPSHOT_EXPORT_COORDINATE_PAGE_ROTATION)?);
    out.push('\n');
    out.push_str("  },\n");
    out.push_str("  \"rendering\": {\n");
    out.push_str("    \"snapshot_source\": ");
    out.push_str(&json_string(SNAPSHOT_EXPORT_RENDERING_SOURCE)?);
    out.push_str(",\n");
    out.push_str("    \"image_format\": ");
    out.push_str(&json_string(SNAPSHOT_EXPORT_RENDERING_IMAGE_FORMAT)?);
    out.push_str(",\n");
    out.push_str("    \"pixels_per_point\": ");
    out.push_str(&json_f32(SNAPSHOT_EXPORT_PIXELS_PER_POINT)?);
    out.push('\n');
    out.push_str("  },\n");
    out.push_str("  \"items\": [\n");

    let item_count = items.len();
    for (index, (item, (width_px, height_px))) in items.iter().zip(image_sizes).enumerate() {
        out.push_str("    {\n");
        out.push_str("      \"id\": ");
        out.push_str(&json_string(&item.id)?);
        out.push_str(",\n");
        out.push_str("      \"kind\": ");
        out.push_str(&json_string(item.kind.as_str())?);
        out.push_str(",\n");
        out.push_str("      \"page\": ");
        out.push_str(&item.page.to_string());
        out.push_str(",\n");
        out.push_str("      \"rect\": {\n");
        out.push_str("        \"x\": ");
        out.push_str(&json_f32(item.rect.x)?);
        out.push_str(",\n");
        out.push_str("        \"y\": ");
        out.push_str(&json_f32(item.rect.y)?);
        out.push_str(",\n");
        out.push_str("        \"width\": ");
        out.push_str(&json_f32(item.rect.width)?);
        out.push_str(",\n");
        out.push_str("        \"height\": ");
        out.push_str(&json_f32(item.rect.height)?);
        out.push('\n');
        out.push_str("      },\n");
        out.push_str("      \"image\": {\n");
        out.push_str("        \"path\": ");
        out.push_str(&json_string(&item.image_path)?);
        out.push_str(",\n");
        out.push_str("        \"width_px\": ");
        out.push_str(&width_px.to_string());
        out.push_str(",\n");
        out.push_str("        \"height_px\": ");
        out.push_str(&height_px.to_string());
        out.push('\n');
        out.push_str("      },\n");
        out.push_str("      \"caption\": ");
        out.push_str(&json_string(&item.metadata.caption)?);
        if let Some(note) = item.metadata.note.as_deref() {
            out.push_str(",\n");
            out.push_str("      \"note\": ");
            out.push_str(&json_string(note)?);
        }
        if let Some(ocr) = item.ocr.as_ref() {
            out.push_str(",\n");
            out.push_str("      \"ocr_text\": ");
            out.push_str(&json_string(&ocr.text)?);
        }
        out.push('\n');
        out.push_str("    }");
        if index + 1 != item_count {
            out.push(',');
        }
        out.push('\n');
    }

    out.push_str("  ]\n");
    out.push_str("}\n");
    Ok(out)
}

/// Renders the deterministic v1 `report.tex` from the normalized ordered item list.
pub fn snapshot_export_report_tex(items: &[SnapshotExportItem]) -> Result<String, String> {
    validate_snapshot_export_contract_items(items)?;

    let mut out = String::from(concat!(
        "\\documentclass{article}\n",
        "\\usepackage[T1]{fontenc}\n",
        "\\usepackage[utf8]{inputenc}\n",
        "\\usepackage{graphicx}\n",
        "\\usepackage{caption}\n",
        "\\usepackage{csquotes}\n",
        "\\begin{document}\n\n",
        "\\begin{enumerate}\n"
    ));
    for item in items {
        let item_text = item.metadata.note.as_deref().map(str::trim).filter(|text| !text.is_empty()).unwrap_or(item.metadata.caption.as_str());
        out.push_str(&format!("    \\item Conforme a Figura~\\ref{{fig:{}}}, {}.\n\n", item.id, latex_escape_plain_text(item_text)));
        out.push_str("\\noindent\\begin{minipage}[t]{\\linewidth}\n");
        out.push_str("    \\vspace{0pt}\n");
        out.push_str("    \\centering\n");
        out.push_str(&format!("    \\fbox{{\\includegraphics[width=\\linewidth]{{{}}}}}\n", item.image_path));
        out.push_str("    \\captionsetup{justification=centering}\n");
        out.push_str(&format!("    \\captionof{{figure}}{{{}}}\n", latex_escape_plain_text(&item.metadata.caption)));
        out.push_str(&format!("    \\label{{fig:{}}}\n", item.id));
        out.push_str("\\end{minipage}\n");

        if let Some(ocr) = item.ocr.as_ref().map(|ocr| ocr.text.trim()).filter(|text| !text.is_empty()) {
            out.push('\n');
            out.push_str(&format!("\\noindent OCR.: {}\n", latex_escape_plain_text(ocr)));
        }

        out.push('\n');
    }
    out.push_str("\\end{enumerate}\n\n\\end{document}\n");

    Ok(out)
}

/// Renders a machine-readable report for AI pipelines from the canonical ordered item list.
pub fn snapshot_export_marker_json(items: &[SnapshotExportItem]) -> Result<String, String> {
    validate_snapshot_export_contract_items(items)?;
    let value = serde_json::json!({
        "package_kind": SNAPSHOT_EXPORT_PACKAGE_KIND,
        "manifest_version": SNAPSHOT_EXPORT_MANIFEST_VERSION,
        "items": items
            .iter()
            .map(|item| {
                serde_json::json!({
                    "id": item.id,
                    "kind": item.kind.as_str(),
                    "page": item.page,
                    "image_path": item.image_path,
                    "caption": item.metadata.caption,
                    "note": item.metadata.note,
                    "ocr_text": item.ocr.as_ref().map(|ocr| ocr.text.clone()),
                })
            })
            .collect::<Vec<_>>()
    });
    serde_json::to_string_pretty(&value)
        .map(|mut out| {
            out.push('\n');
            out
        })
        .map_err(|e| format!("failed to serialize snapshot export marker JSON: {e}"))
}

#[derive(Clone, Debug, Default)]
struct SnapshotExportWriteHooks {
    #[cfg(test)]
    fail_before_write_rel_path: Option<String>,
}

impl SnapshotExportWriteHooks {
    fn maybe_fail_before_write(&self, _relative: &str) -> Result<(), String> {
        #[cfg(test)]
        if self.fail_before_write_rel_path.as_deref() == Some(_relative) {
            return Err(format!("injected snapshot export write failure at {_relative}"));
        }
        Ok(())
    }

    #[cfg(test)]
    fn fail_before_write(relative: &str) -> Self {
        Self { fail_before_write_rel_path: Some(relative.to_owned()) }
    }
}

fn validate_snapshot_export_package_input(input: &SnapshotExportPackageInput) -> Result<(), String> {
    if input.render_source.pages != input.render_source.sizes.len() {
        return Err("snapshot export render source page metadata is incomplete".into());
    }
    if input.source_pdf_bytes.is_empty() {
        return Err("snapshot export source.pdf bytes are empty".into());
    }
    validate_snapshot_export_contract_items(&input.items)?;
    for item in &input.items {
        validate_snapshot_export_page(input.render_source.pages, item.page)?;
        let page_size = *input.render_source.sizes.get(item.page - 1).ok_or_else(|| format!("page {} display size is not available", item.page))?;
        validate_snapshot_export_rect_on_page(page_size, item.rect)?;
    }
    Ok(())
}

fn snapshot_export_stage_prefix(name: &str) -> String {
    format!(".{name}.snapshot-export-{}-", std::process::id())
}

fn create_snapshot_export_staging_dir(target: &Path) -> Result<PathBuf, String> {
    let parent = target.parent().filter(|p| !p.as_os_str().is_empty()).unwrap_or(Path::new("."));
    let name = target.file_name().ok_or("snapshot export target must name a directory")?.to_string_lossy().into_owned();

    if target.exists() {
        return Err(format!("snapshot export target already exists: {}", target.display()));
    }

    let prefix = snapshot_export_stage_prefix(&name);
    for attempt in 0..128 {
        let candidate = parent.join(format!("{prefix}{attempt}"));
        match std::fs::create_dir(&candidate) {
            Ok(()) => return Ok(candidate),
            Err(err) if err.kind() == std::io::ErrorKind::AlreadyExists => continue,
            Err(err) => return Err(format!("could not create snapshot export staging directory {}: {err}", candidate.display())),
        }
    }

    Err(format!("could not create a free staging directory beside {}", target.display()))
}

fn create_named_staging_dir(target: &Path, family: &str) -> Result<PathBuf, String> {
    let parent = target.parent().filter(|p| !p.as_os_str().is_empty()).unwrap_or(Path::new("."));
    let name = target.file_name().ok_or_else(|| format!("{family} target must name a directory"))?.to_string_lossy().into_owned();

    if target.exists() {
        return Err(format!("{family} target already exists: {}", target.display()));
    }

    let prefix = format!(".{name}.{family}-{}-", std::process::id());
    for attempt in 0..128 {
        let candidate = parent.join(format!("{prefix}{attempt}"));
        match std::fs::create_dir(&candidate) {
            Ok(()) => return Ok(candidate),
            Err(err) if err.kind() == std::io::ErrorKind::AlreadyExists => continue,
            Err(err) => return Err(format!("could not create {family} staging directory {}: {err}", candidate.display())),
        }
    }

    Err(format!("could not create a free staging directory beside {}", target.display()))
}

fn resolve_snapshot_export_path(root: &Path, relative: &str) -> Result<PathBuf, String> {
    let relative = normalize_package_relative_path(relative)?;
    let mut path = root.to_path_buf();
    for part in relative.split('/') {
        path.push(part);
    }
    Ok(path)
}

fn resolve_crop_review_project_path(root: &Path, relative: &str) -> Result<PathBuf, String> {
    let relative = normalize_package_relative_path(relative)?;
    let mut path = root.to_path_buf();
    for part in relative.split('/') {
        path.push(part);
    }
    Ok(path)
}

fn write_snapshot_export_file(root: &Path, relative: &str, bytes: &[u8], hooks: &SnapshotExportWriteHooks) -> Result<(), String> {
    let path = resolve_snapshot_export_path(root, relative)?;
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| format!("could not create {}: {e}", parent.display()))?;
    }
    hooks.maybe_fail_before_write(relative)?;
    std::fs::write(&path, bytes).map_err(|e| format!("could not write {}: {e}", path.display()))
}

fn write_crop_review_project_file(root: &Path, relative: &str, bytes: &[u8]) -> Result<(), String> {
    let path = resolve_crop_review_project_path(root, relative)?;
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| format!("could not create {}: {e}", parent.display()))?;
    }
    std::fs::write(&path, bytes).map_err(|e| format!("could not write {}: {e}", path.display()))
}

pub fn crop_review_project_json(project: &CropReviewProject) -> Result<String, String> {
    let json = CropReviewProjectJson::from_project(project);
    serde_json::to_string_pretty(&json)
        .map(|mut out| {
            out.push('\n');
            out
        })
        .map_err(|e| format!("failed to serialize crop review project JSON: {e}"))
}

pub fn write_crop_review_project(target: &Path, source_pdf_bytes: Arc<Vec<u8>>, crops: Vec<CropReviewItem>) -> Result<(), String> {
    if source_pdf_bytes.is_empty() {
        return Err("crop review project source.pdf bytes are empty".into());
    }
    for crop in &crops {
        crop.validate()?;
    }
    let project = CropReviewProject { source_pdf_bytes, crops };
    let json = crop_review_project_json(&project)?;
    let stage = create_named_staging_dir(target, "crop-review-project")?;
    let result = (|| {
        write_crop_review_project_file(&stage, CROP_REVIEW_PROJECT_SOURCE_PDF_PATH, project.source_pdf_bytes.as_slice())?;
        write_crop_review_project_file(&stage, CROP_REVIEW_PROJECT_JSON_PATH, json.as_bytes())?;
        validate_crop_review_project_dir(&stage, &project, &json)?;
        std::fs::rename(&stage, target).map_err(|e| format!("could not publish crop review project to {}: {e}", target.display()))?;
        Ok(())
    })();
    if result.is_err() {
        let _ = std::fs::remove_dir_all(&stage);
    }
    result
}

pub fn load_crop_review_project(root: &Path) -> Result<CropReviewProject, String> {
    let json_path = resolve_crop_review_project_path(root, CROP_REVIEW_PROJECT_JSON_PATH)?;
    let json_text = std::fs::read_to_string(&json_path).map_err(|e| format!("could not read {}: {e}", json_path.display()))?;
    let json: CropReviewProjectJson = serde_json::from_str(&json_text).map_err(|e| format!("crop review project JSON is not valid: {e}"))?;
    json.validate()?;

    let source_path = resolve_crop_review_project_path(root, &json.source_pdf)?;
    let source_pdf_bytes = Arc::new(std::fs::read(&source_path).map_err(|e| format!("could not read {}: {e}", source_path.display()))?);
    if source_pdf_bytes.is_empty() {
        return Err("crop review project source.pdf bytes are empty".into());
    }

    let mut crops = Vec::with_capacity(json.crops.len());
    for crop in json.crops {
        crops.push(crop.into_item()?);
    }
    Ok(CropReviewProject { source_pdf_bytes, crops })
}

fn validate_crop_review_project_dir(root: &Path, project: &CropReviewProject, json_text: &str) -> Result<(), String> {
    if !root.is_dir() {
        return Err(format!("crop review project root {} was not created", root.display()));
    }
    let source_path = resolve_crop_review_project_path(root, CROP_REVIEW_PROJECT_SOURCE_PDF_PATH)?;
    let json_path = resolve_crop_review_project_path(root, CROP_REVIEW_PROJECT_JSON_PATH)?;
    let staged_source = std::fs::read(&source_path).map_err(|e| format!("could not read {}: {e}", source_path.display()))?;
    if staged_source != project.source_pdf_bytes.as_slice() {
        return Err("crop review project source.pdf bytes do not match the provided working file".into());
    }
    let staged_json = std::fs::read_to_string(&json_path).map_err(|e| format!("could not read {}: {e}", json_path.display()))?;
    if staged_json != json_text {
        return Err("crop review project JSON does not match the generated JSON".into());
    }
    let parsed: CropReviewProjectJson = serde_json::from_str(&staged_json).map_err(|e| format!("crop review project JSON is not valid: {e}"))?;
    parsed.validate()?;
    Ok(())
}

fn render_snapshot_export_item_png(renderer: &mut PageRenderer, source: &ExportSource, item: &SnapshotExportItem) -> Result<Vec<u8>, String> {
    validate_snapshot_export_page(source.pages, item.page)?;
    let page_size = *source.sizes.get(item.page - 1).ok_or_else(|| format!("page {} display size is not available", item.page))?;
    validate_snapshot_export_rect_on_page(page_size, item.rect)?;
    let tile = snapshot_export_render_tile(item.rect)?;
    let expected_size = snapshot_export_image_size(item.rect)?;
    let rendered = renderer.render(RenderRequest {
        page: item.page - 1,
        kind: RequestKind::Pixels,
        tile: Some(tile),
        scale: SNAPSHOT_EXPORT_PIXELS_PER_POINT,
        tag: 0,
        preview: false,
    });
    if let Some(error) = rendered.error {
        return Err(format!("snapshot item {} on page {} could not be rendered: {error}", item.id, item.page));
    }
    if (rendered.width, rendered.height) != expected_size {
        return Err(format!(
            "snapshot item {} rendered at {}x{} px, expected {}x{} px",
            item.id, rendered.width, rendered.height, expected_size.0, expected_size.1
        ));
    }
    encode_png(rendered.width, rendered.height, &rendered.rgba).map_err(|e| format!("snapshot item {} could not be encoded as PNG: {e}", item.id))
}

fn render_snapshot_export_images(
    root: &Path,
    source: &ExportSource,
    items: &[SnapshotExportItem],
    hooks: &SnapshotExportWriteHooks,
) -> Result<Vec<Vec<u8>>, String> {
    let evidence_dir = resolve_snapshot_export_path(root, SNAPSHOT_EXPORT_EVIDENCE_DIR)?;
    std::fs::create_dir_all(&evidence_dir).map_err(|e| format!("could not create {}: {e}", evidence_dir.display()))?;
    let mut renderer = PageRenderer::new(source.bytes.clone(), source.config.clone());
    let mut rendered = Vec::with_capacity(items.len());
    for item in items {
        let png = render_snapshot_export_item_png(&mut renderer, source, item)?;
        write_snapshot_export_file(root, &item.image_path, &png, hooks)?;
        rendered.push(png);
    }
    Ok(rendered)
}

fn snapshot_export_png_dimensions(bytes: &[u8]) -> Result<(u32, u32), String> {
    let reader = png::Decoder::new(std::io::Cursor::new(bytes)).read_info().map_err(|e| format!("could not read PNG metadata: {e}"))?;
    let info = reader.info();
    Ok((info.width, info.height))
}

fn snapshot_export_report_docx(title: &str, items: &[SnapshotExportItem], images: &[Vec<u8>]) -> Result<Vec<u8>, String> {
    validate_snapshot_export_contract_items(items)?;
    if images.len() != items.len() {
        return Err(format!("snapshot export Word report expects {} rendered images, got {}", items.len(), images.len()));
    }

    let pages = items
        .iter()
        .zip(images)
        .map(|(item, bytes)| {
            let mut blocks = vec![pdfcraft_export::Block {
                text: item.metadata.caption.clone(),
                rect: [72.0, 720.0, 540.0, 744.0],
                size: 14.0,
                bold: true,
                italic: false,
                color: [0.0, 0.0, 0.0],
                rotation: 0.0,
                space_around: [false, false],
            }];
            if let Some(note) = item.metadata.note.as_deref().map(str::trim).filter(|text| !text.is_empty()) {
                blocks.push(pdfcraft_export::Block {
                    text: note.to_owned(),
                    rect: [72.0, 680.0, 540.0, 704.0],
                    size: 11.0,
                    bold: false,
                    italic: false,
                    color: [0.0, 0.0, 0.0],
                    rotation: 0.0,
                    space_around: [false, false],
                });
            }
            if let Some(ocr) = item.ocr.as_ref().map(|ocr| ocr.text.trim()).filter(|text| !text.is_empty()) {
                blocks.push(pdfcraft_export::Block {
                    text: format!("OCR.: {ocr}"),
                    rect: [72.0, 640.0, 540.0, 664.0],
                    size: 10.0,
                    bold: false,
                    italic: false,
                    color: [0.0, 0.0, 0.0],
                    rotation: 0.0,
                    space_around: [false, false],
                });
            }
            let image_height = f64::from(item.rect.height.max(1.0));
            let image_width = f64::from(item.rect.width.max(1.0));
            let images = vec![pdfcraft_export::Image {
                ext: "png",
                bytes: std::sync::Arc::from(bytes.clone()),
                rect: [72.0, 300.0, 72.0 + image_width, 300.0 + image_height],
                size: [image_width, image_height],
                rotation: 0.0,
            }];
            pdfcraft_export::Page { width: 612.0, height: 792.0, blocks, images, background: None }
        })
        .collect::<Vec<_>>();
    Ok(pdfcraft_export::docx(&pages, title))
}

fn validate_snapshot_export_package_dir(
    root: &Path,
    source_pdf_bytes: &[u8],
    items: &[SnapshotExportItem],
    manifest_json: &str,
    report_tex: &str,
    report_docx: Option<&[u8]>,
    marker_json: &str,
) -> Result<(), String> {
    if !root.is_dir() {
        return Err(format!("snapshot export package root {} was not created", root.display()));
    }

    let manifest_path = resolve_snapshot_export_path(root, SNAPSHOT_EXPORT_MANIFEST_PATH)?;
    let source_pdf_path = resolve_snapshot_export_path(root, SNAPSHOT_EXPORT_SOURCE_PDF_PATH)?;
    let report_path = resolve_snapshot_export_path(root, SNAPSHOT_EXPORT_REPORT_PATH)?;
    let report_docx_path = resolve_snapshot_export_path(root, SNAPSHOT_EXPORT_REPORT_DOCX_PATH)?;
    let marker_path = resolve_snapshot_export_path(root, SNAPSHOT_EXPORT_MARKER_PATH)?;
    let evidence_dir = resolve_snapshot_export_path(root, SNAPSHOT_EXPORT_EVIDENCE_DIR)?;

    let staged_manifest = std::fs::read_to_string(&manifest_path).map_err(|e| format!("could not read {}: {e}", manifest_path.display()))?;
    if staged_manifest != manifest_json {
        return Err("snapshot export manifest.json does not match the generated manifest".into());
    }

    let staged_report = std::fs::read_to_string(&report_path).map_err(|e| format!("could not read {}: {e}", report_path.display()))?;
    if staged_report != report_tex {
        return Err("snapshot export report.tex does not match the generated report".into());
    }

    let staged_marker = std::fs::read_to_string(&marker_path).map_err(|e| format!("could not read {}: {e}", marker_path.display()))?;
    if staged_marker != marker_json {
        return Err("snapshot export report.marker.json does not match the generated report".into());
    }

    match report_docx {
        Some(bytes) => {
            let staged_docx = std::fs::read(&report_docx_path).map_err(|e| format!("could not read {}: {e}", report_docx_path.display()))?;
            if staged_docx != bytes {
                return Err("snapshot export report.docx does not match the generated report".into());
            }
        }
        None => {
            if report_docx_path.exists() {
                return Err(format!("snapshot export wrote unexpected {}", report_docx_path.display()));
            }
        }
    }

    let staged_source = std::fs::read(&source_pdf_path).map_err(|e| format!("could not read {}: {e}", source_pdf_path.display()))?;
    if staged_source != source_pdf_bytes {
        return Err("snapshot export source.pdf bytes do not match the provided working file".into());
    }

    if !evidence_dir.is_dir() {
        return Err(format!("snapshot export evidence directory {} is missing", evidence_dir.display()));
    }

    let parsed: serde_json::Value =
        serde_json::from_str(&staged_manifest).map_err(|e| format!("snapshot export manifest.json is not valid JSON: {e}"))?;
    let source_path = parsed["source_pdf"]["path"].as_str().ok_or("snapshot export manifest.json is missing source_pdf.path")?;
    if source_path != SNAPSHOT_EXPORT_SOURCE_PDF_PATH {
        return Err(format!("snapshot export manifest.json source_pdf.path must be {}, got {source_path}", SNAPSHOT_EXPORT_SOURCE_PDF_PATH));
    }
    if !resolve_snapshot_export_path(root, source_path)?.is_file() {
        return Err(format!("snapshot export manifest.json references missing source PDF {source_path}"));
    }

    let manifest_items = parsed["items"].as_array().ok_or("snapshot export manifest.json is missing items")?;
    if manifest_items.len() != items.len() {
        return Err(format!("snapshot export manifest.json lists {} items, expected {}", manifest_items.len(), items.len()));
    }

    for (expected, actual) in items.iter().zip(manifest_items) {
        let id = actual["id"].as_str().ok_or("snapshot export manifest.json item is missing id")?;
        let kind = actual["kind"].as_str().ok_or("snapshot export manifest.json item is missing kind")?;
        let page = actual["page"].as_u64().ok_or("snapshot export manifest.json item is missing page")? as usize;
        let image_path = actual["image"]["path"].as_str().ok_or("snapshot export manifest.json item is missing image.path")?;
        let width_px = actual["image"]["width_px"].as_u64().ok_or("snapshot export manifest.json item is missing image.width_px")? as u32;
        let height_px = actual["image"]["height_px"].as_u64().ok_or("snapshot export manifest.json item is missing image.height_px")? as u32;

        if id != expected.id {
            return Err(format!("snapshot export manifest.json item id {id} does not match expected {}", expected.id));
        }
        if kind != expected.kind.as_str() {
            return Err(format!("snapshot export manifest.json item {} kind {kind} does not match expected {}", expected.id, expected.kind.as_str()));
        }
        if page != expected.page {
            return Err(format!("snapshot export manifest.json item {} page {page} does not match expected {}", expected.id, expected.page));
        }
        if image_path != expected.image_path {
            return Err(format!(
                "snapshot export manifest.json item {} image path {image_path} does not match expected {}",
                expected.id, expected.image_path
            ));
        }

        let expected_size = snapshot_export_image_size(expected.rect)?;
        if (width_px, height_px) != expected_size {
            return Err(format!(
                "snapshot export manifest.json item {} image size {}x{} does not match expected {}x{}",
                expected.id, width_px, height_px, expected_size.0, expected_size.1
            ));
        }

        let image_bytes =
            std::fs::read(resolve_snapshot_export_path(root, image_path)?).map_err(|e| format!("could not read package image {image_path}: {e}"))?;
        let actual_size = snapshot_export_png_dimensions(&image_bytes)?;
        if actual_size != expected_size {
            return Err(format!(
                "snapshot export image {} is {}x{} px, expected {}x{} px",
                image_path, actual_size.0, actual_size.1, expected_size.0, expected_size.1
            ));
        }
    }

    Ok(())
}

/// Writes one snapshot export package to `target`, staging in a sibling directory and publishing
/// only after all artifacts validate.
pub fn write_snapshot_export_package(target: &Path, input: SnapshotExportPackageInput) -> Result<(), String> {
    write_snapshot_export_package_with_hooks(target, input, &SnapshotExportWriteHooks::default())
}

fn write_snapshot_export_package_with_hooks(
    target: &Path,
    input: SnapshotExportPackageInput,
    hooks: &SnapshotExportWriteHooks,
) -> Result<(), String> {
    validate_snapshot_export_package_input(&input)?;
    let manifest_json = snapshot_export_manifest_json(&input.items)?;
    let report_tex = snapshot_export_report_tex(&input.items)?;
    let marker_json = snapshot_export_marker_json(&input.items)?;
    let stage = create_snapshot_export_staging_dir(target)?;

    let result = (|| {
        write_snapshot_export_file(&stage, SNAPSHOT_EXPORT_MANIFEST_PATH, manifest_json.as_bytes(), hooks)?;
        write_snapshot_export_file(&stage, SNAPSHOT_EXPORT_SOURCE_PDF_PATH, input.source_pdf_bytes.as_slice(), hooks)?;
        write_snapshot_export_file(&stage, SNAPSHOT_EXPORT_REPORT_PATH, report_tex.as_bytes(), hooks)?;
        write_snapshot_export_file(&stage, SNAPSHOT_EXPORT_MARKER_PATH, marker_json.as_bytes(), hooks)?;
        let images = render_snapshot_export_images(&stage, &input.render_source, &input.items, hooks)?;
        let report_docx = if input.include_word_report {
            let bytes = snapshot_export_report_docx(&input.title, &input.items, &images)?;
            write_snapshot_export_file(&stage, SNAPSHOT_EXPORT_REPORT_DOCX_PATH, &bytes, hooks)?;
            Some(bytes)
        } else {
            None
        };
        validate_snapshot_export_package_dir(
            &stage,
            input.source_pdf_bytes.as_slice(),
            &input.items,
            &manifest_json,
            &report_tex,
            report_docx.as_deref(),
            &marker_json,
        )?;
        if target.exists() {
            return Err(format!("snapshot export target already exists: {}", target.display()));
        }
        std::fs::rename(&stage, target).map_err(|e| format!("could not publish snapshot export package to {}: {e}", target.display()))?;
        Ok(())
    })();

    if result.is_err() {
        let _ = std::fs::remove_dir_all(&stage);
    }

    result
}

fn render_snapshot_export_item_ocr(
    renderer: &mut PageRenderer,
    source: &ExportSource,
    item: &SnapshotExportItem,
) -> Result<Option<SnapshotExportOcr>, String> {
    validate_snapshot_export_page(source.pages, item.page)?;
    let page_size = *source.sizes.get(item.page - 1).ok_or_else(|| format!("page {} display size is not available", item.page))?;
    validate_snapshot_export_rect_on_page(page_size, item.rect)?;
    let scale = SNAPSHOT_EXPORT_OCR_DPI / 72.0;
    let tile = snapshot_export_render_tile_at_scale(item.rect, scale)?;
    let rendered = renderer.render(RenderRequest { page: item.page - 1, kind: RequestKind::Pixels, tile: Some(tile), scale, tag: 0, preview: false });
    if let Some(error) = rendered.error {
        return Err(format!("snapshot item {} on page {} could not be rendered for OCR: {error}", item.id, item.page));
    }
    let lines =
        crate::ocr::engine().map_err(|e| e.to_string())?.recognize(&rendered.rgba, rendered.width, rendered.height).map_err(|e| e.to_string())?;
    let text = lines.iter().map(|line| line.text()).collect::<Vec<_>>().join("\n").trim().to_string();
    Ok((!text.is_empty()).then_some(SnapshotExportOcr { text }))
}

fn snapshot_export_text_bounds_overlap(a: [f32; 4], b: ExportRect) -> bool {
    let left = a[0].max(b.x);
    let top = a[1].max(b.y);
    let right = a[2].min(b.x + b.width);
    let bottom = a[3].min(b.y + b.height);
    right > left && bottom > top
}

fn extract_snapshot_export_item_text(
    renderer: &mut PageRenderer,
    source: &ExportSource,
    item: &SnapshotExportItem,
) -> Result<Option<SnapshotExportOcr>, String> {
    validate_snapshot_export_page(source.pages, item.page)?;
    let page_size = *source.sizes.get(item.page - 1).ok_or_else(|| format!("page {} display size is not available", item.page))?;
    validate_snapshot_export_rect_on_page(page_size, item.rect)?;

    let rendered = renderer.render(RenderRequest { page: item.page - 1, kind: RequestKind::Text, scale: 1.0, ..Default::default() });
    let Some(text) = rendered.text else {
        return Ok(None);
    };

    let words = words_with(&text, |x, y| [x, y]);
    let text = words
        .into_iter()
        .filter(|(_, rect)| snapshot_export_text_bounds_overlap(*rect, item.rect))
        .map(|(word, _)| word)
        .collect::<Vec<_>>()
        .join(" ")
        .trim()
        .to_string();
    Ok((!text.is_empty()).then_some(SnapshotExportOcr { text }))
}

/// Populates missing OCR slots from the selected portion itself, using the shared OCR engine.
///
/// v1 keeps OCR derived and report-only: callers should run this before package writing so the
/// report can include the selected portion's OCR text without changing the manifest contract.
pub fn populate_snapshot_export_ocr(source: &ExportSource, items: &mut [SnapshotExportItem]) -> Result<(), String> {
    if items.iter().all(|item| item.ocr.is_some() || !item.allow_ocr_fill) {
        return Ok(());
    }
    let mut renderer = PageRenderer::new(source.bytes.clone(), source.config.clone());
    for item in items.iter_mut().filter(|item| item.ocr.is_none() && item.allow_ocr_fill) {
        item.ocr = extract_snapshot_export_item_text(&mut renderer, source, item)?;
        if item.ocr.is_none() && crate::ocr::available() {
            item.ocr = render_snapshot_export_item_ocr(&mut renderer, source, item)?;
        }
    }
    Ok(())
}

impl Exporter {
    pub fn new(doc: &Document) -> Self {
        Self::from_source(doc.export_source())
    }

    /// An exporter that draws a page too large for the renderer's size limits at the largest
    /// resolution they allow, as the interactive Export does; [`Exporter::dpi_used`] says at what.
    /// A source whose `config.reject_oversize` is set refuses such pages instead.
    pub fn from_source(src: ExportSource) -> Self {
        Self { renderer: PageRenderer::new(src.bytes, src.config), pages: src.pages, sizes: src.sizes }
    }

    /// The resolution page `page` is actually exported at when asked for `dpi`: lower than asked
    /// when the page is too large for the renderer's size limits at that resolution.
    pub fn dpi_used(&self, page: usize, dpi: f64) -> f64 {
        let dpi = dpi.clamp(18.0, 1200.0);
        match self.sizes.get(page) {
            Some(&(w, h)) => f64::from(pdfcraft_render::effective_scale(w, h, (dpi / 72.0) as f32)) * 72.0,
            None => dpi,
        }
    }

    fn check(&self, page: usize) -> Result<(), String> {
        if page < self.pages { Ok(()) } else { Err(format!("page {} does not exist", page + 1)) }
    }

    /// Page `page` (0-based) as a PNG at `dpi`, at most the renderer's size limits allow (see
    /// [`Exporter::dpi_used`]); refused instead when the source asked for `reject_oversize`.
    pub fn png(&mut self, page: usize, dpi: f64) -> Result<Vec<u8>, String> {
        self.check(page)?;
        let r = self.renderer.render(RenderRequest { page, scale: (dpi.clamp(18.0, 1200.0) / 72.0) as f32, ..Default::default() });
        if let Some(e) = r.error {
            return Err(format!("page {}: {e}", page + 1));
        }
        encode_png(r.width, r.height, &r.rgba)
    }

    /// Page `page` as an image file of `format`, which says the resolution it was drawn at.
    pub fn image(&mut self, page: usize, dpi: f64, format: ImageFormat) -> Result<Vec<u8>, String> {
        self.check(page)?;
        let r = self.renderer.render(RenderRequest { page, scale: (dpi.clamp(18.0, 1200.0) / 72.0) as f32, ..Default::default() });
        if let Some(e) = r.error {
            return Err(format!("page {}: {e}", page + 1));
        }
        encode_image(r.width, r.height, &r.rgba, format, self.dpi_used(page, dpi))
    }

    /// The reading-order text of a page.
    pub fn text(&mut self, page: usize) -> Result<String, String> {
        self.check(page)?;
        let r = self.renderer.render(RenderRequest { page, kind: RequestKind::Text, scale: 1.0, ..Default::default() });
        match r.text {
            Some(t) => Ok(t.plain_text()),
            None => Err(format!("page {}: {}", page + 1, r.error.unwrap_or_else(|| "no text layer".into()))),
        }
    }

    /// The text of several pages, separated by form feeds (as `pdftotext` does).
    pub fn text_of(&mut self, pages: &[usize]) -> Result<String, String> {
        let mut out = String::new();
        for (k, p) in pages.iter().enumerate() {
            if k > 0 {
                out.push('\u{c}');
            }
            out.push_str(self.text(*p)?.trim_end());
            out.push('\n');
        }
        Ok(out)
    }
}

/// Premultiplied RGBA → an image file of `format`. A JPEG or TIFF says it is `dpi` dots per inch,
/// so other programs show and print it at the page's size. A PNG says nothing: its `pHYs` counts
/// pixels per metre, so 72 dpi would read back as 72.009 and a page made from it would shrink.
pub fn encode_image(width: u32, height: u32, premultiplied: &[u8], format: ImageFormat, dpi: f64) -> Result<Vec<u8>, String> {
    match format {
        ImageFormat::Png => encode_png(width, height, premultiplied),
        ImageFormat::Jpeg { quality } => encode_jpeg(width, height, premultiplied, quality, dpi),
        ImageFormat::Tiff => encode_tiff(width, height, premultiplied, dpi),
    }
}

/// Premultiplied RGBA → PNG (straight alpha). A raster with no translucent pixel (a rendered
/// page always is one: it is drawn on white) is written as RGB.
///
/// Deflate level 3 with adaptive filtering: on rendered pages it writes files the size of the
/// `png` crate's default (level 6) in a third to a fifth of the time. Measured with RGB at
/// level 3 against RGBA at the default: a 300-dpi scan drawn at 220 dpi took 805 → 167 ms
/// (4.46 → 4.47 MB), a text page at 220 dpi 70 → 40 ms (0.85 → 0.89 MB), and an annual
/// report page at 150 dpi 179 → 64 ms (1.41 → 1.38 MB).
pub fn encode_png(width: u32, height: u32, premultiplied: &[u8]) -> Result<Vec<u8>, String> {
    check_raster(width, height, premultiplied)?;
    let pixels = premultiplied.as_chunks::<4>().0;
    let opaque = pixels.iter().all(|px| px[3] == 255);
    let (color, data) = if opaque {
        (png::ColorType::Rgb, pixels.iter().flat_map(|px| [px[0], px[1], px[2]]).collect::<Vec<u8>>())
    } else {
        let mut rgba = premultiplied.to_vec();
        for px in rgba.as_chunks_mut::<4>().0 {
            let a = u32::from(px[3]);
            if a != 0 && a != 255 {
                for c in &mut px[..3] {
                    *c = ((u32::from(*c) * 255 + a / 2) / a).min(255) as u8;
                }
            }
        }
        (png::ColorType::Rgba, rgba)
    };
    let mut out = Vec::new();
    let mut enc = png::Encoder::new(&mut out, width, height);
    enc.set_color(color);
    enc.set_depth(png::BitDepth::Eight);
    enc.set_deflate_compression(png::DeflateCompression::Level(3));
    enc.set_filter(png::Filter::Adaptive);
    let mut w = enc.write_header().map_err(|e| e.to_string())?;
    w.write_image_data(&data).map_err(|e| e.to_string())?;
    w.finish().map_err(|e| e.to_string())?;
    Ok(out)
}

/// Image file formats for Export ▸ Image.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ImageFormat {
    Png,
    /// Quality 1–100.
    Jpeg {
        quality: u8,
    },
    Tiff,
}

impl ImageFormat {
    pub fn extension(self) -> &'static str {
        match self {
            ImageFormat::Png => "png",
            ImageFormat::Jpeg { .. } => "jpg",
            ImageFormat::Tiff => "tif",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            ImageFormat::Png => "PNG",
            ImageFormat::Jpeg { .. } => "JPEG",
            ImageFormat::Tiff => "TIFF",
        }
    }
}

/// Checks that `premultiplied` is exactly `width × height` RGBA pixels, and at least one,
/// before any encoder sees it: `as_chunks` drops a partial trailing pixel, so the rest would be
/// written as a valid image, and the JPEG and TIFF encoders panic on a buffer that doesn't fit
/// the size.
fn check_raster(width: u32, height: u32, premultiplied: &[u8]) -> Result<(), String> {
    if width == 0 || height == 0 {
        return Err(format!("a {width}×{height} raster has no pixels"));
    }
    let expected = usize::try_from(width).ok().zip(usize::try_from(height).ok()).and_then(|(w, h)| w.checked_mul(h)?.checked_mul(4));
    match expected {
        Some(n) if n == premultiplied.len() => Ok(()),
        Some(n) => Err(format!("a {width}×{height} raster is {n} bytes of RGBA, got {}", premultiplied.len())),
        None => Err(format!("a {width}×{height} raster is too large")),
    }
}

/// Premultiplied RGBA composited over white → RGB.
fn over_white(premultiplied: &[u8]) -> Vec<u8> {
    premultiplied
        .as_chunks::<4>()
        .0
        .iter()
        .flat_map(|p| {
            let k = 255 - p[3];
            [p[0].saturating_add(k), p[1].saturating_add(k), p[2].saturating_add(k)]
        })
        .collect()
}

/// Premultiplied RGBA → baseline JPEG at `dpi` (pages are opaque: transparency becomes white paper).
pub fn encode_jpeg(width: u32, height: u32, premultiplied: &[u8], quality: u8, dpi: f64) -> Result<Vec<u8>, String> {
    use image::ImageEncoder;
    use image::codecs::jpeg::{JpegEncoder, PixelDensity};
    check_raster(width, height, premultiplied)?;
    let rgb = over_white(premultiplied);
    let mut out = Vec::new();
    let mut enc = JpegEncoder::new_with_quality(&mut out, quality.clamp(1, 100));
    // The JFIF density is whole dots per inch.
    enc.set_pixel_density(PixelDensity::dpi(dpi.round() as u16));
    enc.write_image(&rgb, width, height, image::ExtendedColorType::Rgb8).map_err(|e| e.to_string())?;
    Ok(out)
}

/// Premultiplied RGBA → TIFF (RGB, LZW-compressed) at `dpi`. Written with `tiff` directly:
/// `image`'s `TiffEncoder` always says 1 × 1 with no unit and leaves the strips uncompressed.
/// No predictor: LZW alone is the compression TIFF readers support most widely after none.
pub fn encode_tiff(width: u32, height: u32, premultiplied: &[u8], dpi: f64) -> Result<Vec<u8>, String> {
    use tiff::encoder::{Compression, Rational, TiffEncoder, colortype::RGB8};
    use tiff::tags::{ResolutionUnit, Tag};
    let err = |e: tiff::TiffError| e.to_string();
    check_raster(width, height, premultiplied)?;
    let rgb = over_white(premultiplied);
    let mut out = std::io::Cursor::new(Vec::new());
    let mut tiff = TiffEncoder::new(&mut out).map_err(err)?.with_compression(Compression::Lzw);
    let mut image = tiff.new_image::<RGB8>(width, height).map_err(err)?;
    // In hundredths: a page drawn at a lowered resolution (`Exporter::dpi_used`) is rarely a whole number.
    let resolution = Rational { n: (dpi * 100.0).round() as u32, d: 100 };
    let tags = image.encoder();
    tags.write_tag(Tag::ResolutionUnit, ResolutionUnit::Inch).map_err(err)?;
    tags.write_tag(Tag::XResolution, resolution.clone()).map_err(err)?;
    tags.write_tag(Tag::YResolution, resolution).map_err(err)?;
    image.write_data(&rgb).map_err(err)?;
    Ok(out.into_inner())
}

#[cfg(test)]
mod image_name_tests {
    use super::image_file_name;

    fn image(page: usize) -> pdfcraft_create::ExtractedImage {
        pdfcraft_create::ExtractedImage {
            page,
            object: pdfcraft_cos::ObjRef { num: 1, generation: 0 },
            width: 1,
            height: 1,
            extension: "png",
            data: Vec::new(),
        }
    }

    #[test]
    fn page_number_is_padded_to_the_page_count() {
        assert_eq!(image_file_name("doc", &image(0), 1, 1), "doc_Page_1_Image_0001.png");
        assert_eq!(image_file_name("doc", &image(0), 1, 9), "doc_Page_1_Image_0001.png");
        assert_eq!(image_file_name("doc", &image(6), 2, 20), "doc_Page_07_Image_0002.png");
        assert_eq!(image_file_name("doc", &image(6), 3, 120), "doc_Page_007_Image_0003.png");
        assert_eq!(image_file_name("doc", &image(119), 4, 120), "doc_Page_120_Image_0004.png");
        assert_eq!(image_file_name("doc", &image(0), 1, 0), "doc_Page_1_Image_0001.png");
    }
}

#[cfg(test)]
mod snapshot_export_tests {
    use super::{
        CropPdfRect, CropReviewItem, CropReviewOcrMode, CropReviewStore, ExportItemKind, ExportRect, SnapshotExportItemInput, SnapshotExportMetadata,
        SnapshotExportOcr, normalize_package_relative_path, normalize_snapshot_export_items,
    };
    use chrono::{TimeZone, Utc};
    use pdfcraft_render::PageInfo;

    fn metadata(caption: &str, note: Option<&str>) -> SnapshotExportMetadata {
        SnapshotExportMetadata { caption: caption.into(), note: note.map(str::to_owned) }
    }

    fn input(order: usize, page: usize, caption: &str) -> SnapshotExportItemInput {
        SnapshotExportItemInput {
            order,
            page,
            rect: ExportRect { x: 10.0, y: 20.0, width: 30.0, height: 40.0 },
            metadata: metadata(caption, None),
            ocr: None,
            allow_ocr_fill: true,
        }
    }

    fn timestamp(hour: u32, minute: u32, second: u32) -> chrono::DateTime<Utc> {
        Utc.with_ymd_and_hms(2026, 10, 26, hour, minute, second).single().unwrap()
    }

    fn page(width: f32, height: f32, crop: [f32; 4], rotation: u16) -> PageInfo {
        PageInfo { width, height, label: "1".into(), crop, rotation }
    }

    fn crop_item(id: &str, page_number: usize, rect_pdf: CropPdfRect) -> CropReviewItem {
        CropReviewItem {
            id: id.to_owned(),
            kind: ExportItemKind::Snapshot,
            page_number,
            rect_pdf,
            caption: format!("caption-{id}"),
            note: Some(format!("note-{id}")),
            ocr_mode: CropReviewOcrMode::Auto,
            ocr_text: Some(format!("ocr-{id}")),
            ocr_extracted_at: Some(timestamp(9, 15, 0)),
            created_at: timestamp(9, 0, 0),
            updated_at: timestamp(9, 30, 0),
        }
    }

    #[test]
    fn snapshot_items_follow_explicit_order_and_preserve_ties() {
        let items = normalize_snapshot_export_items(3, vec![input(30, 3, "third"), input(10, 2, "first"), input(10, 1, "second")]).unwrap();

        assert_eq!(items.iter().map(|item| item.metadata.caption.as_str()).collect::<Vec<_>>(), vec!["first", "second", "third"]);
        assert_eq!(items.iter().map(|item| item.page).collect::<Vec<_>>(), vec![2, 1, 3]);
    }

    #[test]
    fn typed_metadata_flows_into_normalized_export_items() {
        let items = normalize_snapshot_export_items(
            2,
            vec![SnapshotExportItemInput {
                order: 0,
                page: 2,
                rect: ExportRect { x: 12.0, y: 18.0, width: 22.0, height: 28.0 },
                metadata: metadata("Typed caption", Some("Typed note")),
                ocr: None,
                allow_ocr_fill: true,
            }],
        )
        .unwrap();

        assert_eq!(items[0].metadata, metadata("Typed caption", Some("Typed note")));
    }

    #[test]
    fn normalized_items_keep_an_explicit_ocr_slot() {
        let items = normalize_snapshot_export_items(
            1,
            vec![SnapshotExportItemInput {
                order: 0,
                page: 1,
                rect: ExportRect { x: 10.0, y: 20.0, width: 30.0, height: 40.0 },
                metadata: metadata("Caption", None),
                ocr: Some(SnapshotExportOcr { text: "derived OCR".into() }),
                allow_ocr_fill: true,
            }],
        )
        .unwrap();

        assert_eq!(items[0].ocr, Some(SnapshotExportOcr { text: "derived OCR".into() }));
    }

    #[test]
    fn snapshot_ids_and_image_paths_derive_from_the_canonical_order() {
        let items = normalize_snapshot_export_items(2, vec![input(20, 1, "second"), input(10, 2, "first")]).unwrap();

        assert_eq!(items.iter().map(|item| item.id.as_str()).collect::<Vec<_>>(), vec!["item_1", "item_2"]);
        assert_eq!(items.iter().map(|item| item.image_path.as_str()).collect::<Vec<_>>(), vec!["evidence/item_1.png", "evidence/item_2.png"]);
        assert_eq!(items.iter().map(|item| item.kind).collect::<Vec<_>>(), vec![ExportItemKind::Snapshot, ExportItemKind::Snapshot]);
        assert_eq!(items.iter().map(|item| item.kind.as_str()).collect::<Vec<_>>(), vec!["snapshot", "snapshot"]);
    }

    #[test]
    fn package_paths_are_normalized_to_relative_forward_slash_paths() {
        assert_eq!(normalize_package_relative_path("./evidence//item_1.png"), Ok("evidence/item_1.png".into()));
        assert_eq!(normalize_package_relative_path("nested/./dir///item_2.png"), Ok("nested/dir/item_2.png".into()));
    }

    #[test]
    fn package_paths_reject_invalid_or_escaping_inputs() {
        for path in ["", ".", "/evidence/item_1.png", "../evidence/item_1.png", "evidence/../item_1.png", r"evidence\item_1.png"] {
            assert!(normalize_package_relative_path(path).is_err(), "{path}");
        }
    }

    #[test]
    fn invalid_pages_and_rectangles_fail_through_the_normal_result_path() {
        let bad_page = normalize_snapshot_export_items(2, vec![input(0, 0, "bad page")]).unwrap_err();
        assert_eq!(bad_page, "page 0 does not exist");

        let negative_rect = normalize_snapshot_export_items(
            2,
            vec![SnapshotExportItemInput {
                order: 0,
                page: 1,
                rect: ExportRect { x: -1.0, y: 20.0, width: 30.0, height: 40.0 },
                metadata: metadata("bad rect", None),
                ocr: None,
                allow_ocr_fill: true,
            }],
        )
        .unwrap_err();
        assert_eq!(negative_rect, "snapshot rectangle coordinates must be non-negative");

        let bad_rect = normalize_snapshot_export_items(
            2,
            vec![SnapshotExportItemInput {
                order: 0,
                page: 1,
                rect: ExportRect { x: 10.0, y: 20.0, width: 0.0, height: 40.0 },
                metadata: metadata("bad rect", None),
                ocr: None,
                allow_ocr_fill: true,
            }],
        )
        .unwrap_err();
        assert_eq!(bad_rect, "snapshot rectangle size must be finite and positive");

        let non_finite = normalize_snapshot_export_items(
            2,
            vec![SnapshotExportItemInput {
                order: 0,
                page: 1,
                rect: ExportRect { x: f32::NAN, y: 20.0, width: 30.0, height: 40.0 },
                metadata: metadata("bad rect", None),
                ocr: None,
                allow_ocr_fill: true,
            }],
        )
        .unwrap_err();
        assert_eq!(non_finite, "snapshot rectangle coordinates must be finite");
    }

    #[test]
    fn crop_items_order_by_page_display_y_display_x_then_id() {
        let mut store = CropReviewStore::new();
        store.upsert(crop_item("page-1-bottom", 1, CropPdfRect::from_user_rect([15.0, 100.0, 55.0, 140.0]).unwrap())).unwrap();
        store.upsert(crop_item("page-1-top-right", 1, CropPdfRect::from_user_rect([40.0, 680.0, 80.0, 720.0]).unwrap())).unwrap();
        store.upsert(crop_item("page-1-top-left-b", 1, CropPdfRect::from_user_rect([10.0, 680.0, 50.0, 720.0]).unwrap())).unwrap();
        store.upsert(crop_item("page-2", 2, CropPdfRect::from_user_rect([5.0, 500.0, 25.0, 540.0]).unwrap())).unwrap();
        store.upsert(crop_item("page-1-top-left-a", 1, CropPdfRect::from_user_rect([10.0, 680.0, 50.0, 720.0]).unwrap())).unwrap();

        let pages = [page(300.0, 800.0, [0.0, 0.0, 300.0, 800.0], 0), page(300.0, 600.0, [0.0, 0.0, 300.0, 600.0], 0)];
        assert_eq!(store.ordered_ids(&pages), vec!["page-1-top-left-a", "page-1-top-left-b", "page-1-top-right", "page-1-bottom", "page-2"]);
    }

    #[test]
    fn crop_geometry_is_stored_in_pdf_space_and_round_trips_through_page_transforms() {
        let info = page(100.0, 200.0, [0.0, 0.0, 200.0, 100.0], 90);
        let display = ExportRect { x: 40.0, y: 20.0, width: 30.0, height: 60.0 };
        let rect_pdf =
            CropPdfRect::from_user_rect(info.view_rect_to_user([display.x, display.y, display.x + display.width, display.y + display.height]))
                .unwrap();

        assert!((rect_pdf.left - 20.0).abs() < 1e-3, "left: {}", rect_pdf.left);
        assert!((rect_pdf.bottom - 40.0).abs() < 1e-3, "bottom: {}", rect_pdf.bottom);
        assert!((rect_pdf.right - 80.0).abs() < 1e-3, "right: {}", rect_pdf.right);
        assert!((rect_pdf.top - 70.0).abs() < 1e-3, "top: {}", rect_pdf.top);
        assert_ne!(rect_pdf.left, display.x, "stored x is in PDF space, not viewport space");
        assert_ne!(rect_pdf.bottom, display.y, "stored y is in PDF space, not viewport space");
        let round_trip = rect_pdf.to_export_rect(&info).unwrap();
        assert!((round_trip.x - display.x).abs() < 1e-3, "x: {}", round_trip.x);
        assert!((round_trip.y - display.y).abs() < 1e-3, "y: {}", round_trip.y);
        assert!((round_trip.width - display.width).abs() < 1e-3, "width: {}", round_trip.width);
        assert!((round_trip.height - display.height).abs() < 1e-3, "height: {}", round_trip.height);
    }

    #[test]
    fn crop_store_round_trips_identifier_metadata_ocr_and_timestamps() {
        let mut store = CropReviewStore::new();
        let item = CropReviewItem {
            id: "crop-123".to_owned(),
            kind: ExportItemKind::Snapshot,
            page_number: 3,
            rect_pdf: CropPdfRect::from_user_rect([12.5, 44.0, 90.0, 120.0]).unwrap(),
            caption: "Figure caption".to_owned(),
            note: Some("Reviewer note".to_owned()),
            ocr_mode: CropReviewOcrMode::Auto,
            ocr_text: Some("Recognized text".to_owned()),
            ocr_extracted_at: Some(timestamp(10, 5, 0)),
            created_at: timestamp(9, 55, 0),
            updated_at: timestamp(10, 6, 0),
        };

        store.upsert(item.clone()).unwrap();
        let stored = store.get("crop-123").unwrap();
        assert_eq!(stored, &item);
        assert_eq!(stored.kind.as_str(), "snapshot");
        assert_eq!(stored.metadata(), SnapshotExportMetadata { caption: "Figure caption".to_owned(), note: Some("Reviewer note".to_owned()) });
        assert_eq!(stored.ocr(), Some(SnapshotExportOcr { text: "Recognized text".to_owned() }));
    }
}

#[cfg(test)]
mod crop_review_project_tests {
    use super::{CropPdfRect, CropReviewItem, CropReviewOcrMode, ExportItemKind, load_crop_review_project, write_crop_review_project};
    use chrono::{TimeZone, Utc};
    use std::sync::Arc;

    struct TestDir {
        path: std::path::PathBuf,
    }

    impl TestDir {
        fn new(name: &str) -> Self {
            static NEXT: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
            let n = NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            let path = std::env::temp_dir().join(format!("pdfcraft-crop-project-{name}-{}-{n}", std::process::id()));
            let _ = std::fs::remove_dir_all(&path);
            std::fs::create_dir_all(&path).unwrap();
            Self { path }
        }

        fn child(&self, name: &str) -> std::path::PathBuf {
            self.path.join(name)
        }
    }

    impl Drop for TestDir {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.path);
        }
    }

    fn timestamp(hour: u32, minute: u32, second: u32) -> chrono::DateTime<Utc> {
        Utc.with_ymd_and_hms(2026, 10, 26, hour, minute, second).single().unwrap()
    }

    fn crop(id: &str, page_number: usize, caption: &str, note: Option<&str>, ocr_text: Option<&str>) -> CropReviewItem {
        CropReviewItem {
            id: id.to_owned(),
            kind: ExportItemKind::Snapshot,
            page_number,
            rect_pdf: CropPdfRect { left: 12.5, bottom: 44.0, right: 90.0, top: 120.0 },
            caption: caption.to_owned(),
            note: note.map(str::to_owned),
            ocr_mode: CropReviewOcrMode::Auto,
            ocr_text: ocr_text.map(str::to_owned),
            ocr_extracted_at: ocr_text.map(|_| timestamp(10, 5, 0)),
            created_at: timestamp(9, 55, 0),
            updated_at: timestamp(10, 6, 0),
        }
    }

    #[test]
    fn crop_review_project_round_trips_all_fields() {
        let dir = TestDir::new("round-trip");
        let target = dir.child("project");
        let source_pdf = Arc::new(crate::tests::fixture(2));
        let crops =
            vec![crop("crop-0001", 1, "Resumo", Some("Observacao com acento"), Some("ocr 日本語")), crop("crop-0002", 2, "第二", Some("nota"), None)];

        write_crop_review_project(&target, source_pdf.clone(), crops.clone()).unwrap();
        let loaded = load_crop_review_project(&target).unwrap();

        assert_eq!(loaded.source_pdf_bytes.as_slice(), source_pdf.as_slice());
        assert_eq!(loaded.crops, crops);
    }

    #[test]
    fn crop_review_project_writes_expected_folder_shape() {
        let dir = TestDir::new("shape");
        let target = dir.child("project");
        write_crop_review_project(&target, Arc::new(crate::tests::fixture(1)), vec![crop("crop-0001", 1, "Caption", None, None)]).unwrap();

        assert!(target.join("source.pdf").is_file());
        assert!(target.join("project.json").is_file());

        let json = std::fs::read_to_string(target.join("project.json")).unwrap();
        assert!(json.contains("\"schema_version\": 1"));
        assert!(json.contains("\"source_pdf\": \"source.pdf\""));
        assert!(json.contains("\"crops\""));
    }

    #[test]
    fn crop_review_project_json_remains_utf8_readable() {
        let dir = TestDir::new("utf8");
        let target = dir.child("project");
        write_crop_review_project(
            &target,
            Arc::new(crate::tests::fixture(1)),
            vec![crop("crop-0001", 1, "Résumé 日本語", Some("ação"), Some("ocr çã"))],
        )
        .unwrap();

        let json = std::fs::read_to_string(target.join("project.json")).unwrap();
        assert!(json.contains("Résumé 日本語"), "{json}");
        assert!(json.contains("ação"), "{json}");
        assert!(json.contains("ocr çã"), "{json}");
        assert!(!json.contains("\\u"), "{json}");
    }

    #[test]
    fn crop_review_project_rejects_invalid_source_pdf_reference() {
        let dir = TestDir::new("bad-source");
        let target = dir.child("project");
        std::fs::create_dir_all(&target).unwrap();
        std::fs::write(target.join("source.pdf"), crate::tests::fixture(1)).unwrap();
        std::fs::write(target.join("project.json"), b"{\n  \"schema_version\": 1,\n  \"source_pdf\": \"../outside.pdf\",\n  \"crops\": []\n}\n")
            .unwrap();

        let err = load_crop_review_project(&target).unwrap_err();
        assert!(err.contains("source_pdf must be source.pdf"), "{err}");
    }

    #[test]
    fn crop_review_project_rejects_unsupported_schema_version() {
        let dir = TestDir::new("bad-version");
        let target = dir.child("project");
        std::fs::create_dir_all(&target).unwrap();
        std::fs::write(target.join("source.pdf"), crate::tests::fixture(1)).unwrap();
        std::fs::write(target.join("project.json"), b"{\n  \"schema_version\": 2,\n  \"source_pdf\": \"source.pdf\",\n  \"crops\": []\n}\n").unwrap();

        let err = load_crop_review_project(&target).unwrap_err();
        assert!(err.contains("schema version 2 is not supported"), "{err}");
    }
}

#[cfg(test)]
mod export_embedded_crop_review_tests {
    use std::sync::Arc;

    use chrono::{TimeZone, Utc};

    use super::{CropPdfRect, CropReviewItem, CropReviewOcrMode, EMBEDDED_CROP_REVIEW_ATTACHMENT_NAME, ExportItemKind, crop_review_store_json};

    fn timestamp(hour: u32, minute: u32, second: u32) -> chrono::DateTime<Utc> {
        Utc.with_ymd_and_hms(2026, 10, 26, hour, minute, second).single().unwrap()
    }

    fn crop(id: &str, page_number: usize) -> CropReviewItem {
        CropReviewItem {
            id: id.to_owned(),
            kind: ExportItemKind::Snapshot,
            page_number,
            rect_pdf: CropPdfRect { left: 12.5, bottom: 44.0, right: 90.0, top: 120.0 },
            caption: format!("caption-{id}"),
            note: Some(format!("note-{id}")),
            ocr_mode: CropReviewOcrMode::Auto,
            ocr_text: Some(format!("ocr-{id}")),
            ocr_extracted_at: Some(timestamp(10, 5, 0)),
            created_at: timestamp(9, 55, 0),
            updated_at: timestamp(10, 6, 0),
        }
    }

    fn session_with(n: usize) -> (crate::Session, crate::DocId) {
        let mut session = crate::Session::new().with_clock(|| 1_700_000_000);
        let id = session.open("fixture.pdf", None, Arc::new(crate::tests::fixture(n)), None).unwrap();
        (session, id)
    }

    fn attachment_keys(bytes: &Arc<Vec<u8>>) -> Vec<String> {
        let doc = pdfcraft_cos::Document::open(bytes.clone()).unwrap();
        pdfcraft_organize::attachment_keys(&doc)
    }

    fn embedded_fixture_with_crop_review(crops: &[CropReviewItem]) -> Arc<Vec<u8>> {
        let mut cos = pdfcraft_cos::Document::open(Arc::new(crate::tests::fixture(2))).unwrap();
        let json = crop_review_store_json(crops).unwrap();
        pdfcraft_organize::add_attachment(
            &mut cos,
            EMBEDDED_CROP_REVIEW_ATTACHMENT_NAME,
            json.as_bytes(),
            Some("PdfCraft embedded crop review workspace"),
            pdfcraft_organize::mime_for_name(EMBEDDED_CROP_REVIEW_ATTACHMENT_NAME),
            Some("D:20261026100500Z"),
        )
        .unwrap();
        Arc::new(pdfcraft_cos::write_full(&cos, &pdfcraft_cos::SaveOptions::default()).unwrap())
    }

    #[test]
    fn export_open_restores_crop_review_store_from_reserved_embedded_attachment() {
        let expected = vec![crop("crop-0001", 1), crop("crop-0002", 2)];
        let bytes = embedded_fixture_with_crop_review(&expected);

        assert!(attachment_keys(&bytes).contains(&EMBEDDED_CROP_REVIEW_ATTACHMENT_NAME.to_owned()));

        let mut session = crate::Session::new().with_clock(|| 1_700_000_000);
        let id = session.open("embedded.pdf", None, bytes, None).unwrap();

        assert_eq!(session.crop_review(id).unwrap().items(), expected.as_slice());
    }

    #[test]
    fn export_save_and_reopen_preserve_reserved_embedded_crop_review_attachment() {
        let expected = crop("crop-0001", 1);
        let (mut session, id) = session_with(2);

        // Crop-review mutations alone do not force a save yet, so pair the persistence check with
        // one ordinary document edit and assert that the embedded mirror survives the round trip.
        session.apply(id, crate::Edit::SetInfo { key: "Title".into(), value: "Crop review".into() }).unwrap();
        session.crop_review_mut(id).unwrap().upsert(expected.clone()).unwrap();

        let saved = session.save_bytes(id).unwrap();
        assert!(attachment_keys(&saved).contains(&EMBEDDED_CROP_REVIEW_ATTACHMENT_NAME.to_owned()));

        session.mark_saved(id, saved.clone(), None).unwrap();
        assert_eq!(session.crop_review(id).unwrap().items(), std::slice::from_ref(&expected));

        let mut reopened = crate::Session::new().with_clock(|| 1_700_000_000);
        let reopened_id = reopened.open("saved.pdf", None, saved, None).unwrap();
        assert_eq!(reopened.crop_review(reopened_id).unwrap().items(), &[expected]);
    }

    #[test]
    fn export_save_full_removes_reserved_embedded_attachment_when_crop_review_store_is_empty() {
        let existing = crop("crop-0001", 1);
        let bytes = embedded_fixture_with_crop_review(std::slice::from_ref(&existing));
        let mut session = crate::Session::new().with_clock(|| 1_700_000_000);
        let id = session.open("embedded.pdf", None, bytes, None).unwrap();

        assert_eq!(session.crop_review(id).unwrap().items(), std::slice::from_ref(&existing));
        assert!(session.crop_review_mut(id).unwrap().remove(&existing.id).is_some());
        assert!(session.crop_review(id).unwrap().items().is_empty());

        let saved = session.save_full_bytes(id).unwrap();
        assert!(!attachment_keys(&saved).contains(&EMBEDDED_CROP_REVIEW_ATTACHMENT_NAME.to_owned()));

        session.mark_saved(id, saved.clone(), None).unwrap();
        assert!(session.crop_review(id).unwrap().items().is_empty());

        let mut reopened = crate::Session::new().with_clock(|| 1_700_000_000);
        let reopened_id = reopened.open("saved.pdf", None, saved, None).unwrap();
        assert!(reopened.crop_review(reopened_id).unwrap().items().is_empty());
    }

    #[test]
    fn export_unchanged_documents_without_crop_review_do_not_gain_reserved_embedded_attachment() {
        let (session, id) = session_with(1);
        let original = session.get(id).unwrap().bytes.clone();

        assert!(session.crop_review(id).unwrap().items().is_empty());

        let saved = session.save_bytes(id).unwrap();
        assert_eq!(saved, original);
        assert!(!attachment_keys(&saved).contains(&EMBEDDED_CROP_REVIEW_ATTACHMENT_NAME.to_owned()));

        let mut reopened = crate::Session::new().with_clock(|| 1_700_000_000);
        let reopened_id = reopened.open("saved.pdf", None, saved, None).unwrap();
        assert!(reopened.crop_review(reopened_id).unwrap().items().is_empty());
    }
}

#[cfg(test)]
mod snapshot_export_artifact_tests {
    use serde_json::Value;

    use super::{
        ExportRect, SnapshotExportItem, SnapshotExportItemInput, SnapshotExportMetadata, SnapshotExportOcr, normalize_snapshot_export_items,
        snapshot_export_manifest_json, snapshot_export_report_tex,
    };

    fn metadata(caption: &str, note: Option<&str>) -> SnapshotExportMetadata {
        SnapshotExportMetadata { caption: caption.into(), note: note.map(str::to_owned) }
    }

    fn sample_items() -> Vec<SnapshotExportItem> {
        normalize_snapshot_export_items(
            4,
            vec![
                SnapshotExportItemInput {
                    order: 20,
                    page: 4,
                    rect: ExportRect { x: 90.5, y: 244.0, width: 180.0, height: 110.0 },
                    metadata: metadata("Comment {callout} _ review", None),
                    ocr: Some(SnapshotExportOcr { text: "Recognized crop text two".into() }),
                    allow_ocr_fill: true,
                },
                SnapshotExportItemInput {
                    order: 10,
                    page: 2,
                    rect: ExportRect { x: 72.0, y: 128.0, width: 216.0, height: 96.0 },
                    metadata: metadata("50% approved & signed #1", Some(r"Path C:\exports\item_{1} uses ~ temp ^ dir and costs $5.")),
                    ocr: Some(SnapshotExportOcr { text: "Recognized crop text one".into() }),
                    allow_ocr_fill: true,
                },
            ],
        )
        .unwrap()
    }

    #[test]
    fn manifest_json_matches_the_v1_contract_golden_output() {
        let manifest = snapshot_export_manifest_json(&sample_items()).unwrap();

        let expected = concat!(
            "{\n",
            "  \"package_kind\": \"pdfcraft.snapshot_export\",\n",
            "  \"manifest_version\": \"1.1\",\n",
            "  \"source_pdf\": {\n",
            "    \"path\": \"source.pdf\",\n",
            "    \"state\": \"working-copy\"\n",
            "  },\n",
            "  \"coordinate_space\": {\n",
            "    \"page_origin\": \"top-left\",\n",
            "    \"page_space\": \"display\",\n",
            "    \"units\": \"pt\",\n",
            "    \"page_rotation\": \"already-applied\"\n",
            "  },\n",
            "  \"rendering\": {\n",
            "    \"snapshot_source\": \"display-equivalent\",\n",
            "    \"image_format\": \"png\",\n",
            "    \"pixels_per_point\": 2.0\n",
            "  },\n",
            "  \"items\": [\n",
            "    {\n",
            "      \"id\": \"item_1\",\n",
            "      \"kind\": \"snapshot\",\n",
            "      \"page\": 2,\n",
            "      \"rect\": {\n",
            "        \"x\": 72.0,\n",
            "        \"y\": 128.0,\n",
            "        \"width\": 216.0,\n",
            "        \"height\": 96.0\n",
            "      },\n",
            "      \"image\": {\n",
            "        \"path\": \"evidence/item_1.png\",\n",
            "        \"width_px\": 432,\n",
            "        \"height_px\": 192\n",
            "      },\n",
            "      \"caption\": \"50% approved & signed #1\",\n",
            "      \"note\": \"Path C:\\\\exports\\\\item_{1} uses ~ temp ^ dir and costs $5.\",\n",
            "      \"ocr_text\": \"Recognized crop text one\"\n",
            "    },\n",
            "    {\n",
            "      \"id\": \"item_2\",\n",
            "      \"kind\": \"snapshot\",\n",
            "      \"page\": 4,\n",
            "      \"rect\": {\n",
            "        \"x\": 90.5,\n",
            "        \"y\": 244.0,\n",
            "        \"width\": 180.0,\n",
            "        \"height\": 110.0\n",
            "      },\n",
            "      \"image\": {\n",
            "        \"path\": \"evidence/item_2.png\",\n",
            "        \"width_px\": 360,\n",
            "        \"height_px\": 220\n",
            "      },\n",
            "      \"caption\": \"Comment {callout} _ review\",\n",
            "      \"ocr_text\": \"Recognized crop text two\"\n",
            "    }\n",
            "  ]\n",
            "}\n"
        );

        assert_eq!(manifest, expected);
    }

    #[test]
    fn report_tex_matches_the_snapshot_template_golden_output() {
        let tex = snapshot_export_report_tex(&sample_items()).unwrap();

        let expected = concat!(
            "\\documentclass{article}\n",
            "\\usepackage[T1]{fontenc}\n",
            "\\usepackage[utf8]{inputenc}\n",
            "\\usepackage{graphicx}\n",
            "\\usepackage{caption}\n",
            "\\usepackage{csquotes}\n",
            "\\begin{document}\n\n",
            "\\begin{enumerate}\n",
            "    \\item Conforme a Figura~\\ref{fig:item_1}, Path C:\\textbackslash{}exports\\textbackslash{}item\\_\\{1\\} uses \\~{} temp \\^{} dir and costs \\$5..\n\n",
            "\\noindent\\begin{minipage}[t]{\\linewidth}\n",
            "    \\vspace{0pt}\n",
            "    \\centering\n",
            "    \\fbox{\\includegraphics[width=\\linewidth]{evidence/item_1.png}}\n",
            "    \\captionsetup{justification=centering}\n",
            "    \\captionof{figure}{50\\% approved \\& signed \\#1}\n",
            "    \\label{fig:item_1}\n",
            "\\end{minipage}\n\n",
            "\\noindent OCR.: Recognized crop text one\n\n",
            "    \\item Conforme a Figura~\\ref{fig:item_2}, Comment \\{callout\\} \\_ review.\n\n",
            "\\noindent\\begin{minipage}[t]{\\linewidth}\n",
            "    \\vspace{0pt}\n",
            "    \\centering\n",
            "    \\fbox{\\includegraphics[width=\\linewidth]{evidence/item_2.png}}\n",
            "    \\captionsetup{justification=centering}\n",
            "    \\captionof{figure}{Comment \\{callout\\} \\_ review}\n",
            "    \\label{fig:item_2}\n",
            "\\end{minipage}\n\n",
            "\\noindent OCR.: Recognized crop text two\n\n",
            "\\end{enumerate}\n\n",
            "\\end{document}\n"
        );

        assert_eq!(tex, expected);
    }

    #[test]
    fn manifest_items_and_latex_blocks_stay_aligned() {
        let items = sample_items();
        let manifest = snapshot_export_manifest_json(&items).unwrap();
        let tex = snapshot_export_report_tex(&items).unwrap();

        let parsed: Value = serde_json::from_str(&manifest).unwrap();
        let manifest_items = parsed["items"].as_array().unwrap();
        assert_eq!(manifest_items.len(), items.len());
        assert_eq!(tex.matches("\\begin{minipage}[t]{\\linewidth}").count(), items.len());

        for manifest_item in manifest_items {
            let id = manifest_item["id"].as_str().unwrap();
            let path = manifest_item["image"]["path"].as_str().unwrap();
            assert!(tex.contains(&format!("\\label{{fig:{id}}}")), "{id}");
            assert!(tex.contains(&format!("\\fbox{{\\includegraphics[width=\\linewidth]{{{path}}}}}")), "{id}");
        }
    }

    #[test]
    fn report_tex_includes_an_ocr_paragraph_when_present() {
        let items = normalize_snapshot_export_items(
            1,
            vec![SnapshotExportItemInput {
                order: 0,
                page: 1,
                rect: ExportRect { x: 10.0, y: 20.0, width: 30.0, height: 40.0 },
                metadata: metadata("Caption", Some("Note text")),
                ocr: Some(SnapshotExportOcr { text: "Derived OCR text".into() }),
                allow_ocr_fill: true,
            }],
        )
        .unwrap();

        let tex = snapshot_export_report_tex(&items).unwrap();
        assert!(tex.contains("\\noindent OCR.: Derived OCR text\n"), "{tex}");
    }
}

#[cfg(test)]
mod snapshot_export_package_tests {
    use std::sync::Arc;

    use pdfcraft_render::inspect;

    use super::{
        ExportRect, ExportSource, SnapshotExportItemInput, SnapshotExportMetadata, SnapshotExportPackageInput, SnapshotExportWriteHooks,
        normalize_snapshot_export_items, populate_snapshot_export_ocr, write_snapshot_export_package, write_snapshot_export_package_with_hooks,
    };

    fn metadata(caption: &str, note: Option<&str>) -> SnapshotExportMetadata {
        SnapshotExportMetadata { caption: caption.into(), note: note.map(str::to_owned) }
    }

    struct TestDir {
        path: std::path::PathBuf,
    }

    impl TestDir {
        fn new(name: &str) -> Self {
            let path = std::env::temp_dir().join(format!("pdfcraft-engine-export-{name}-{}-{}", std::process::id(), unique_suffix()));
            std::fs::create_dir_all(&path).unwrap();
            Self { path }
        }

        fn child(&self, name: &str) -> std::path::PathBuf {
            self.path.join(name)
        }
    }

    impl Drop for TestDir {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.path);
        }
    }

    fn unique_suffix() -> u64 {
        use std::sync::atomic::{AtomicU64, Ordering};
        static NEXT: AtomicU64 = AtomicU64::new(1);
        NEXT.fetch_add(1, Ordering::Relaxed)
    }

    fn source(bodies: &[&str], boxes: &[[f32; 4]], rotates: &[u16]) -> ExportSource {
        assert_eq!(bodies.len(), boxes.len());
        assert_eq!(bodies.len(), rotates.len());

        let kids = (0..bodies.len()).map(|i| format!("{} 0 R", 4 + i * 2)).collect::<Vec<_>>().join(" ");
        let mut objects = vec![
            "<< /Type /Catalog /Pages 2 0 R >>".to_owned(),
            format!("<< /Type /Pages /Kids [{kids}] /Count {} >>", bodies.len()),
            "<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica >>".to_owned(),
        ];

        let mut sizes = Vec::with_capacity(bodies.len());
        for (i, (body, crop)) in bodies.iter().zip(boxes.iter()).enumerate() {
            let rotate = rotates[i];
            let media = format!("[{} {} {} {}]", crop[0], crop[1], crop[2], crop[3]);
            objects.push(format!(
                "<< /Type /Page /Parent 2 0 R /MediaBox {media} /CropBox {media} /Rotate {rotate} /Contents {} 0 R /Resources << /Font << /F1 3 0 R >> >> >>",
                5 + i * 2
            ));
            objects.push(format!("<< /Length {} >>\nstream\n{body}\nendstream", body.len()));
            let width = crop[2] - crop[0];
            let height = crop[3] - crop[1];
            let size = if rotate.is_multiple_of(180) { (width, height) } else { (height, width) };
            sizes.push(size);
        }

        let mut bytes = b"%PDF-1.7\n".to_vec();
        let mut offsets = Vec::new();
        for (i, object) in objects.iter().enumerate() {
            offsets.push(bytes.len());
            bytes.extend_from_slice(format!("{} 0 obj\n{object}\nendobj\n", i + 1).as_bytes());
        }
        let xref = bytes.len();
        bytes.extend_from_slice(format!("xref\n0 {}\n0000000000 65535 f \n", objects.len() + 1).as_bytes());
        for offset in offsets {
            bytes.extend_from_slice(format!("{offset:010} 00000 n \n").as_bytes());
        }
        bytes.extend_from_slice(format!("trailer\n<< /Root 1 0 R /Size {} >>\nstartxref\n{xref}\n%%EOF\n", objects.len() + 1).as_bytes());

        ExportSource { bytes: Arc::new(bytes), config: Default::default(), pages: bodies.len(), sizes }
    }

    fn package_input(items: Vec<super::SnapshotExportItem>, source: ExportSource) -> SnapshotExportPackageInput {
        SnapshotExportPackageInput {
            items,
            source_pdf_bytes: source.bytes.clone(),
            render_source: source,
            title: "Export test".into(),
            include_word_report: false,
        }
    }

    fn png_size(path: &std::path::Path) -> (u32, u32) {
        let bytes = std::fs::read(path).unwrap();
        let reader = png::Decoder::new(std::io::Cursor::new(bytes)).read_info().unwrap();
        let info = reader.info();
        (info.width, info.height)
    }

    fn png_pixels(path: &std::path::Path) -> (png::ColorType, Vec<u8>) {
        let bytes = std::fs::read(path).unwrap();
        let mut reader = png::Decoder::new(std::io::Cursor::new(bytes)).read_info().unwrap();
        let mut pixels = vec![0; reader.output_buffer_size().unwrap()];
        let frame = reader.next_frame(&mut pixels).unwrap();
        pixels.truncate(frame.buffer_size());
        (frame.color_type, pixels)
    }

    #[test]
    fn one_item_package_export_creates_expected_files_and_content() {
        let source = source(&["0 0 0 rg 20 30 40 50 re f"], &[[0.0, 0.0, 200.0, 300.0]], &[0]);
        let items = normalize_snapshot_export_items(
            source.pages,
            vec![SnapshotExportItemInput {
                order: 0,
                page: 1,
                rect: ExportRect { x: 20.0, y: 30.0, width: 40.0, height: 50.0 },
                metadata: metadata("Black box", Some("One item")),
                ocr: None,
                allow_ocr_fill: true,
            }],
        )
        .unwrap();

        let dir = TestDir::new("one-item");
        let target = dir.child("package");
        let input = package_input(items.clone(), source.clone());

        write_snapshot_export_package(&target, input).unwrap();

        assert!(target.is_dir());
        assert!(target.join("manifest.json").is_file());
        assert!(target.join("source.pdf").is_file());
        assert!(target.join("report.tex").is_file());
        assert!(target.join("evidence").is_dir());
        assert!(target.join("evidence/item_1.png").is_file());
        assert_eq!(std::fs::read(target.join("source.pdf")).unwrap(), source.bytes.as_slice());

        let manifest = std::fs::read_to_string(target.join("manifest.json")).unwrap();
        assert!(manifest.contains("\"path\": \"source.pdf\""));
        assert!(manifest.contains("\"path\": \"evidence/item_1.png\""));

        let report = std::fs::read_to_string(target.join("report.tex")).unwrap();
        assert!(report.contains("\\includegraphics[width=\\linewidth]{evidence/item_1.png}"));
        assert!(report.contains("\\label{fig:item_1}"));

        assert_eq!(png_size(&target.join("evidence/item_1.png")), (80, 100));
    }

    #[test]
    fn multi_item_export_across_pages_keeps_manifest_latex_and_images_aligned() {
        let source =
            source(&["0 0 0 rg 10 240 40 30 re f", "0 0 0 rg 90 110 50 40 re f"], &[[0.0, 0.0, 200.0, 300.0], [0.0, 0.0, 200.0, 300.0]], &[0, 0]);
        let items = normalize_snapshot_export_items(
            source.pages,
            vec![
                SnapshotExportItemInput {
                    order: 20,
                    page: 2,
                    rect: ExportRect { x: 90.0, y: 110.0, width: 50.0, height: 40.0 },
                    metadata: metadata("Second page", None),
                    ocr: None,
                    allow_ocr_fill: true,
                },
                SnapshotExportItemInput {
                    order: 10,
                    page: 1,
                    rect: ExportRect { x: 10.0, y: 30.0, width: 40.0, height: 30.0 },
                    metadata: metadata("First page", Some("ordered first")),
                    ocr: None,
                    allow_ocr_fill: true,
                },
            ],
        )
        .unwrap();

        let dir = TestDir::new("multi-item");
        let target = dir.child("package");
        write_snapshot_export_package(&target, package_input(items.clone(), source)).unwrap();

        let manifest: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(target.join("manifest.json")).unwrap()).unwrap();
        let items_json = manifest["items"].as_array().unwrap();
        assert_eq!(items_json.len(), 2);
        assert_eq!(items_json[0]["id"], "item_1");
        assert_eq!(items_json[0]["page"], 1);
        assert_eq!(items_json[0]["image"]["path"], "evidence/item_1.png");
        assert_eq!(items_json[1]["id"], "item_2");
        assert_eq!(items_json[1]["page"], 2);
        assert_eq!(items_json[1]["image"]["path"], "evidence/item_2.png");

        let report = std::fs::read_to_string(target.join("report.tex")).unwrap();
        assert!(report.contains("\\label{fig:item_1}"));
        assert!(report.contains("\\label{fig:item_2}"));
        assert!(report.contains("\\includegraphics[width=\\linewidth]{evidence/item_1.png}"));
        assert!(report.contains("\\includegraphics[width=\\linewidth]{evidence/item_2.png}"));

        assert_eq!(png_size(&target.join("evidence/item_1.png")), (80, 60));
        assert_eq!(png_size(&target.join("evidence/item_2.png")), (100, 80));
    }

    #[test]
    fn package_export_can_include_a_word_report() {
        let source = source(&["0 0 0 rg 20 30 40 50 re f"], &[[0.0, 0.0, 200.0, 300.0]], &[0]);
        let items = normalize_snapshot_export_items(
            source.pages,
            vec![SnapshotExportItemInput {
                order: 0,
                page: 1,
                rect: ExportRect { x: 20.0, y: 30.0, width: 40.0, height: 50.0 },
                metadata: metadata("Black box", Some("One item")),
                ocr: None,
                allow_ocr_fill: true,
            }],
        )
        .unwrap();

        let dir = TestDir::new("one-item-word");
        let target = dir.child("package");
        let mut input = package_input(items, source);
        input.include_word_report = true;

        write_snapshot_export_package(&target, input).unwrap();

        let docx = std::fs::read(target.join("report.docx")).unwrap();
        assert!(docx.windows(b"word/document.xml".len()).any(|w| w == b"word/document.xml"));
        assert!(docx.windows(b"word/media/image1.png".len()).any(|w| w == b"word/media/image1.png"));
    }

    #[test]
    fn rotated_page_crop_behavior_uses_display_space_after_rotation() {
        let source = source(&["1 0 0 rg 20 40 60 30 re f 0 0 1 rg 120 40 60 30 re f"], &[[0.0, 0.0, 200.0, 100.0]], &[90]);
        let info = inspect(source.bytes.clone(), None).unwrap();
        assert_eq!((info.pages[0].width, info.pages[0].height), (100.0, 200.0));

        let items = normalize_snapshot_export_items(
            source.pages,
            vec![SnapshotExportItemInput {
                order: 0,
                page: 1,
                rect: ExportRect { x: 40.0, y: 20.0, width: 30.0, height: 60.0 },
                metadata: metadata("Rotated red band", None),
                ocr: None,
                allow_ocr_fill: true,
            }],
        )
        .unwrap();

        let dir = TestDir::new("rotated");
        let target = dir.child("package");
        write_snapshot_export_package(&target, package_input(items, source)).unwrap();

        let image = target.join("evidence/item_1.png");
        assert_eq!(png_size(&image), (60, 120));

        let (color, pixels) = png_pixels(&image);
        assert_eq!(color, png::ColorType::Rgb);
        let mut red = 0usize;
        let mut blue = 0usize;
        for px in pixels.as_chunks::<3>().0 {
            if px[0] > 200 && px[1] < 80 && px[2] < 80 {
                red += 1;
            }
            if px[2] > 200 && px[0] < 80 && px[1] < 80 {
                blue += 1;
            }
        }
        assert!(red > 1000, "the rotated display-space crop should capture the red band");
        assert_eq!(blue, 0, "the crop should exclude the blue band");
    }

    #[test]
    fn injected_write_failure_leaves_no_published_final_package_directory() {
        let source = source(&["0 0 0 rg 20 30 40 50 re f"], &[[0.0, 0.0, 200.0, 300.0]], &[0]);
        let items = normalize_snapshot_export_items(
            source.pages,
            vec![SnapshotExportItemInput {
                order: 0,
                page: 1,
                rect: ExportRect { x: 20.0, y: 30.0, width: 40.0, height: 50.0 },
                metadata: metadata("Black box", None),
                ocr: None,
                allow_ocr_fill: true,
            }],
        )
        .unwrap();

        let dir = TestDir::new("write-fail");
        let target = dir.child("package");
        let err = write_snapshot_export_package_with_hooks(
            &target,
            package_input(items, source),
            &SnapshotExportWriteHooks::fail_before_write("evidence/item_1.png"),
        )
        .unwrap_err();

        assert!(err.contains("injected snapshot export write failure"));
        assert!(!target.exists(), "no false-success package directory is published");

        let names = std::fs::read_dir(&dir.path).unwrap().map(|entry| entry.unwrap().file_name().to_string_lossy().into_owned()).collect::<Vec<_>>();
        assert_eq!(names, Vec::<String>::new(), "staging directories are removed after failure");
    }

    #[test]
    fn invalid_rect_refuses_before_any_package_is_published() {
        let source = source(&["0 0 0 rg 20 30 40 50 re f"], &[[0.0, 0.0, 200.0, 300.0]], &[0]);
        let items = normalize_snapshot_export_items(
            source.pages,
            vec![SnapshotExportItemInput {
                order: 0,
                page: 1,
                rect: ExportRect { x: 180.0, y: 30.0, width: 40.0, height: 50.0 },
                metadata: metadata("Out of bounds", None),
                ocr: None,
                allow_ocr_fill: true,
            }],
        )
        .unwrap();

        let dir = TestDir::new("bad-rect");
        let target = dir.child("package");
        let err = write_snapshot_export_package(&target, package_input(items, source)).unwrap_err();

        assert!(err.contains("exceeds page bounds"));
        assert!(!target.exists(), "invalid exports must not leave a published directory");
    }

    #[test]
    fn ocr_population_follows_model_availability_rule() {
        let source = source(&["BT /F1 24 Tf 20 150 Td (Page 1) Tj ET"], &[[0.0, 0.0, 200.0, 300.0]], &[0]);
        let mut items = normalize_snapshot_export_items(
            source.pages,
            vec![SnapshotExportItemInput {
                order: 0,
                page: 1,
                rect: ExportRect { x: 10.0, y: 120.0, width: 110.0, height: 50.0 },
                metadata: metadata("OCR crop", None),
                ocr: None,
                allow_ocr_fill: true,
            }],
        )
        .unwrap();

        // The rectangle covers native page text, which is preferred over bitmap OCR (17649c1b), so the
        // text is filled with or without OCR models; models only matter for regions without a text layer.
        let result = populate_snapshot_export_ocr(&source, &mut items);
        assert!(result.is_ok(), "{result:?}");
        let text = items[0].ocr.as_ref().map(|ocr| ocr.text.clone()).unwrap_or_default();
        assert!(text.contains("Page 1"), "{text:?}");
    }

    #[test]
    fn ocr_population_prefers_native_page_text_inside_the_selected_rectangle() {
        let source = source(&["BT /F1 24 Tf 20 150 Td (Thermal bow result) Tj ET"], &[[0.0, 0.0, 200.0, 300.0]], &[0]);
        let mut items = normalize_snapshot_export_items(
            source.pages,
            vec![SnapshotExportItemInput {
                order: 0,
                page: 1,
                rect: ExportRect { x: 15.0, y: 130.0, width: 170.0, height: 30.0 },
                metadata: metadata("Text crop", None),
                ocr: None,
                allow_ocr_fill: true,
            }],
        )
        .unwrap();

        populate_snapshot_export_ocr(&source, &mut items).unwrap();

        assert_eq!(items[0].ocr.as_ref().map(|ocr| ocr.text.as_str()), Some("Thermal bow result"));
    }

    #[test]
    fn ocr_population_skips_fill_when_item_disallows_it() {
        let source = source(&["BT /F1 24 Tf 20 150 Td (Thermal bow result) Tj ET"], &[[0.0, 0.0, 200.0, 300.0]], &[0]);
        let mut items = normalize_snapshot_export_items(
            source.pages,
            vec![SnapshotExportItemInput {
                order: 0,
                page: 1,
                rect: ExportRect { x: 15.0, y: 130.0, width: 170.0, height: 30.0 },
                metadata: metadata("Text crop", None),
                ocr: None,
                allow_ocr_fill: false,
            }],
        )
        .unwrap();

        populate_snapshot_export_ocr(&source, &mut items).unwrap();

        assert!(items[0].ocr.is_none());
    }
}

#[cfg(test)]
mod raster_tests {
    use super::{ImageFormat, encode_image, encode_png};

    const FORMATS: [ImageFormat; 3] = [ImageFormat::Png, ImageFormat::Jpeg { quality: 90 }, ImageFormat::Tiff];

    /// The colour type and pixels of a PNG.
    fn decode(png: &[u8]) -> (png::ColorType, Vec<u8>) {
        let mut reader = png::Decoder::new(std::io::Cursor::new(png)).read_info().unwrap();
        let mut pixels = vec![0; reader.output_buffer_size().unwrap()];
        let frame = reader.next_frame(&mut pixels).unwrap();
        pixels.truncate(frame.buffer_size());
        (frame.color_type, pixels)
    }

    /// A rendered page has no translucent pixel: it is written as RGB, each pixel exactly as
    /// rendered (the alpha channel would only make the file bigger and slower to write).
    #[test]
    fn opaque_rasters_are_written_as_rgb_with_their_exact_pixels() {
        let rgba = [[255, 255, 255, 255], [0, 0, 0, 255], [200, 30, 90, 255], [1, 2, 3, 255], [17, 250, 128, 255], [90, 90, 90, 255]];
        let (color, pixels) = decode(&encode_png(3, 2, rgba.as_flattened()).unwrap());
        assert_eq!(color, png::ColorType::Rgb);
        assert_eq!(pixels, rgba.iter().flat_map(|p| [p[0], p[1], p[2]]).collect::<Vec<_>>());
    }

    /// A raster with translucent pixels keeps its alpha, with the colours un-premultiplied.
    #[test]
    fn translucent_rasters_keep_straight_alpha() {
        let rgba = [[64, 0, 32, 128], [0, 0, 0, 0], [10, 20, 30, 255]];
        let (color, pixels) = decode(&encode_png(3, 1, rgba.as_flattened()).unwrap());
        assert_eq!(color, png::ColorType::Rgba);
        assert_eq!(pixels, [128, 0, 64, 128, 0, 0, 0, 0, 10, 20, 30, 255]);
    }

    /// A buffer that is not exactly `width × height` RGBA pixels is an error in every format,
    /// opaque or not. A partial trailing pixel used to be dropped and the rest written as a
    /// valid image, and a buffer too short for its size panicked in the JPEG and TIFF encoders.
    #[test]
    fn rasters_of_the_wrong_length_are_rejected() {
        let cases: [(&str, u32, u32, &[u8]); 7] = [
            ("opaque, a partial pixel after", 1, 1, &[10, 20, 30, 255, 0]),
            ("opaque, three bytes after", 1, 1, &[10, 20, 30, 255, 0, 0, 0]),
            ("translucent, a partial pixel after", 1, 1, &[10, 20, 30, 128, 0]),
            ("opaque, a pixel short", 2, 1, &[10, 20, 30, 255]),
            ("translucent, a pixel too many", 1, 1, &[10, 20, 30, 128, 1, 2, 3, 4]),
            ("empty", 1, 1, &[]),
            ("a size whose byte count overflows", u32::MAX, u32::MAX, &[10, 20, 30, 255]),
        ];
        for format in FORMATS {
            for (what, width, height, data) in cases {
                assert!(encode_image(width, height, data, format, 72.0).is_err(), "{}, {what}", format.label());
            }
        }
    }

    /// A raster of no pixels is the right length (no bytes), and refused before any encoder sees
    /// it, in every format.
    #[test]
    fn rasters_of_no_pixels_are_an_error() {
        for format in FORMATS {
            for (width, height) in [(0, 0), (0, 3), (3, 0)] {
                let refused = encode_image(width, height, &[], format, 72.0).err();
                assert_eq!(refused, Some(format!("a {width}×{height} raster has no pixels")), "{}", format.label());
            }
        }
    }
}
