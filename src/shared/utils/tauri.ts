import { invoke as tauriInvoke } from '@tauri-apps/api/core'

type InvokeArgs = Record<string, unknown>
type InvokeOptions = Parameters<typeof tauriInvoke>[2]

export function isTauri() {
  if (typeof window === 'undefined') return false

  const tauriWindow = window as Window & {
    __TAURI_INTERNALS__?: {
      invoke?: unknown
    }
  }

  return typeof tauriWindow.__TAURI_INTERNALS__?.invoke === 'function'
}

export async function invoke<T = unknown>(
  command: string,
  args?: InvokeArgs,
  options?: InvokeOptions,
) {
  if (!isTauri()) {
    throw new Error(`Tauri invoke unavailable for command "${command}" outside the Tauri webview`)
  }

  return tauriInvoke<T>(command, args, options)
}
