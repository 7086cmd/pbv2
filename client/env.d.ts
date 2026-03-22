/// <reference types="vite-plus/client" />

declare module "*.vue" {
  import { DefineComponent } from "vue";
  const component: DefineComponent<{}, {}, any>;
  export default component;
}

declare module "katex/contrib/auto-render" {
  function renderMathInElement(elem: Element, options?: Record<string, unknown>): void;
  export default renderMathInElement;
}

declare module "katex/contrib/mhchem" {}
