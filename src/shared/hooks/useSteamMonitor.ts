import type { Game, InvokeRunningProcess } from '@/shared/types'
import { listen } from '@tauri-apps/api/event'
import { useEffect, useRef } from 'react'
import { useIdleStore, useStateStore, useUserStore } from '@/shared/stores'
import { isTauri, logEvent } from '@/shared/utils'
import { invoke } from '@/shared/utils/tauri'

export function useSteamMonitor() {
  const userSummary = useUserStore(state => state.userSummary)
  const setIsCardFarming = useStateStore(state => state.setIsCardFarming)
  const setIsAchievementUnlocker = useStateStore(state => state.setIsAchievementUnlocker)
  const setShowSteamWarning = useStateStore(state => state.setShowSteamWarning)
  const setIdleGamesList = useIdleStore(state => state.setIdleGamesList)
  const processSignatureRef = useRef('')

  // Listen for Steam status changes
  useEffect(() => {
    if (!isTauri()) return
    logEvent('[Steam Monitor] Listening for Steam status changes')
    const unlistenPromise = listen<boolean>('steam_status_changed', event => {
      const isSteamRunning = event.payload
      logEvent(`[Steam Monitor] Steam status changed: ${isSteamRunning ? 'running' : 'stopped'}`)
      if (!isSteamRunning && userSummary) {
        invoke('kill_all_steamutil_processes')
          .then(() => logEvent('[Steam Monitor] SteamUtility processes killed'))
          .catch(error => logEvent(`[Error] in (kill_all_steamutil_processes): ${error}`))
        setIsCardFarming(false)
        setIsAchievementUnlocker(false)
        setShowSteamWarning(true)
        logEvent('[Steam Monitor] Stopped active Steam tasks after Steam closed')
      }
    })

    return () => {
      unlistenPromise.then(unlisten => unlisten())
    }
  }, [userSummary, setIsAchievementUnlocker, setIsCardFarming, setShowSteamWarning])

  // Listen for running processes changes
  useEffect(() => {
    if (!isTauri()) return
    logEvent('[Steam Monitor] Listening for running process changes')
    const unlistenPromise = listen('running_processes_changed', event => {
      const response = event.payload as InvokeRunningProcess
      const processes = response?.processes ?? []
      const processSignature = processes.map(process => process.appid).join(',')
      const previousProcessCount = processSignatureRef.current
        ? processSignatureRef.current.split(',').length
        : 0

      if (processSignatureRef.current !== processSignature) {
        const countChanged = previousProcessCount !== processes.length
        logEvent(
          countChanged
            ? `[Steam Monitor] Running process count changed: ${processes.length}`
            : '[Steam Monitor] Running process list changed',
        )
        processSignatureRef.current = processSignature
      }

      setIdleGamesList((prevList: Game[]) => {
        if (prevList.length !== processes.length) {
          return processes.map(process => {
            const existingGame = prevList.find(game => game.appid === process.appid)
            return {
              ...process,
              // Track start time for idle timer
              startTime: existingGame?.startTime || Date.now(),
            }
          })
        }

        // Only update if the list of games has actually changed
        const prevMap = new Map(prevList.map(item => [item.appid, item]))
        const newMap = new Map(processes.map(item => [item.appid, item]))

        if (
          prevList.some(item => !newMap.has(item.appid)) ||
          processes.some(item => !prevMap.has(item.appid))
        ) {
          return processes
        }

        return prevList
      })
    })

    return () => {
      unlistenPromise.then(unlisten => unlisten())
    }
  }, [setIdleGamesList])
}
