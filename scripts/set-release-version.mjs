import { readFileSync, writeFileSync } from 'node:fs'

const version = process.argv[2]
if (!version || !/^\d+\.\d+\.\d+(?:-[0-9A-Za-z.-]+)?$/.test(version)) {
  throw new Error('Usage: node scripts/set-release-version.mjs <semver-without-v>')
}

const updateFile = (file, transform) => {
  const source = readFileSync(file, 'utf8')
  const output = transform(source)
  if (output === source) return
  writeFileSync(file, output)
}

updateFile('package.json', source => {
  const pattern = /^(\s*"version"\s*:\s*)"[^"]+"/m
  if (!pattern.test(source)) throw new Error('Could not update version in package.json')
  return source.replace(pattern, `$1"${version}"`)
})

updateFile('src-tauri/Cargo.toml', source => {
  const pattern = /^(\[package\][\s\S]*?^version\s*=\s*)"[^"]+"/m
  if (!pattern.test(source))
    throw new Error('Could not update package version in src-tauri/Cargo.toml')
  return source.replace(pattern, `$1"${version}"`)
})

updateFile('src-tauri/tauri.conf.json', source => {
  const pattern = /^(\s*"version"\s*:\s*)"[^"]+"/m
  if (!pattern.test(source))
    throw new Error('Could not update version in src-tauri/tauri.conf.json')
  return source.replace(pattern, `$1"${version}"`)
})

process.stdout.write(`Pinned application manifests to ${version}.\n`)
