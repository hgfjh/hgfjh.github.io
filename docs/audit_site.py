"""Audit a freshly exported academic homepage using only Python's standard library.

Run from repository root: python docs/audit_site.py [--strict]. Strict mode
rejects unexpected published files; use it in CI, whose dist starts empty.
This checks static artifacts, not live CDN behavior or WCAG conformance.
"""
import argparse
import gzip
import json
import struct
from html.parser import HTMLParser
from pathlib import Path
from urllib.parse import urlsplit

ORIGIN = "https://oluolorode.com/"
VIEWER = ORIGIN + "CoxeterViewer5D/"
ASSETS = {
    "style": "static/style.css",
    "videos": "static/videos.js",
    "surface": "static/surface.js",
    "portrait": "static/images/portrait.webp",
    "mesh": "static/meshes/genus2.bin",
    "icon": "static/favicon.svg",
    "share": "static/images/share.jpg",
}


def require(condition, message):
    if not condition:
        raise ValueError(message)


def fingerprint(data):
    value = 0xcbf29ce484222325
    for byte in data:
        value = ((value ^ byte) * 0x100000001b3) & 0xffffffffffffffff
    return f"{value:016x}"


def fingerprinted(path, data):
    stem, extension = path.rsplit(".", 1)
    return f"{stem}.{fingerprint(data)}.{extension}"


class Document(HTMLParser):
    def __init__(self, html):
        super().__init__()
        self.tags = []
        self.feed(html)

    def handle_starttag(self, tag, attrs):
        self.tags.append((tag, dict(attrs)))

    def find(self, tag, **attrs):
        return [a for t, a in self.tags if t == tag and
                all(a.get(k) == v for k, v in attrs.items())]


