import { CONFIG_FILE, getDocsPath, getRemote, readConfig, setup } from './config.js';
import { filterDocs, findDoc, loadLocalDocs, printDocs, readLocalDoc, searchDocs } from './docs.js';
import { getRepoStatus, printRepoWarning } from './git.js';
import { assertGhAvailable, loadRemoteDocs, readRemoteDoc } from './remote.js';

export async function main(argv = process.argv) {
  const [command = 'help', ...args] = argv.slice(2);

  if (command === 'help' || command === '--help' || command === '-h') return printHelp();
  if (command === 'setup') return setup(args[0]);
  if (command === 'config') return printConfig();
  if (command === 'doctor') return doctor();

  const options = parseOptions(args);
  const source = loadSource(options.remote);

  if (command === 'decisions') {
    return printDocs(filterDocs(source.docs, { type: 'decision', guild: options.guild, status: options.status }).slice(0, options.limit));
  }

  if (command === 'search') {
    const query = options.positionals.join(' ');
    if (!query) throw new Error('search requires a query');
    return printDocs(searchDocs(source.docs, query).slice(0, options.limit));
  }

  if (command === 'show') {
    const selector = options.positionals.join(' ');
    if (!selector) throw new Error('show requires a path or id');
    const doc = findDoc(source.docs, selector);
    if (!doc) throw new Error(`doc not found: ${selector}`);
    console.log(source.remote ? readRemoteDoc(getRemote(), doc.path) : readLocalDoc(getDocsPath(), doc.path));
    return;
  }

  throw new Error(`unknown command: ${command}`);
}

function loadSource(forceRemote = false) {
  const docsPath = getDocsPath();

  if (!forceRemote && docsPath) {
    printRepoWarning(getRepoStatus(docsPath));
    return { docs: loadLocalDocs(docsPath), remote: false };
  }

  assertGhAvailable();
  return { docs: loadRemoteDocs(getRemote()), remote: true };
}

function printConfig() {
  console.log(`config: ${CONFIG_FILE}`);
  console.log(JSON.stringify(readConfig(), null, 2));
}

function doctor() {
  const docsPath = getDocsPath();
  if (!docsPath) {
    console.log('px-docs path: not configured');
    console.log('remote fallback: available through gh');
    return;
  }

  console.log(`px-docs path: ${docsPath}`);
  const status = getRepoStatus(docsPath);
  if (!status.ok) {
    console.log(`git status: ${status.message}`);
    return;
  }

  console.log(`upstream: ${status.upstream}`);
  console.log(`behind: ${status.behind}`);
  console.log(`ahead: ${status.ahead}`);
}

function parseOptions(args) {
  const options = { positionals: [], limit: 20, remote: false };

  for (let index = 0; index < args.length; index += 1) {
    const arg = args[index];
    if (arg === '--remote') options.remote = true;
    else if (arg === '--guild') options.guild = args[++index];
    else if (arg === '--status') options.status = args[++index];
    else if (arg === '--limit') options.limit = Number(args[++index]);
    else options.positionals.push(arg);
  }

  return options;
}

function printHelp() {
  console.log(`pxdocs - discover PX docs from local files or GitHub\n\nUsage:\n  pxdocs setup [path]             configure local px-docs path\n  pxdocs doctor                   check config and whether repo is behind remote\n  pxdocs decisions [options]      list decision docs\n  pxdocs search <query> [options] search docs metadata\n  pxdocs show <path|id> [options] print a doc\n  pxdocs config                   print config\n\nOptions:\n  --guild <front|back|qa>         filter guild docs\n  --status <status>               filter by status text\n  --limit <number>                max results, default 20\n  --remote                        use gh CLI instead of local files\n`);
}
