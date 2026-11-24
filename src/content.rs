//! Academic content for the homepage.
//!
//! Keeping these facts outside the HTML template means future updates to the
//! biography or education do not require editing markup.

/// One entry in the education section.
pub struct EducationEntry {
    pub institution: &'static str,
    pub degree: &'static str,
    pub dates: &'static str,
    pub details: &'static str,
}

/// The information needed by our one-page academic homepage.
pub struct AcademicProfile {
    pub display_name: &'static str,
    pub full_name: &'static str,
    pub role: &'static str,
    pub institution: &'static str,
    pub research_interests: &'static str,
    pub biography: &'static str,
    pub email: &'static str,
    pub github_url: &'static str,
    pub education: Vec<EducationEntry>,
    pub advisor_name: &'static str,
    pub advisor_url: &'static str,
}

/// Construct the profile from the public facts in the academic CV.
///
/// The short display name matches the current homepage; the full name is kept
/// for the accessible page description and for the CV itself.
pub fn academic_profile() -> AcademicProfile {
    AcademicProfile {
        display_name: "Olu Olorode",
        full_name: "Oluwadamilola Olorode",
        role: "Mathematics PhD candidate",
        institution: "Cornell University",
        research_interests: "Geometric group theory, hyperbolic geometry, low-dimensional topology, and machine learning",
        biography: "My research interests include geometric group theory, hyperbolic geometry, low-dimensional topology, and machine learning.",
        email: "oio3@cornell.edu",
        github_url: "https://github.com/hgfjh",
        education: vec![
            EducationEntry {
                institution: "Cornell University",
                degree: "PhD in Mathematics (expected May 2027)",
                dates: "2021–present",
                details: "Advisor: Jason Manning",
            },
            EducationEntry {
                institution: "Rutgers University–New Brunswick",
                degree: "B.S. in Mathematics",
                dates: "2019–2021",
                details: "Highest Honors, summa cum laude",
            },
        ],
        advisor_name: "Jason Manning",
        advisor_url: "https://sites.google.com/cornell.edu/jasonmanning/home",
    }
}

/// A manuscript and its optional public resources.
/// `None` means the corresponding resource is not published on the site.
pub struct Paper {
    pub title: &'static str,
    pub authors: &'static str,
    pub status: &'static str,
    pub abstract_text: Option<&'static str>,
    pub pdf_url: Option<&'static str>,
    pub youtube_id: Option<&'static str>,
    pub transcript_url: Option<&'static str>,
}

/// Research entries from the CV. Private manuscripts remain available on request.
pub fn papers() -> Vec<Paper> {
    vec![
        Paper {
            title: "An Improved Combination Theorem for A/QI Triples",
            authors: "Oluwadamilola Olorode",
            status: "Preprint available upon request.",
            abstract_text: None,
            pdf_url: None,
            youtube_id: None,
            transcript_url: None,
        },
        Paper {
            title: "The Ratcliffe–Tschantz Manifold Is Isometric to Martelli’s Fibered 5-Manifold",
            authors: "Joint work with I. Broudy, C. Kelln, and L. Xu",
            status: "Preprint available upon request.",
            abstract_text: None,
            pdf_url: None,
            youtube_id: None,
            transcript_url: None,
        },
    ]
}

/// One selected research presentation.
pub struct Talk {
    pub title: &'static str,
    pub venue: &'static str,
    pub year: &'static str,
}

/// The invited talks listed in the CV, most recent year first.
pub fn talks() -> Vec<Talk> {
    vec![
        Talk {
            title: "New Tools for Finding Exotic Hyperbolic Groups",
            venue: "CUNY Geometry and Topology Seminar, Manhattan, NY",
            year: "2026",
        },
        Talk {
            title: "An Improved Combination Theorem for A/QI Triples",
            venue: "Spring Topology and Dynamics Conference, Birmingham, AL",
            year: "2026",
        },
        Talk {
            title: "An Improved Combination Theorem for A/QI Triples",
            venue: "Brandeis Topology Seminar, Waltham, MA",
            year: "2025",
        },
    ]
}
