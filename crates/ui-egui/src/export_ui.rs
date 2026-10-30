//! Export a PDF ▸ Image (PNG, JPEG, TIFF) and Text (Acrobat's Export a PDF tool, first formats).
//!
//! On the desktop the export runs on a worker thread and reports progress in the notice bar;
//! on the web it runs in place and downloads the files.

use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use egui::{Align, Layout};
use pdfcraft_engine::export::{
    ExportSource, Exporter, ImageFormat, SnapshotExportPackageInput, populate_snapshot_export_ocr, write_snapshot_export_package,
};

use crate::marks_ui::PageRange;
use crate::theme::{self, Tokens};
use crate::{PdfCraftApp, widgets};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ExportKind {
    Image,
    Text,
    /// Export all images: the images pages use, as files.
    AllImages,
    /// Export the persisted crop-review dataset as a snapshot package.
    CropReviewPackage,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ExportDraft {
    pub dpi: f64,
    pub range: PageRange,
    pub format: ImageFormat,
    /// Export all images ▸ skip images with fewer pixels than this on their shorter side.
    pub min_side: u32,
    /// Parent folder where package exports are created.
    pub destination_dir: String,
    /// Name of the package directory to create inside `destination_dir`.
    pub package_dir_name: String,
    /// Whether crop review package export should also write report.docx.
    pub include_word_report: bool,
}

impl Default for ExportDraft {
    fn default() -> Self {
        Self {
            dpi: 150.0,
            range: PageRange::default(),
            format: ImageFormat::Png,
            min_side: 0,
            destination_dir: String::new(),
            package_dir_name: String::new(),
            include_word_report: false,
        }
    }
}

/// Progress of a background export: (done, total, final message once finished).
pub type ExportStatus = Arc<Mutex<Option<(usize, usize, Option<String>)>>>;

fn export_stem(name: &str) -> String {
    let stem = name.trim_end_matches(".pdf").trim_end_matches(".PDF").trim();
    if stem.is_empty() { "export".into() } else { stem.to_string() }
}

fn suggested_crop_review_package_dir_name(stem: &str) -> String {
    format!("{stem}_crop_review_package")
}

fn typed_export_dir(path: &str) -> Result<PathBuf, String> {
    let path = path.trim();
    if path.is_empty() {
        return Err("Choose a destination folder or browse to one.".into());
    }
    let dir = PathBuf::from(path);
    if !dir.exists() {
        return Err(format!("Destination folder does not exist: {}", dir.display()));
    }
    if !dir.is_dir() {
        return Err(format!("Destination path is not a folder: {}", dir.display()));
    }
    Ok(dir)
}

fn typed_package_dir_name(name: &str) -> Result<String, String> {
    let name = name.trim();
    if name.is_empty() {
        return Err("Choose a package folder name.".into());
    }
    if matches!(name, "." | "..") {
        return Err("Package folder name must not be . or ...".into());
    }
    if name.contains('/') || name.contains('\\') {
        return Err("Package folder name must not contain path separators.".into());
    }
    Ok(name.to_string())
}

fn crop_review_export_parent_dir(app: &PdfCraftApp) -> Result<PathBuf, String> {
    app.export_dir_override.clone().map(PathBuf::from).map_or_else(|| typed_export_dir(&app.export_draft.destination_dir), Ok)
}

pub(crate) fn body(ui: &mut egui::Ui, app: &mut PdfCraftApp, t: &Tokens, kind: ExportKind) -> (bool, bool) {
    if kind == ExportKind::CropReviewPackage {
        ui.label(egui::RichText::new(tl!("Export Crop Review Package")).font(theme::semibold(18.0)));
        ui.add_space(8.0);
        let count = app.active_ids().and_then(|(_, id)| app.session.get(id)).map(|d| d.crop_review().items().len()).unwrap_or(0);
        ui.label(
            egui::RichText::new(crate::i18n::fmt(
                tl!("Exports {n} stored crop items through the shared snapshot package writer."),
                &[("n", &count.to_string())],
            ))
            .small()
            .color(t.text_faint),
        );
        ui.add_space(10.0);
        let suggested_name = app
            .active_ids()
            .and_then(|(_, id)| app.session.get(id))
            .map(|d| export_stem(&d.name))
            .map(|stem| suggested_crop_review_package_dir_name(&stem))
            .unwrap_or_else(|| "export_crop_review_package".into());
        if app.export_draft.package_dir_name.trim().is_empty() {
            app.export_draft.package_dir_name = suggested_name.clone();
        }
        let label = ui.label(tl!("Destination folder"));
        ui.horizontal(|ui| {
            ui.add(widgets::line(&mut app.export_draft.destination_dir).desired_width(320.0)).labelled_by(label.id);
            if widgets::pill_button(ui, tl!("Browse…"), false).clicked() {
                #[cfg(not(target_arch = "wasm32"))]
                {
                    let dialog = rfd::AsyncFileDialog::new().set_title(tl!("Choose a folder for the exported files").to_string());
                    let _ = app.ask_one(crate::pickers::Ask::Folder(dialog), None, |app, dir| {
                        app.export_draft.destination_dir = dir.to_string_lossy().into_owned();
                    });
                }
                #[cfg(target_arch = "wasm32")]
                {
                    app.notify_tr("Crop review package export requires the desktop app");
                }
            }
        });
        ui.add_space(6.0);
        let label = ui.label(tl!("Package folder name"));
        ui.add(widgets::line(&mut app.export_draft.package_dir_name).desired_width(320.0)).labelled_by(label.id);
        ui.add_space(6.0);
        let final_folder = match (crop_review_export_parent_dir(app), typed_package_dir_name(&app.export_draft.package_dir_name)) {
            (Ok(dir), Ok(name)) => dir.join(name).display().to_string(),
            (Ok(dir), Err(_)) => dir.join(app.export_draft.package_dir_name.trim()).display().to_string(),
            _ => {
                let name = app.export_draft.package_dir_name.trim();
                if name.is_empty() { suggested_name.clone() } else { name.to_string() }
            }
        };
        ui.label(egui::RichText::new(crate::i18n::fmt(tl!("Final package folder: {path}"), &[("path", &final_folder)])).small().color(t.text_faint));
        ui.label(
            egui::RichText::new(tl!("Choose the parent folder and the package folder name PdfCraft will create inside it."))
                .small()
                .color(t.text_faint),
        );
        ui.add_space(8.0);
        ui.add(widgets::checkbox(&mut app.export_draft.include_word_report, tl!("Also write report.docx")));
        ui.add_space(12.0);
        let (mut apply, mut cancel) = (false, false);
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            let ok = count > 0 && crop_review_export_parent_dir(app).is_ok() && typed_package_dir_name(&app.export_draft.package_dir_name).is_ok();
            if ui.add_enabled_ui(ok, |ui| widgets::pill_button(ui, tl!("Export"), true)).inner.clicked() {
                apply = true;
            }
            if widgets::pill_button(ui, tl!("Cancel"), false).clicked() {
                cancel = true;
            }
        });
        return (apply, cancel);
    }

    let count = app.active_ids().and_then(|(_, id)| app.session.get(id)).map(|d| d.info.pages.len()).unwrap_or(0);
    let d = &mut app.export_draft;
    ui.label(
        egui::RichText::new(match kind {
            ExportKind::Image => tl!("Export to Image"),
            ExportKind::Text => tl!("Export to Text"),
            ExportKind::AllImages => tl!("Export All Images"),
            ExportKind::CropReviewPackage => tl!("Export Crop Review Package"),
        })
        .font(theme::semibold(18.0)),
    );
    ui.add_space(8.0);
    if kind == ExportKind::Image {
        ui.horizontal(|ui| {
            ui.label(tl!("Resolution"));
            egui::ComboBox::from_id_salt("export-dpi")
                .selected_text(crate::i18n::fmt(tl!("{dpi} pixels/inch"), &[("dpi", &d.dpi.to_string())]))
                .show_ui(ui, |ui| {
                    for dpi in [72.0, 96.0, 150.0, 300.0, 600.0] {
                        ui.selectable_value(&mut d.dpi, dpi, crate::i18n::fmt(tl!("{dpi} pixels/inch"), &[("dpi", &dpi.to_string())]));
                    }
                });
        });
        ui.horizontal(|ui| {
            ui.label(tl!("Format"));
            let mut quality = match d.format {
                ImageFormat::Jpeg { quality } => quality,
                _ => 85,
            };
            egui::ComboBox::from_id_salt("export-format").selected_text(d.format.label()).show_ui(ui, |ui| {
                for f in [ImageFormat::Png, ImageFormat::Jpeg { quality }, ImageFormat::Tiff] {
                    let on = std::mem::discriminant(&d.format) == std::mem::discriminant(&f);
                    if ui.selectable_label(on, f.label()).clicked() {
                        d.format = f;
                    }
                }
            });
            if let ImageFormat::Jpeg { .. } = d.format {
                ui.label(tl!("Quality"));
                if ui.add(egui::Slider::new(&mut quality, 10..=100)).changed() {
                    d.format = ImageFormat::Jpeg { quality };
                }
            }
        });
        ui.label(
            egui::RichText::new(crate::i18n::fmt(tl!("One {f} file per page, named after the document."), &[("f", d.format.label())]))
                .small()
                .color(t.text_faint),
        );
    } else if kind == ExportKind::AllImages {
        ui.horizontal(|ui| {
            ui.label(tl!("Exclude images smaller than"));
            let label = move |n: u32| {
                if n == 0 { tl!("No limit").to_string() } else { crate::i18n::trn(crate::i18n::current(), u64::from(n), "{n} pixel", "{n} pixels") }
            };
            egui::ComboBox::from_id_salt("export-min").selected_text(label(d.min_side)).show_ui(ui, |ui| {
                for n in [0, 16, 32, 64, 128, 256] {
                    ui.selectable_value(&mut d.min_side, n, label(n));
                }
            });
        });
        ui.label(
            egui::RichText::new(tl!("Each image once, named after the document and page. JPEG images are saved unchanged; others as PNG."))
                .small()
                .color(t.text_faint),
        );
    } else {
        ui.label(egui::RichText::new(tl!("Plain text in reading order; pages are separated by form feeds.")).small().color(t.text_faint));
    }
    ui.add_space(6.0);
    ui.label(egui::RichText::new(tl!("Pages")).font(theme::semibold(12.5)));
    d.range.ui(ui, count);
    ui.add_space(12.0);
    let (mut apply, mut cancel) = (false, false);
    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
        let ok = !d.range.pages(count).is_empty();
        if ui.add_enabled_ui(ok, |ui| widgets::pill_button(ui, tl!("Export"), true)).inner.clicked() {
            apply = true;
        }
        if widgets::pill_button(ui, tl!("Cancel"), false).clicked() {
            cancel = true;
        }
    });
    (apply, cancel)
}

