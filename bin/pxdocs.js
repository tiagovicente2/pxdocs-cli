#!/usr/bin/env node
import { main } from '../src/cli.js';

main(process.argv).catch((error) => {
  console.error(`pxdocs: ${error.message}`);
  process.exitCode = 1;
});
