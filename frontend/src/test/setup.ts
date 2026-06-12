import '@testing-library/jest-dom/vitest';

// jsdom 缺失的浏览器 API（UnifiedTree focusRequest 路径依赖）
if (typeof globalThis.CSS === 'undefined') {
  (globalThis as Record<string, unknown>).CSS = {
    escape: (s: string) => s.replace(/[^a-zA-Z0-9_ -￿-]/g, (c) => `\\${c}`),
  };
}
if (typeof Element !== 'undefined' && !Element.prototype.scrollIntoView) {
  Element.prototype.scrollIntoView = () => {};
}
