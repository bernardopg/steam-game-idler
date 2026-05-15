import type { CardFarmingUser, UserSummary } from '@/shared/types'
import { invoke } from '@/shared/utils/tauri'

const DEFAULT_LOCAL_DEV_STEAM_ID = ''

function parseLocalDevSteamIds(value: string | undefined) {
  return (value || '')
    .split(',')
    .map(steamId => steamId.trim())
    .filter(Boolean)
}

export function getLocalDevSteamId() {
  return process.env.NEXT_PUBLIC_SGI_DEV_STEAM_ID?.trim() || DEFAULT_LOCAL_DEV_STEAM_ID
}

export function getLocalDevSteamIds() {
  return Array.from(
    new Set([
      ...parseLocalDevSteamIds(process.env.NEXT_PUBLIC_SGI_DEV_STEAM_IDS),
      getLocalDevSteamId(),
    ]),
  ).filter(Boolean)
}

export function isLocalDevSignInEnabled() {
  return process.env.NEXT_PUBLIC_SGI_DEV_SIGN_IN === 'true'
}

export function isLocalDevUserSummary(userSummary: UserSummary) {
  return isLocalDevSignInEnabled() && userSummary?.steamId === getLocalDevSteamId()
}

export function promoteLocalDevUserSummary(
  userSummary: UserSummary,
  cardFarmingUser: CardFarmingUser,
) {
  if (!isLocalDevUserSummary(userSummary)) {
    return userSummary
  }

  return {
    steamId: cardFarmingUser.steamId,
    personaName: cardFarmingUser.personaName,
    avatar: cardFarmingUser.avatar,
    mostRecent: userSummary?.mostRecent,
  }
}

export async function migrateLocalDevProfile(sourceSteamId: string, targetSteamId: string) {
  if (!sourceSteamId || !targetSteamId || sourceSteamId === targetSteamId) {
    return false
  }

  try {
    const response = await invoke<{ success: boolean }>('migrate_local_dev_profile', {
      sourceSteamId,
      targetSteamId,
    })

    return response?.success === true
  } catch (error) {
    console.error('Error in migrateLocalDevProfile:', error)
    return false
  }
}
