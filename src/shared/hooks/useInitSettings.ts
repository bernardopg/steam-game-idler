import type { InvokeSettings } from '@/shared/types'
import { useEffect } from 'react'
import { useUserStore } from '@/shared/stores'
import { logEvent } from '@/shared/utils'
import { invoke, isTauri } from '@/shared/utils/tauri'

export function useInitSettings() {
  const userSummary = useUserStore(state => state.userSummary)
  const setUserSettings = useUserStore(state => state.setUserSettings)

  useEffect(() => {
    const getAndSetUserSettings = async () => {
      if (!isTauri()) return

      if (userSummary) {
        try {
          logEvent(`[Settings Init] Loading settings for ${userSummary.steamId}`)
          const response = await invoke<InvokeSettings>('get_user_settings', {
            steamId: userSummary.steamId,
          })
          setUserSettings(response.settings)
          logEvent(`[Settings Init] Settings loaded for ${userSummary.steamId}`)
        } catch (error) {
          console.error('Error in (getAndSetUserSettings):', error)
          logEvent(`[Error] in (getAndSetUserSettings): ${error}`)
        }
      }
    }
    getAndSetUserSettings()
  }, [userSummary, setUserSettings])
}
