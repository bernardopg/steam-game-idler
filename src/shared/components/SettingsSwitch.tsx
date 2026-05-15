import type {
  AchievementUnlockerSettings,
  CardFarmingSettings,
  GeneralSettings,
} from '@/shared/types'
import { useTranslation } from 'react-i18next'
import { cn, Switch } from '@heroui/react'
import {
  handleCheckboxChange,
  handleRunAtStartupChange,
  useAchievementSettings,
  useCardSettings,
  useGeneralSettings,
} from '@/features/settings'
import { useUserStore } from '@/shared/stores'
import { antiAwayStatus } from '@/shared/utils'

interface SettingsCheckboxProps {
  type: 'general' | 'cardFarming' | 'achievementUnlocker'
  name: string
  isProSetting?: boolean
}

export const SettingsSwitch = ({ type, name, isProSetting = false }: SettingsCheckboxProps) => {
  const { t } = useTranslation()
  const userSummary = useUserStore(state => state.userSummary)
  const userSettings = useUserStore(state => state.userSettings)
  const setUserSettings = useUserStore(state => state.setUserSettings)
  const isPro = useUserStore(state => state.isPro)
  const { startupState, setStartupState } = useGeneralSettings()

  useCardSettings()
  useAchievementSettings()

  const isSettingEnabled = () => {
    if (!userSettings) return false

    if (type === 'general') {
      return Boolean((userSettings.general as GeneralSettings)[name as keyof GeneralSettings])
    }
    if (type === 'cardFarming') {
      return Boolean(
        (userSettings.cardFarming as CardFarmingSettings)[name as keyof CardFarmingSettings],
      )
    }
    if (type === 'achievementUnlocker') {
      return Boolean(
        (userSettings.achievementUnlocker as AchievementUnlockerSettings)[
          name as keyof AchievementUnlockerSettings
        ],
      )
    }
    return false
  }

  const switchLabel =
    type === 'general'
      ? t(`settings.general.${name}`, { defaultValue: name })
      : type === 'cardFarming'
        ? t(`settings.cardFarming.${name}`, { defaultValue: name })
        : t(`settings.achievementUnlocker.${name}`, { defaultValue: name })

  const switchClassNames = {
    wrapper: cn(
      '!bg-switch group-data-[hover=true]:!bg-inputhover',
      'group-data-[selected=true]:!bg-dynamic',
      'transition-colors duration-200',
    ),
    thumb: cn('!bg-content transition-all duration-200', 'group-data-[selected=true]:!ms-4'),
  }

  if (name === 'antiAway') {
    return (
      <Switch
        aria-label={switchLabel}
        size='sm'
        name={name}
        isSelected={isSettingEnabled()}
        classNames={switchClassNames}
        onChange={(e: React.ChangeEvent<HTMLInputElement>) => {
          handleCheckboxChange(e, 'general', userSummary?.steamId, setUserSettings)
          antiAwayStatus(isSettingEnabled() ? null : undefined)
        }}
      />
    )
  }

  if (name === 'runAtStartup') {
    return (
      <Switch
        aria-label={switchLabel}
        size='sm'
        name={name}
        isSelected={startupState || false}
        classNames={switchClassNames}
        onChange={() => handleRunAtStartupChange(setStartupState)}
      />
    )
  }

  return (
    <Switch
      aria-label={switchLabel}
      size='sm'
      name={name}
      isSelected={isSettingEnabled()}
      isDisabled={isProSetting && !isPro}
      classNames={switchClassNames}
      onChange={e => {
        if (type === 'general') {
          handleCheckboxChange(e, 'general', userSummary?.steamId, setUserSettings)
        } else if (type === 'cardFarming') {
          handleCheckboxChange(e, 'cardFarming', userSummary?.steamId, setUserSettings)
        } else {
          handleCheckboxChange(e, 'achievementUnlocker', userSummary?.steamId, setUserSettings)
        }
      }}
    />
  )
}
