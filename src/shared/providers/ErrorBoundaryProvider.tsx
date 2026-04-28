import { Component } from 'react'
import { Button, cn } from '@heroui/react'
import { ExtLink } from '@/shared/components'

const CHUNK_RELOAD_KEY = 'sgi:chunk-load-reload-at'

function isChunkLoadError(error: unknown) {
  const errorLike = error as { message?: unknown; name?: unknown }
  const message = String(errorLike?.message ?? error ?? '')
  const name = String(errorLike?.name ?? '')

  return (
    name === 'ChunkLoadError' ||
    message.includes('ChunkLoadError') ||
    message.includes('Failed to load chunk') ||
    message.includes('Loading chunk') ||
    message.includes('/_next/static/chunks/')
  )
}

function recoverFromChunkLoadError(error: unknown) {
  if (typeof window === 'undefined' || !isChunkLoadError(error)) return false

  const now = Date.now()
  const lastReload = Number(window.sessionStorage.getItem(CHUNK_RELOAD_KEY) || 0)

  if (now - lastReload < 10000) return false

  window.sessionStorage.setItem(CHUNK_RELOAD_KEY, String(now))
  window.setTimeout(() => window.location.reload(), 50)
  return true
}

interface ErrorBoundaryState {
  hasError: boolean
  error: Error | null
  errorInfo: React.ErrorInfo | null
  recoveringChunk: boolean
}

interface ErrorBoundaryProps {
  children: React.ReactNode
}

export class ErrorBoundaryProvider extends Component<ErrorBoundaryProps, ErrorBoundaryState> {
  constructor(props: ErrorBoundaryProps) {
    super(props)
    this.state = {
      hasError: false,
      error: null,
      errorInfo: null,
      recoveringChunk: false,
    }
  }

  static getDerivedStateFromError(): Partial<ErrorBoundaryState> {
    return { hasError: true }
  }

  componentDidMount() {
    window.setTimeout(() => {
      window.sessionStorage.removeItem(CHUNK_RELOAD_KEY)
    }, 15000)

    window.addEventListener('error', this.handleWindowError)
    window.addEventListener('unhandledrejection', this.handleUnhandledRejection)
  }

  componentWillUnmount() {
    window.removeEventListener('error', this.handleWindowError)
    window.removeEventListener('unhandledrejection', this.handleUnhandledRejection)
  }

  componentDidCatch(error: Error, errorInfo: React.ErrorInfo) {
    const recoveringChunk = recoverFromChunkLoadError(error)

    this.setState({
      error,
      errorInfo,
      recoveringChunk,
    })

    console.error('Client side error caught by ErrorBoundary:', error, errorInfo)
  }

  private handleWindowError = (event: ErrorEvent) => {
    recoverFromChunkLoadError(event.error ?? event.message)
  }

  private handleUnhandledRejection = (event: PromiseRejectionEvent) => {
    recoverFromChunkLoadError(event.reason)
  }

  render(): React.ReactNode {
    if (this.state.hasError) {
      const { error, errorInfo, recoveringChunk } = this.state

      const issueTitle = error && String(error)
      const issueBody = `### Description
<give a brief description of what you were doing when the error occurred>

### Steps to reproduce
<give a step-by-step description of how to reproduce the error>

### Stack
\`\`\`
${errorInfo && errorInfo.componentStack}
\`\`\``
      const encodedTitle = encodeURIComponent(issueTitle || 'Error in Steam Game Idler')
      const encodedBody = encodeURIComponent(issueBody)

      return (
        <div className='bg-base h-screen w-screen'>
          <div
            className={cn(
              'absolute top-0 left-0 w-full h-12 select-none',
              'flex justify-center items-center bg-sidebar',
            )}
            data-tauri-drag-region
          >
            <p className='text-content font-bold'>Uh-oh!</p>
          </div>

          <div className='flex flex-col items-center justify-center gap-2 h-full text-content'>
            <div
              className={cn(
                'flex flex-col justify-center gap-4 h-[65%] w-[80%]',
                'bg-tab-panel rounded-lg border border-border p-4',
              )}
            >
              <p className='text-sm'>
                {recoveringChunk
                  ? 'Reloading after a development chunk update...'
                  : 'An error occurred while rendering the application'}
              </p>

              <div className='flex flex-col'>
                <p className='font-bold'>Error</p>
                <p className='text-sm font-mono text-danger font-semibold'>
                  {error && String(error).replace('Error: ', '')}
                </p>
              </div>

              <div className='flex flex-col overflow-hidden'>
                <p className='font-bold'>Stack</p>
                <div className='bg-base border border-border rounded-lg h-full w-full p-1 overflow-hidden'>
                  <div className='overflow-y-scroll h-full'>
                    <pre className='text-xs text-altwhite font-semibold text-left text-wrap p-1'>
                      {errorInfo && errorInfo.componentStack}
                    </pre>
                  </div>
                </div>
              </div>
            </div>

            <div className='flex gap-4'>
              <ExtLink
                href={`https://github.com/zevnda/steam-game-idler/issues/new?title=${encodedTitle}&body=${encodedBody}`}
              >
                <div className='bg-warning p-2 font-semibold rounded-lg'>
                  <p className='text-xs'>Report on GitHub</p>
                </div>
              </ExtLink>

              <Button
                size='sm'
                className='font-semibold rounded-lg bg-dynamic'
                onPress={() => window.location.reload()}
              >
                Reload
              </Button>
            </div>
          </div>
        </div>
      )
    }

    return this.props.children
  }
}
