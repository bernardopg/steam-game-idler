import type { UserSummary } from '@/shared/types'
import { emit } from '@tauri-apps/api/event'
import { useEffect } from 'react'
import { useTranslation } from 'react-i18next'
import { useLoaderStore, useStateStore, useUserStore } from '@/shared/stores'
import { getLocalDevSteamId, isLocalDevSignInEnabled, isTauri, logEvent } from '@/shared/utils'
import { invoke } from '@/shared/utils/tauri'

function getLocalDevUserSummary() {
  if (process.env.NODE_ENV === 'production') return null
  if (!isLocalDevSignInEnabled()) return null

  const steamId = getLocalDevSteamId()
  if (!steamId) return null

  return {
    steamId,
    personaName: process.env.NEXT_PUBLIC_SGI_DEV_PERSONA_NAME || 'Local Dev User',
    avatar: process.env.NEXT_PUBLIC_SGI_DEV_AVATAR || '',
    mostRecent: 1,
  }
}

export function useInit() {
  const setLoadingUserSummary = useStateStore(state => state.setLoadingUserSummary)
  const setUserSummary = useUserStore(state => state.setUserSummary)
  const { hideLoader } = useLoaderStore()
  const { t, i18n } = useTranslation()

  console.debug('Monitor for rerenders')

  useEffect(() => {
    if (!isTauri()) return
    logEvent('[App Init] Initialization started')
    // Emit ready event to backend
    emit('ready')
      .then(() => logEvent('[App Init] Ready event emitted'))
      .catch(error => logEvent(`[Error] in (emit ready): ${error}`))
    // Start the Steam status monitor once globally
    invoke('start_steam_status_monitor')
      .then(() => logEvent('[App Init] Steam status monitor started'))
      .catch(error => logEvent(`[Error] in (start_steam_status_monitor): ${error}`))
    // Start the processes monitor once globally
    invoke('start_processes_monitor')
      .then(() => logEvent('[App Init] Processes monitor started'))
      .catch(error => logEvent(`[Error] in (start_processes_monitor): ${error}`))
  }, [])

  useEffect(() => {
    if (!isTauri()) return
    invoke('update_tray_menu', {
      show: t('tray.show'),
      update: t('tray.update'),
      quit: t('tray.quit'),
    })
      .then(() => logEvent(`[App Init] Tray menu updated for ${i18n.language}`))
      .catch(error => logEvent(`[Error] in (update_tray_menu): ${error}`))
  }, [t, i18n.language])

  useEffect(() => {
    // Set user summary data
    const savedUserSummary = JSON.parse(localStorage.getItem('userSummary') || '{}') as UserSummary
    const localDevUserSummary = getLocalDevUserSummary()
    const userSummary = savedUserSummary?.steamId ? savedUserSummary : localDevUserSummary

    if (userSummary?.steamId) {
      if (!savedUserSummary?.steamId && localDevUserSummary?.steamId) {
        localStorage.setItem('userSummary', JSON.stringify(localDevUserSummary))
      }

      setUserSummary(userSummary)
      logEvent(`[App Init] Loaded saved user summary for ${userSummary.steamId}`)
    } else {
      logEvent('[App Init] No saved user summary found')
    }

    setTimeout(() => {
      hideLoader()
      setTimeout(() => {
        setLoadingUserSummary(false)
      }, 250)
    }, 1500)
  }, [setUserSummary, setLoadingUserSummary, hideLoader])
}
