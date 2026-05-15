import { addToast } from '@heroui/react'
import i18next from 'i18next'
import { ErrorToast } from '@/shared/components'

export function showSuccessToast(description: string, title?: string) {
  addToast({
    title: title ?? i18next.t('common.done'),
    description,
    color: 'success',
  })
}

export function showPrimaryToast(description: string, title?: string) {
  addToast({
    title: title ?? i18next.t('common.notice'),
    description,
    color: 'primary',
  })
}

export function showWarningToast(description: string, title?: string) {
  addToast({
    title: title ?? i18next.t('common.notice'),
    description,
    color: 'warning',
  })
}

export function showDangerToast(description: string, title?: string) {
  addToast({
    title: title ?? i18next.t('common.errorTitle'),
    description,
    color: 'danger',
  })
}

export function showSteamNotRunningToast() {
  addToast({
    title: i18next.t('toast.steam'),
    description: (
      <ErrorToast href='https://steamgameidler.com/docs/faq#error-messages:~:text=Steam%20is%20not%20running' />
    ),
    color: 'danger',
  })
}

export function showAccountMismatchToast(color: 'danger' | 'warning') {
  addToast({
    title: i18next.t('toast.mismatch'),
    description: (
      <ErrorToast href='https://steamgameidler.com/docs/faq#error-messages:~:text=Account%20mismatch%20between%20Steam%20and%20SGI' />
    ),
    color,
  })
}

export function showMissingCredentialsToast() {
  addToast({
    title: i18next.t('toast.missingCredentials'),
    description: (
      <ErrorToast href='https://steamgameidler.com/docs/faq#error-messages:~:text=Missing%20card%20farming%20credentials%20in%20%E2%80%9Csettings%20%3E%20card%20farming%22' />
    ),
    color: 'danger',
  })
}

export function showOutdatedCredentialsToast() {
  addToast({
    title: i18next.t('toast.outdatedCredentials'),
    description: (
      <ErrorToast href='https://steamgameidler.com/docs/faq#error-messages:~:text=Card%20farming%20credentials%20need%20to%20be%20updated%20in%20%E2%80%9Csettings%20%3E%20general%22' />
    ),
    color: 'danger',
  })
}

export function showEnableAllGamesToast() {
  addToast({
    title: i18next.t('toast.enableAllGames'),
    description: (
      <ErrorToast href='https://steamgameidler.com/docs/faq#error-messages:~:text=Add%20some%20games%20to%20your%20card%20farming%20list%20or%20enable%20%E2%80%9Call%20games%E2%80%9D%20in%20%E2%80%9Csettings%20%3E%20card%20farming%22' />
    ),
    color: 'danger',
  })
}

export function showIncorrectCredentialsToast() {
  addToast({
    title: i18next.t('toast.incorrectCredentials'),
    description: (
      <ErrorToast href='https://steamgameidler.com/docs/faq#error-messages:~:text=Incorrect%20card%20farming%20credentials' />
    ),
    color: 'danger',
  })
}

export function showNoGamesToast() {
  addToast({
    title: i18next.t('toast.noGames'),
    description: (
      <ErrorToast href='https://steamgameidler.com/docs/faq#:~:text=There%20are%20no%20games%20in%20your%20list' />
    ),
    color: 'danger',
  })
}

export function showPriceFetchCooldownToast(cooldown: number) {
  addToast({
    title: i18next.t('toast.tradingCards.cooldown', { cooldown }),
    description: (
      <ErrorToast href='https://steamgameidler.com/docs/faq#error-messages:~:text=Please%20wait%20X%20seconds%20before%20fetching%20more%20card%20prices' />
    ),
    color: 'warning',
  })
}

export function showPriceFetchRateLimitToast() {
  addToast({
    title: i18next.t('toast.tradingCards.rateLimit'),
    description: (
      <ErrorToast href='https://steamgameidler.com/docs/faq#:~:text=Rate%20limited%20when%20fetching%20card%20prices' />
    ),
    color: 'danger',
  })
}
