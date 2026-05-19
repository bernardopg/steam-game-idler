import { isEnabled } from '@tauri-apps/plugin-autostart'
import { platform } from '@tauri-apps/plugin-os'
import { useEffect, useState } from 'react'
import { useUserStore } from '@/shared/stores'
import { isTauri } from '@/shared/utils/tauri'

export const useGeneralSettings = () => {
  const userSettings = useUserStore(state => state.userSettings)
  const [startupState, setStartupState] = useState<boolean | null>(null)
  const [osPlatform, setOsPlatform] = useState<string>('Windows')
  const [keyValue, setKeyValue] = useState('')
  const [hasKey, setHasKey] = useState(false)
  const [sliderLabel, setSliderLabel] = useState('')

  useEffect(() => {
    const checkStartupState = async () => {
      if (!isTauri()) {
        setStartupState(false)
        return
      }

      const [isEnabledState, p] = await Promise.all([isEnabled(), platform()])
      setStartupState(isEnabledState)
      setOsPlatform(p === 'linux' ? 'Linux' : 'Windows')
    }
    checkStartupState()
  }, [])

  useEffect(() => {
    const apiKey = userSettings.general.apiKey
    if (apiKey && apiKey.length > 0) {
      setHasKey(true)
      setKeyValue(apiKey)
    }
  }, [userSettings.general.apiKey])

  return {
    startupState,
    setStartupState,
    osPlatform,
    keyValue,
    setKeyValue,
    hasKey,
    setHasKey,
    sliderLabel,
    setSliderLabel,
  }
}
