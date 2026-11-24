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