/// Write the files (`name`, bytes) produced for `pages`, reporting progress.
#[allow(clippy::too_many_arguments)]
fn run(
    src: ExportSource,
    kind: ExportKind,
    dpi: f64,
    format: ImageFormat,
    min_side: u32,
    pages: Vec<usize>,
    stem: String,
    mut sink: impl FnMut(&str, Vec<u8>) -> Result<(), String>,
    status: &ExportStatus,
    lang: crate::i18n::Lang,
) -> String {
    // May run on a worker thread: messages are drawn in the UI's language.
    crate::i18n::set_current(lang);
    let total = pages.len();
    let set = |done: usize, msg: Option<String>| {
        if let Ok(mut s) = status.lock() {
            *s = Some((done, total, msg));
        }
    };
    if kind == ExportKind::AllImages {
        set(0, None);
        let out = match pdfcraft_engine::export::extract_images(&src, &pages, min_side) {
            Ok(o) => o,
            Err(e) => return crate::i18n::fmt(tl!("Export stopped: {e}"), &[("e", &e.to_string())]),
        };
        for (k, img) in out.images.iter().enumerate() {
            if let Err(e) = sink(&pdfcraft_engine::export::image_file_name(&stem, img, k + 1, src.pages), img.data.clone()) {
                return crate::i18n::fmt(tl!("Export stopped: {e}"), &[("e", &e.to_string())]);
            }
        }
        let n = out.images.len();
        let mut msg = if n == 1 {
            crate::i18n::fmt(tl!("Exported 1 image"), &[])
        } else {
            crate::i18n::fmt(tl!("Exported {n} images"), &[("n", &n.to_string())])
        };
        if !out.skipped.is_empty() {
            msg.push_str(&crate::i18n::fmt(
                tl!(" ({s} not exported: {first})"),
                &[("s", &out.skipped.len().to_string()), ("first", &out.skipped[0].2)],
            ));
        }
        return msg;
    }
    let mut ex = Exporter::from_source(src);
    match kind {
        // Handled above.
        ExportKind::AllImages => String::new(),
        ExportKind::CropReviewPackage => String::new(),
        ExportKind::Image => {
            for (k, p) in pages.iter().enumerate() {
                set(k, None);
                let result = ex.image(*p, dpi, format).and_then(|img| sink(&format!("{stem}_page_{}.{}", p + 1, format.extension()), img));
                if let Err(e) = result {
                    return crate::i18n::fmt(tl!("Export stopped: {e}"), &[("e", &e.to_string())]);
                }
            }
            let mut msg = if total == 1 {
                crate::i18n::fmt(tl!("Exported 1 image"), &[])
            } else {
                crate::i18n::fmt(tl!("Exported {n} images"), &[("n", &total.to_string())])
            };
            // Pages too large for the renderer at the chosen resolution are drawn at the most it
            // allows: say so, rather than leave a smaller image unexplained.
            let lowered: Vec<f64> = pages.iter().map(|p| ex.dpi_used(*p, dpi)).filter(|used| *used < dpi.clamp(18.0, 1200.0) - 0.5).collect();
            if let Some(lowest) = lowered.iter().copied().reduce(f64::min) {
                msg.push_str(&crate::i18n::fmt(
                    tl!(" ({n} at {dpi} dpi, the largest size they can be drawn at)"),
                    &[("n", &lowered.len().to_string()), ("dpi", &format!("{}", lowest.floor()))],
                ));
            }
            msg
        }
        ExportKind::Text => {
            set(0, None);
            match ex.text_of(&pages).and_then(|text| sink(&format!("{stem}.txt"), text.into_bytes())) {
                Ok(()) => {
                    if total == 1 {
                        crate::i18n::fmt(tl!("Exported the text of 1 page"), &[])
                    } else {
                        crate::i18n::fmt(tl!("Exported the text of {n} pages"), &[("n", &total.to_string())])
                    }
                }
                Err(e) => crate::i18n::fmt(tl!("Export stopped: {e}"), &[("e", &e.to_string())]),
            }
        }
    }
}

