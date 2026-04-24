<script type="module">
// mdx-gen mermaid hydrator — finds every <pre class="mermaid">
// container on the page and replaces its contents with inline
// SVG rendered by mermaid.js. Safe to include on pages that have
// no mermaid blocks; it short-circuits on an empty query.
(async function hydrate() {
  if (!document.querySelector('pre.mermaid')) return;
  try {
    const mermaid = (await import(
      'https://cdn.jsdelivr.net/npm/mermaid@10/dist/mermaid.esm.min.mjs'
    )).default;
    mermaid.initialize({ startOnLoad: false, securityLevel: 'loose' });
    await mermaid.run({ querySelector: 'pre.mermaid' });
  } catch (err) {
    console.warn('mdx-gen: mermaid load/render failed', err);
    for (const el of document.querySelectorAll('pre.mermaid')) {
      const msg = document.createElement('div');
      msg.style.cssText =
        'padding:0.5rem 0.75rem;color:#8a1f1f;background:#fdecea;' +
        'border-left:3px solid #8a1f1f;border-radius:3px;' +
        'font:12px/1.4 system-ui';
      msg.textContent = `[mdx-gen mermaid] ${err && err.message ? err.message : err}`;
      el.replaceChildren(msg);
    }
  }
})();
</script>
