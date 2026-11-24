/* A dependency-free WebGL 2 renderer for the Rust-generated genus-two mesh. */
const vertexSource = `#version 300 es
in vec3 aPosition;
in vec3 aNormal;
uniform mat3 uRotation;
uniform float uAspect;
uniform float uRadius;
out vec3 vNormal;
out vec3 vView;

void main() {
    // One orientation, applied to positions and analytic normals alike.
    mat3 rotation = uRotation;
    vec3 p = rotation * (aPosition / uRadius);
    // Camera at the origin, looking down -z. The radius normalization
    // and aspect-aware distance keep every rotation within the viewport.
    p.z -= 3.15 * max(1.0, 1.0 / uAspect);
    vNormal = rotation * aNormal;
    vView = -p;
    // Perspective with near=0.1, far=20, vertical field of view ~39 degrees.
    float f = 2.80;
    float near = 0.1, far = 20.0;
    gl_Position = vec4(f*p.x/uAspect, f*p.y,
        -(far+near)/(far-near)*p.z - 2.0*far*near/(far-near), -p.z);
}`;

const fragmentSource = `#version 300 es
precision highp float;
in vec3 vNormal;
in vec3 vView;
out vec4 color;
void main() {
    vec3 n = normalize(vNormal);
    vec3 light = normalize(vec3(-0.45, 0.75, 1.0));
    vec3 fill = normalize(vec3(0.8, -0.2, 0.5));
    vec3 view = normalize(vView);
    float diffuse = max(dot(n, light), 0.0);
    float bounce = max(dot(n, fill), 0.0);
    float specular = pow(max(dot(n, normalize(light + view)), 0.0), 36.0);
    // Restrained navy, with a soft blue highlight and a little fill light.
    vec3 dark = vec3(0.045, 0.055, 0.17);
    vec3 bright = vec3(0.27, 0.33, 0.67);
    vec3 rgb = mix(dark, bright, 0.15 + 0.68*diffuse + 0.17*bounce);
    rgb += vec3(0.25, 0.29, 0.42) * specular * 0.42 * diffuse;
    color = vec4(rgb, 1.0);
}`;

/** Read the versioned little-endian format without assuming host endianness. */
export function decodeMesh(buffer) {
    const view = new DataView(buffer);
    if (view.byteLength < 16 || view.getUint32(0, false) !== 0x47324d31) {
        throw new Error("Invalid G2M1 header");
    }
    const vertices = view.getUint32(4, true);
    const faces = view.getUint32(8, true);
    if (!vertices || vertices > 65536 || !faces || view.getUint32(12, true) !== 0 ||
        view.byteLength !== 16 + 24 * vertices + 6 * faces) {
        throw new Error("Invalid G2M1 dimensions");
    }
    let offset = 16;
    const readVectors = () => {
        const values = new Float32Array(3 * vertices);
        for (let i = 0; i < values.length; i++, offset += 4) {
            values[i] = view.getFloat32(offset, true);
            if (!Number.isFinite(values[i])) throw new Error("Nonfinite mesh coordinate");
        }
        return values;
    };
    const positions = readVectors();
    const normals = readVectors();
    const indices = new Uint16Array(3 * faces);
    for (let i = 0; i < indices.length; i++, offset += 2) {
        indices[i] = view.getUint16(offset, true);
        if (indices[i] >= vertices) throw new Error("Mesh index out of range");
    }
    let radius = 0;
    for (let i = 0; i < positions.length; i += 3) {
        radius = Math.max(radius, Math.hypot(positions[i], positions[i + 1], positions[i + 2]));
        const norm = Math.hypot(normals[i], normals[i + 1], normals[i + 2]);
        if (Math.abs(norm - 1) > 0.001) throw new Error("Invalid mesh normal");
    }
    if (!radius) throw new Error("Empty geometric extent");
    return { positions, normals, indices, radius };
}

// Unit quaternions in [w, x, y, z] order. Multiplication composes rotations
// right-to-left, just like matrix multiplication.
export function multiply(a, b) {
    const [w, x, y, z] = a;
    const [v, i, j, k] = b;
    return [w*v-x*i-y*j-z*k, w*i+x*v+y*k-z*j,
        w*j-x*k+y*v+z*i, w*k+x*j-y*i+z*v];
}

export function unit(q) {
    const length = Math.hypot(...q);
    if (!Number.isFinite(length) || length === 0) throw new Error("Invalid quaternion");
    return q.map(x => x / length);
}

