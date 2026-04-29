import type { InvokeSettings, InvokeSteamCredentials, UserSettings } from '@/shared/types'
import i18next from 'i18next'
import { showDangerToast, showSuccessToast } from '@/shared/components'
import { useUserStore } from '@/shared/stores'
import { logEvent } from '@/shared/utils'
import { invoke } from '@/shared/utils/tauri'

export const handleShowStoreLoginWindow = async (
  setUserSettings: (value: UserSettings | ((prev: UserSettings) => UserSettings)) => void,
) => {
  const { userSummary } = useUserStore.getState()

  const result = await invoke<InvokeSteamCredentials>('open_store_login_window')

  if (!result || result.success === false) {
    showDangerToast(i18next.t('common.error'))
    logEvent(`[Error] in (handleShowStoreLoginWindow): ${result?.message || 'Unknown error'}`)
    return
  }

  if (result.success) {
    const response = await invoke<InvokeSettings>('update_user_settings', {
      steamId: userSummary?.steamId,
      key: 'general.autoRedeemFreeGames',
      value: true,
    })

    setUserSettings(response.settings)

    showSuccessToast(
      i18next.t('toast.autoRedeem.authenticated', { user: userSummary?.personaName }),
    )
  }
}

export const handleSignOutCurrentStoreUser = async (
  setUserSettings: (value: UserSettings | ((prev: UserSettings) => UserSettings)) => void,
) => {
  const { userSummary } = useUserStore.getState()

  const result = await invoke<InvokeSteamCredentials>('delete_store_cookies')

  if (!result || result.success === false) {
    showDangerToast(i18next.t('common.error'))
    logEvent(
      `[Error] in (handleSignOutCurrentStoreUser) this error can occur if you are not already signed in: ${result?.message || 'Unknown error'}`,
    )
    return
  }
  logEvent('[Settings - Free Games] Store cookies deleted')

  const response = await invoke<InvokeSettings>('update_user_settings', {
    steamId: userSummary?.steamId,
    key: 'general.autoRedeemFreeGames',
    value: false,
  })

  setUserSettings(response.settings)
  logEvent('[Settings - Free Games] Signed out current store user')
}
