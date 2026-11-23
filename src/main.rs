use askama::Template;
use std::{error::Error, fs, path::Path};

// Askama reads templates/index.html at compile time and generates
// the implementation of Template for this struct.
#[derive(Template)]
#[template(path = "index.html")]
struct HomePage<'a> {
    name: &'a str,
    research: &'a str,
}

fn main() -> Result<(), Box<dyn Error>> {
    // GitHub Pages will eventually serve this generated directory.
    let output = Path::new("dist");
    fs::create_dir_all(output.join("static"))?;

    // Page content is kept in Rust for now; later commits will move
    // publications and other entries into structured site data.
    let page = HomePage {
        name: "Olu Olorode",
        research: "Geometric group theory, hyperbolic geometry, low-dimensional topology, and machine learning",
    };

    // Generate the static homepage, then copy its stylesheet.
    fs::write(output.join("index.html"), page.render()?)?;
    fs::copy("static/style.css", output.join("static/style.css"))?;

    println!("Generated {}", output.join("index.html").display());
    Ok(())
}
