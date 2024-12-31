#!/usr/bin/env node

// src/index.ts
import { readFile } from "node:fs/promises";
import { autoCommit, init, install, runHook } from "@doremijs/igit-core";
var args = process.argv.slice(2);
var helpMessage = `Usage: igit <command> [options]

Commands:
  init    Initialize the igit configuration file
  install Install hooks to the git repository
  run     Run a specific hook
  commit  Auto commit with ai
  version Print the version`;
async function start() {
  if (args.length === 0) {
    console.log(helpMessage);
    return;
  }
  const command = args[0];
  const options = args.slice(1);
  switch (command) {
    case "init":
      init();
      break;
    case "install":
      install();
      break;
    case "run":
      try {
        runHook(options[0], options.slice(1));
      } catch (error) {
        console.error("\x1B[31mHooks failed to run\x1B[0m");
        process.exit(1);
      }
      break;
    case "commit":
      autoCommit(options.includes("-y"));
      break;
    case "version": {
      const pkg = await readFile("../package.json", "utf-8");
      const version = JSON.parse(pkg).version;
      console.log(`Version: ${version}`);
      break;
    }
    default:
      console.log(helpMessage);
  }
}
start();
