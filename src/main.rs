mod assets;
mod content;
mod geometry;
mod mesh_binary;
mod topology;

use askama::Template;
use std::{error::Error, fs, io, path::Path};

use assets::AssetUrls;
use content::{AcademicProfile, Paper, Talk, academic_profile, papers, talks};

/// Askama checks `templates/index.html` against this Rust context at compile time.
#[derive(Template)]
#[template(path = "index.html")]
struct HomePage {
    assets: AssetUrls,
    profile: AcademicProfile,
    papers: Vec<Paper>,
    talks: Vec<Talk>,
}

impl HomePage {
    fn new(assets: AssetUrls) -> Self {
        Self {
            assets,
            profile: academic_profile(),
            papers: papers(),
            talks: talks(),
        }
    }
}

/// Render the website into a directory suitable for GitHub Pages.
fn export_site(output: &Path) -> Result<(), Box<dyn Error>> {
    // Generate and verify the mesh before writing any website files.
    let mesh = geometry::genus_two().map_err(io::Error::other)?;
    fs::create_dir_all(output.join("static/images"))?;
    fs::create_dir_all(output.join("static/meshes"))?;

    let mut binary = Vec::new();
    mesh_binary::write(&mesh, &mut binary)?;
    let assets = AssetUrls::new(&binary);
    assets.export(output, &binary)?;
    let page = HomePage::new(assets);
    fs::write(output.join("index.html"), page.render()?)?;

    // The PDF in assets/ is a *public* copy: private contact details were
    // actually redacted from the source PDF, not just visually covered.
    fs::copy("assets/cv-public.pdf", output.join("cv.pdf"))?;

    // Retain the full-precision JSON for inspection.
    // Serialize into a buffer to avoid one filesystem write per coordinate.
    let mut json = Vec::new();
    mesh.write_json(&mut json)?;
    fs::write(output.join("static/meshes/genus2.json"), json)?;
    println!("Verified genus-two mesh: {:?}", mesh.stats);

    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    let output = Path::new("dist");
    export_site(output)?;
    println!(
        "Generated {} and {}",
        output.join("index.html").display(),
        output.join("cv.pdf").display()
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn homepage_has_public_cv_and_profile_links() {
        let html = HomePage::new(AssetUrls::new(b"test mesh"))
            .render()
            .expect("Askama should render a valid homepage");

        assert!(html.contains("Olu Olorode"));
        assert!(html.contains("Cornell University"));
        assert!(html.contains("Jason Manning"));
        assert!(html.contains("https://sites.google.com/cornell.edu/jasonmanning/home"));
        assert!(html.contains("./cv.pdf"));
        assert!(html.contains(&AssetUrls::new(b"test mesh").portrait));
        assert!(html.contains("alt=\"Portrait of Olu Olorode\""));
        let viewer_href = "href=\"https://hgfjh.github.io/CoxeterViewer5D/\"";
        assert!(html.contains(&format!(
            "<a class=\"nav-demo\" {viewer_href}>Try CoxeterViewer</a>"
        )));
        assert_eq!(html.matches(&format!("<a {viewer_href}>")).count(), 1);
        assert_eq!(html.matches(viewer_href).count(), 2);
        assert!(html.contains("href=\"https://github.com/hgfjh/CoxeterViewer5D\""));
        assert!(!html.contains("The browser demo is not yet deployed."));
        assert!(html.contains("id=\"surface-mount\""));
        assert!(html.contains("data-genus-surface"));
        let assets = AssetUrls::new(b"test mesh");
        assert!(html.contains(&format!("data-mesh=\"{}\"", assets.mesh)));
        assert!(html.contains("data-toggle type=\"button\" disabled"));
        assert!(html.contains("id=\"surface-instructions\""));
        assert!(html.contains("data-rotation-icon aria-hidden=\"true\""));
        assert!(html.contains(&format!("type=\"module\" src=\"{}\"", assets.surface)));
        assert!(html.contains(&format!("href=\"{}\"", assets.style)));
        assert!(html.contains(&format!("src=\"{}\"", assets.videos)));
        assert!(html.contains(&format!("href=\"{}\"", assets.icon)));
        assert!(html.contains("rel=\"canonical\" href=\"https://hgfjh.github.io/\""));
        assert!(html.contains("property=\"og:image\""));
        assert!(html.contains("data-adjust"));
        assert_eq!(html.matches("data-orient=").count(), 5);
        assert!(!html.contains("?v=8.5"));
        assert!(html.contains("mailto:oio3@cornell.edu"));
        assert!(html.contains("https://github.com/hgfjh"));
        assert!(!html.contains("Village Circle"));
    }

    #[test]
    fn homepage_has_research_and_omits_absent_resources() {
        // Keep this test valid even after real public resources are added.
        let mut page = HomePage::new(AssetUrls::new(b"test mesh"));
        for paper in &mut page.papers {
            paper.abstract_text = None;
            paper.pdf_url = None;
            paper.youtube_id = None;
            paper.transcript_url = None;
        }
        let html = page.render().unwrap();
        assert!(html.contains("An Improved Combination Theorem for A/QI Triples"));
        assert!(html.contains("Martelli"));
        assert!(html.contains("CUNY Geometry and Topology Seminar"));
        assert!(html.contains("Request preprint"));
        assert!(!html.contains("Read paper"));
        assert!(!html.contains("<iframe"));
        assert!(!html.contains("youtube.com"));
        assert!(!html.contains("data-video-id="));
    }

    #[test]
    fn optional_research_resources_render_without_eager_video() {
        // Synthetic metadata: used only by tests, never published.
        let mut page = HomePage::new(AssetUrls::new(b"test mesh"));
        page.papers[0].abstract_text = Some("A test abstract.");
        page.papers[0].pdf_url = Some("https://example.org/paper.pdf");
        page.papers[0].youtube_id = Some("abcdefghijk");
        page.papers[0].transcript_url = Some("https://example.org/transcript");
        let html = page.render().unwrap();
        assert!(html.contains("A test abstract."));
        assert!(html.contains("https://example.org/paper.pdf"));
        assert!(html.contains("https://www.youtube.com/watch?v=abcdefghijk"));
        assert!(html.contains("data-video-id=\"abcdefghijk\""));
        assert!(html.contains("https://example.org/transcript"));
        assert!(!html.contains("<iframe"));
    }
}
