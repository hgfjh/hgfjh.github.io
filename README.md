# Academic Site — Commit 2

A minimal Rust + Askama static-site exporter with automated publishing to GitHub Pages.
The GitHub Pages site serves generated HTML/CSS; it does **not** run a Rust server.

## Prerequisites

- Git and a working Rust toolchain (with Cargo, rustfmt, and Clippy)
- A GitHub account named `hgfjh`
- A **public** repository named exactly `hgfjh.github.io`, with default branch `main`

## Run locally (PowerShell)

```powershell
cargo run
python -m http.server 8000 --directory dist
```

Open http://localhost:8000/ . For Python installations using the launcher, replace `python` with `py`.
This produces `dist/index.html` and `dist/static/style.css`.

## Check before publishing

```powershell
cargo fmt --all -- --check
cargo clippy --all-targets --locked -- -D warnings
cargo test --locked
cargo run --release --locked
```

**Important:** `--locked` expects a committed `Cargo.lock`. If it is missing, run
`cargo generate-lockfile` once, then `git add Cargo.lock` and commit it. Cargo
creates this lockfile when you first run `cargo run`, too.

## Publish the site

1. On GitHub, create a *public* repository named `hgfjh.github.io` owned by `hgfjh`.
   If using an existing local project, create an **empty** GitHub repository
   (do not pre-populate with a separate README or `.gitignore`).
2. Navigate to **Settings → Pages → Build and deployment → Source** and choose
   **GitHub Actions**.
3. Ensure that `main` is your default branch and `Cargo.lock` is tracked.
4. Add `.github/workflows/deploy.yml` to your existing project, then run:

   ```powershell
   git add .github/workflows/deploy.yml README.md Cargo.lock
   git commit -m "ci: deploy generated Rust site to GitHub Pages"
   git remote add origin https://github.com/hgfjh/hgfjh.github.io.git
   git push -u origin main
   ```

   If `origin` already exists, **skip** `git remote add origin`.
   If the repository already has commits or a remote, review the existing
   branch/history rather than force-pushing.

5. Open the repository's **Actions** tab. Select **Build and deploy academic site**
   and inspect the latest run. After `build` and `deploy` succeed, visit
   https://hgfjh.github.io/ .

### What the workflow does

- Runs on every push to `main`, or when started manually in the Actions tab.
- Checks out the source and installs the stable Rust toolchain.
- Runs formatting checks, Clippy, and Rust tests with the lockfile.
- Generates the website with `cargo run --release --locked`.
- Checks that the HTML and CSS exist.
- Uploads **only** `dist/` as a GitHub Pages deployment artifact.
- Deploys the artifact with the `github-pages` environment and minimum deployment permissions.

The `dist/` and `target/` directories are intentionally excluded from Git.

## Troubleshooting

- **`Cargo.lock needs to be updated`**: run `cargo generate-lockfile` locally and commit the lockfile.
- **`Resource not accessible by integration` or Pages deployment denied**:
  confirm **Settings → Pages → Source = GitHub Actions** and review Actions/Pages permissions and environment restrictions.
- **404**: verify the repository is named exactly `hgfjh.github.io`, the Actions
  run completed successfully, and the browser URL is `https://hgfjh.github.io/`.
- **CSS missing**: verify `dist/static/style.css` exists in the generated artifact.
- **Action not running**: confirm the workflow file is committed on `main`.
