import { resolve, dirname } from 'node:path'
import { readFile, writeFile, readdir } from 'node:fs/promises'
import { fileURLToPath } from 'node:url'

const __dirname = dirname(fileURLToPath(import.meta.url))
const version = process.argv[2]

const packageJsonPath = resolve(__dirname, '../packages/cli/package.json')
const packageJson = await readFile(packageJsonPath, 'utf-8')
const packageJsonObj = JSON.parse(packageJson)
packageJsonObj.version = version
await writeFile(packageJsonPath, JSON.stringify(packageJsonObj, null, 2))

async function updatePackageJsonVersion(parentDir, version) {
  const packageJsonPath = resolve(parentDir, 'package.json')
  const packageJson = await readFile(packageJsonPath, 'utf-8')
  const newContent = packageJson.replace(
    /(")?version("?): "[0-9]+\.[0-9]+\.[0-9]+"/,
    `"version": "${version}"`
  )
  await writeFile(packageJsonPath, newContent)
}

async function updateCargoTomlVersion(parentDir, version) {
  const cargoTomlPath = resolve(parentDir, 'Cargo.toml')
  const cargoToml = await readFile(cargoTomlPath, 'utf-8')
  const newContent = cargoToml.replace(
    /version = "[0-9]+\.[0-9]+\.[0-9]+"/,
    `version = "${version}"`
  )
  await writeFile(cargoTomlPath, newContent)
}

async function start() {
  await updatePackageJsonVersion(resolve(__dirname, '../'), version)
  await updatePackageJsonVersion(resolve(__dirname, '../packages/cli'), version)
  await updatePackageJsonVersion(resolve(__dirname, '../packages/core'), version)
  await updateCargoTomlVersion(resolve(__dirname, '../packages/core'), version)
  // 读取../packages/core/npm目录下所有文件并替换
  const npmDir = resolve(__dirname, '../packages/core/npm')
  const files = await readdir(npmDir)
  for (const file of files) {
    await updatePackageJsonVersion(resolve(npmDir, file), version)
  }
}

start()
