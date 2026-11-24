# Final academic-site audit — Commit 9

## Design and scope

This update preserves the approved portrait, content, geometry, shader colors,
quaternion orientation and 32-second rotation period. No Cargo dependency,
lockfile, graphics framework, CV, or research manuscript is changed. A small
orientation-control disclosure adds a single-click/tap alternative to dragging.
It is collapsed by default; there is still no visible explanatory caption.

## Automatic cache invalidation

`src/assets.rs` computes a deterministic 64-bit FNV-1a fingerprint of the bytes
of each public rendering asset (including the generated mesh). It writes the
same bytes at `name.<16 hexadecimal digits>.extension` and a stable alias.
Askama emits fingerprinted URLs. A changed asset receives a different URL,
without editing a version number. FNV-1a is a non-cryptographic cache key, not
an integrity signature. The static audit recomputes the key and checks that
both copies are byte-identical. Stable aliases support the local preview and
pre-9 HTML; a fresh deployment does not retain all earlier fingerprint names.
HTML and CV cache policy is controlled by GitHub Pages. Refresh if an old
cached HTML document requests a removed fingerprint. Do not rely on URL
fingerprinting as a guarantee of atomic CDN propagation.

The JPEG sharing image is a format conversion of the existing approved public
WebP; the original WebP is unchanged. Social crawlers may fetch the JPEG, but
the ordinary browser homepage does not. The custom SVG favicon and root ICO
are a stylized two-handle silhouette, not a rendering of the mesh.

## Automated checks and payload budget

Run from the repository root, with Python 3 and Node 22+ available:

```powershell
cargo fmt --all
cargo fmt --all -- --check
cargo clippy --all-targets --locked -- -D warnings
cargo test --locked
cargo run --release --locked
node --test docs/surface-decoder.test.mjs docs/surface-rotation.test.mjs docs/surface-lifecycle.test.mjs
python docs/audit_site.py
```

The site audit independently verifies the actual generated artifact:

- Source/alias bytes and content-based names, public CV and portrait.
- Binary and JSON coordinate/index agreement and expected mesh certificate.
- Canonical and social metadata, local URL references, IDs, accessible names
  and labels, both viewer links, and no eager iframe or external script/image.
- Uncompressed cold homepage budget of 420 KiB, including the binary mesh.
- Strict mode also rejects unexpected files; CI uses it on a fresh export.

Local builds can retain old fingerprints or the standalone preview. The
normal audit reports these, rather than deleting files. Do not upload a dirty
local `dist` directory; the existing Actions workflow builds a fresh one.

Representative independently generated fixture (not the Rust build):
390,179 bytes raw and 246,220 bytes summed gzip estimate before the tiny skip
link change. Includes HTML, CSS, local video and surface scripts, WebP, binary
mesh and SVG icon. Excludes the CV, full-precision JSON, social JPEG and
CoxeterViewer assets, which the homepage does not request on startup. It also
excludes HTTP/TLS headers and an optional browser request for favicon.ico.
The duplicated stable paths increase the deployment artifact size, not the
ordinary homepage transfer. The audit prints the actual numbers after build.
No measured CDN startup or GPU-power claim is made here.

## Deployment

CI retains the Rust format, lint, tests, export and file checks, adds Node 24
and the dependency-free renderer tests, and runs the Python static audit.
The affected official Pages actions move to configure-pages v6,
upload-pages-artifact v5 and deploy-pages v5 (Node 24 runtime or a composite
using the newer upload action). Existing least-privilege permissions and
concurrency remain. Upstream dependency deprecation warnings may still occur;
do not suppress them. Check the new workflow logs after publishing.

## Manual browser acceptance (after release build and after deployment)

Serve `dist` with `python -m http.server 8000 --directory dist`. Test desktop
and mobile widths (including 320 CSS pixels), zoom to 200%, tab through the
page, verify the skip link, try canvas keyboard controls and the optional
orientation buttons with a mouse/touch, and check pause/restart and dragging.
Switch tabs or scroll offscreen while dragging and verify capture is cleared;
rotation should resume when visible if previously playing. Emulate reduced
motion and confirm no automatic rotation until opted in. Disable WebGL or
JavaScript to see the static fallback. Verify that the CV, advisor, research
and both CoxeterViewer links still work. Test the public page in a fresh
browser session and confirm the fingerprinted assets load without 404s.

In Chrome DevTools Network, enable Disable cache for a cold test; record
actual transferred bytes and timings. With ordinary cache enabled, reload for
a warm test. Confirm no viewer assets, remote video iframe, social JPEG or
JSON mesh are fetched before explicitly needed. Use Performance to compare
visible animation against paused, offscreen and hidden states. Run Lighthouse
or another accessibility audit as a supplementary check, not a substitute for
manual keyboard, screen-reader and mobile testing.

## Verification boundaries

The independent numerical fixture passed the static audit (and intentionally
missing/modified fingerprinted file checks). The ten Node tests passed with
simulated DOM/WebGL interfaces, including non-drag controls and interrupted
capture. Inline Chromium fallback layouts passed at 1440, 390 and 320 pixels.
HTTP navigation and WebGL were blocked in the preparation environment; no Rust
compiler was available. The full Rust exporter, local browser/assistive-
technology behavior, live CDN timings and new GitHub Actions workflow remain
to be validated by the developer. This is not a WCAG conformance claim.

Implementation assistance: ChatGPT prepared the patch, static verifier,
regression tests and documentation. Final deployment remains under developer
control.
