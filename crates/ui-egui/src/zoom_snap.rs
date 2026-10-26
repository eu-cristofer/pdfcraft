//! View ▸ Zoom ▸ Marquee Zoom (drag a rectangle to fill the window with it; click to zoom in)
//! and Edit ▸ Take a Snapshot (drag a rectangle to copy that area as an image).

use chrono::Utc;
use egui::{Color32, CornerRadius, Pos2, Rect, Stroke};
use pdfcraft_engine::{
    Document,
    export::{CropPdfRect, CropReviewItem, ExportItemKind},
};
use pdfcraft_render::{RenderConfig, RenderRequest, RequestKind, Tile};

use crate::canvas::{DocView, PageXform};
use crate::page_pointer::PagePointer;
use crate::{Dialog, PdfCraftApp, QuickTool, RightPanel};

/// A finished gesture: (page, the rectangle on screen, the same in page view points
/// [x0, y0, x1, y1]); a plain click has an empty rectangle.
pub type Marquee = (usize, Rect, [f32; 4]);

/// Shared marquee drag state for Marquee Zoom, Snapshot, and crop-review rectangle creation.
pub(crate) fn marquee_input(
    resp: &egui::Response,
    xf: &PageXform,
    ptr: &PagePointer,
    page: usize,
    marquee: &mut Option<(usize, Pos2)>,
    marquee_done: &mut Option<Marquee>,
) -> bool {
    let (pointer, origin) = (ptr.pos, ptr.origin);
    let mut used = false;

    if resp.drag_started()
        && let Some(o) = origin.filter(|o| xf.rect.contains(*o))
    {
        *marquee = Some((page, o));
        used = true;
    }
    if let Some((p, start)) = *marquee
        && p == page
        && let Some(end) = pointer
    {
        let end = Pos2::new(end.x.clamp(xf.rect.left(), xf.rect.right()), end.y.clamp(xf.rect.top(), xf.rect.bottom()));
        let r = Rect::from_two_pos(start, end);
        if resp.drag_stopped() {
            *marquee = None;
            let (a, b) = (xf.screen_to_view(r.left_top()), xf.screen_to_view(r.right_bottom()));
            *marquee_done = Some((page, r, [a.0.min(b.0), a.1.min(b.1), a.0.max(b.0), a.1.max(b.1)]));
        }
        used = true;
    }
    if !used
        && resp.clicked()
        && let Some(p) = pointer.filter(|p| xf.rect.contains(*p))
    {
        let v = xf.screen_to_view(p);
        *marquee_done = Some((page, Rect::from_min_size(p, egui::Vec2::ZERO), [v.0, v.1, v.0, v.1]));
        used = true;
    }

    used
}

pub(crate) fn paint_marquee(ui: &egui::Ui, xf: &PageXform, pointer: Option<Pos2>, page: usize, marquee: Option<(usize, Pos2)>) {
    if pointer.is_some_and(|p| xf.rect.contains(p)) {
        ui.ctx().set_cursor_icon(egui::CursorIcon::Crosshair);
    }
    if let Some((p, start)) = marquee
        && p == page
        && let Some(end) = pointer
    {
        let end = Pos2::new(end.x.clamp(xf.rect.left(), xf.rect.right()), end.y.clamp(xf.rect.top(), xf.rect.bottom()));
        let r = Rect::from_two_pos(start, end);
        ui.painter().rect_stroke(r, CornerRadius::ZERO, Stroke::new(1.0, Color32::from_rgb(0x14, 0x73, 0xE6)), egui::StrokeKind::Middle);
    }
}

/// Drag a rectangle on page `page` (Marquee Zoom, Snapshot).
pub(crate) fn page_input(ui: &egui::Ui, resp: &egui::Response, xf: &PageXform, ptr: &PagePointer, page: usize, view: &mut DocView) {
    let _ = marquee_input(resp, xf, ptr, page, &mut view.marquee, &mut view.marquee_done);
    paint_marquee(ui, xf, ptr.pos, page, view.marquee);
}

