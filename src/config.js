import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import readline from 'node:readline/promises';
import { stdin as input, stdout as output } from 'node:process';

const CONFIG_DIR = path.join(os.homedir(), '.config', 'pxdocs');
const CONFIG_FILE = path.join(CONFIG_DIR, 'config.json');

export function readConfig() {
  if (!fs.existsSync(CONFIG_FILE)) return {};
  return JSON.parse(fs.readFileSync(CONFIG_FILE, 'utf8'));
}

export function writeConfig(config) {
  fs.mkdirSync(CONFIG_DIR, { recursive: true });
  fs.writeFileSync(CONFIG_FILE, `${JSON.stringify(config, null, 2)}\n`);
}

export async function setup(pathArg) {
  let docsPath = pathArg;

  if (!docsPath) {
    const rl = readline.createInterface({ input, output });
    docsPath = await rl.question('px-docs local path: ');
    rl.close();
  }

  docsPath = path.resolve(docsPath.trim().replace(/^~/, os.homedir()));

  if (!fs.existsSync(docsPath)) {
    throw new Error(`path does not exist: ${docsPath}`);
  }

  if (!fs.existsSync(path.join(docsPath, 'docs'))) {
    throw new Error(`path does not look like px-docs: ${docsPath}`);
  }

  const config = { ...readConfig(), docsPath, remote: 'px-center/px-docs' };
  writeConfig(config);
  console.log(`configured px-docs path: ${docsPath}`);
}

export function getDocsPath() {
  return readConfig().docsPath;
}

export function getRemote() {
  return readConfig().remote ?? 'px-center/px-docs';
}

export { CONFIG_FILE };
