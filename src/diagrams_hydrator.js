<script type="importmap">
{
  "imports": {
    "three": "https://esm.sh/three@0.160.0",
    "three/addons/": "https://esm.sh/three@0.160.0/examples/jsm/"
  }
}
</script>
<script type="module">
// mdx-gen diagram hydrator — renders mermaid, geojson, topojson,
// and ASCII STL containers to inline SVG. Safe to include on pages
// that have no diagrams; each renderer short-circuits when its
// selector finds nothing.
//
// CDNs:
//   mermaid    — jsdelivr (self-contained ESM bundle)
//   d3-geo     — esm.sh (reliable bare-import resolution)
//   topojson   — esm.sh
//   three + addons — esm.sh via the import map above, so STLLoader /
//                    SVGRenderer share the same THREE instance as
//                    the top-level import.
(async function hydrate() {
  const has = sel => document.querySelector(sel) !== null;

  // Replace the <pre> inside `el` with an error pill so the user
  // sees *something* went wrong rather than staring at an empty
  // container.
  const fail = (el, label, err) => {
    console.warn(`mdx-gen: ${label}`, err);
    const msg = document.createElement('div');
    msg.style.cssText =
      'padding:0.5rem 0.75rem;color:#8a1f1f;background:#fdecea;' +
      'border-left:3px solid #8a1f1f;border-radius:3px;font:12px/1.4 system-ui';
    msg.textContent = `[mdx-gen ${label}] ${err && err.message ? err.message : err}`;
    el.replaceChildren(msg);
  };

  // ── Mermaid ───────────────────────────────────────────────────────
  if (has('pre.mermaid')) {
    try {
      const mermaid = (await import(
        'https://cdn.jsdelivr.net/npm/mermaid@10/dist/mermaid.esm.min.mjs'
      )).default;
      mermaid.initialize({ startOnLoad: false, securityLevel: 'loose' });
      await mermaid.run({ querySelector: 'pre.mermaid' });
    } catch (e) {
      for (const el of document.querySelectorAll('pre.mermaid')) {
        fail(el, 'mermaid', e);
      }
    }
  }

  // ── Shared geo→SVG helper ────────────────────────────────────────
  const ns = 'http://www.w3.org/2000/svg';

  function toFeatureList(g) {
    if (g && g.type === 'FeatureCollection') return g.features || [];
    if (g && g.type === 'Feature') return [g];
    // Bare geometry — wrap in a minimal Feature so d3.geoPath can
    // walk `.geometry`.
    return [{ type: 'Feature', geometry: g, properties: {} }];
  }

  function geoSvg(d3, geojson) {
    const w = 640, h = 400;
    const proj = d3.geoMercator().fitSize([w, h], geojson);
    const path = d3.geoPath(proj);
    const svg = document.createElementNS(ns, 'svg');
    svg.setAttribute('viewBox', `0 0 ${w} ${h}`);
    svg.setAttribute('width', String(w));
    svg.setAttribute('height', String(h));
    svg.setAttribute('xmlns', ns);
    svg.style.maxWidth = '100%';
    svg.style.height = 'auto';
    // Soft backdrop so boundaries + edges read against the page.
    const bg = document.createElementNS(ns, 'rect');
    bg.setAttribute('width', String(w));
    bg.setAttribute('height', String(h));
    bg.setAttribute('fill', '#f6f8fa');
    svg.append(bg);
    for (const f of toFeatureList(geojson)) {
      const d = path(f);
      if (!d) continue;
      const props = f.properties || {};
      const p = document.createElementNS(ns, 'path');
      p.setAttribute('d', d);
      p.setAttribute('fill', props.fill || '#cfd8dc');
      p.setAttribute('stroke', props.stroke || '#37474f');
      p.setAttribute(
        'stroke-width',
        String(props['stroke-width'] ?? 1.0),
      );
      if (props['fill-opacity'] != null) {
        p.setAttribute('fill-opacity', String(props['fill-opacity']));
      }
      svg.append(p);
    }
    return svg;
  }

  // ── GeoJSON ──────────────────────────────────────────────────────
  if (has('[data-mdx-diagram="geojson"]')) {
    let d3;
    try {
      d3 = await import('https://esm.sh/d3-geo@3');
    } catch (e) {
      for (const el of document.querySelectorAll('[data-mdx-diagram="geojson"]')) {
        fail(el, 'geojson (d3-geo load)', e);
      }
    }
    if (d3) {
      for (const el of document.querySelectorAll('[data-mdx-diagram="geojson"]')) {
        try {
          const data = JSON.parse(el.querySelector('pre').textContent);
          el.replaceChildren(geoSvg(d3, data));
        } catch (e) { fail(el, 'geojson render', e); }
      }
    }
  }

  // ── TopoJSON ─────────────────────────────────────────────────────
  if (has('[data-mdx-diagram="topojson"]')) {
    let d3, topo;
    try {
      d3 = await import('https://esm.sh/d3-geo@3');
      topo = await import('https://esm.sh/topojson-client@3');
    } catch (e) {
      for (const el of document.querySelectorAll('[data-mdx-diagram="topojson"]')) {
        fail(el, 'topojson (libs load)', e);
      }
    }
    if (d3 && topo) {
      for (const el of document.querySelectorAll('[data-mdx-diagram="topojson"]')) {
        try {
          const data = JSON.parse(el.querySelector('pre').textContent);
          const objects = data.objects || {};
          const keys = Object.keys(objects);
          if (keys.length === 0) {
            throw new Error('topology has no objects');
          }
          // Merge every named object into a single Feature
          // Collection so multi-object topologies render in one
          // projection. Per-object `properties` flow through to
          // `geoSvg`, which uses `fill` / `stroke` from them.
          const features = [];
          for (const key of keys) {
            const decoded = topo.feature(data, objects[key]);
            if (decoded.type === 'FeatureCollection') {
              features.push(...decoded.features);
            } else {
              features.push(decoded);
            }
          }
          const fc = { type: 'FeatureCollection', features };
          el.replaceChildren(geoSvg(d3, fc));
        } catch (e) { fail(el, 'topojson render', e); }
      }
    }
  }

  // ── ASCII STL ────────────────────────────────────────────────────
  if (has('[data-mdx-diagram="stl"]')) {
    let THREE, STLLoader, SVGRenderer;
    try {
      THREE = await import('three');
      ({ STLLoader } = await import('three/addons/loaders/STLLoader.js'));
      ({ SVGRenderer } = await import('three/addons/renderers/SVGRenderer.js'));
    } catch (e) {
      for (const el of document.querySelectorAll('[data-mdx-diagram="stl"]')) {
        fail(el, 'stl (three.js load — check the import map is in place)', e);
      }
    }
    if (THREE && STLLoader && SVGRenderer) {
      for (const el of document.querySelectorAll('[data-mdx-diagram="stl"]')) {
        try {
          const src = el.querySelector('pre').textContent;
          const loader = new STLLoader();
          const geom = loader.parse(src);
          // Phong shading needs normals per face group; STLLoader
          // gives us normals already, but smoothing + recomputing
          // bounds after recentering improves the look.
          geom.computeBoundingBox();
          geom.computeVertexNormals();
          const box = geom.boundingBox;
          const size = box.getSize(new THREE.Vector3());
          const center = box.getCenter(new THREE.Vector3());
          geom.translate(-center.x, -center.y, -center.z);
          const extent =
            Math.max(size.x, size.y, size.z) || 1;

          const scene = new THREE.Scene();
          scene.background = new THREE.Color(0xf6f8fa);

          // Iso-style camera so the cube reads as 3-D from the
          // first frame.
          const cam = new THREE.PerspectiveCamera(
            35, 640 / 400, 0.1, extent * 20,
          );
          const d = extent * 2.6;
          cam.position.set(d, d * 0.9, d);
          cam.lookAt(0, 0, 0);

          // AmbientLight keeps shadowed faces from going black;
          // DirectionalLight drives the face-to-face contrast that
          // makes shape legible. Wireframe overlay kept subtle so
          // silhouette reads even on flat-shaded faces.
          scene.add(new THREE.AmbientLight(0xffffff, 0.55));
          const key = new THREE.DirectionalLight(0xffffff, 0.9);
          key.position.set(5, 8, 5);
          scene.add(key);
          const fill = new THREE.DirectionalLight(0xffffff, 0.35);
          fill.position.set(-4, -2, -6);
          scene.add(fill);

          const mat = new THREE.MeshPhongMaterial({
            color: 0x4a90d9,
            specular: 0x222222,
            shininess: 24,
            flatShading: true,
          });
          const mesh = new THREE.Mesh(geom, mat);
          scene.add(mesh);

          const edges = new THREE.LineSegments(
            new THREE.EdgesGeometry(geom, 30),
            new THREE.LineBasicMaterial({ color: 0x1b3d6e }),
          );
          scene.add(edges);

          const renderer = new SVGRenderer();
          renderer.setQuality('high');
          renderer.setSize(640, 400);
          renderer.render(scene, cam);
          const svg = renderer.domElement;
          svg.style.maxWidth = '100%';
          svg.style.height = 'auto';
          el.replaceChildren(svg);
        } catch (e) { fail(el, 'stl render', e); }
      }
    }
  }
})();
</script>
