import { test } from "node:test";
import assert from "node:assert/strict";
import { decodeMesh } from "../static/surface.js";

// Minimal synthetic rendering fixture, not a genus-two topology certificate.
function fixture() {
    const bytes = new ArrayBuffer(16 + 24 * 3 + 6);
    const d = new DataView(bytes);
    d.setUint32(0, 0x47324d31);
    d.setUint32(4, 3, true);
    d.setUint32(8, 1, true);
    const values = [0,0,0, 1,0,0, 0,1,0, 0,0,1, 0,0,1, 0,0,1];
    values.forEach((v,i) => d.setFloat32(16 + i*4, v, true));
    [0,1,2].forEach((v,i) => d.setUint16(16 + 72 + i*2, v, true));
    return bytes;
}
test("decodes little-endian geometry", () => {
    const m = decodeMesh(fixture());
    assert.deepEqual([...m.positions], [0,0,0,1,0,0,0,1,0]);
    assert.deepEqual([...m.indices], [0,1,2]);
    assert.equal(m.radius, 1);
});
test("rejects truncated, malformed and invalid payloads", () => {
    assert.throws(() => decodeMesh(fixture().slice(0,-1)));
    for (const mutate of [
        d => d.setUint32(0, 0),
        d => d.setUint32(4, 0, true),
        d => d.setUint32(12, 1, true),
        d => d.setFloat32(16, NaN, true),
        d => d.setFloat32(16 + 36 + 8, 0, true),
        d => d.setUint16(16 + 72, 3, true),
    ]) {
        const bytes = fixture(); mutate(new DataView(bytes));
        assert.throws(() => decodeMesh(bytes));
    }
});