def audit(root, dist, strict=False):
    def read(path):
        result = (dist / path).read_bytes()
        require(bool(result), f"Empty file: {path}")
        return result

    expected = {"index.html", "cv.pdf", "favicon.ico", "static/meshes/genus2.json"}
    urls, data = {}, {}
    for name, path in ASSETS.items():
        raw = read(path)
        if name != "mesh":
            require(raw == (root / path).read_bytes(), f"Source differs: {path}")
        hashed = fingerprinted(path, raw)
        require(raw == read(hashed), f"Fingerprint alias differs: {hashed}")
        expected.update((path, hashed))
        urls[name] = "./" + hashed
        data[name] = raw

    require(read("favicon.ico") == (root / "static/favicon.ico").read_bytes(),
            "ICO differs from source")
    require(read("cv.pdf") == (root / "assets/cv-public.pdf").read_bytes(),
            "Public CV differs from source")
    require(data["portrait"].startswith(b"RIFF") and data["portrait"][8:12] == b"WEBP",
            "Portrait is not WebP")
    require(data["share"].startswith(b"\xff\xd8"), "Sharing image is not JPEG")
    require(read("favicon.ico").startswith(b"\x00\x00\x01\x00"), "Invalid ICO")

    # The binary representation must agree with the verified f64 JSON geometry,
    # not merely have the correct total number of bytes.
    mesh = json.loads(read("static/meshes/genus2.json"))
    require(mesh["stats"] == dict(vertices=6142, edges=18432, faces=12288, euler=-2, genus=2),
            "Unexpected mesh certificate")
    v, f = 6142, 12288
    require(len(mesh["positions"]) == len(mesh["normals"]) == v and
            len(mesh["triangles"]) == f, "Mesh counts differ")
    require(all(len(row) == 3 for key in ("positions", "normals", "triangles")
                for row in mesh[key]), "Mesh triples malformed")
    binary = struct.pack("<4sIII", b"G2M1", v, f, 0)
    for key in ("positions", "normals"):
        binary += struct.pack(f"<{3*v}f", *(x for row in mesh[key] for x in row))
    binary += struct.pack(f"<{3*f}H", *(x for row in mesh["triangles"] for x in row))
    # JSON numeric rendering may omit the sign of an exact zero. Compare floats
    # numerically (including signed zeros) while requiring exact indices/header.
    packed = data["mesh"]
    require(len(packed) == len(binary) == 221152, "Mesh binary size differs")
    require(packed[:16] == binary[:16] and packed[16+24*v:] == binary[16+24*v:],
            "Mesh binary header or triangle indices differ")
    require(struct.unpack_from(f"<{6*v}f", packed, 16) ==
            struct.unpack_from(f"<{6*v}f", binary, 16), "Mesh coordinates differ")

    html = read("index.html")
    doc = Document(html.decode("utf-8"))
    require(len(doc.find("html", lang="en")) == 1 and len(doc.find("h1")) == 1,
            "Missing document language or unique h1")
    require(len(doc.find("main", id="main", tabindex="-1")) == 1, "Skip target is not focusable")
    require(len(doc.find("title")) == 1 and len(doc.find("meta", name="description")) == 1,
            "Missing page title or description")
    require(len(doc.find("link", rel="canonical", href=ORIGIN)) == 1,
            "Missing canonical URL")
    require(len(doc.find("meta", property="og:url", content=ORIGIN)) == 1,
            "Open Graph origin is incorrect")
    require(len(doc.find("meta", property="og:image", content=ORIGIN + urls["share"][2:])) == 1,
            "Sharing image URL is incorrect")
    require(len(doc.find("link", rel="stylesheet", href=urls["style"])) == 1,
            "Stylesheet fingerprint absent")
    require(len(doc.find("link", rel="icon", href=urls["icon"])) == 1,
            "Favicon fingerprint absent")
    require(len(doc.find("script", src=urls["surface"], type="module")) == 1 and
            len(doc.find("script", src=urls["videos"])) == 1, "Script fingerprint absent")
    require(len(doc.find("img", src=urls["portrait"], width="750", height="1000")) == 1,
            "Portrait URL or intrinsic dimensions absent")
    require(len(doc.find("figure", **{"data-mesh": urls["mesh"]})) == 1,
            "Mesh fingerprint absent")
    require(len(doc.find("a", href=VIEWER)) == 2, "Viewer links changed")
    require("https://hgfjh.github.io/" not in html.decode("utf-8"),
            "Legacy GitHub Pages origin remains in the homepage")
    require(bool(doc.find("a", href="./cv.pdf")), "CV link missing")
    require(not doc.find("iframe"), "An iframe loads eagerly")
    ids = [a["id"] for _, a in doc.tags if "id" in a]
    require(len(ids) == len(set(ids)), "Duplicate HTML IDs")
    for tag, a in doc.tags:
        for key in ("aria-describedby", "aria-labelledby"):
            if key in a:
                require(all(x in ids for x in a[key].split()), f"Broken {key}")
        if tag == "img":
            require(bool(a.get("alt")), "Missing image alternative text")
        if tag in ("script", "img"):
            require(a.get("src", "").startswith("./"), f"Unexpected eager external {tag}")
        if tag == "link" and a.get("rel") in ("stylesheet", "icon", "preload", "preconnect"):
            require(a.get("href", "").startswith("./"), "External eager link")
        if tag == "a" and a.get("href", "").startswith("#"):
            require(a["href"][1:] in ids, f"Broken anchor {a['href']}")
        for key in ("src", "href", "data-mesh"):
            value = a.get(key, "")
            if value.startswith("./"):
                path = urlsplit(value).path[2:]
                require(path in expected and (dist / path).is_file(), f"Broken local URL: {value}")
    require(len(doc.find("button", **{"data-orient": "Home"})) == 1,
            "Single-pointer reset control missing")
    require("?v=8.5" not in html.decode(), "Stale manual version query")

    # Model the cold homepage payload, with WebGL enabled and the surface visible.
    # This is an uncompressed byte budget, not a measured CDN/network timing.
    initial = [html, *(data[k] for k in ("style", "videos", "surface", "portrait", "mesh", "icon"))]
    raw_size = sum(map(len, initial))
    gzip_estimate = sum(len(gzip.compress(x, mtime=0)) for x in initial)
    require(raw_size <= 420 * 1024, f"Homepage payload exceeds 420 KiB: {raw_size}")
    actual = {p.relative_to(dist).as_posix() for p in dist.rglob("*") if p.is_file()}
    extra = sorted(actual - expected)
    if strict:
        require(not extra, f"Unexpected files in fresh deployment: {extra}")
    elif extra:
        print(f"Local dist has extra files (not checked for deployment): {extra}")
    print(f"Static audit passed: {len(expected)} expected files; {raw_size:,} B cold raw payload; "
          f"{gzip_estimate:,} B gzip estimate (all resources, including WebGL mesh).")
    for name, content in data.items():
        print(f"  {name:>8}: {len(content):>9,} B")
    return raw_size, gzip_estimate


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--strict", action="store_true")
    parser.add_argument("--dist", type=Path, default=Path("dist"))
    args = parser.parse_args()
    audit(Path.cwd(), args.dist, args.strict)
