import fs from 'node:fs';
import path from 'node:path';

const DOC_TYPES = [
  { type: 'decision', marker: '/decisions/' },
  { type: 'adr', marker: '/ADRs/' },
  { type: 'rfc', marker: '/RFCs/' },
  { type: 'rfc', marker: '/rfcs/' },
  { type: 'guide', marker: '/guides/' },
];

export function walkMarkdown(rootDir) {
  const files = [];

  function walk(dir) {
    for (const entry of fs.readdirSync(dir, { withFileTypes: true })) {
      const fullPath = path.join(dir, entry.name);
      if (entry.isDirectory()) {
        walk(fullPath);
      } else if (entry.isFile() && entry.name.endsWith('.md')) {
        files.push(fullPath);
      }
    }
  }

  walk(path.join(rootDir, 'docs'));
  return files.sort();
}

export function parseDoc(filePath, content, rootDir = '') {
  const relativePath = rootDir ? path.relative(rootDir, filePath) : filePath;
  const normalized = relativePath.replaceAll(path.sep, '/');
  const title = content.match(/^#\s+(.+)$/m)?.[1]?.trim() ?? path.basename(filePath, '.md');
  const status = content.match(/^Status:\s*(.+)$/im)?.[1]?.trim() ?? parseFrontmatter(content).status ?? 'unknown';
  const date = content.match(/^Data:\s*(.+)$/im)?.[1]?.trim() ?? parseFrontmatter(content).date ?? 'unknown';
  const type = DOC_TYPES.find((candidate) => normalized.includes(candidate.marker))?.type ?? 'doc';
  const guild = normalized.match(/^docs\/(front|back|qa)-guild\//)?.[1] ?? undefined;
  const id = path.basename(normalized).match(/^(?:adr-|rfc-)?(\d+)/i)?.[1] ?? undefined;
  const summary = content
    .split('\n')
    .find((line) => line.trim() && !line.startsWith('#') && !line.includes(':'))
    ?.trim();

  return { path: normalized, title, status, date, type, guild, id, summary };
}

export function loadLocalDocs(rootDir) {
  return walkMarkdown(rootDir).map((file) => parseDoc(file, fs.readFileSync(file, 'utf8'), rootDir));
}

export function readLocalDoc(rootDir, docPath) {
  return fs.readFileSync(path.join(rootDir, docPath), 'utf8');
}

export function findDocs(docs, selector, options = {}) {
  const scopedDocs = filterDocs(docs, options);

  return scopedDocs.filter((doc) => doc.path === selector)
    .concat(scopedDocs.filter((doc) => doc.id === selector))
    .concat(scopedDocs.filter((doc) => doc.path.includes(selector)))
    .filter((doc, index, all) => all.findIndex((candidate) => candidate.path === doc.path) === index);
}

export function buildDocFromPath(filePath) {
  const normalized = filePath.replaceAll(path.sep, '/');
  const baseName = path.basename(normalized, '.md');
  const title = baseName
    .replace(/^(?:adr-|rfc-)?\d+-?/i, '')
    .replaceAll('-', ' ')
    .trim() || baseName;
  const type = DOC_TYPES.find((candidate) => normalized.includes(candidate.marker))?.type ?? 'doc';
  const guild = normalized.match(/^docs\/(front|back|qa)-guild\//)?.[1] ?? undefined;
  const id = baseName.match(/^(?:adr-|rfc-)?(\d+)/i)?.[1] ?? undefined;

  return { path: normalized, title, status: 'unknown', date: 'unknown', type, guild, id };
}

export function filterDocs(docs, options = {}) {
  return docs.filter((doc) => {
    if (options.type && doc.type !== options.type) return false;
    if (options.guild && doc.guild !== options.guild) return false;
    if (options.status && !doc.status.toLowerCase().includes(options.status.toLowerCase())) return false;
    return true;
  });
}

export function searchDocs(docs, query) {
  const terms = query.toLowerCase().split(/\s+/).filter(Boolean);
  return docs
    .map((doc) => {
      const haystack = `${doc.title} ${doc.path} ${doc.status} ${doc.summary ?? ''}`.toLowerCase();
      const score = terms.reduce((total, term) => total + (haystack.includes(term) ? 1 : 0), 0);
      return { ...doc, score };
    })
    .filter((doc) => doc.score > 0)
    .sort((a, b) => b.score - a.score || a.path.localeCompare(b.path));
}

export function printDocs(docs) {
  for (const doc of docs) {
    const suffix = [doc.type, doc.guild, doc.status].filter(Boolean).join(' · ');
    console.log(`${doc.path}\n  ${doc.title}${suffix ? ` (${suffix})` : ''}`);
  }
}

function parseFrontmatter(content) {
  const match = content.match(/^---\n([\s\S]*?)\n---/);
  if (!match) return {};
  return Object.fromEntries(match[1].split('\n').map((line) => line.split(':').map((part) => part.trim())).filter(([key, value]) => key && value));
}
