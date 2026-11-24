# Academic Site — Commit 4

Rust + Askama static academic homepage. GitHub Pages serves the exported HTML,
CSS, JavaScript and public CV; Rust only runs during the build.

## Apply Commit 4

Copy the changed files from the archive into the existing repository. Preserve
its `.git` history, `Cargo.lock`, `Cargo.toml` and public CV. No dependencies
changed. The archive preserves the advisor link from the current repository.

```powershell
cargo fmt --all
cargo clippy --all-targets --locked -- -D warnings
cargo test --locked
cargo run --release --locked
python -m http.server 8000 --directory dist
```

Visit http://localhost:8000/ and check Research, Selected talks, the advisor
link and CV. The current papers do not have any public abstracts, paper URLs,
or video IDs supplied to this project, so these optional controls are omitted.

## Editing research content

Edit `papers()` and `talks()` in `src/content.rs`. For each paper, `None`
means no public resource; change it to `Some("...")` when ready:

- `abstract_text`: approved plain-text abstract.
- `pdf_url`: approved public URL (arXiv, DOI landing page, or PDF). Update the
  `status` field accordingly.
- `youtube_id`: just the 11-character ID from the YouTube watch URL, not the URL.
  Make the video public or unlisted and allow embedding.
- `transcript_url`: a public transcript URL, if available.

The script `static/videos.js` reveals a local play button and creates a
privacy-enhanced YouTube iframe only after activation. The direct YouTube link
works with JavaScript disabled. No remote thumbnail, player or YouTube API is
loaded at page startup. Test an actual video ID when you have one; the Rust
rendering test uses synthetic metadata that is never exported.

Only put public materials in the repository. Do not commit private manuscripts
or the original unredacted CV. The exporter copies `assets/cv-public.pdf`.

## Publish

```powershell
git add src/content.rs src/main.rs templates/index.html static/style.css static/videos.js .github/workflows/deploy.yml README.md
git commit -m "feat: add research publications and video-ready entries"
git push origin main
```

The existing workflow checks and exports the site, including the video script.
