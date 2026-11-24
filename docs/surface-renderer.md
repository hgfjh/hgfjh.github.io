# Native WebGL renderer

The existing Rust generator and manifold verification remain unchanged.
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

The shader rotates positions and normals with the same orthogonal matrix:
`Rz(-0.12) Rx(0.42) Rz(theta) Rx(0.25)`. Angles are in radians. The surface
spins around a tilted axis rather than passing through an edge-on view. Its
normal stays within a cone about the camera direction. The angular velocity is
2 pi / 32 radians per second. Only the angle and aspect uniforms change between
ordinary frames. Vertex and element buffers are uploaded once.

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
start with a static rendered view and can opt in. Moving the angle slider pauses
the animation. The canvas resolution follows its CSS size with DPR capped at 2.

## Local workflow

Run `cargo fmt --all`, `cargo clippy --all-targets --locked -- -D warnings`,
`cargo test --locked`, and `cargo run --release --locked`. Then copy
`docs/surface-preview.html` into `dist/` and serve `dist/` over HTTP (see README).
The preview is local-only by default. The new JS and binary are exported but
are not requested by the academic homepage in this commit.

Optional decoder tests (Node 22+): `node --test docs/surface-decoder.test.mjs`.
Inspect the surface at several angles, try start/pause, resize the window,
scroll away and back, and test the OS/browser reduced-motion preference.

Implementation assistance: ChatGPT generated the renderer, binary exporter,
preview and tests; local Rust compilation and full browser WebGL validation
remain the developer's responsibility.
