import type { InvokeSettings, UserSettings, UserSummary } from '@/shared/types'
import { invoke, isTauri } from '@/shared/utils/tauri'

const updateTradingCardsSetting = (
  setUserSettings: (value: UserSettings | ((prev: UserSettings) => UserSettings)) => void,
  value: Partial<UserSettings['tradingCards']>,
) => {
  setUserSettings(prev => ({
    ...prev,
    tradingCards: {
      ...prev.tradingCards,
      ...value,
    },
  }))
}

export const handleSellOptionChange = async (
  key: string,
  userSummary: UserSummary,
  setUserSettings: (value: UserSettings | ((prev: UserSettings) => UserSettings)) => void,
) => {
  if (!isTauri()) {
    updateTradingCardsSetting(setUserSettings, {
      sellOptions: key as UserSettings['tradingCards']['sellOptions'],
    })
    return
  }

  const updateResponse = await invoke<InvokeSettings>('update_user_settings', {
    steamId: userSummary?.steamId,
    key: 'tradingCards.sellOptions',
    value: key,
  })

  setUserSettings(updateResponse.settings)
}

export const handlePriceAdjustmentChange = async (
  value: number,
  userSummary: UserSummary,
  setUserSettings: (value: UserSettings | ((prev: UserSettings) => UserSettings)) => void,
  setPriceAdjustment: React.Dispatch<React.SetStateAction<number>>,
) => {
  setPriceAdjustment(value)

  if (!isTauri()) {
    updateTradingCardsSetting(setUserSettings, { priceAdjustment: value })
    return
  }

  const updateResponse = await invoke<InvokeSettings>('update_user_settings', {
    steamId: userSummary?.steamId,
    key: 'tradingCards.priceAdjustment',
    value,
  })

  setUserSettings(updateResponse.settings)
}

export const handleSellLimitMinChange = async (
  value: number,
  userSummary: UserSummary,
  setUserSettings: (value: UserSettings | ((prev: UserSettings) => UserSettings)) => void,
  sellLimitMax: number,
  setSellLimitMin: React.Dispatch<React.SetStateAction<number>>,
) => {
  setSellLimitMin(value)

  if (!isTauri()) {
    updateTradingCardsSetting(setUserSettings, {
      sellLimit: {
        min: value,
        max: sellLimitMax,
      },
    })
    return
  }

  const updateResponse = await invoke<InvokeSettings>('update_user_settings', {
    steamId: userSummary?.steamId,
    key: 'tradingCards.sellLimit',
    value: {
      min: value,
      max: sellLimitMax,
    },
  })

  setUserSettings(updateResponse.settings)
}

export const handleSellLimitMaxChange = async (
  value: number,
  userSummary: UserSummary,
  setUserSettings: (value: UserSettings | ((prev: UserSettings) => UserSettings)) => void,
  sellLimitMin: number,
  setSellLimitMax: React.Dispatch<React.SetStateAction<number>>,
) => {
  setSellLimitMax(value)

  if (!isTauri()) {
    updateTradingCardsSetting(setUserSettings, {
      sellLimit: {
        min: sellLimitMin,
        max: value,
      },
    })
    return
  }

  const updateResponse = await invoke<InvokeSettings>('update_user_settings', {
    steamId: userSummary?.steamId,
    key: 'tradingCards.sellLimit',
    value: {
      min: sellLimitMin,
      max: value,
    },
  })

  setUserSettings(updateResponse.settings)
}

export const handleSellDelayChange = async (
  value: number,
  userSummary: UserSummary,
  setUserSettings: (value: UserSettings | ((prev: UserSettings) => UserSettings)) => void,
  setSellDelay: React.Dispatch<React.SetStateAction<number>>,
) => {
  setSellDelay(value)

  if (!isTauri()) {
    updateTradingCardsSetting(setUserSettings, { sellDelay: value })
    return
  }

  const updateResponse = await invoke<InvokeSettings>('update_user_settings', {
    steamId: userSummary?.steamId,
    key: 'tradingCards.sellDelay',
    value,
  })

  setUserSettings(updateResponse.settings)
}
