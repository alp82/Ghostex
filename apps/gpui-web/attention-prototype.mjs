// THROWAWAY: build and serve the real GPUI sidebar with in-memory attention fixtures.
// Run from the repository root: bun apps/gpui-web/attention-prototype.mjs
import { spawnSync } from 'node:child_process';
import { homedir } from 'node:os';
import path from 'node:path';
const cwd = path.resolve(import.meta.dirname, '../..');
const build = spawnSync('bun', ['run', 'web:build'], {
  cwd, stdio: 'inherit', env: { ...process.env, PATH: `${homedir()}/.cargo/bin:${process.env.PATH}` },
});
if (build.status !== 0) process.exit(build.status ?? 1);
console.log('Native sidebar prototype: http://localhost:4187/?prototype=attention&variant=B');
spawnSync('python3', ['-m', 'http.server', '4187', '--bind', '127.0.0.1', '--directory', 'apps/gpui-web/www/dist'], { cwd, stdio: 'inherit' });
