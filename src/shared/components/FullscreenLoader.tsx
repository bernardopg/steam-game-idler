import { cn, Spinner } from '@heroui/react'
import { Unbounded } from 'next/font/google'

const unbounded = Unbounded({
  subsets: ['latin'],
  variable: '--font-unbounded',
})

export const FullscreenLoader = ({ loaderFadeOut = false }: { loaderFadeOut?: boolean }) => {
  return (
    <div
      className={cn(
        'fixed inset-0 z-9998 h-screen w-screen bg-black transition-opacity duration-250',
        {
          'opacity-0 pointer-events-none': loaderFadeOut,
          'opacity-100': !loaderFadeOut,
        },
      )}
    >
      <video
        src='/loader.webm'
        autoPlay
        loop
        muted
        playsInline
        className='absolute inset-0 h-screen w-screen object-cover blur'
      />
      <div className='absolute inset-0 z-10 flex flex-col items-center justify-center space-y-10'>
        <p className={`${unbounded.className} text-4xl font-black uppercase text-white`}>
          Steam Game Idler
        </p>
        <Spinner className='text-white' color='current' size='lg' />
      </div>
    </div>
  )
}
