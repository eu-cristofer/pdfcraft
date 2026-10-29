//! Names and identity of this build (Costa Inspection fork of PdfCraft).
//!
//! One place, so the final suite name (U1) is a one-file change; packaging files copy these
//! values and say `keep in sync with crates/ui-egui/src/branding.rs`.

/// Name of the suite this app belongs to (placeholder until the final suite name, U1, is chosen).
pub const SUITE_NAME: &str = "<Suite>";

/// Display name of this application.
pub const APP_NAME: &str = "<Suite> PDF Review";

/// Application identifier: lowercase reverse-DNS. Placeholder until U1.
pub const APP_ID: &str = "com.costainspection.suite-pdf";

/// Home page of this fork's source repository.
pub const HOMEPAGE_URL: &str = "https://github.com/eu-cristofer/pdfcraft";

/// Plain-text statement of origin. Allowed; it carries no ArtCraft logo or mark.
pub const BASED_ON: &str = "Based on PdfCraft by the ArtCraft Team and the PdfCraft contributors (MIT OR Apache-2.0).";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn app_name_starts_with_suite_name() {
        assert!(APP_NAME.starts_with(SUITE_NAME));
    }

    #[test]
    fn no_ui_source_embeds_the_artcraft_brand_files() {
        let needle = concat!("docs/", "brand/");
        for src in [include_str!("widgets.rs"), include_str!("home.rs"), include_str!("dialogs.rs")] {
            assert!(!src.contains(needle));
        }
    }
}
