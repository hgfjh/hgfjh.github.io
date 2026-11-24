import { test } from "node:test";
import assert from "node:assert/strict";
import { advance, axisAngle, between, initialOrientation, multiply, rotationMatrix, spinAxis, trackball, unit } from "../static/surface.js";

function rotate(q, [x,y,z]) {
    const m = rotationMatrix(q);
    return [m[0]*x+m[3]*y+m[6]*z, m[1]*x+m[4]*y+m[7]*z, m[2]*x+m[5]*y+m[8]*z];
}
function close(a,b,eps=2e-7) {
    assert.equal(a.length,b.length);
    a.forEach((x,i)=>assert.ok(Math.abs(x-b[i])<eps, `${x} vs ${b[i]}`));
}
test("matrix convention and composition agree", () => {
    const a=axisAngle([0,0,1],Math.PI/2), b=axisAngle([1,0,0],Math.PI/2);
    close(rotate(a,[1,0,0]),[0,1,0]);
    close(rotate(b,[0,1,0]),[0,0,1]);
    close(rotate(multiply(a,b),[0.3,0.4,0.5]),rotate(a,rotate(b,[0.3,0.4,0.5])));
});
test("trackball gives unit vectors and shortest rotations, including antipodes", () => {
    const rect={left:10,top:20,width:200,height:100};
    const a=trackball(110,70,rect), b=trackball(135,55,rect), c=trackball(500,70,rect);
    close(a,[0,0,1]); close(c,[1,0,0]);
    close([Math.hypot(...b)],[1]);
    close(rotate(between(a,b),a),b);
    close(rotate(between(a,a),a),a);
    close(rotate(between(c,[-1,0,0]),c),[-1,0,0]);
});
test("body-axis spin preserves the reoriented world axis", () => {
    const q=initialOrientation(), d=axisAngle([0,1,0],1.1), changed=unit(multiply(d,q));
    const axis=unit(spinAxis), world=rotate(changed,axis);
    assert.ok(Math.hypot(...world.map((x,i)=>x-rotate(q,axis)[i]))>0.1);
    close(world,rotate(d,rotate(q,axis)));
    for (const t of [0,0.1,1.7,Math.PI,2*Math.PI]) close(rotate(advance(changed,t),axis),world);
    close([...rotationMatrix(advance(changed,2*Math.PI))],[...rotationMatrix(changed)]);
});
test("long animation keeps unit quaternions and proper orthonormal matrices", () => {
    let q=initialOrientation();
    for(let i=0;i<50000;i++) q=advance(q,2*Math.PI/32/60);
    close([Math.hypot(...q)],[1],1e-12);
    const m=rotationMatrix(q), a=[...m.slice(0,3)],b=[...m.slice(3,6)],c=[...m.slice(6,9)];
    close([Math.hypot(...a),Math.hypot(...b),Math.hypot(...c)],[1,1,1]);
    close([a[1]*b[2]-a[2]*b[1],a[2]*b[0]-a[0]*b[2],a[0]*b[1]-a[1]*b[0]],c);
});
