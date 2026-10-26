//! Export a PDF ▸ Image and Text (execution plan M10.3, the first formats).
//!
//! [`Exporter`] renders pages to PNG, JPEG or TIFF at a resolution and extracts reading-order
//! text, from a
//! document's working file (so unsaved edits and hidden layers are respected, as on screen).

use std::path::{Component, Path, PathBuf};
use std::sync::Arc;

use pdfcraft_render::{PageRenderer, RenderRequest, RequestKind, Tile};

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

/// Raw snapshot export data before canonical ordering, IDs and image paths are assigned.
#[derive(Clone, Debug, PartialEq)]
pub struct SnapshotExportItemInput {
    pub order: usize,
    /// 1-based page number in displayed-page space.
    pub page: usize,
    pub rect: ExportRect,
    pub caption: String,
    pub note: Option<String>,
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

/// One normalized item in the canonical export order.
#[derive(Clone, Debug, PartialEq)]
pub struct SnapshotExportItem {
    pub id: String,
    pub kind: ExportItemKind,
    /// 1-based page number in displayed-page space.
    pub page: usize,
    pub rect: ExportRect,
    pub image_path: String,
    pub caption: String,
    pub note: Option<String>,
}

/// End-to-end inputs for writing one snapshot export package.
#[derive(Clone)]
pub struct SnapshotExportPackageInput {
    pub items: Vec<SnapshotExportItem>,
    /// Display-equivalent bytes/config for crop rendering.
    pub render_source: ExportSource,
    /// Working-file bytes for `source.pdf`.
    pub source_pdf_bytes: Arc<Vec<u8>>,
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
        caption: selected_comment_caption(annotation)?,
        note: selected_comment_note(annotation.contents.as_deref()),
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
            caption: item.caption,
            note: item.note,
        });
    }
    Ok(normalized)
}

const SNAPSHOT_EXPORT_PACKAGE_KIND: &str = "pdfcraft.snapshot_export";
const SNAPSHOT_EXPORT_MANIFEST_VERSION: &str = "1.0";
const SNAPSHOT_EXPORT_MANIFEST_PATH: &str = "manifest.json";
const SNAPSHOT_EXPORT_REPORT_PATH: &str = "report.tex";
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

fn snapshot_export_tile_axis(start: f32, span: f32) -> Result<(u32, u32), String> {
    if !start.is_finite() || !span.is_finite() {
        return Err("snapshot rectangle coordinates must be finite".into());
    }
    let origin = (f64::from(start) * f64::from(SNAPSHOT_EXPORT_PIXELS_PER_POINT)).floor();
    let end = (f64::from(start + span) * f64::from(SNAPSHOT_EXPORT_PIXELS_PER_POINT)).ceil();
    if !origin.is_finite() || !end.is_finite() || origin < 0.0 || end <= origin || end > f64::from(u32::MAX) {
        return Err("snapshot image dimensions are out of range".into());
    }
    let origin = origin as u32;
    let end = end as u32;
    Ok((origin, end - origin))
}