export function axisAngle(axis, angle) {
    const length = Math.hypot(...axis);
    if (!Number.isFinite(length) || length === 0) throw new Error("Invalid rotation axis");
    const s = Math.sin(angle / 2) / length;
    return [Math.cos(angle / 2), ...axis.map(x => s * x)];
}

/** Column-major rotation matrix for GLSL. */
export function rotationMatrix(q) {
    const [w, x, y, z] = unit(q);
    return new Float32Array([
        1-2*(y*y+z*z), 2*(x*y+w*z), 2*(x*z-w*y),
        2*(x*y-w*z), 1-2*(x*x+z*z), 2*(y*z+w*x),
        2*(x*z+w*y), 2*(y*z-w*x), 1-2*(x*x+y*y)
    ]);
}

/** Screen point on a virtual unit sphere, or on its equator outside it. */
export function trackball(x, y, rect) {
    const radius = Math.min(rect.width, rect.height) / 2;
    if (radius <= 0) return [0, 0, 1];
    const u = (x - rect.left - rect.width / 2) / radius;
    const v = (rect.top + rect.height / 2 - y) / radius;
    const r2 = u*u + v*v;
    if (r2 <= 1) return [u, v, Math.sqrt(1-r2)];
    return [u/Math.sqrt(r2), v/Math.sqrt(r2), 0];
}

/** Shortest rotation carrying unit vector a to unit vector b. */
export function between(a, b) {
    const dot = Math.max(-1, Math.min(1, a[0]*b[0]+a[1]*b[1]+a[2]*b[2]));
    const cross = (u,v) => [u[1]*v[2]-u[2]*v[1], u[2]*v[0]-u[0]*v[2], u[0]*v[1]-u[1]*v[0]];
    if (dot < -1 + 1e-12) {
        // The antipodal case has no unique shortest axis; choose a stable one.
        return axisAngle(cross(a, Math.abs(a[0]) < 0.8 ? [1,0,0] : [0,1,0]), Math.PI);
    }
    return unit([1+dot, ...cross(a,b)]);
}

export const spinAxis = Object.freeze([0.35, 0.56, 0.75]);
export const initialOrientation = () => unit(multiply(axisAngle([0,0,1], -0.12), axisAngle([1,0,0], 0.67)));
export function advance(q, radians) {
    // Right multiplication spins about a body-fixed axis. After a drag d,
    // the world-space axis is d times its old direction and stays there.
    return unit(multiply(q, axisAngle(spinAxis, radians)));
}

function compile(gl, kind, source) {
    const shader = gl.createShader(kind);
    if (!shader) throw new Error("Cannot allocate shader");
    gl.shaderSource(shader, source);
    gl.compileShader(shader);
    if (!gl.getShaderParameter(shader, gl.COMPILE_STATUS)) {
        const message = gl.getShaderInfoLog(shader);
        gl.deleteShader(shader);
        throw new Error(message || "Shader compilation failed");
    }
    return shader;
}

function makeRenderer(gl, mesh) {
    const vertex = compile(gl, gl.VERTEX_SHADER, vertexSource);
    const fragment = compile(gl, gl.FRAGMENT_SHADER, fragmentSource);
    const program = gl.createProgram();
    if (!program) throw new Error("Cannot allocate program");
    gl.attachShader(program, vertex);
    gl.attachShader(program, fragment);
    gl.linkProgram(program);
    gl.deleteShader(vertex);
    gl.deleteShader(fragment);
    if (!gl.getProgramParameter(program, gl.LINK_STATUS)) {
        throw new Error(gl.getProgramInfoLog(program) || "Shader link failed");
    }
    gl.useProgram(program);
    const vao = gl.createVertexArray();
    if (!vao) throw new Error("Cannot allocate vertex array");
    gl.bindVertexArray(vao);
    for (const [name, values] of [["aPosition", mesh.positions], ["aNormal", mesh.normals]]) {
        const buffer = gl.createBuffer();
        if (!buffer) throw new Error("Cannot allocate vertex buffer");
        gl.bindBuffer(gl.ARRAY_BUFFER, buffer);
        gl.bufferData(gl.ARRAY_BUFFER, values, gl.STATIC_DRAW);
        const location = gl.getAttribLocation(program, name);
        if (location < 0) throw new Error(`Missing attribute ${name}`);
        gl.enableVertexAttribArray(location);
        gl.vertexAttribPointer(location, 3, gl.FLOAT, false, 0, 0);
    }
    const indexBuffer = gl.createBuffer();
    if (!indexBuffer) throw new Error("Cannot allocate index buffer");
    gl.bindBuffer(gl.ELEMENT_ARRAY_BUFFER, indexBuffer);
    gl.bufferData(gl.ELEMENT_ARRAY_BUFFER, mesh.indices, gl.STATIC_DRAW);
    const rotation = gl.getUniformLocation(program, "uRotation");
    const aspect = gl.getUniformLocation(program, "uAspect");
    gl.uniform1f(gl.getUniformLocation(program, "uRadius"), mesh.radius);
    gl.enable(gl.DEPTH_TEST);
    gl.enable(gl.CULL_FACE);
    gl.cullFace(gl.BACK);
    gl.clearColor(0, 0, 0, 0);
    if (gl.getError() !== gl.NO_ERROR) throw new Error("WebGL buffer initialization failed");
    return (orientation, width, height) => {
        gl.viewport(0, 0, width, height);
        gl.clear(gl.COLOR_BUFFER_BIT | gl.DEPTH_BUFFER_BIT);
        gl.uniformMatrix3fv(rotation, false, rotationMatrix(orientation));
        gl.uniform1f(aspect, width / height);
        gl.drawElements(gl.TRIANGLES, mesh.indices.length, gl.UNSIGNED_SHORT, 0);
    };
}

