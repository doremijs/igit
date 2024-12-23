#!/usr/bin/env node

// src/index.ts
import {init, install, runHook} from "@doremijs/igit-core";
var run = function() {
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
};
var args = process.argv.slice(2);
run();
