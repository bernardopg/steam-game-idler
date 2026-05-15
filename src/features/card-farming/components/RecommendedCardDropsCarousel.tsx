import type { Game } from '@/shared/types'
import { useTranslation } from 'react-i18next'
import { TbPlus } from 'react-icons/tb'
import { Button, Spinner } from '@heroui/react'
import Image from 'next/image'

interface RecommendedCardDropsCarouselProps {
  gamesWithDrops: Game[]
  onAddGame: (game: Game) => void
  isLoading: boolean
}

export const RecommendedCardDropsCarousel = ({
  gamesWithDrops,
  onAddGame,
  isLoading,
}: RecommendedCardDropsCarouselProps) => {
  const { t } = useTranslation()

  const handleImageError = (event: React.SyntheticEvent<HTMLImageElement, Event>) => {
    ;(event.target as HTMLImageElement).src = '/fallback.webp'
  }

  if (isLoading) {
    return (
      <div className='mb-6 px-6 mt-4'>
        <p className='text-lg font-black mb-3'>{t('gamesList.recommended')}</p>
        <div className='h-48 flex items-center justify-center'>
          <Spinner size='lg' />
        </div>
      </div>
    )
  }

  if (!gamesWithDrops || gamesWithDrops.length === 0) {
    return <div />
  }

  return (
    <div className='mb-6 px-6 mt-4'>
      <p className='text-lg font-black mb-3'>{t('gamesList.recommended')}</p>

      <div className='grid gap-4 grid-cols-2 sm:grid-cols-3 lg:grid-cols-4 xl:grid-cols-5 2xl:grid-cols-6'>
        {gamesWithDrops.map(game => (
          <div key={game.appid} className='group select-none'>
            <div className='relative overflow-hidden rounded-lg'>
              <div className='aspect-460/215 relative overflow-hidden rounded-lg'>
                <Image
                  src={`https://cdn.cloudflare.steamstatic.com/steam/apps/${game.appid}/header.jpg`}
                  width={460}
                  height={215}
                  alt={`${game.name} image`}
                  priority={true}
                  onError={handleImageError}
                  className='w-full h-full object-cover rounded-lg duration-150 group-hover:scale-105'
                />
                <div
                  className='pointer-events-none absolute inset-0 rounded-lg opacity-0 group-hover:opacity-100 transition-opacity duration-150'
                  style={{ boxShadow: 'inset 0 0 0 2px hsl(var(--heroui-dynamic))' }}
                />
                <Button
                  isIconOnly
                  size='sm'
                  radius='full'
                  className='absolute top-1.5 right-1.5 opacity-0 group-hover:opacity-100 bg-black/60 hover:bg-black/80 text-white transition-all duration-150'
                  onPress={() => onAddGame(game)}
                >
                  <TbPlus size={16} />
                </Button>
              </div>
            </div>

            <div className='flex justify-between items-center pt-2 gap-2'>
              <h3 className='text-xs font-bold text-altwhite group-hover:text-content truncate duration-150 flex-1'>
                {game.name}
              </h3>
              <span className='text-xs bg-white text-black font-semibold px-2 py-0.5 rounded-full shrink-0'>
                {t('customLists.cardFarming.drops', { count: game.remaining || 0 })}
              </span>
            </div>
          </div>
        ))}
      </div>
    </div>
  )
}
