// Runs the catalogue bake from the repo root so rustup finds rust-toolchain.toml,
// which it resolves from the working directory and not from --manifest-path.
import { execFileSync } from 'node:child_process';
import { resolve, dirname } from 'node:path';
import { fileURLToPath } from 'node:url';

const here = dirname(fileURLToPath(import.meta.url));
const pkgDir = resolve(here, '..');
const rootDir = resolve(pkgDir, '../..');
const manifest = resolve(rootDir, 'Cargo.toml');

const args = [
  'run', '-p', 'water-ui-wasm', '--example', 'bake_catalogue', '--release',
  '--manifest-path', manifest,
  '--', resolve(pkgDir, 'assets'), resolve(here, 'bake-manifest.json'),
];
console.log(`> cargo ${args.join(' ')}`);
execFileSync('cargo', args, { cwd: rootDir, stdio: 'inherit', shell: process.platform === 'win32' });
