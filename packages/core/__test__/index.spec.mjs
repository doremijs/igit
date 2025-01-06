import fs from 'node:fs/promises'
import path from 'node:path'
import { fileURLToPath } from 'node:url'
import { execSync } from 'node:child_process'
import test from 'ava'

import { init, install, runHook } from '../index.js'

const testDir = path.join(fileURLToPath(import.meta.url), '../..')

test.beforeEach('mkdir test dir', async (t) => {
  execSync('git config --global init.defaultBranch main')
  execSync('git config --global user.email "test@example.com"')
  execSync('git config --global user.name "Test User"')
  execSync('git init')
  await new Promise(resolve => setTimeout(resolve, 100))
  console.log('Finish git init')
})
test.afterEach('rm test dir', async (t) => {
   // process.chdir(startDir)
   await fs.rm(path.join(testDir, '.config'), { recursive: true, force: true })
   await fs.rm(path.join(testDir, '.git'), { recursive: true, force: true })
   console.log('Finish rm test dir')
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
    runHook('pre-push', [])
    const prePushResult = await fs.readFile(path.join(testDir, 'pre-push.txt'), 'utf-8')
    t.is(prePushResult, 'hello')
  } catch (err) {
    t.fail(err.message)
  } finally {
    await fs.rm(path.join(testDir, 'pre-push.txt'), { force: true })
  }
})
