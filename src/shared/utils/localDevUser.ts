import type { CardFarmingUser, UserSummary } from '@/shared/types'
import { invoke } from '@/shared/utils/tauri'

const DEFAULT_LOCAL_DEV_STEAM_ID = '76561198000000000'

export function getLocalDevSteamId() {
  return process.env.NEXT_PUBLIC_SGI_DEV_STEAM_ID || DEFAULT_LOCAL_DEV_STEAM_ID
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
