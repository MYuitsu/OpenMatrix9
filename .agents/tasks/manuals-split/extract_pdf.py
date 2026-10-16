"""Extract text, images and index from one PDF using PyMuPDF.
Usage: python extract_pdf.py <pdf> <pdfkey> <out_root>
Output: <out_root>/<pdfkey>/{text/,images/,index.json}. Only that dir is overwritten."""
import sys, json, hashlib, shutil, re, collections
from pathlib import Path
import fitz

MIN_PX = 64
DPI = 150

def main(pdf, key, out_root):
    out = Path(out_root) / key
    if out.exists():
        shutil.rmtree(out)  # own staging output only
    (out / "text").mkdir(parents=True); (out / "images").mkdir()
    doc = fitz.open(pdf)
    # repeated header/footer detection
    band = collections.Counter()
    pages_blocks = []
    for p in doc:
        h = p.rect.height; bl = []
        for b in p.get_text("blocks"):
            if b[6] != 0: continue
            t = re.sub(r"\d+", "#", b[4].strip())
            if t and (b[3] < h * 0.08 or b[1] > h * 0.92): band[t] += 1
        pages_blocks.append(bl)
    n = len(doc); repeated = {t for t, c in band.items() if c >= max(3, n * 0.3)}
    sizes = collections.Counter()
    for p in doc:
        for b in p.get_text("dict")["blocks"]:
            for l in b.get("lines", []):
                for s in l["spans"]: sizes[round(s["size"])] += len(s["text"])
    body = sizes.most_common(1)[0][0] if sizes else 10
    seen = {}; img_hash_count = collections.Counter(); pages = []; ocr = False
    try:
        fitz.get_tessdata(); ocr = True
    except Exception: pass
    for p in doc:
        pn = p.number + 1
        h = p.rect.height; lines = []; heads = []
        d = p.get_text("dict", sort=True)
        for b in d["blocks"]:
            if b["type"] != 0: continue
            if b["bbox"][3] < h * .08 or b["bbox"][1] > h * .92:
                if re.sub(r"\d+", "#", " ".join(s["text"] for l in b["lines"] for s in l["spans"]).strip()) in repeated: continue
            for l in b["lines"]:
                txt = "".join(s["text"] for s in l["spans"]).strip()
                if not txt: continue
                sz = max(s["size"] for s in l["spans"]); bold = any(s["flags"] & 16 for s in l["spans"])
                if sz >= body + 2 and len(txt) < 120:
                    lvl = 1 if sz >= body + 6 else 2 if sz >= body + 4 else 3
                    heads.append({"level": lvl, "text": txt}); lines.append("#" * lvl + " " + txt)
                else: lines.append(txt)
        text = "\n".join(lines)
        tf = f"{pn:04d}.md"
        (out / "text" / tf).write_text(f"<!-- page {pn} -->\n\n{text}\n", encoding="utf-8")
        imgs = []; k = 0
        for im in p.get_images(full=True):
            xref = im[0]
            try:
                pix = fitz.Pixmap(doc, xref)
                if pix.width < MIN_PX or pix.height < MIN_PX: continue
                if pix.alpha: pix = fitz.Pixmap(pix, 0)
                if pix.colorspace is None or pix.colorspace.name not in ("DeviceGray", "DeviceRGB"): pix = fitz.Pixmap(fitz.csRGB, pix)
                hh = hashlib.sha256(pix.samples).hexdigest()
            except Exception: continue
            img_hash_count[hh] += 1
            if hh in seen: imgs.append({"file": seen[hh], "duplicate": True}); continue
            if len(doc) > 4 and img_hash_count[hh] and False: pass
            k += 1; name = f"{key}_p{pn}_img{k:02d}.png"
            pix.save(out / "images" / name); seen[hh] = name
            imgs.append({"file": name, "w": pix.width, "h": pix.height, "sha256": hh, "type": "embedded"})
        draws = len(p.get_drawings()); raster_area = 0
        for r in p.get_image_info(): 
            bb = fitz.Rect(r["bbox"]); raster_area += bb.width * bb.height
        mostly_vector = draws >= 15 and raster_area < 0.15 * p.rect.width * p.rect.height
        scanned = len(text.strip()) < 5 and (raster_area > 0.5 * p.rect.width * p.rect.height)
        if mostly_vector or scanned:
            name = f"{key}_p{pn}_img{k+1:02d}.png"
            p.get_pixmap(dpi=DPI).save(out / "images" / name)
            imgs.append({"file": name, "type": "page_render_scanned" if scanned else "page_render_vector", "dpi": DPI})
        pages.append({"page": pn, "text_file": f"text/{tf}", "chars": len(text), "images": imgs,
                      "headings": heads, "scanned_no_text": scanned, "needs_ocr": scanned and not ocr})
    # drop repeated logos: same hash on >=30% of pages (min 3)
    logos = {h for h, c in img_hash_count.items() if c >= max(3, n * .3)}
    for h in logos:
        f = seen[h]; (out / "images" / f).unlink(missing_ok=True)
        for pg in pages: pg["images"] = [i for i in pg["images"] if i.get("file") != f]
    cmd = {h["text"] for pg in pages for h in pg["headings"] if h["level"] >= 2 and re.match(r"^[A-Za-z][\w ./-]{1,40}$", h["text"])}
    cmd |= {t[1].strip() for t in doc.get_toc() if re.match(r"^[A-Za-z][\w ./-]{1,40}$", t[1].strip())}  # bookmark titles
    cmd = sorted(cmd)
    idx = {"pdf": str(pdf), "pdfkey": key, "page_count": n, "body_font_size": body, "ocr_available": ocr,
           "toc": doc.get_toc(), "scanned_pages": [p["page"] for p in pages if p["scanned_no_text"]],
           "repeated_headers_footers": sorted(repeated), "removed_logo_hashes": sorted(logos),
           "command_name_candidates": cmd, "pages": pages}
    (out / "index.json").write_text(json.dumps(idx, ensure_ascii=False, indent=1), encoding="utf-8")
    print(f"{key}: {n} pages, {len(list((out/'images').iterdir()))} images")

if __name__ == "__main__":
    if len(sys.argv) != 4: sys.exit(__doc__)
    main(*sys.argv[1:])
