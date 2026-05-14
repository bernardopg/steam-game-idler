import { useEffect } from 'react'
import { startAutoIdleGames } from '@/shared/utils'
import { isTauri } from '@/shared/utils/tauri'

export function useAutoIdleGames() {
  useEffect(() => {
    if (!isTauri()) return

    // Start idling games in auto idle list
    startAutoIdleGames()
  }, [])
}