fn run_crop_review_package_export(input: SnapshotExportPackageInput, target: PathBuf, status: &ExportStatus, lang: crate::i18n::Lang) {
    crate::i18n::set_current(lang);
    let total = input.items.len();
    let set = |done: usize, msg: Option<String>| {
        if let Ok(mut s) = status.lock() {
            *s = Some((done, total, msg));
        }
    };

    set(0, None);
    let shown = target.display().to_string();
    let msg = match write_snapshot_export_package(&target, input) {
        Ok(()) => crate::i18n::fmt(tl!("Exported crop review package to {dir}"), &[("dir", &shown)]),
        Err(e) => crate::i18n::fmt(tl!("Export stopped: {e}"), &[("e", &e)]),
    };
    set(1, Some(msg));
}

impl PdfCraftApp {
    /// Start exporting the active document with the dialog's settings.
    pub(crate) fn start_export(&mut self, kind: ExportKind) {
        // What's typed in a form field is part of the document (#166).
        if !self.commit_form_typing() {
            return;
        }
        let Some((_, id)) = self.active_ids() else { return };
        if !self.may_copy(id) {
            return;
        }
        let Some(doc) = self.session.get(id) else { return };
        let src = doc.export_source();
        let stem = export_stem(&doc.name);
        if kind == ExportKind::CropReviewPackage {
            let mut input = match doc.crop_review_snapshot_export_package_input() {
                Ok(mut input) => {
                    input.include_word_report = self.export_draft.include_word_report;
                    input
                }
                Err(e) => {
                    self.notify_fmt("Export stopped: {e}", &[("e", &e)]);
                    return;
                }
            };
            if let Err(e) = populate_snapshot_export_ocr(&input.render_source, &mut input.items) {
                self.notify_fmt("Export stopped: {e}", &[("e", &e)]);
                return;
            }
            let status: ExportStatus = Arc::new(Mutex::new(Some((0, input.items.len(), None))));
            #[cfg(not(target_arch = "wasm32"))]
            {
                let start = move |app: &mut Self, target: PathBuf, inline: bool| {
                    let st = status.clone();
                    let lang = crate::i18n::current();
                    let work = move || run_crop_review_package_export(input, target, &st, lang);
                    if inline {
                        work();
                    } else {
                        std::thread::Builder::new().name("pdfcraft-crop-review-export".into()).spawn(work).ok();
                    }
                    app.export_status = Some(status);
                };
                match (crop_review_export_parent_dir(self), typed_package_dir_name(&self.export_draft.package_dir_name)) {
                    (Ok(dir), Ok(name)) => start(self, dir.join(name), true),
                    (Err(e), _) => self.notify(e),
                    (_, Err(e)) => self.notify(e),
                }
            }
            #[cfg(target_arch = "wasm32")]
            {
                self.notify_tr("Crop review package export requires the desktop app");
            }
            return;
        }

        let pages = self.export_draft.range.pages(src.pages);
        let dpi = self.export_draft.dpi;
        let format = self.export_draft.format;
        let min_side = self.export_draft.min_side;
        let status: ExportStatus = Arc::new(Mutex::new(Some((0, pages.len(), None))));
        #[cfg(not(target_arch = "wasm32"))]
        {
            let start = move |app: &mut Self, dir: std::path::PathBuf, inline: bool| {
                let st = status.clone();
                let shown = dir.display().to_string();
                let lang = crate::i18n::current();
                let work = move || {
                    let sink = |name: &str, bytes: Vec<u8>| {
                        crate::editing::write_atomically(&dir.join(name).to_string_lossy(), &bytes).map_err(|e| format!("{name}: {e}"))
                    };
                    let msg = run(src, kind, dpi, format, min_side, pages, stem, sink, &st, lang);
                    if let Ok(mut s) = st.lock() {
                        let (done, total) = s.as_ref().map_or((0, 0), |(d, t, _)| (*d, *t));
                        let done_msg = crate::i18n::fmt(tl!("{msg} to {dir}"), &[("msg", &msg), ("dir", &shown)]);
                        *s = Some((done.max(total), total, Some(done_msg)));
                    }
                };
                if inline {
                    work(); // tests and automation: synchronous
                } else {
                    std::thread::Builder::new().name("pdfcraft-export".into()).spawn(work).ok();
                }
                app.export_status = Some(status);
            };
            match self.export_dir_override.clone() {
                Some(d) => start(self, d.into(), true),
                None => {
                    let dialog = rfd::AsyncFileDialog::new().set_title(tl!("Choose a folder for the exported files").to_string());
                    self.ask_one(crate::pickers::Ask::Folder(dialog), None, move |app, dir| start(app, dir, false));
                }
            }
        }
        #[cfg(target_arch = "wasm32")]
        {
            let msg = run(
                src,
                kind,
                dpi,
                format,
                min_side,
                pages,
                stem,
                |name, bytes| crate::editing::download(name, &bytes),
                &status,
                crate::i18n::current(),
            );
            if let Ok(mut s) = status.lock() {
                *s = Some((0, 0, Some(msg)));
            }
            self.export_status = Some(status);
        }
    }

    /// Show export progress, and the result once it is done.
    pub(crate) fn poll_export(&mut self) {
        let Some(st) = self.export_status.clone() else { return };
        let snapshot = st.lock().ok().and_then(|s| s.clone());
        match snapshot {
            Some((_, _, Some(msg))) => {
                self.export_status = None;
                self.notify(msg);
            }
            Some((done, total, None)) if total > 1 => {
                let m = crate::i18n::fmt(tl!("Exporting… {d} of {t}"), &[("d", &done.to_string()), ("t", &total.to_string())]);
                self.notify(m);
            }
            _ => {}
        }
        if let Some(ctx) = &self.ctx {
            ctx.request_repaint_after(std::time::Duration::from_millis(200));
        }
    }
}
