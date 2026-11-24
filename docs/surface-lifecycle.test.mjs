import { test } from "node:test";
import assert from "node:assert/strict";
import { mountSurface } from "../static/surface.js";

// Exercise the real lifecycle and event handlers, not GPU rendering. The fake
// DOM deliberately does not reflect an SVG's .hidden property to an attribute.
class Element {
    constructor() { this.listeners = {}; this.attrs = new Map(); this.dataset = {}; this.hidden = false; this.disabled = false; }
    addEventListener(name, fn) { (this.listeners[name] ??= []).push(fn); }
    emit(name, event = {}) { for (const fn of this.listeners[name] ?? []) fn(event); }
    setAttribute(name, value) { this.attrs.set(name, value); }
    removeAttribute(name) { this.attrs.delete(name); }
    querySelector() { return null; }
    getBoundingClientRect() { return {left:0,top:0,width:200,height:200}; }
    setPointerCapture(id) { this.capture = id; }
    hasPointerCapture(id) { return this.capture === id; }
    releasePointerCapture() { this.capture = null; }
}
function fixture() {
    const buffer = new ArrayBuffer(16+24*3+6), d = new DataView(buffer);
    d.setUint32(0,0x47324d31); d.setUint32(4,3,true); d.setUint32(8,1,true);
    [0,0,0,1,0,0,0,1,0,0,0,1,0,0,1,0,0,1].forEach((x,i)=>d.setFloat32(16+4*i,x,true));
    [0,1,2].forEach((x,i)=>d.setUint16(88+2*i,x,true));
    return buffer;
}
function harness(t, {reduced=false, available=true} = {}) {
    const matrices=[], frames=new Map(); let sequence=0, observer, fetches=0;
    const gl = new Proxy({
        NO_ERROR:0, getError:()=>0, isContextLost:()=>false,
        getShaderParameter:()=>true,getProgramParameter:()=>true,
        getAttribLocation:()=>0, uniformMatrix3fv:(_loc,_transpose,m)=>matrices.push([...m]),
    }, { get:(target,key)=>key in target?target[key]:(key.toUpperCase()===key?1:()=>({})) });
    const root=new Element(), canvas=new Element(), fallback=new Element(), toggle=new Element(), status=new Element();
    root.dataset.mesh="mesh.bin"; canvas.hidden=true; toggle.disabled=true;
    canvas.getContext=()=>available?gl:null;
    const adjust=new Element(); adjust.hidden=true;
    const directions=["ArrowLeft","ArrowRight","ArrowUp","ArrowDown","Home"].map(key=>{const b=new Element();b.dataset.orient=key;return b;});
    adjust.querySelectorAll=()=>directions;
    const children={"[data-adjust]":adjust,canvas,"[data-fallback]":fallback,"[data-toggle]":toggle,"[data-status]":status};
    root.querySelector=key=>children[key]??null;
    const document=new Element(); document.hidden=false; document.baseURI="https://example.test/";
    const motion=new Element(); motion.matches=reduced;
    const globals={document,matchMedia:()=>motion,devicePixelRatio:1,
        fetch:async()=>{fetches++;return {ok:true,arrayBuffer:async()=>fixture()};},
        requestAnimationFrame:fn=>{frames.set(++sequence,fn);return sequence;},
        cancelAnimationFrame:id=>frames.delete(id),
        ResizeObserver:class {observe(){}},
        IntersectionObserver:class {constructor(fn){observer=fn;}observe(){}}
    };
    const saved=Object.fromEntries(Object.keys(globals).map(k=>[k,Object.getOwnPropertyDescriptor(globalThis,k)]));
    for(const [k,v] of Object.entries(globals)) Object.defineProperty(globalThis,k,{value:v,configurable:true,writable:true});
    t.after(()=>{for(const [k,v] of Object.entries(saved)) if(v) Object.defineProperty(globalThis,k,v); else delete globalThis[k];});
    mountSurface(root);
    return {root,canvas,fallback,toggle,status,motion,document,matrices,frames,adjust,directions,fetches:()=>fetches,
        visible:async value=>{observer([{isIntersecting:value}]);await new Promise(resolve=>setImmediate(resolve));},
        tick:now=>{const pending=[...frames];frames.clear();for(const [,fn] of pending) fn(now);}};
}
test("lazy initialization, real SVG attribute, dragging and animation lifecycle", async t=>{
    const h=harness(t); assert.equal(h.fetches(),0); await h.visible(true);
    assert.equal(h.root.dataset.state,"ready"); assert.equal(h.fetches(),1);
    assert.ok(h.fallback.attrs.has("hidden")); assert.equal(h.canvas.hidden,false);
    assert.equal(h.frames.size,1); h.tick(0); h.tick(16);
    const before=h.matrices.at(-1);
    h.canvas.emit("pointerdown",{button:0,pointerId:1,clientX:100,clientY:100});
    assert.equal(h.frames.size,0); assert.equal(h.canvas.capture,1);
    h.canvas.emit("pointermove",{pointerId:1,clientX:145,clientY:120});
    assert.notDeepEqual(h.matrices.at(-1),before);
    const dragged=h.matrices.at(-1);
    h.canvas.emit("pointerup",{pointerId:1}); assert.equal(h.frames.size,1);
    h.tick(32); h.tick(48); assert.notDeepEqual(h.matrices.at(-1),dragged);
    h.toggle.emit("click"); assert.equal(h.frames.size,0);
    h.canvas.emit("pointerdown",{button:0,pointerId:2,clientX:100,clientY:100});
    h.canvas.emit("pointercancel",{pointerId:2}); assert.equal(h.frames.size,0);
    const paused=h.matrices.at(-1);
    h.canvas.emit("keydown",{key:"ArrowRight",preventDefault(){}});assert.notDeepEqual(h.matrices.at(-1),paused);
    h.toggle.emit("click"); await h.visible(false); assert.equal(h.frames.size,0);
    await h.visible(true); assert.equal(h.frames.size,1);
    h.document.hidden=true;h.document.emit("visibilitychange");assert.equal(h.frames.size,0);
    h.canvas.emit("webglcontextlost");assert.equal(h.root.dataset.state,"fallback");assert.ok(!h.fallback.attrs.has("hidden"));
});
test("reduced-motion preference requires opt-in and stops active animation",async t=>{
    const h=harness(t,{reduced:true});await h.visible(true);assert.equal(h.frames.size,0);
    h.toggle.emit("click");assert.equal(h.frames.size,1);
    h.motion.emit("change",{matches:true});assert.equal(h.frames.size,0);
    h.motion.emit("change",{matches:false});assert.equal(h.frames.size,0);
});
test("missing WebGL leaves schematic visible without downloading mesh",async t=>{
    const h=harness(t,{available:false});await h.visible(true);assert.equal(h.root.dataset.state,"fallback");
    assert.equal(h.fetches(),0);assert.equal(h.canvas.hidden,true);assert.ok(!h.fallback.attrs.has("hidden"));
});

test("single-click orientation controls and interrupted drag",async t=>{
    const h=harness(t);await h.visible(true);assert.equal(h.adjust.hidden,false);
    h.toggle.emit("click");const before=h.matrices.at(-1);h.directions[0].emit("click");
    assert.notDeepEqual(h.matrices.at(-1),before);h.directions[4].emit("click");
    assert.deepEqual(h.matrices.at(-1),before);assert.equal(h.frames.size,0);
    h.toggle.emit("click");h.canvas.emit("pointerdown",{button:0,pointerId:3,clientX:100,clientY:100});
    assert.equal(h.frames.size,0);h.document.hidden=true;h.document.emit("visibilitychange");
    assert.equal(h.canvas.capture,null);h.document.hidden=false;h.document.emit("visibilitychange");
    assert.equal(h.frames.size,1);h.canvas.emit("webglcontextlost");assert.equal(h.adjust.hidden,true);
    assert.equal(h.frames.size,0);
});
