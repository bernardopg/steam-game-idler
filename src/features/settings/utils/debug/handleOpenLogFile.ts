import i18next from 'i18next'
import { showDangerToast, showSuccessToast } from '@/shared/components'
import { logEvent } from '@/shared/utils'
import { invoke, isTauri } from '@/shared/utils/tauri'

export const handleOpenLogFile = async () => {
  if (!isTauri()) return

  try {
    await invoke('open_file_explorer', { path: 'log.txt' })
    showSuccessToast(i18next.t('toast.openLogFile.success'))
  } catch (error) {
    showDangerToast(i18next.t('common.error'))
    console.error('Error in (handleOpenLogFile):', error)
    logEvent(`[Error] in (handleOpenLogFile): ${error}`)
  }
}
