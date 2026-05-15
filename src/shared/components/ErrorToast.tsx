import { useTranslation } from 'react-i18next'
import { ExtLink } from '@/shared/components'

interface ErrorToastProps {
  href: string
}

export const ErrorToast = ({ href }: ErrorToastProps) => {
  const { t } = useTranslation()

  return (
    <ExtLink href={href}>
      <span className='text-xs text-dynamic hover:text-dynamic-hover underline underline-offset-2'>
        {t('common.learnMore')}
      </span>
    </ExtLink>
  )
}
