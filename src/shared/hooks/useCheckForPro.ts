import type { ProTier } from '@/shared/utils'
import { useEffect } from 'react'
import { useUserStore } from '@/shared/stores'
import { GRANDFATHER_CUTOFF, logEvent } from '@/shared/utils'
import { invoke, isTauri } from '@/shared/utils/tauri'

const DEFAULT_SUBSCRIPTIONS_URL = 'https://apibase.vercel.app/api/subscriptions'
const VALID_PRO_TIERS = ['casual', 'gamer'] as const

async function isLocalDevBuild() {
  if (process.env.NODE_ENV !== 'production') return true
  if (!isTauri()) return false

  try {
    return await invoke<boolean>('is_dev')
  } catch {
    return false
  }
}

function parseProTier(value: string | undefined) {
  if (!value) return undefined
  return VALID_PRO_TIERS.includes(value as NonNullable<ProTier>) ? (value as ProTier) : undefined
}

export function useCheckForPro() {
  const userSummary = useUserStore(state => state.userSummary)
  const setIsPro = useUserStore(state => state.setIsPro)
  const setProTier = useUserStore(state => state.setProTier)

  // Check for active subscription and set isPro + proTier
  useEffect(() => {
    const steamId = userSummary?.steamId

    if (!steamId) return

    const checkSubscription = async () => {
      try {
        const isDevBuild = await isLocalDevBuild()
        const forcedTier = isDevBuild
          ? parseProTier(process.env.NEXT_PUBLIC_SGI_FORCE_PRO_TIER)
          : undefined

        if (forcedTier) {
          setIsPro(true)
          setProTier(forcedTier)
          logEvent(`[PRO] Using local dev PRO tier override: ${forcedTier}`)
          return
        }

        const subscriptionsUrl =
          isDevBuild && process.env.NEXT_PUBLIC_SGI_SUBSCRIPTIONS_URL
            ? process.env.NEXT_PUBLIC_SGI_SUBSCRIPTIONS_URL
            : DEFAULT_SUBSCRIPTIONS_URL

        const response = await fetch(subscriptionsUrl, {
          method: 'POST',
          headers: {
            'Content-Type': 'application/json',
          },
          body: JSON.stringify({ steamId }),
        })

        const data = await response.json()

        if (data?.results?.status) {
          setIsPro(true)

          const createdAt = data?.results?.created_at
          const tier = data?.results?.tier as ProTier

          // Grandfather: subscribers before cutoff get full Gamer access regardless of plan
          if (createdAt && new Date(createdAt) < GRANDFATHER_CUTOFF) {
            setProTier('gamer')
          } else {
            setProTier(tier ?? null)
          }
        } else {
          setIsPro(false)
          setProTier(null)
        }
      } catch (error) {
        console.error('Error checking subscription:', error)
        logEvent(`[Error] in checkSubscription: ${error}`)
        setIsPro(false)
        setProTier(null)
      }
    }

    checkSubscription()
  }, [userSummary?.steamId, setIsPro, setProTier])
}
