import type { InvokeUserSummary } from '@/shared/types'
import { decrypt, logEvent } from '@/shared/utils'
import { invoke } from '@/shared/utils/tauri'

export const fetchUserSummary = async (steamId: string, apiKey: string | null) => {
  try {
    const res = await invoke<InvokeUserSummary>('get_user_summary', {
      steamId,
      apiKey: apiKey ? decrypt(apiKey) : null,
    })
    return {
      steamId: res.response.players[0]?.steamid ?? '',
      personaName: res.response.players[0]?.personaname ?? '',
      avatar: res.response.players[0]?.avatar?.replace('.jpg', '_full.jpg') ?? '',
    }
  } catch (error) {
    // The Web API key is optional. Saving valid Steam cookies must not depend
    // on the profile endpoint being reachable or returning JSON.
    console.warn('Steam user summary is unavailable; using the signed-in profile instead.', error)
    logEvent('[Settings - Card Farming] User summary unavailable; using the signed-in profile')
    return { steamId: '', personaName: '', avatar: '' }
  }
}
