import { readFileSync } from 'node:fs'

const expected = process.argv[2]
if (expected && !/^\d+\.\d+\.\d+(?:-[0-9A-Za-z.-]+)?$/.test(expected)) {
  throw new Error('Expected version must be semver without a leading v.')
}

const packageVersion = JSON.parse(readFileSync('package.json', 'utf8')).version
const cargoToml = readFileSync('src-tauri/Cargo.toml', 'utf8')
const cargoVersion = cargoToml.match(/^\[package\][\s\S]*?^version\s*=\s*"([^"]+)"/m)?.[1]
const tauriVersion = JSON.parse(readFileSync('src-tauri/tauri.conf.json', 'utf8')).version
const versions = {
  'package.json': packageVersion,
  'src-tauri/Cargo.toml': cargoVersion,
  'src-tauri/tauri.conf.json': tauriVersion,
}

const uniqueVersions = new Set(Object.values(versions))
if (uniqueVersions.size !== 1 || !packageVersion) {
  throw new Error(
    `Runtime version mismatch: ${Object.entries(versions)
      .map(([file, version]) => `${file}=${version ?? 'missing'}`)
      .join(', ')}`,
  )
}
if (expected && packageVersion !== expected) {
  throw new Error(`Runtime version ${packageVersion} does not match requested release ${expected}.`)
}

process.stdout.write(`Runtime manifests are aligned on ${packageVersion}.\n`)
