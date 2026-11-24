# Genus-two mesh — Commit 8.2

## Geometry and topology

Use A=2.65, B=1.35, a=1.12, r=0.53, lambda=0.43. Put

- q0 = 1 - (x/A)^2 - (y/B)^2;
- q1 = (x-a)^2 + y^2 - r^2;
- q2 = (x+a)^2 + y^2 - r^2;
- P = q0 q1 q2.

The two holes are disjoint (a>r) and inside the ellipse: on either closed
hole disk, |x| <= a+r and |y| <= r, so (x/A)^2+(y/B)^2 < 1.
Thus P is positive exactly in the interior of the planar pair of pants D,
zero on its three boundary curves and negative outside. The surface F=0,
where F=z^2-lambda^2 P, is its double. The gradient is nonzero: away from
z=0 the z derivative is nonzero, and at a rim exactly one regular factor
vanishes. This proves smoothness and embeddedness of the analytic surface.
The double is connected, closed and orientable with Euler characteristic
2 chi(D) = -2, hence genus two.

## Triangulation

The right half of D is the convex half-ellipse minus a circle centered
at (a,0). The ray direction (cos theta,sin theta) hits the outer ellipse at
its positive quadratic root or the straight seam at -a/cos theta. The
corner angle atan2(B,-a) separates the regimes. Sample the arc in 64
intervals and the seam in 32 intervals, including the corners exactly.
The 16 radial intervals use t=(1-cos(pi*j/16))/2, concentrating samples
near the rounded rims. At radius r+(rho-r)t, each ray samples nested chords.

Each angular wedge has angle strictly between 0 and pi. In the basis of
its two bounding rays, the ring chords have equation u/R_i+v/R_{i+1}=1
with u,v>=0. As both radii increase strictly with t, these chords are
strictly nested, and successive chords bound a convex quadrilateral.
Splitting each quadrilateral gives a non-overlapping planar triangulation.
Different wedges have disjoint interiors; reflection across x=0 has
interior in the other half-plane. Seam vertices share indices, not a
floating-point proximity test. The planar polygon is an approximation of D:
inner boundary chords are not on the exact circular holes between samples.
The lifted mesh is therefore a PL approximation, not the exact level set.

Lift to heights +/-lambda sqrt(P) at vertices; use one shared index at
boundary vertices, where z=0. The planar edge check verifies that no
interior edge has two boundary endpoints and no triangle has three boundary
vertices. Consequently the piecewise-linear height is strictly positive
in the interior of the polygonal region. Both sheets are graphs over that
region, and intersect only along the actual polygonal boundary. This is
the geometric embedding argument for this specific generator in exact
arithmetic. Floating-point checks detect invalid local areas and heights;
they are not an interval-arithmetic proof or a generic intersection checker.

Orient upper faces counterclockwise in projection and reverse lower faces.
Use the normalized analytic gradient (-lambda^2 P_x,-lambda^2 P_y,2z)
for smooth shading. These are normals to the analytic surface at the sampled
vertices, not flat normals to individual PL triangles.

## Verification

`src/topology.rs` verifies distinct valid indices, no duplicate faces,
exactly two opposite directed incidences on each edge, one circular link
per vertex, and connectedness. Only then is genus computed from Euler
characteristic. This detects, for example, two spheres pinched at a vertex,
which edge incidence alone cannot detect. It does not certify arbitrary
geometric realizations. The geometry module additionally checks projected
areas, boundary incidence and positive interior polynomial values.

For the default mesh, the planar grid has 2*96*17 - 33 = 3231 vertices:
33 seam vertices are shared. Its boundary has 2*64 + 2*96 = 320 vertices.
Doubling gives V=6142, F=12288, E=18432, chi=-2. The tests check those counts,
unit normals, level-set residuals, alternative resolutions, and invalid
combinatorial surfaces.

## Export and scope

`cargo run --release --locked` writes `dist/static/meshes/genus2.json`.
The schema has `positions` and `normals` (arrays of triples), `triangles`
(zero-based index triples), and `stats` (vertices, edges, faces, euler, genus).
The homepage renderer now uses the binary export described in
[surface-renderer.md](surface-renderer.md). The original JSON remains an
inspection format. The 8.5 refinement below changes the geometry, not the
abstract triangulation.

No new crates are needed. ChatGPT assisted with this implementation and the
mathematical documentation. The accompanying independent Python reference
check is not a substitute for compiling and running the Rust tests.

## Refined double-torus shape (Commit 8.5)

Apply the ambient diffeomorphism

    Psi(x,y,z) = (x, s(x)y, t(x)z)
    e(x) = exp(-(x/0.65)^2)
    s(x) = 1 - 0.60 e(x)
    t(x) = 1.12 (1 - 0.40 e(x)).

Both scales are strictly positive (s >= 0.40, t >= 0.672). The inverse divides
by the same scales at fixed x. This narrows the central bridge and slightly
thickens the lobes, without changing the topology of the smooth surface.
The new analytic normals are normalized J(Psi)^(-T) N, namely the vector

    (Nx - s'(x)y Ny/s(x) - t'(x)z Nz/t(x), Ny/s(x), Nz/t(x)).

Here (x,y,z) is the original point. The exported f64 positions and normals
are those of the deformed surface. The level-set residual test evaluates the
original equation at the inverse-transformed vertex.

A nonlinear diffeomorphism alone does not establish that the straight triangles
between deformed vertices remain embedded. To address this, `check_waist`
verifies positive signed projected areas and the original boundary incidence
on the new planar coordinates. Temporarily fill each original polygonal hole
with triangles from its centre to the boundary edges. These fans complete the
original planar domain to the convex outer polygon, since the holes are convex
inscribed polygons. Check the orientations of these triangles before and after
moving the vertices; the hole centres are fixed by the map. The PL map on this
filled disk keeps x fixed. On each triangle its affine Jacobian has positive
orientation, so the new y is strictly increasing along each vertical segment.
The same is true on vertical edges by continuity (the adjacent affine maps
agree). Each vertical fibre of the original convex polygon is an interval;
joining the finitely many pieces shows that the map is strictly increasing on
the entire fibre. Distinct x coordinates remain distinct. Thus the PL map is
injective, including on the punctured disk. The new upper/lower heights retain
the original sign and the zero rim, so the two-graph embedding argument applies
again. The temporary triangles are not exported. All counts remain unchanged.

This is a construction-specific exact-arithmetic argument with floating-point
checks, not an interval-arithmetic certificate. The independent Python reference
checks the same projected triangle areas and fan areas at all four resolutions.
