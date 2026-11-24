# Academic Site — Commit 3

A small Rust + Askama static-site exporter for an academic homepage. The deployed
site contains HTML, CSS, and a public CV PDF; GitHub Pages does **not** run Rust.

## What's new in Commit 3

- `src/content.rs` provides the academic profile and education entries.
- `templates/index.html` renders the biography, education, and contact links.
- `assets/cv-public.pdf` is a public copy of the four-page CV. The residential
  address and telephone number have been **redacted from the PDF content**.
- `src/main.rs` copies the sanitized PDF to `dist/cv.pdf` during static export.
- The GitHub Actions workflow verifies that the public CV was exported.

The homepage keeps **Olu Olorode** as its display name, as in the existing
GitHub repository, while the CV retains the full professional name.

## Local build (PowerShell)

```powershell
cargo fmt --all -- --check
cargo clippy --all-targets --locked -- -D warnings
cargo test --locked
cargo run --release --locked
python -m http.server 8000 --directory dist
```

Visit <http://localhost:8000/> and check the **Download CV** link. If `python`
isn't available, try `py -m http.server 8000 --directory dist`.

`--locked` uses your existing committed `Cargo.lock`. It is not intentionally
replaced by this commit, since no dependency versions have changed.

## Apply on top of Commit 2

Replace the corresponding files in your existing repository with those from
this archive; add `src/content.rs` and `assets/cv-public.pdf`. Keep your own
`Cargo.lock` and `.git` history.

```powershell
git status
git add src/main.rs src/content.rs templates/index.html static/style.css assets/cv-public.pdf .github/workflows/deploy.yml .gitignore README.md
git commit -m "feat: add academic profile and public CV"
git push origin main
```

The existing `main` deployment workflow will build and publish `dist/` on push.

## Privacy and future editing

**Do not add the original `cv.pdf` to this public repository.** If you later
update your CV, prepare and inspect a fresh public version before replacing
`assets/cv-public.pdf`. A white rectangle over sensitive text is not secure
redaction; the underlying text must be removed.

Publication entries and YouTube video metadata come in Commit 4; the homepage
uses only the biographical and education information from the supplied CV.
