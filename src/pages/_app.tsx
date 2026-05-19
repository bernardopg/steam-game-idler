import type { AppProps } from 'next/app'
import { TbX } from 'react-icons/tb'
import { HeroUIProvider, ToastProvider } from '@heroui/react'
import { FullscreenLoader, Layout } from '@/shared/components'
import { ErrorBoundaryProvider, I18nProvider, ThemeProvider } from '@/shared/providers'
import { useLoaderStore, useStateStore } from '@/shared/stores'
import '@/styles/globals.css'

const App = ({ Component, pageProps }: AppProps) => {
  const { loadingUserSummary } = useStateStore()
  const { loaderFadeOut } = useLoaderStore()

  return (
    <ErrorBoundaryProvider>
      <I18nProvider>
        <ThemeProvider
          attribute='class'
          themes={['dark', 'blue', 'red', 'purple', 'pink', 'gold', 'black']}
          enableSystem={true}
          defaultTheme='dark'
          disableTransitionOnChange
        >
          <HeroUIProvider>
            <ToastProvider
              placement='top-right'
              toastOffset={44}
              maxVisibleToasts={4}
              regionProps={{
                classNames: {
                  base: 'z-[2147483647] pointer-events-none !w-auto',
                },
              }}
              toastProps={{
                radius: 'md',
                variant: 'flat',
                timeout: 4500,
                shouldShowTimeoutProgress: true,
                closeIcon: <TbX size={14} className='text-content/60' />,
                classNames: {
                  base: [
                    'pointer-events-auto z-[2147483647] cursor-default',
                    'border border-border/60 bg-sidebar/95 shadow-lg backdrop-blur-md',
                    '!w-[300px] !max-w-[300px]',
                  ],
                  title: ['text-content text-xs font-semibold leading-snug'],
                  description: ['text-content/70 text-xs font-normal leading-snug mt-0.5'],
                  closeButton: [
                    'opacity-60 hover:opacity-100 absolute right-1 top-1 hover:bg-item-hover transition-opacity',
                  ],
                  progressTrack: ['bg-border/30 h-[2px]'],
                  progressIndicator: ['h-[2px]'],
                },
              }}
            />
            {loadingUserSummary && <FullscreenLoader loaderFadeOut={loaderFadeOut} />}
            <Layout>
              <Component {...pageProps} />
            </Layout>
          </HeroUIProvider>
        </ThemeProvider>
      </I18nProvider>
    </ErrorBoundaryProvider>
  )
}

export default App