fn snapshot_export_render_tile(rect: ExportRect) -> Result<Tile, String> {
    let rect = rect.validate()?;
    let (x, w) = snapshot_export_tile_axis(rect.x, rect.width)?;
    let (y, h) = snapshot_export_tile_axis(rect.y, rect.height)?;
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
        out.push_str(&json_string(&item.caption)?);
        if let Some(note) = item.note.as_deref() {
            out.push_str(",\n");
            out.push_str("      \"note\": ");
            out.push_str(&json_string(note)?);
            out.push('\n');
        } else {
            out.push('\n');
        }
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

    let mut out = String::from("\\section*{Snapshot Evidence}\n\n");
    for (index, item) in items.iter().enumerate() {
        out.push_str("\\begin{figure}[htbp]\n");
        out.push_str("\\centering\n");
        out.push_str(&format!("\\includegraphics[width=\\linewidth]{{{}}}\n", item.image_path));
        out.push_str(&format!("\\caption{{{}}}\n", latex_escape_plain_text(&item.caption)));
        out.push_str(&format!("\\label{{fig:{}}}\n", item.id));
        out.push_str("\\end{figure}\n");

        if let Some(note) = item.note.as_deref() {
            out.push('\n');
            out.push_str("\\paragraph{Note.}\n");
            out.push_str(&latex_escape_plain_text(note));
            out.push('\n');
        }

        if index + 1 != items.len() {
            out.push('\n');
        }
    }

    Ok(out)
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

fn resolve_snapshot_export_path(root: &Path, relative: &str) -> Result<PathBuf, String> {
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
) -> Result<(), String> {
    let evidence_dir = resolve_snapshot_export_path(root, SNAPSHOT_EXPORT_EVIDENCE_DIR)?;
    std::fs::create_dir_all(&evidence_dir).map_err(|e| format!("could not create {}: {e}", evidence_dir.display()))?;
    let mut renderer = PageRenderer::new(source.bytes.clone(), source.config.clone());
    for item in items {
        let png = render_snapshot_export_item_png(&mut renderer, source, item)?;
        write_snapshot_export_file(root, &item.image_path, &png, hooks)?;
    }
    Ok(())
}

fn snapshot_export_png_dimensions(bytes: &[u8]) -> Result<(u32, u32), String> {
    let reader = png::Decoder::new(std::io::Cursor::new(bytes)).read_info().map_err(|e| format!("could not read PNG metadata: {e}"))?;
    let info = reader.info();
    Ok((info.width, info.height))
}

fn validate_snapshot_export_package_dir(
    root: &Path,
    source_pdf_bytes: &[u8],
    items: &[SnapshotExportItem],
    manifest_json: &str,
    report_tex: &str,
) -> Result<(), String> {
    if !root.is_dir() {
        return Err(format!("snapshot export package root {} was not created", root.display()));
    }

    let manifest_path = resolve_snapshot_export_path(root, SNAPSHOT_EXPORT_MANIFEST_PATH)?;
    let source_pdf_path = resolve_snapshot_export_path(root, SNAPSHOT_EXPORT_SOURCE_PDF_PATH)?;
    let report_path = resolve_snapshot_export_path(root, SNAPSHOT_EXPORT_REPORT_PATH)?;
    let evidence_dir = resolve_snapshot_export_path(root, SNAPSHOT_EXPORT_EVIDENCE_DIR)?;

    let staged_manifest = std::fs::read_to_string(&manifest_path).map_err(|e| format!("could not read {}: {e}", manifest_path.display()))?;
    if staged_manifest != manifest_json {
        return Err("snapshot export manifest.json does not match the generated manifest".into());
    }

    let staged_report = std::fs::read_to_string(&report_path).map_err(|e| format!("could not read {}: {e}", report_path.display()))?;
    if staged_report != report_tex {
        return Err("snapshot export report.tex does not match the generated report".into());
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
    let stage = create_snapshot_export_staging_dir(target)?;

    let result = (|| {
        write_snapshot_export_file(&stage, SNAPSHOT_EXPORT_MANIFEST_PATH, manifest_json.as_bytes(), hooks)?;
        write_snapshot_export_file(&stage, SNAPSHOT_EXPORT_SOURCE_PDF_PATH, input.source_pdf_bytes.as_slice(), hooks)?;
        write_snapshot_export_file(&stage, SNAPSHOT_EXPORT_REPORT_PATH, report_tex.as_bytes(), hooks)?;
        render_snapshot_export_images(&stage, &input.render_source, &input.items, hooks)?;
        validate_snapshot_export_package_dir(&stage, input.source_pdf_bytes.as_slice(), &input.items, &manifest_json, &report_tex)?;
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
    use super::{ExportItemKind, ExportRect, SnapshotExportItemInput, normalize_package_relative_path, normalize_snapshot_export_items};

    fn input(order: usize, page: usize, caption: &str) -> SnapshotExportItemInput {
        SnapshotExportItemInput { order, page, rect: ExportRect { x: 10.0, y: 20.0, width: 30.0, height: 40.0 }, caption: caption.into(), note: None }
    }

    #[test]
    fn snapshot_items_follow_explicit_order_and_preserve_ties() {
        let items = normalize_snapshot_export_items(3, vec![input(30, 3, "third"), input(10, 2, "first"), input(10, 1, "second")]).unwrap();

        assert_eq!(items.iter().map(|item| item.caption.as_str()).collect::<Vec<_>>(), vec!["first", "second", "third"]);
        assert_eq!(items.iter().map(|item| item.page).collect::<Vec<_>>(), vec![2, 1, 3]);
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
                caption: "bad rect".into(),
                note: None,
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
                caption: "bad rect".into(),
                note: None,
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
                caption: "bad rect".into(),
                note: None,
            }],
        )
        .unwrap_err();
        assert_eq!(non_finite, "snapshot rectangle coordinates must be finite");
    }
}

#[cfg(test)]
mod snapshot_export_artifact_tests {
    use serde_json::Value;

    use super::{
        ExportRect, SnapshotExportItem, SnapshotExportItemInput, normalize_snapshot_export_items, snapshot_export_manifest_json,
        snapshot_export_report_tex,
    };

    fn sample_items() -> Vec<SnapshotExportItem> {
        normalize_snapshot_export_items(
            4,
            vec![
                SnapshotExportItemInput {
                    order: 20,
                    page: 4,
                    rect: ExportRect { x: 90.5, y: 244.0, width: 180.0, height: 110.0 },
                    caption: "Comment {callout} _ review".into(),
                    note: None,
                },
                SnapshotExportItemInput {
                    order: 10,
                    page: 2,
                    rect: ExportRect { x: 72.0, y: 128.0, width: 216.0, height: 96.0 },
                    caption: "50% approved & signed #1".into(),
                    note: Some(r"Path C:\exports\item_{1} uses ~ temp ^ dir and costs $5.".into()),
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
            "  \"manifest_version\": \"1.0\",\n",
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
            "      \"note\": \"Path C:\\\\exports\\\\item_{1} uses ~ temp ^ dir and costs $5.\"\n",
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
            "      \"caption\": \"Comment {callout} _ review\"\n",
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
            "\\section*{Snapshot Evidence}\n\n",
            "\\begin{figure}[htbp]\n",
            "\\centering\n",
            "\\includegraphics[width=\\linewidth]{evidence/item_1.png}\n",
            "\\caption{50\\% approved \\& signed \\#1}\n",
            "\\label{fig:item_1}\n",
            "\\end{figure}\n\n",
            "\\paragraph{Note.}\n",
            "Path C:\\textbackslash{}exports\\textbackslash{}item\\_\\{1\\} uses \\~{} temp \\^{} dir and costs \\$5.\n\n",
            "\\begin{figure}[htbp]\n",
            "\\centering\n",
            "\\includegraphics[width=\\linewidth]{evidence/item_2.png}\n",
            "\\caption{Comment \\{callout\\} \\_ review}\n",
            "\\label{fig:item_2}\n",
            "\\end{figure}\n"
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
        assert_eq!(tex.matches("\\begin{figure}[htbp]").count(), items.len());

        for manifest_item in manifest_items {
            let id = manifest_item["id"].as_str().unwrap();
            let path = manifest_item["image"]["path"].as_str().unwrap();
            assert!(tex.contains(&format!("\\label{{fig:{id}}}")), "{id}");
            assert!(tex.contains(&format!("\\includegraphics[width=\\linewidth]{{{path}}}")), "{id}");
        }
    }
}

#[cfg(test)]
mod snapshot_export_package_tests {
    use std::sync::Arc;

    use pdfcraft_render::inspect;

    use super::{
        ExportRect, ExportSource, SnapshotExportItemInput, SnapshotExportPackageInput, SnapshotExportWriteHooks, normalize_snapshot_export_items,
        write_snapshot_export_package, write_snapshot_export_package_with_hooks,
    };

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
        SnapshotExportPackageInput { items, source_pdf_bytes: source.bytes.clone(), render_source: source }
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
                caption: "Black box".into(),
                note: Some("One item".into()),
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
                    caption: "Second page".into(),
                    note: None,
                },
                SnapshotExportItemInput {
                    order: 10,
                    page: 1,
                    rect: ExportRect { x: 10.0, y: 30.0, width: 40.0, height: 30.0 },
                    caption: "First page".into(),
                    note: Some("ordered first".into()),
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
                caption: "Rotated red band".into(),
                note: None,
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
                caption: "Black box".into(),
                note: None,
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
                caption: "Out of bounds".into(),
                note: None,
            }],
        )
        .unwrap();

        let dir = TestDir::new("bad-rect");
        let target = dir.child("package");
        let err = write_snapshot_export_package(&target, package_input(items, source)).unwrap_err();

        assert!(err.contains("exceeds page bounds"));
        assert!(!target.exists(), "invalid exports must not leave a published directory");
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