impl PdfCraftApp {
    /// Finish a marquee gesture for the current tool.
    pub(crate) fn finish_marquee(&mut self, index: usize, done: Marquee) {
        let (page, rect, view_rect) = done;
        if self.views.get(index).is_some_and(|view| view.crop_review.creating) {
            if view_rect[2] - view_rect[0] < 2.0 || view_rect[3] - view_rect[1] < 2.0 {
                return;
            }
            let Some(id) = self.views.get(index).map(|view| view.id) else { return };
            let item = match self.session.get(id).ok_or_else(|| "no document".to_string()).and_then(|doc| crop_review_item(doc, page, view_rect)) {
                Ok(item) => item,
                Err(e) => {
                    self.notify_fmt("Couldn't add the crop: {e}", &[("e", &e)]);
                    return;
                }
            };
            let created_id = item.id.clone();
            let inserted = self.session.crop_review_mut(id).map_err(|e| e.to_string()).and_then(|store| store.upsert(item).map(|_| ()));
            match inserted {
                Ok(()) => {
                    if self.right.is_none() {
                        self.choose_right_panel(Some(RightPanel::Crops));
                    }
                    if let Some(view) = self.views.get_mut(index) {
                        view.crop_review.creating = false;
                        view.crop_review.select(created_id.as_str());
                    }
                }
                Err(e) => self.notify_fmt("Couldn't add the crop: {e}", &[("e", &e)]),
            }
            return;
        }

        match self.quick_tool {
            QuickTool::MarqueeZoom => {
                let view = &mut self.views[index];
                if rect.width() < 6.0 || rect.height() < 6.0 {
                    // A click zooms in one step around the point.
                    let z = (view.zoom * 1.5).min(64.0);
                    view.zoom_at(z, rect.center());
                } else {
                    view.zoom_to_rect(rect);
                }
            }
            QuickTool::Snapshot => {
                if view_rect[2] - view_rect[0] < 2.0 || view_rect[3] - view_rect[1] < 2.0 {
                    return;
                }
                // The tool outlives the tab it was chosen on.
                if self.views.get(index).map(|v| v.id).is_some_and(|id| !self.may_copy(id)) {
                    return;
                }
                match self.snapshot(index, page, view_rect) {
                    Ok((w, h)) => {
                        self.notify_fmt("The selected area has been copied ({w} × {h} pixels)", &[("w", &w.to_string()), ("h", &h.to_string())])
                    }
                    Err(e) => self.notify_fmt("Couldn't copy the snapshot: {e}", &[("e", &e.to_string())]),
                }
                self.export_draft.snapshot_package =
                    Some(crate::export_ui::SnapshotPackageDraft { page, rect: view_rect, caption: String::new(), note: String::new() });
                self.dialog = Some(Dialog::Export(crate::export_ui::ExportKind::Image));
            }
            _ => {}
        }
    }

    /// View ▸ Zoom ▸ Fit Visible on the current page: find where the page has ink (a quick
    /// low-resolution render) and zoom so that spans the window's width. A blank page fits the
    /// width.
    pub fn fit_visible(&mut self, index: usize) -> Result<(), String> {
        let view = &self.views[index];
        let page = view.current;
        let doc = self.session.get(view.id).ok_or("no document")?;
        let config = RenderConfig { password: doc.password.as_deref().map(std::sync::Arc::from), ..RenderConfig::default() };
        let mut r = pdfcraft_render::PageRenderer::new(doc.bytes.clone(), config);
        // About 800 px along the longer side is plenty to find the margins.
        let size = doc.info.pages.get(page).map_or(792.0, |p| p.width.max(p.height));
        let out =
            r.render(RenderRequest { page, kind: RequestKind::Pixels, tile: None, scale: (800.0 / size).clamp(0.1, 4.0), tag: 0, preview: false });
        if let Some(e) = out.error {
            return Err(e);
        }
        let rot = view.rotation;
        let view = &mut self.views[index];
        match ink_bounds(out.width as usize, out.height as usize, &out.rgba) {
            Some([u0, v0, u1, v1]) => {
                // View space → the rotated view on screen.
                let rotate = |u: f32, v: f32| match rot {
                    90 => (1.0 - v, u),
                    180 => (1.0 - u, 1.0 - v),
                    270 => (v, 1.0 - u),
                    _ => (u, v),
                };
                let (a, b) = (rotate(u0, v0), rotate(u1, v1));
                if !view.fit_content(page, [a.0.min(b.0), a.1.min(b.1), a.0.max(b.0), a.1.max(b.1)]) {
                    view.fit = crate::canvas::Fit::Width;
                }
            }
            None => view.fit = crate::canvas::Fit::Width,
        }
        Ok(())
    }

