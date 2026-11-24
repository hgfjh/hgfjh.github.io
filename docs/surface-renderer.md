# Native WebGL renderer

The Rust generator supplies the verified, waist-deformed mesh described in
[genus-two.md](genus-two.md). The combinatorial manifold verifier is unchanged.
The JSON is the full-precision inspection format; `genus2.bin` is a rendering
representation, not an independent topological certificate.

## Binary layout (G2M1)

All integer and floating-point fields are little-endian except the four literal
magic bytes `G2M1`. There is no padding beyond the reserved header word.

| Offset | Type | Contents |
| --- | --- | --- |
| 0 | 4 bytes | G2M1 |
| 4 | u32 | V, vertex count (1..65536) |
| 8 | u32 | F, triangle count (positive) |
| 12 | u32 | reserved zero |
| 16 | 3V f32 | positions, xyz triples |
| 16+12V | 3V f32 | analytic normals, xyz triples |
| 16+24V | 3F u16 | zero-based triangle indices |

Total length is 16+24V+6F. The default mesh has 6,142 vertices and 12,288 faces,
so its binary is 221,152 bytes. The decoder checks length, magic, dimensions,
finite coordinates, approximately unit normals, index bounds, and extent. Rust
verifies topology before export. Neither the binary decoder nor float conversion
is a general embeddedness verifier.

## Geometry and projection

The renderer stores its orientation as a unit quaternion q in [w,x,y,z]
order. The shader receives a column-major rotation matrix derived from q and
applies it to both positions and normals. The initial orientation is
Rz(-0.12) Rx(0.67). The body-fixed spin axis is the normalized vector
(0.35,0.56,0.75); angular velocity is 2 pi / 32 radians per second. Unlike
the previous spin, this can reveal edge-on views. The camera, palette and
lighting are unchanged.

A pointer is projected to the front hemisphere of a virtual unit sphere (or
its equator outside the circle). For successive unit vectors a and b the
shortest rotation has quaternion proportional to (1+a dot b, a cross b),
with an explicit antipodal case. For a drag rotation d we update q <- d q
(screen coordinates). For animation we update q <- q r, where r is a small
rotation around the body-fixed axis. These multiplications have different
orders intentionally. For a body axis u, the world axis q u q^-1 becomes
 d (q u q^-1) d^-1 after a drag, and remains fixed during subsequent spin.
Normalization prevents accumulated floating-point drift. During ordinary
frames only the rotation and aspect uniforms change. Vertex and index
buffers are uploaded once.

The vertex shader scales positions by the maximum distance from the origin,
then translates them down the camera's negative z axis. Perspective uses
near=0.1, far=20 and f=2.8 (vertical field of view approximately 39 degrees).
The perspective division is performed by the GPU. A depth buffer and back-face
culling resolve occlusion; outward triangle orientation comes from Rust.
The fragment shader renormalizes interpolated normals and combines ambient,
diffuse, fill and a small Blinn-Phong highlight. The navy palette is chosen to
match the site's visual identity, not to encode mathematical data.

## Progressive enhancement

The standalone page remains useful without JavaScript or WebGL 2. The renderer
waits until its figure is visible, checks for WebGL, fetches and decodes the
binary, compiles shaders, uploads buffers and draws a first frame before hiding
the fallback. Missing graphics capability, failed requests or context loss
leave the schematic visible. Reload after a context loss to retry. The loop
stops when paused, offscreen or the document is hidden. Reduced-motion users
start with a static rendered view and can opt in. Pointer capture supports mouse, pen and touch dragging. The loop stops during
a drag and resumes on release if rotation was enabled before it. Arrow keys
rotate the focused canvas; Home or double-click resets its orientation.
The standalone angle slider resets to a selected spin phase and pauses.
The homepage has a labelled pause/play icon and screen-reader-only
instructions and status; the visible caption is removed. Touch dragging
consumes gestures inside the canvas, while the surrounding page scrolls normally.
The SVG is hidden using its hidden attribute, not an HTMLElement property. The canvas resolution follows its CSS size with DPR capped at 2.

## Local workflow

Run `cargo fmt --all`, `cargo clippy --all-targets --locked -- -D warnings`,
`cargo test --locked`, and `cargo run --release --locked`. Then copy
`docs/surface-preview.html` into `dist/` and serve `dist/` over HTTP (see README).
The preview is local-only by default. The homepage loads the renderer and requests the binary only when its
surface enters the viewport. Explicit 8.5 query versions on CSS, JavaScript
and the binary reduce stale-asset problems; automatic fingerprinting is
left for the final audit.

Optional tests (Node 22+):
`node --test docs/surface-decoder.test.mjs docs/surface-rotation.test.mjs docs/surface-lifecycle.test.mjs`.
The quaternion tests check matrix conventions, antipodal trackball cases,
axis preservation after dragging and long-run normalization. Lifecycle
tests use a mock GL/DOM; actual browser GPU validation is still needed.
Inspect the surface at several angles, try start/pause, resize the window,
scroll away and back, and test the OS/browser reduced-motion preference.

Implementation assistance: ChatGPT generated the renderer, binary exporter,
preview and tests; local Rust compilation and full browser WebGL validation
remain the developer's responsibility.
