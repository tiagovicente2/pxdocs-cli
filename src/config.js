import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import readline from 'node:readline/promises';
import { stdin as input, stdout as output } from 'node:process';

const CONFIG_DIR = path.join(os.homedir(), '.config', 'pxdocs');
const CONFIG_FILE = path.join(CONFIG_DIR, 'config.json');
const FETCH_TTL_MS = 10 * 60 * 1000;

export function readConfig() {
  if (!fs.existsSync(CONFIG_FILE)) return {};
  return JSON.parse(fs.readFileSync(CONFIG_FILE, 'utf8'));
}

export function writeConfig(config) {
  fs.mkdirSync(CONFIG_DIR, { recursive: true });
  fs.writeFileSync(CONFIG_FILE, `${JSON.stringify(config, null, 2)}\n`);
}

export async function setup(pathArg) {
  const docsPath = await askForDocsPath(pathArg, { allowEmpty: false });
  saveDocsPath(docsPath);
  console.log(`configured px-docs path: ${docsPath}`);
}

export async function promptForDocsPath() {
  console.error('px-docs path is not configured.');
  const docsPath = await askForDocsPath(undefined, { allowEmpty: true });

  if (!docsPath) {
    console.error('using GitHub fallback through gh');
    return undefined;
  }

  saveDocsPath(docsPath);
  console.error(`configured px-docs path: ${docsPath}`);
  return docsPath;
}

async function askForDocsPath(pathArg, { allowEmpty }) {
  let docsPath = pathArg;

  if (!docsPath) {
    const rl = readline.createInterface({ input, output });
    const suffix = allowEmpty ? ', or press enter to use GitHub fallback' : '';
    docsPath = await rl.question(`px-docs local path${suffix}: `);
    rl.close();
  }

  docsPath = docsPath.trim();
  if (!docsPath && allowEmpty) return undefined;

  docsPath = path.resolve(docsPath.replace(/^~/, os.homedir()));

  if (!fs.existsSync(docsPath)) {
    throw new Error(`path does not exist: ${docsPath}`);
  }

  if (!fs.existsSync(path.join(docsPath, 'docs'))) {
    throw new Error(`path does not look like px-docs: ${docsPath}`);
  }

  return docsPath;
}

function saveDocsPath(docsPath) {
  const config = { ...readConfig(), docsPath, remote: 'px-center/px-docs' };
  writeConfig(config);
}

export function getDocsPath() {
  return readConfig().docsPath;
}

export function getRemote() {
  return readConfig().remote ?? 'px-center/px-docs';
}

export function shouldFetchDocs(docsPath, { force = false, skip = false } = {}) {
  if (skip) return false;
  if (force) return true;

  const lastFetchAt = readConfig().lastFetchAtByPath?.[docsPath];
  return !lastFetchAt || Date.now() - lastFetchAt > FETCH_TTL_MS;
}

export function markDocsFetched(docsPath) {
  const config = readConfig();
  writeConfig({
    ...config,
    lastFetchAtByPath: {
      ...(config.lastFetchAtByPath ?? {}),
      [docsPath]: Date.now(),
    },
  });
}

export { CONFIG_FILE, FETCH_TTL_MS };
