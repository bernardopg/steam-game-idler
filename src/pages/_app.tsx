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
                  base: 'z-[2147483647] pointer-events-none',
                },
              }}
              toastProps={{
                radius: 'sm',
                variant: 'flat',
                timeout: 4500,
                shouldShowTimeoutProgress: true,
                closeIcon: <TbX size={16} className='text-content' />,
                classNames: {
                  base: [
                    'pointer-events-auto z-[2147483647] cursor-default border border-border bg-sidebar/95 shadow-2xl backdrop-blur-md',
                  ],
                  title: ['text-content text-sm font-semibold leading-snug'],
                  description: ['text-content/80 text-xs font-medium'],
                  closeButton: ['opacity-100 absolute right-1 top-1 hover:bg-item-hover'],
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
