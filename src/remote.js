import { execFileSync } from 'node:child_process';
import { buildDocFromPath } from './docs.js';

function gh(args) {
  return execFileSync('gh', args, {
    encoding: 'utf8',
    stdio: ['ignore', 'pipe', 'pipe'],
    timeout: 30000,
  }).trim();
}

export function loadRemoteDocs(remote) {
  const tree = JSON.parse(gh(['api', `repos/${remote}/git/trees/HEAD?recursive=1`]));
  return tree.tree
    .filter((item) => item.type === 'blob' && item.path.startsWith('docs/') && item.path.endsWith('.md'))
    .map((item) => buildDocFromPath(item.path))
    .sort((a, b) => a.path.localeCompare(b.path));
}

export function readRemoteDoc(remote, filePath) {
  const payload = JSON.parse(gh(['api', `repos/${remote}/contents/${filePath}`]));
  return Buffer.from(payload.content.replace(/\n/g, ''), 'base64').toString('utf8');
}

export function assertGhAvailable() {
  try {
    gh(['--version']);
  } catch {
    throw new Error('gh CLI is required for remote fallback');
  }
}
