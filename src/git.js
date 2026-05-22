import { execFileSync } from 'node:child_process';
import fs from 'node:fs';
import path from 'node:path';

function git(cwd, args, options = {}) {
  return execFileSync('git', args, {
    cwd,
    encoding: 'utf8',
    stdio: ['ignore', 'pipe', options.silent ? 'ignore' : 'pipe'],
    timeout: options.timeout ?? 10000,
  }).trim();
}

export function isGitRepo(repoPath) {
  return fs.existsSync(path.join(repoPath, '.git'));
}

export function getRepoStatus(repoPath, { fetch = true } = {}) {
  if (!repoPath || !isGitRepo(repoPath)) return { ok: false, message: 'not a git repo' };

  try {
    const upstream = git(repoPath, ['rev-parse', '--abbrev-ref', '--symbolic-full-name', '@{u}'], { silent: true });
    if (fetch) git(repoPath, ['fetch', '--quiet'], { silent: true, timeout: 30000 });
    const counts = git(repoPath, ['rev-list', '--left-right', '--count', `${upstream}...HEAD`], { silent: true });
    const [behind, ahead] = counts.split(/\s+/).map(Number);

    return { ok: true, upstream, behind, ahead };
  } catch (error) {
    return { ok: false, message: error.message };
  }
}

export function printRepoWarning(status) {
  if (!status.ok) {
    console.error(`warning: could not check px-docs git status (${status.message})`);
    return;
  }

  if (status.behind > 0) {
    console.error(`warning: px-docs is ${status.behind} commit(s) behind ${status.upstream}`);
  }
}
