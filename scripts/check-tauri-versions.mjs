import { readFileSync } from 'node:fs'

const packageJson = JSON.parse(readFileSync(new URL('../package.json', import.meta.url)))
const cargoToml = readFileSync(new URL('../src-tauri/Cargo.toml', import.meta.url), 'utf8')

const majorMinor = (version, dependency) => {
  const match = version.match(/\d+\.\d+/)
  if (!match) {
    throw new Error(`Could not read a semver version for ${dependency}: ${version}`)
  }
  return match[0]
}

const cargoVersion = cargoToml.match(/^tauri\s*=\s*\{\s*version\s*=\s*"([^"]+)"/m)?.[1]
if (!cargoVersion) {
  throw new Error('Could not find the tauri dependency in src-tauri/Cargo.toml')
}

const dependencies = {
  'tauri (Rust)': cargoVersion,
  '@tauri-apps/api': packageJson.dependencies['@tauri-apps/api'],
  '@tauri-apps/cli': packageJson.devDependencies['@tauri-apps/cli'],
}

const expected = majorMinor(cargoVersion, 'tauri (Rust)')
for (const [dependency, version] of Object.entries(dependencies)) {
  if (!version || majorMinor(version, dependency) !== expected) {
    throw new Error(
      `Tauri version skew: expected ${dependency} to use ${expected}.x; found ${version ?? 'missing'}.`,
    )
  }
}

process.stdout.write(`Tauri host, API, and CLI are aligned on ${expected}.x.\n`)
