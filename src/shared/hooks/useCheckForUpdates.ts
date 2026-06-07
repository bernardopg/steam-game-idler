import { listen } from '@tauri-apps/api/event'
import { platform } from '@tauri-apps/plugin-os'
import { relaunch } from '@tauri-apps/plugin-process'
import { check } from '@tauri-apps/plugin-updater'
import { useEffect } from 'react'
import { useTranslation } from 'react-i18next'
import { showDangerToast, showPrimaryToast } from '@/shared/components'
import { useUpdateStore } from '@/shared/stores'
import {
  fetchLatest,
  isPortableCheck,
  isTauri,
  logEvent,
  preserveKeysAndClearData,
} from '@/shared/utils'
import { invoke } from '@/shared/utils/tauri'

export function useCheckForUpdates() {
  const { t } = useTranslation()
  const setUpdateAvailable = useUpdateStore(state => state.setUpdateAvailable)
  const setShowChangelog = useUpdateStore(state => state.setShowChangelog)

  useEffect(() => {
    // Check for updates - immediate update for major, or show notification
    const checkForUpdates = async () => {
      if (!isTauri()) return
      try {
        const isPortable = await isPortableCheck()
        if (isPortable) return

        if ((await platform()) === 'linux') {
          const latest = await fetchLatest()
          if (!latest?.platforms?.['linux-x86_64']) return
        }

        const update = await check()
        if (update) {
          const latest = await fetchLatest()
          if (latest?.major) {
            localStorage.setItem('hasUpdated', 'true')
            await invoke('kill_all_steamutil_processes')
            await update.downloadAndInstall()
            await preserveKeysAndClearData()
            await relaunch()
          } else {
            setUpdateAvailable(true)
          }
        }
      } catch (error) {
        showDangerToast(t('toast.checkUpdate.error'))
        console.error('Error in (checkForUpdates):', error)
        logEvent(`Error in (checkForUpdates): ${error}`)
      }
    }
    checkForUpdates()
    const intervalId = setInterval(checkForUpdates, 5 * 60 * 1000)
    return () => {
      clearInterval(intervalId)
    }
  }, [setUpdateAvailable, t])

  useEffect(() => {
    // Show changelog after updates
    const hasUpdated = localStorage.getItem('hasUpdated')
    if (hasUpdated) {
      localStorage.removeItem('hasUpdated')
      setShowChangelog(true)
    }
  }, [setShowChangelog])

  useEffect(() => {
    // Surface feedback for the tray "Check for updates" action, which runs in
    // the Rust backend. Native notifications are disabled on Linux, so the
    // backend emits this event as the cross-platform feedback channel.
    if (!isTauri()) return

    const unlistenPromise = listen<string>('update_check_status', event => {
      switch (event.payload) {
        case 'available':
          setUpdateAvailable(true)
          break
        case 'none':
          showPrimaryToast(t('toast.checkUpdate.none'))
          break
        case 'managed_by_package_manager':
          showPrimaryToast(t('toast.checkUpdate.managed_by_package_manager'))
          break
        case 'error':
          showDangerToast(t('toast.checkUpdate.error'))
          break
      }
    })

    return () => {
      unlistenPromise.then(unlisten => unlisten())
    }
  }, [setUpdateAvailable, t])
}
