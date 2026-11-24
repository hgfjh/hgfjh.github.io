//! Search discovery files and structured academic identity, generated at build time.
use crate::{assets::AssetUrls, content::AcademicProfile};
use std::{fmt::Write as _, fs, io, path::Path};

/// Encode a JSON string for embedding inside an HTML script data block.
/// Escaping '<' prevents a value such as </script> from closing the element.
/// JSON escapes preserve the original data after decoding; HTML entities do not.
fn json_string(value: &str) -> String {
    let mut result = String::from("\"");
    for c in value.chars() {
        match c {
            '"' => result.push_str("\\\""),
            '\\' => result.push_str("\\\\"),
            '<' => result.push_str("\\u003c"),
            '>' => result.push_str("\\u003e"),
            '&' => result.push_str("\\u0026"),
            c if c <= '\u{1f}' || matches!(c, '\u{2028}' | '\u{2029}') => {
                write!(result, "\\u{:04x}", u32::from(c)).expect("writing to String cannot fail");
            }
            c => result.push(c),
        }
    }
    result.push('"');
    result
}

/// Describe only the public facts already present in the homepage profile.
/// No guessed ORCID, Scholar identifier, dates or publication claims.
pub fn profile_json(profile: &AcademicProfile, assets: &AssetUrls) -> String {
    let interests = profile
        .research_interests
        .split(',')
        .map(|s| s.trim().trim_start_matches("and "))
        .filter(|s| !s.is_empty())
        .map(json_string)
        .collect::<Vec<_>>()
        .join(", ");
    format!(
        r#"{{
  "@context": "https://schema.org",
  "@type": "ProfilePage",
  "@id": {page_id},
  "url": {url},
  "mainEntity": {{
    "@type": "Person",
    "@id": {person_id},
    "name": {name},
    "alternateName": {alternate_name},
    "url": {url},
    "image": {image},
    "jobTitle": {role},
    "affiliation": {{
      "@type": "CollegeOrUniversity",
      "name": {institution}
    }},
    "knowsAbout": [{interests}],
    "sameAs": [{github}]
  }}
}}"#,
        page_id = json_string(&format!("{}#profile", assets.origin)),
        person_id = json_string(&format!("{}#person", assets.origin)),
        url = json_string(assets.origin),
        name = json_string(profile.full_name),
        alternate_name = json_string(profile.display_name),
        image = json_string(&assets.share_absolute),
        role = json_string(profile.role),
        institution = json_string(profile.institution),
        github = json_string(profile.github_url),
    )
}

fn robots(assets: &AssetUrls) -> String {
    // Preserve the existing unrestricted crawl policy. Training and search
    // can be separated later; no training-specific opt-out is assumed here.
    format!(
        "User-agent: *\nAllow: /\n\nSitemap: {}sitemap.xml\n",
        assets.origin
    )
}

fn sitemap(assets: &AssetUrls) -> String {
    // These two URLs are controlled constants, with no XML-sensitive characters.
    // The public CV is linked from the homepage but not separately promoted here.
    format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n\
         <urlset xmlns=\"http://www.sitemaps.org/schemas/sitemap/0.9\">\n  \
         <url><loc>{}</loc></url>\n  \
         <url><loc>{}</loc></url>\n\
         </urlset>\n",
        assets.origin, assets.viewer,
    )
}

/// Write root-level files for discovery across the shared public domain.
pub fn export(output: &Path, assets: &AssetUrls) -> io::Result<()> {
    fs::write(output.join("robots.txt"), robots(assets))?;
    fs::write(output.join("sitemap.xml"), sitemap(assets))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::content::academic_profile;

    #[test]
    fn json_strings_are_safe_in_script_data() {
        assert_eq!(json_string("hello"), "\"hello\"");
        assert_eq!(
            json_string("\"\\\n\t</script>&\u{2028}"),
            r#""\"\\\u000a\u0009\u003c/script\u003e\u0026\u2028""#
        );
        assert_eq!(json_string("Géométrie"), "\"Géométrie\"");
    }

    #[test]
    fn profile_uses_public_facts_and_safe_encoding() {
        let mut profile = academic_profile();
        profile.display_name = "A & B </script>";
        let data = profile_json(&profile, &AssetUrls::new(b"test"));
        assert!(data.contains(r#""@type": "ProfilePage""#));
        assert!(data.contains(r#""@type": "Person""#));
        assert!(data.contains(r#""alternateName": "A \u0026 B \u003c/script\u003e""#));
        assert!(!data.contains("</script>"));
        assert!(!data.contains(profile.email));
    }

    #[test]
    fn discovery_files_use_production_urls() {
        let assets = AssetUrls::new(b"test");
        assert_eq!(
            robots(&assets),
            "User-agent: *\nAllow: /\n\nSitemap: https://oluolorode.com/sitemap.xml\n"
        );
        let xml = sitemap(&assets);
        assert!(xml.contains("<loc>https://oluolorode.com/</loc>"));
        assert!(xml.contains("<loc>https://oluolorode.com/CoxeterViewer5D/</loc>"));
        assert_eq!(xml.matches("<loc>").count(), 2);
        assert!(!xml.contains("cv.pdf"));
    }
}
