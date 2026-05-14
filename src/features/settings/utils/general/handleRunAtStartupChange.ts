import { disable, enable, isEnabled } from '@tauri-apps/plugin-autostart'
import i18next from 'i18next'
import { showDangerToast } from '@/shared/components'
import { logEvent } from '@/shared/utils'
import { isTauri } from '@/shared/utils/tauri'

export const handleRunAtStartupChange = async (
  setStartupState: React.Dispatch<React.SetStateAction<boolean | null>>,
) => {
  if (!isTauri()) return

  try {
    const isEnabledState = await isEnabled()
    if (isEnabledState) {
      await disable()
    } else {
      await enable()
    }
    setStartupState(!isEnabledState)
  } catch (error) {
    showDangerToast(i18next.t('common.error'))
    console.error('Error in (handleRunAtStartupChange):', error)
    logEvent(`[Error] in (handleRunAtStartupChange): ${error}`)
  }
}
