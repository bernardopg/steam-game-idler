import type { InvokeSettings, UserSettings, UserSummary } from '@/shared/types'
import { useUserStore } from '@/shared/stores'
import { invoke, isTauri } from '@/shared/utils/tauri'

export const handleNextTaskChange = async (
  feature: 'cardFarming' | 'achievementUnlocker',
  currentKey: string,
  userSummary: UserSummary,
  setUserSettings: (value: UserSettings) => void,
) => {
  if (!isTauri()) {
    const currentSettings = useUserStore.getState().userSettings
    setUserSettings({
      ...currentSettings,
      [feature]: {
        ...currentSettings[feature],
        nextTask: currentKey,
      },
    })
    return
  }

  const response = await invoke<InvokeSettings>('update_user_settings', {
    steamId: userSummary?.steamId,
    key: `${feature}.nextTask`,
    value: currentKey,
  })

  setUserSettings(response.settings)
}
