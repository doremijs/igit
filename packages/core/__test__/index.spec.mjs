import fs from 'node:fs/promises'
import fsSync from 'node:fs'
import path from 'node:path'
import { fileURLToPath } from 'node:url'
import { execSync } from 'node:child_process'
import test from 'ava'

import { init, install, collectHookCommands } from '../index.js'

const testDir = path.join(fileURLToPath(import.meta.url), '../../_test')
if (!fsSync.existsSync(testDir)) {
  fsSync.mkdirSync(testDir, { recursive: true })
}
process.chdir(testDir)

execSync('git config --global init.defaultBranch main')
execSync('git config --global user.email "test@example.com"')
execSync('git config --global user.name "Test User"')

test.beforeEach('mkdir test dir', async (t) => {
  // 添加安全目录配置，否则会报错
  // execSync('git config --global --add safe.directory "*"')
  execSync('git init')
  console.log('Finish git init')
})

test.afterEach('rm test dir', async (t) => {
   await fs.rm(path.join(testDir, '.config'), { recursive: true, force: true })
   await fs.rm(path.join(testDir, '.git'), { recursive: true, force: true })
   console.log('Finish rm test dir')
})

test.after('Finally rm test dir', async (t) => {
  try {
    await new Promise(resolve => setTimeout(resolve, 100))
    await fs.rm(path.join(testDir), { recursive: true, force: true })
    console.log('Finish rm test dir')
  } catch (error) {
    console.log('Error rm test dir', error)
  }
})

test.serial('init from native', async(t) => {
  init()
  const configPath = path.join(testDir, '.config/igit.yaml')
  try {
    await fs.access(configPath)
    t.pass()
  } catch (err) {
    t.fail(err.message)
  }
})

test.serial('install hooks', async (t) => {
  try {
    init()
    install()
    const preCommitPath = '.git/hooks/pre-commit'
    const commitMsgPath = '.git/hooks/commit-msg'
    await Promise.all([
      fs.access(preCommitPath),
      fs.access(commitMsgPath)
    ])
    t.pass()
  } catch (err) {
    t.fail(err.message)
  }
})

test.serial('run hook', async (t) => {
  try {
    init()
    const configPath = path.join(testDir, '.config/igit.yaml')
    const config = await fs.readFile(configPath, 'utf-8')
    await fs.writeFile(configPath, config.replace('hooks: {}', 'hooks: \n    pre-push: printf "hello" > pre-push.txt'))
    install()
    const hookCommands = collectHookCommands('pre-push', [])
    t.is(hookCommands[0].command, `printf "hello" > pre-push.txt`)
    // const prePushResult = await fs.readFile(path.join(testDir, 'pre-push.txt'), 'utf-8')
    // t.is(prePushResult, 'hello')
  } catch (err) {
    t.fail(err.message)
  } finally {
    await fs.rm(path.join(testDir, 'pre-push.txt'), { force: true })
  }
})
