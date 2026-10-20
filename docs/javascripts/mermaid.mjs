import mermaid from 'https://unpkg.com/mermaid@11/dist/mermaid.esm.min.mjs';
import elkLayouts from 'https://unpkg.com/@mermaid-js/layout-elk@0.2/dist/mermaid-layout-elk.esm.min.mjs';

mermaid.registerLayoutLoaders(elkLayouts);
mermaid.initialize({
  startOnLoad: false,
  securityLevel: "loose",
  layout: "elk",
});

// Important: necessary to make it visible to Zensical
window.mermaid = mermaid;