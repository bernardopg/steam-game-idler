import type { LogEntry } from '@/shared/types'
import { useEffect, useState } from 'react'
import i18next from 'i18next'
import { showDangerToast } from '@/shared/components'
import { logEvent } from '@/shared/utils'
import { invoke, isTauri } from '@/shared/utils/tauri'

export const useLogs = () => {
  const [logs, setLogs] = useState<LogEntry[]>([])

  useEffect(() => {
    if (!isTauri()) {
      setLogs([])
      return
    }

    const fetchLogs = async () => {
      try {
        const logContents = await invoke<string>('read_log_file')

        // Process log contents
        const logEntries = logContents
          .split('\n')
          .filter(entry => entry.trim() !== '')
          .map(entry => {
            const separatorIndex = entry.indexOf(' + ')
            const timestamp = separatorIndex >= 0 ? entry.slice(0, separatorIndex) : ''
            const message = separatorIndex >= 0 ? entry.slice(separatorIndex + 3) : entry
            return { timestamp, message }
          })
        setLogs(logEntries)
      } catch (error) {
        showDangerToast(i18next.t('common.error'))
        console.error('Error in (fetchLogs):', error)
        logEvent(`[Error] in (fetchLogs): ${error}`)
      }
    }
    fetchLogs()

    const intervalId = setInterval(fetchLogs, 1000)

    return () => clearInterval(intervalId)
  }, [])

  return { logs }
}
