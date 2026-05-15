import { useEffect, useState } from 'react'
import { useTranslation } from 'react-i18next'
import { TbChevronRight, TbEraser } from 'react-icons/tb'
import { Button, cn, Divider, Input } from '@heroui/react'
import { useTheme } from 'next-themes'
import Image from 'next/image'
import {
  handleBackgroundDelete,
  handleBackgroundSave,
  handleThemeChange,
} from '@/features/settings'
import { ProBadge, SettingsSwitch } from '@/shared/components'
import { useStateStore, useUserStore } from '@/shared/stores'

interface Theme {
  key: string
  label: string
  isProTheme: boolean
}

export const CustomizationSettings = () => {
  const { t } = useTranslation()
  const { setTheme, resolvedTheme } = useTheme()
  const [mounted, setMounted] = useState(false)
  const [selectedTheme, setSelectedTheme] = useState('dark')
  const setProModalOpen = useStateStore(state => state.setProModalOpen)
  const setUserSettings = useUserStore(state => state.setUserSettings)
  const isPro = useUserStore(state => state.isPro)

  // Themes
  const themes: Theme[] = [
    { key: 'dark', label: 'Default', isProTheme: false },
    { key: 'blue', label: 'Blue', isProTheme: true },
    { key: 'red', label: 'Red', isProTheme: true },
    { key: 'purple', label: 'Purple', isProTheme: true },
    { key: 'pink', label: 'Pink', isProTheme: true },
    { key: 'gold', label: 'Gold', isProTheme: true },
    { key: 'black', label: 'Black', isProTheme: true },
  ]

  useEffect(() => {
    const localTheme = localStorage.getItem('theme')
    if (!localTheme) {
      localStorage.setItem('theme', 'dark')
      setTheme('dark')
      setSelectedTheme('dark')
    } else {
      setTheme(localTheme)
      setSelectedTheme(localTheme)
    }
    setMounted(true)
  }, [setTheme])

  useEffect(() => {
    if (resolvedTheme) setSelectedTheme(resolvedTheme)
  }, [resolvedTheme])

  if (!mounted) return null

  return (
    <div className='relative flex flex-col gap-4 mt-9 pb-16 w-4/5'>
      <div className='flex flex-col gap-0 select-none'>
        <p className='flex items-center text-xs text-altwhite font-bold'>
          {t('settings.title')}
          <span>
            <TbChevronRight size={12} />
          </span>
        </p>
        <p className='text-3xl font-black'>{t('settings.customization.title')}</p>
      </div>

      <div className='flex flex-col gap-3 mt-4'>
        <div className='flex justify-between items-center'>
          <div className='flex flex-col gap-2 w-1/2'>
            <p className='text-sm text-content font-bold'>
              {t('settings.general.disableTooltips')}
            </p>
            <p className='text-xs text-altwhite'>
              {t('settings.general.disableTooltips.description')}
            </p>
          </div>
          <SettingsSwitch type='general' name='disableTooltips' />
        </div>

        <Divider className='bg-border/70 my-4' />

        <div className='flex justify-between items-center'>
          <div className='flex flex-col gap-2 w-1/2'>
            <p className='text-sm text-content font-bold'>
              {t('settings.general.showRecommendedCarousel')}
            </p>
            <p className='text-xs text-altwhite'>
              {t('settings.general.showRecommendedCarousel.description')}
            </p>
          </div>
          <SettingsSwitch type='general' name='showRecommendedCarousel' />
        </div>

        <Divider className='bg-border/70 my-4' />

        <div className='flex justify-between items-center'>
          <div className='flex flex-col gap-2 w-1/2'>
            <p className='text-sm text-content font-bold'>
              {t('settings.general.showRecentCarousel')}
            </p>
            <p className='text-xs text-altwhite'>
              {t('settings.general.showRecentCarousel.description')}
            </p>
          </div>
          <SettingsSwitch type='general' name='showRecentCarousel' />
        </div>

        <Divider className='bg-border/70 my-4' />

        <div className='flex justify-between items-center'>
          <div className='flex flex-col gap-2 w-1/2'>
            <p className='text-sm text-content font-bold'>
              {t('settings.general.showCardDropsCarousel')}
            </p>
            <p className='text-xs text-altwhite'>
              {t('settings.general.showCardDropsCarousel.description')}
            </p>
          </div>
          <SettingsSwitch type='general' name='showCardDropsCarousel' />
        </div>

        <Divider className='bg-border/70 my-4' />

        <div className='flex justify-between items-start'>
          <div className='flex flex-col gap-2 w-1/2'>
            <div className='flex items-center'>
              <p className='text-sm text-content font-bold'>
                {t('settings.customization.backgroundImage')}
              </p>
              {!isPro && <ProBadge className='scale-65' />}
            </div>
            <p className='text-xs text-altwhite'>
              {t('settings.customization.backgroundImage.description')}
            </p>
          </div>

          <div
            className='flex flex-col gap-4 w-62.5'
            onClick={() => !isPro && setProModalOpen(true)}
          >
            <Input
              type='file'
              accept='image/*'
              className='max-w-62.5'
              isDisabled={!isPro}
              classNames={{
                base: '',
                inputWrapper: cn(
                  'bg-input data-[hover=true]:!bg-inputhover !cursor-pointer',
                  'rounded-lg group-data-[focus-within=true]:!bg-inputhover',
                ),
                input: ['!text-content cursor-pointer'],
              }}
              onChange={e => handleBackgroundSave(e, setUserSettings)}
            />

            <div className='flex justify-end'>
              <Button
                size='sm'
                variant='light'
                radius='full'
                color='danger'
                onPress={() => handleBackgroundDelete(setUserSettings)}
                startContent={<TbEraser size={20} />}
              >
                {t('common.clear')}
              </Button>
            </div>
          </div>
        </div>

        <Divider className='bg-border/70 my-4' />

        <div className='flex flex-col justify-between gap-6'>
          <div className='flex flex-col gap-2 w-1/2'>
            <p className='text-sm text-content font-bold'>{t('settings.customization.theme')}</p>
            <p className='text-xs text-altwhite'>{t('settings.customization.theme.description')}</p>
          </div>

          <div
            role='radiogroup'
            aria-label={t('settings.customization.theme')}
            className='grid grid-cols-5 gap-x-8 gap-y-6'
          >
            {themes.map(theme => {
              const isSelected = selectedTheme === theme.key
              const isLocked = theme.isProTheme && !isPro

              return (
                <button
                  key={theme.key}
                  type='button'
                  role='radio'
                  aria-checked={isSelected}
                  aria-disabled={isLocked}
                  className={cn(
                    'group flex w-38 flex-col items-start gap-2 rounded-lg text-left outline-none',
                    'transition-transform duration-150 hover:-translate-y-0.5 active:translate-y-0',
                    'focus-visible:ring-2 focus-visible:ring-dynamic focus-visible:ring-offset-2 focus-visible:ring-offset-base',
                    isLocked && 'cursor-pointer opacity-70 hover:opacity-100',
                  )}
                  onClick={() => {
                    if (isLocked) {
                      setProModalOpen(true)
                      return
                    }

                    setSelectedTheme(theme.key)
                    handleThemeChange(theme.key, setTheme)
                  }}
                >
                  <span
                    className={cn(
                      'relative block h-11.25 w-36.75 overflow-hidden rounded-lg border bg-input',
                      'transition-colors duration-150',
                      isSelected
                        ? 'border-dynamic shadow-[0_0_0_2px_hsl(var(--heroui-dynamic)/0.35)]'
                        : 'border-border group-hover:border-dynamic/70',
                    )}
                  >
                    <Image
                      src={`/themes/${theme.key}.webp`}
                      alt={theme.label}
                      width={147}
                      height={45}
                      className='h-full w-full object-cover'
                    />
                    {isSelected && (
                      <span className='pointer-events-none absolute inset-0 rounded-[7px] ring-2 ring-inset ring-dynamic' />
                    )}
                  </span>
                  <span className='flex min-w-0 items-center gap-2'>
                    <span
                      className={cn(
                        'truncate text-sm font-medium',
                        isSelected ? 'text-content' : 'text-altwhite',
                      )}
                    >
                      {theme.label}
                    </span>
                    {isLocked && <ProBadge className='scale-75' />}
                  </span>
                </button>
              )
            })}
          </div>
        </div>
      </div>
    </div>
  )
}