/** Progressive enhancement: the fallback remains visible until a frame draws. */
export function mountSurface(root) {
    const canvas = root.querySelector("canvas");
    const fallback = root.querySelector("[data-fallback]");
    const toggle = root.querySelector("[data-toggle]");
    const slider = root.querySelector("[data-angle]");
    const adjust = root.querySelector("[data-adjust]");
    const status = root.querySelector("[data-status]");
    if (!canvas || !fallback || !toggle || !status) return;
    const motion = matchMedia("(prefers-reduced-motion: reduce)");
    let playing = !motion.matches;
    let visible = false, failed = false, started = false, ready = false;
    let draw = null, frame = null, last = null;
    let orientation = initialOrientation();
    let drag = null;
    let loadController = null;
    let theta = 0; // standalone slider tracks a reset-relative spin phase
    const speed = 2 * Math.PI / 32; // radians per second, 32-second revolution
    const tau = 2 * Math.PI;

    function stop() {
        if (frame !== null) cancelAnimationFrame(frame);
        frame = null;
        last = null;
        root.dataset.animating = "false";
    }
    function clearDrag() {
        if (drag === null) return;
        const id = drag.id;
        drag = null; // lostpointercapture may dispatch another event
        if (canvas.hasPointerCapture(id)) canvas.releasePointerCapture(id);
    }
    function fail(message) {
        failed = true;
        ready = false;
        stop();
        canvas.hidden = true;
        fallback.removeAttribute("hidden");
        toggle.disabled = true;
        clearDrag();
        loadController?.abort();
        draw = null;
        if (adjust) adjust.hidden = true;
        if (slider) slider.disabled = true;
        root.dataset.state = "fallback";
        status.textContent = message;
    }
    function render() {
        if (!draw || failed) return false;
        const rect = canvas.getBoundingClientRect();
        const scale = Math.min(devicePixelRatio || 1, 2);
        const width = Math.max(1, Math.round(rect.width * scale));
        const height = Math.max(1, Math.round(rect.height * scale));
        if (canvas.width !== width) canvas.width = width;
        if (canvas.height !== height) canvas.height = height;
        draw(orientation, width, height);
        if (slider) slider.value = String(Math.round(theta * 180 / Math.PI) % 360);
        return true;
    }
    function active() { return ready && playing && drag === null && visible && !document.hidden && !failed; }
    function tick(now) {
        frame = null;
        if (!active()) { stop(); return; }
        if (last !== null) {
            const delta = speed * Math.min((now - last) / 1000, 0.1);
            orientation = advance(orientation, delta);
            theta = (theta + delta) % tau;
        }
        last = now;
        render();
        frame = requestAnimationFrame(tick);
    }
    function reconcile() {
        const label = playing ? "Pause rotation" : "Start rotation";
        toggle.setAttribute("aria-label", label);
        toggle.title = label;
        const icon = toggle.querySelector("[data-rotation-icon]");
        if (icon) {
            icon.textContent = playing ? "Ⅱ" : "▶";
        } else toggle.textContent = label;
        if (active()) {
            if (frame === null) {
                root.dataset.animating = "true";
                frame = requestAnimationFrame(tick);
            }
        } else stop();
    }
    async function start() {
        if (started || failed || document.hidden) return;
        started = true;
        root.dataset.state = "loading";
        status.textContent = "Loading surface…";
        try {
            const gl = canvas.getContext("webgl2", { alpha: true, antialias: true });
            if (!gl) throw new Error("WebGL 2 unavailable");
            loadController = new AbortController();
            const response = await fetch(new URL(root.dataset.mesh, document.baseURI), { signal: loadController.signal });
            if (!response.ok) throw new Error(`Mesh request: ${response.status}`);
            const mesh = decodeMesh(await response.arrayBuffer());
            if (failed) return;
            draw = makeRenderer(gl, mesh);
            canvas.hidden = false;
            render();
            if (gl.getError() !== gl.NO_ERROR || gl.isContextLost()) throw new Error("Initial draw failed");
            fallback.setAttribute("hidden", "");
            ready = true;
            toggle.disabled = false;
            if (adjust) adjust.hidden = false;
            if (slider) slider.disabled = false;
            root.dataset.state = "ready";
            status.textContent = "Genus-two surface · generated in Rust";
            reconcile();
        } catch (error) {
            if (!failed) {
                console.warn("Genus-two surface:", error);
                fail("Genus-two surface (static fallback).");
            }
        } finally {
            loadController = null;
        }
    }
    toggle.addEventListener("click", () => { playing = !playing; reconcile(); });
    if (slider) slider.addEventListener("input", () => {
        playing = false;
        theta = Number(slider.value) * Math.PI / 180 % tau;
        orientation = advance(initialOrientation(), theta);
        reconcile();
        render();
    });
    function reorient(delta) {
        orientation = unit(multiply(delta, orientation));
        render();
    }
    function reset() {
        orientation = initialOrientation();
        theta = 0;
        render();
    }
    canvas.addEventListener("pointerdown", (event) => {
        if (!ready || drag !== null || event.button !== 0) return;
        drag = { id: event.pointerId, point: trackball(event.clientX, event.clientY, canvas.getBoundingClientRect()) };
        canvas.setPointerCapture(event.pointerId);
        reconcile();
    });
    canvas.addEventListener("pointermove", (event) => {
        if (drag?.id !== event.pointerId || !ready) return;
        const next = trackball(event.clientX, event.clientY, canvas.getBoundingClientRect());
        reorient(between(drag.point, next));
        drag.point = next;
    });
    function endDrag(event) {
        if (drag?.id !== event.pointerId) return;
        clearDrag();
        reconcile();
    }
    for (const name of ["pointerup", "pointercancel", "lostpointercapture"]) {
        canvas.addEventListener(name, endDrag);
    }
    canvas.addEventListener("dblclick", () => { if (ready) reset(); });
    function orientByKey(key) {
        const step = Math.PI / 18;
        const rotations = {
            ArrowLeft: [[0,1,0], -step], ArrowRight: [[0,1,0], step],
            ArrowUp: [[1,0,0], -step], ArrowDown: [[1,0,0], step]
        };
        if (!ready) return false;
        if (key === "Home") reset();
        else if (Object.hasOwn(rotations, key)) reorient(axisAngle(...rotations[key]));
        else return false;
        return true;
    }
    canvas.addEventListener("keydown", (event) => {
        if (event.altKey || event.ctrlKey || event.metaKey) return;
        if (orientByKey(event.key)) event.preventDefault();
    });
    // Single-pointer alternative to dragging, including on touch screens.
    if (adjust) {
        for (const button of adjust.querySelectorAll("[data-orient]")) {
            button.addEventListener("click", () => orientByKey(button.dataset.orient));
        }
    }
    motion.addEventListener("change", (event) => {
        // Never restart automatically when a motion preference is relaxed.
        if (event.matches) playing = false;
        reconcile();
    });
    document.addEventListener("visibilitychange", () => {
        if (document.hidden) clearDrag();
        if (!document.hidden && visible) {
            void start();
            if (ready && !playing) render();
        }
        reconcile();
    });
    canvas.addEventListener("webglcontextlost", () => fail("Graphics context lost; showing the static illustration. Reload to retry."));
    const resize = () => {
        if (ready && visible && !document.hidden && !active()) render();
    };
    if (typeof ResizeObserver === "function") new ResizeObserver(resize).observe(root);
    else window.addEventListener("resize", resize);
    if (typeof IntersectionObserver === "function") {
        new IntersectionObserver(([entry]) => {
            visible = entry.isIntersecting;
            if (!visible) clearDrag();
            if (visible) {
                void start();
                if (ready && !playing && !document.hidden) render();
            }
            reconcile();
        }).observe(root);
    } else { visible = true; void start(); }
}

if (typeof document !== "undefined") {
    for (const root of document.querySelectorAll("[data-genus-surface]")) mountSurface(root);
}
