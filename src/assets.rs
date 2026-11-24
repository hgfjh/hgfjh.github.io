//! Content-addressed public assets. Stable aliases support the standalone preview.
use std::{fs, io, path::Path};

// Canonical public origin, including its trailing slash. Local asset URLs
// stay relative so local previews do not depend on production DNS.
const SITE_ORIGIN: &str = "https://oluolorode.com/";

const STYLE: &[u8] = include_bytes!("../static/style.css");
const VIDEOS: &[u8] = include_bytes!("../static/videos.js");
const SURFACE: &[u8] = include_bytes!("../static/surface.js");
const PORTRAIT: &[u8] = include_bytes!("../static/images/portrait.webp");
const ICON: &[u8] = include_bytes!("../static/favicon.svg");
const SHARE: &[u8] = include_bytes!("../static/images/share.jpg");

// FNV-1a is a deterministic, non-cryptographic cache key, not a signature.
fn fingerprint(bytes: &[u8]) -> u64 {
    bytes.iter().fold(0xcbf29ce484222325, |hash, &byte| {
        (hash ^ u64::from(byte)).wrapping_mul(0x100000001b3)
    })
}

fn url(path: &str, bytes: &[u8]) -> String {
    let (stem, extension) = path.rsplit_once('.').expect("asset has an extension");
    format!("./{stem}.{:016x}.{extension}", fingerprint(bytes))
}

pub struct AssetUrls {
    pub origin: &'static str,
    pub viewer: String,
    pub style: String,
    pub videos: String,
    pub surface: String,
    pub portrait: String,
    pub mesh: String,
    pub icon: String,
    pub share: String,
    pub share_absolute: String,
}

impl AssetUrls {
    pub fn new(mesh: &[u8]) -> Self {
        let share = url("static/images/share.jpg", SHARE);
        let share_absolute = format!("{SITE_ORIGIN}{}", &share[2..]);
        Self {
            origin: SITE_ORIGIN,
            viewer: format!("{SITE_ORIGIN}CoxeterViewer5D/"),
            style: url("static/style.css", STYLE),
            videos: url("static/videos.js", VIDEOS),
            surface: url("static/surface.js", SURFACE),
            portrait: url("static/images/portrait.webp", PORTRAIT),
            mesh: url("static/meshes/genus2.bin", mesh),
            icon: url("static/favicon.svg", ICON),
            share,
            share_absolute,
        }
    }

    /// Write exactly the bytes used to compute the URLs, not a second source read.
    pub fn export(&self, output: &Path, mesh: &[u8]) -> io::Result<()> {
        for (stable, fingerprinted, bytes) in [
            ("static/style.css", &self.style, STYLE),
            ("static/videos.js", &self.videos, VIDEOS),
            ("static/surface.js", &self.surface, SURFACE),
            ("static/images/portrait.webp", &self.portrait, PORTRAIT),
            ("static/meshes/genus2.bin", &self.mesh, mesh),
            ("static/favicon.svg", &self.icon, ICON),
            ("static/images/share.jpg", &self.share, SHARE),
        ] {
            fs::write(output.join(stable), bytes)?;
            fs::write(output.join(fingerprinted), bytes)?;
        }
        // Some browsers request this conventional URL independently of the link.
        fs::write(
            output.join("favicon.ico"),
            include_bytes!("../static/favicon.ico"),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fingerprint_is_stable_and_changes_with_content() {
        assert_eq!(fingerprint(b""), 0xcbf29ce484222325);
        assert_eq!(fingerprint(b"hello"), 0xa430d84680aabd0b);
        assert_eq!(
            url("static/test.js", b"hello"),
            "./static/test.a430d84680aabd0b.js"
        );
        assert_ne!(
            url("static/test.js", b"hello"),
            url("static/test.js", b"Hello")
        );
    }
}
