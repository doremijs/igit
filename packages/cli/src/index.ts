#!/usr/bin/env node
import { init, install, runHook } from "@doremijs/igit-core";

const args = process.argv.slice(2);

function run() {
  const command = args[0];
  const options = args.slice(1);
  try {
    switch (command) {
      case "install":
        install();
        break;
      case "init":
        init();
        break;
      case "run":
        runHook(options[0], options.slice(1));
        break;
    }
  } catch (error) {
    process.exit(1);
  }
}

run()
