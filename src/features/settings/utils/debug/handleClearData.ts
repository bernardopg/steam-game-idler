import type { UserSummary } from '@/shared/types'
import { handleClearLogs } from '@/features/settings'
import { logEvent, preserveKeysAndClearData } from '@/shared/utils'

export const handleClearData = async (
  onClose: () => void,
  setUserSummary: (value: UserSummary | ((prev: UserSummary) => UserSummary)) => void,
) => {
  onClose()
  await handleClearLogs(false)
  logEvent('[Settings - Clear Data] Log file cleared')
  await preserveKeysAndClearData()
  setUserSummary(null)
  logEvent('[Settings] Cleared all data successfully')
}
