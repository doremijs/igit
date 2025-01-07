#!/usr/bin/env node
import { resolve } from 'node:path'
import { readFile } from 'node:fs/promises'
import { spawnSync } from 'node:child_process'
import { init, install, collectStagedCommands, collectHookCommands, autoCommit, type ShellCommand } from "@doremijs/igit-core";

const currentPath = new URL('.', import.meta.url).pathname;
const args = process.argv.slice(2);

const LOG_PREFIX = "\x1b[33m[iGit]\x1b[0m ";

const helpMessage = `Usage: igit <command> [options]

Commands:
  init              Initialize the igit configuration file
  install           Install hooks to the git repository
  run [hook] [args] Run a specific hook
  commit [options]  Auto commit with ai (-d or --dry-run to dry run, -y to directly commit)
  version           Print the version`;

function runCommand(shellCommand: ShellCommand) {
  // console.log(`${LOG_PREFIX}Running command: ${shellCommand.command} ${shellCommand.args?.join(' ')}`);
  return spawnSync(shellCommand.command, (shellCommand.args ?? []), {
    stdio: 'inherit',
    shell: true,
  });
}

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
      if (options[0] === 'pre-commit') {
        const commands = collectStagedCommands();
        for (const command of commands) {
          runCommand(command);
        }
      }
      const hookCommands = collectHookCommands(options[0], options.slice(1));
      for (const command of hookCommands) {
        runCommand(command);
      }
			break;
    case "commit":
      const commit = options.includes('-y');
      const dryRun = options.includes('-d') || options.includes('--dry-run');
      const message = await autoCommit();
      if (dryRun) {
        console.log(`${LOG_PREFIX} AI generated commit message is: \n${message}`);
        break;
      }
      const escapedMessage = message.replace(/"/g, '\\"').replace(/`/g, '\\`');

      if (commit) {
        runCommand({ command: 'git', args: ['commit', '-m', `"${escapedMessage}"`] });
      } else {
        runCommand({ command: 'git', args: ['commit', '-e', '-m', `"${escapedMessage}"`] });
      }
      break;
		case "version": {
			const pkg = await readFile(resolve(currentPath, "../package.json"), "utf-8");
			const version = JSON.parse(pkg).version;
			console.log(`${LOG_PREFIX}Version: ${version}`);
			break;
		}
		default:
			console.log(helpMessage);
	}
}

start().catch(error => {
  if (error instanceof Error) {
    console.error(`${LOG_PREFIX}\x1b[31m${error.message}\x1b[0m`);
  }
  process.exit(1)

});
