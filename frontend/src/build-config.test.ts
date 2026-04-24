// @vitest-environment node
import config from '../vite.config';

describe('vite build config', () => {
  it('uses relative asset paths for the desktop bundle', () => {
    expect(config.base).toBe('./');
  });
});
