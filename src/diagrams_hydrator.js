<script type="module">
// mdx-gen diagram hydrator — renders mermaid, geojson, topojson,
// and ASCII STL containers to inline SVG. Safe to include on pages
// that have no diagrams; each renderer short-circuits when its
// selector finds nothing.

(async function hydrate() {
  const has = sel => document.querySelector(sel) !== null;

  if (has('pre.mermaid')) {
    try {
      const mermaid = (await import(
        'https://cdn.jsdelivr.net/npm/mermaid@10/dist/mermaid.esm.min.mjs'
      )).default;
      mermaid.initialize({ startOnLoad: false });
      await mermaid.run({ querySelector: 'pre.mermaid' });
    } catch (e) { console.warn('mdx-gen: mermaid load failed', e); }
  }

  const ns = 'http://www.w3.org/2000/svg';
  function geoSvg(d3, geojson) {
    const w = 640, h = 400;
    const proj = d3.geoMercator().fitSize([w, h], geojson);
    const path = d3.geoPath(proj);
    const svg = document.createElementNS(ns, 'svg');
    svg.setAttribute('viewBox', `0 0 ${w} ${h}`);
    svg.setAttribute('width', w);
    svg.setAttribute('height', h);
    const features = geojson.features || (geojson.type === 'FeatureCollection' ? geojson.features : [geojson]);
    for (const f of features) {
      const p = document.createElementNS(ns, 'path');
      p.setAttribute('d', path(f) || '');
      p.setAttribute('fill', '#e6e6e6');
      p.setAttribute('stroke', '#333');
      p.setAttribute('stroke-width', '0.6');
      svg.append(p);
    }
    return svg;
  }

  if (has('[data-mdx-diagram="geojson"]')) {
    try {
      const d3 = await import('https://cdn.jsdelivr.net/npm/d3-geo@3/+esm');
      for (const el of document.querySelectorAll('[data-mdx-diagram="geojson"]')) {
        try {
          const data = JSON.parse(el.querySelector('pre').textContent);
          el.replaceChildren(geoSvg(d3, data));
        } catch (e) { console.warn('mdx-gen: geojson render failed', e); }
      }
    } catch (e) { console.warn('mdx-gen: d3-geo load failed', e); }
  }

  if (has('[data-mdx-diagram="topojson"]')) {
    try {
      const d3 = await import('https://cdn.jsdelivr.net/npm/d3-geo@3/+esm');
      const topo = await import('https://cdn.jsdelivr.net/npm/topojson-client@3/+esm');
      for (const el of document.querySelectorAll('[data-mdx-diagram="topojson"]')) {
        try {
          const data = JSON.parse(el.querySelector('pre').textContent);
          const key = Object.keys(data.objects || {})[0];
          if (!key) throw new Error('topology has no objects');
          const feat = topo.feature(data, data.objects[key]);
          el.replaceChildren(geoSvg(d3, feat));
        } catch (e) { console.warn('mdx-gen: topojson render failed', e); }
      }
    } catch (e) { console.warn('mdx-gen: topojson libs load failed', e); }
  }

  if (has('[data-mdx-diagram="stl"]')) {
    try {
      const THREE = await import('https://cdn.jsdelivr.net/npm/three@0.160.0/+esm');
      const { STLLoader } = await import('https://cdn.jsdelivr.net/npm/three@0.160.0/examples/jsm/loaders/STLLoader.js/+esm');
      const { SVGRenderer } = await import('https://cdn.jsdelivr.net/npm/three@0.160.0/examples/jsm/renderers/SVGRenderer.js/+esm');
      for (const el of document.querySelectorAll('[data-mdx-diagram="stl"]')) {
        try {
          const src = el.querySelector('pre').textContent;
          const loader = new STLLoader();
          const geom = loader.parse(src);
          geom.computeBoundingBox();
          const box = geom.boundingBox;
          const size = box.getSize(new THREE.Vector3());
          const center = box.getCenter(new THREE.Vector3());
          geom.translate(-center.x, -center.y, -center.z);
          const scene = new THREE.Scene();
          const scale = Math.max(size.x, size.y, size.z) * 1.6;
          const cam = new THREE.PerspectiveCamera(45, 1.6, 0.1, scale * 10);
          cam.position.set(scale, scale, scale);
          cam.lookAt(0, 0, 0);
          scene.add(new THREE.AmbientLight(0xffffff, 1));
          const mat = new THREE.MeshBasicMaterial({ color: 0xa0a0a0, wireframe: false });
          scene.add(new THREE.Mesh(geom, mat));
          const renderer = new SVGRenderer();
          renderer.setSize(640, 400);
          renderer.render(scene, cam);
          el.replaceChildren(renderer.domElement);
        } catch (e) { console.warn('mdx-gen: stl render failed', e); }
      }
    } catch (e) { console.warn('mdx-gen: three.js load failed', e); }
  }
})();
</script>