    /// Render `view_rect` (page view points) of `page` at the current zoom and copy it to the
    /// clipboard (and keep it in `last_snapshot`).
    pub fn snapshot(&mut self, index: usize, page: usize, view_rect: [f32; 4]) -> Result<(u32, u32), String> {
        let view = &self.views[index];
        let doc = self.session.get(view.id).ok_or("no document")?;
        let ppp = self.ctx.as_ref().map_or(2.0, |c| c.pixels_per_point());
        let scale = (view.zoom * ppp).clamp(0.5, 8.0);
        let tile = Tile {
            x: (view_rect[0] * scale).floor().max(0.0) as u32,
            y: (view_rect[1] * scale).floor().max(0.0) as u32,
            w: ((view_rect[2] - view_rect[0]) * scale).ceil().max(1.0) as u32,
            h: ((view_rect[3] - view_rect[1]) * scale).ceil().max(1.0) as u32,
        };
        if (tile.w as u64) * (tile.h as u64) > 64_000_000 {
            return Err("the area is too large at this zoom".into());
        }
        let config = RenderConfig { password: doc.password.as_deref().map(std::sync::Arc::from), ..RenderConfig::default() };
        let mut r = pdfcraft_render::PageRenderer::new(doc.bytes.clone(), config);
        let out = r.render(RenderRequest { page, kind: RequestKind::Pixels, tile: Some(tile), scale, tag: 0, preview: false });
        if let Some(e) = out.error {
            return Err(e);
        }
        let (w, h) = (out.width, out.height);
        #[cfg(not(target_arch = "wasm32"))]
        if self.system_clipboard {
            let img = arboard::ImageData { width: w as usize, height: h as usize, bytes: std::borrow::Cow::Borrowed(&out.rgba) };
            arboard::Clipboard::new().and_then(|mut c| c.set_image(img)).map_err(|e| e.to_string())?;
        }
        self.last_snapshot = Some((w, h, out.rgba));
        Ok((w, h))
    }
}

fn crop_review_item(doc: &Document, page: usize, view_rect: [f32; 4]) -> Result<CropReviewItem, String> {
    let page_info = doc.info.pages.get(page).ok_or("no page")?;
    let a = page_info.view_to_user(view_rect[0], view_rect[1]);
    let b = page_info.view_to_user(view_rect[2], view_rect[3]);
    let rect_pdf = CropPdfRect::from_user_rect([a[0], a[1], b[0], b[1]])?;
    let now = Utc::now();
    Ok(CropReviewItem {
        id: next_crop_review_id(doc),
        kind: ExportItemKind::Snapshot,
        page_number: page + 1,
        rect_pdf,
        caption: String::new(),
        note: None,
        ocr_text: None,
        ocr_extracted_at: None,
        created_at: now,
        updated_at: now,
    })
}

fn next_crop_review_id(doc: &Document) -> String {
    let next = doc
        .crop_review()
        .items()
        .iter()
        .filter_map(|item| item.id.strip_prefix("crop-"))
        .filter_map(|suffix| suffix.parse::<u32>().ok())
        .max()
        .unwrap_or(0)
        .saturating_add(1);
    format!("crop-{next:04}")
}

/// The normalised bounds [u0, v0, u1, v1] of the pixels that aren't (near) white, if any.
pub(crate) fn ink_bounds(w: usize, h: usize, rgba: &[u8]) -> Option<[f32; 4]> {
    if w == 0 || h == 0 || rgba.len() < w * h * 4 {
        return None;
    }
    let ink = |p: &[u8]| p[3] > 16 && (p[0] < 240 || p[1] < 240 || p[2] < 240);
    let (mut x0, mut y0, mut x1, mut y1) = (w, h, 0, 0);
    for (y, row) in rgba.chunks_exact(w * 4).take(h).enumerate() {
        for (x, p) in row.as_chunks::<4>().0.iter().enumerate() {
            if ink(p) {
                x0 = x0.min(x);
                x1 = x1.max(x + 1);
                y0 = y0.min(y);
                y1 = y1.max(y + 1);
            }
        }
    }
    (x1 > x0 && y1 > y0).then(|| [x0 as f32 / w as f32, y0 as f32 / h as f32, x1 as f32 / w as f32, y1 as f32 / h as f32])
}

#[cfg(test)]
mod tests {
    #[test]
    fn ink_bounds_finds_the_marked_area() {
        let (w, h) = (10, 20);
        let mut px = vec![255u8; w * h * 4];
        for (x, y) in [(2, 5), (7, 15)] {
            px[(y * w + x) * 4] = 0;
        }
        assert_eq!(super::ink_bounds(w, h, &px), Some([0.2, 0.25, 0.8, 0.8]));
        assert_eq!(super::ink_bounds(w, h, &vec![255u8; w * h * 4]), None);
    }
}
