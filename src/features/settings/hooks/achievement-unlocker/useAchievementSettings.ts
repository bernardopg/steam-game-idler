import { useCallback, useEffect, useState } from 'react'
import { useTranslation } from 'react-i18next'
import { useUserStore } from '@/shared/stores'

const DEFAULT_INTERVAL: [number, number] = [30, 130]

export const useAchievementSettings = () => {
  const { t } = useTranslation()
  const userSettings = useUserStore(state => state.userSettings)
  const [sliderLabel, setSliderLabel] = useState('')
  const [sliderValue, setSliderValue] = useState<[number, number]>(DEFAULT_INTERVAL)

  const getSliderLabel = useCallback(
    (interval: [number, number]) =>
      t('settings.achievementUnlocker.interval', {
        min: interval[0],
        max: interval[1],
      }),
    [t],
  )

  // Sync local settings with global settings when they change
  useEffect(() => {
    const interval = userSettings.achievementUnlocker?.interval ?? DEFAULT_INTERVAL
    setSliderValue(interval)
    setSliderLabel(getSliderLabel(interval))
  }, [userSettings.achievementUnlocker?.interval, getSliderLabel])

  return { sliderLabel, setSliderLabel, sliderValue, setSliderValue, getSliderLabel }
}
