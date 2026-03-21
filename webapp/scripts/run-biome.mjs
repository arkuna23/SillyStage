import { spawnSync } from 'node:child_process'
import { existsSync, readdirSync } from 'node:fs'
import { dirname, join } from 'node:path'
import { fileURLToPath } from 'node:url'

const currentFile = fileURLToPath(import.meta.url)
const projectRoot = dirname(dirname(currentFile))
const biomeNamespaceDir = join(projectRoot, 'node_modules', '.pnpm', 'node_modules', '@biomejs')

function resolveBiomeBinary() {
  if (!existsSync(biomeNamespaceDir)) {
    throw new Error('Biome is not installed. Run "pnpm install" first.')
  }

  const cliPackageName = readdirSync(biomeNamespaceDir).find((entry) => entry.startsWith('cli-'))

  if (!cliPackageName) {
    throw new Error('Unable to find the installed Biome CLI package.')
  }

  const binaryName = process.platform === 'win32' ? 'biome.exe' : 'biome'
  const binaryPath = join(biomeNamespaceDir, cliPackageName, binaryName)

  if (!existsSync(binaryPath)) {
    throw new Error(`Unable to find the Biome binary at "${binaryPath}".`)
  }

  return binaryPath
}

const result = spawnSync(resolveBiomeBinary(), process.argv.slice(2), {
  cwd: projectRoot,
  stdio: 'inherit',
})

if (result.error) {
  throw result.error
}

process.exit(result.status ?? 1)
