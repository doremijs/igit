import fs from 'node:fs/promises'
import test from 'ava'

import { init } from '../index.js'

test('init from native', (t) => {
  init()
  const configPath = '.config/igit.yaml'
  return fs.access(configPath).then(() => t.pass()).catch(() => t.fail())
})
