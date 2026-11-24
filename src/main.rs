mod content;

use askama::Template;
use std::{error::Error, fs, path::Path};

use content::{AcademicProfile, academic_profile};

/// Askama checks `templates/index.html` against this Rust context at compile time.
#[derive(Template)]
#[template(path = "index.html")]
struct HomePage {
    profile: AcademicProfile,
}

/// Render the website into a directory suitable for GitHub Pages.
fn export_site(output: &Path) -> Result<(), Box<dyn Error>> {
    fs::create_dir_all(output.join("static"))?;

    let page = HomePage {
        profile: academic_profile(),
    };

    fs::write(output.join("index.html"), page.render()?)?;
    fs::copy("static/style.css", output.join("static/style.css"))?;

    // The PDF in assets/ is a *public* copy: private contact details were
    // actually redacted from the source PDF, not just visually covered.
    fs::copy("assets/cv-public.pdf", output.join("cv.pdf"))?;

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
        let html = HomePage {
            profile: academic_profile(),
        }
        .render()
        .expect("Askama should render a valid homepage");

        assert!(html.contains("Olu Olorode"));
        assert!(html.contains("Cornell University"));
        assert!(html.contains("Jason Manning"));
        assert!(html.contains("./cv.pdf"));
        assert!(html.contains("mailto:oio3@cornell.edu"));
        assert!(html.contains("https://github.com/hgfjh"));
        assert!(!html.contains("Village Circle"));
    }
}
