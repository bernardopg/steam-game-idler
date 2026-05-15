import type { LogEntry } from '@/shared/types'
import { useState } from 'react'
import { useTranslation } from 'react-i18next'
import {
  TbAlertTriangle,
  TbArrowBarToDown,
  TbChevronRight,
  TbEraser,
  TbFolders,
} from 'react-icons/tb'
import { Button, cn, useDisclosure } from '@heroui/react'
import { GeistMono } from 'geist/font/mono'
import {
  ClearData,
  ExportSettings,
  handleClearLogs,
  handleOpenLogFile,
  OpenSettings,
  ResetSettings,
  useLogs,
  useSettings,
} from '@/features/settings'
import { CustomModal } from '@/shared/components'

export const Logs = () => {
  const { t } = useTranslation()
  const { logs }: { logs: LogEntry[] } = useLogs()
  const { setRefreshKey } = useSettings()
  const { isOpen, onOpen, onOpenChange } = useDisclosure()
  const [autoScroll, setAutoScroll] = useState(true)

  const handleConfirmClearLogs = (onClose: () => void) => {
    handleClearLogs()
    onClose()
  }

  return (
    <div className='relative flex flex-col gap-4 mt-9 pr-10'>
      <div className='flex flex-col gap-0 select-none'>
        <p className='flex items-center text-xs text-altwhite font-bold'>
          {t('settings.title')}
          <span>
            <TbChevronRight size={12} />
          </span>
        </p>
        <p className='text-3xl font-black'>{t('settings.debug.title')}</p>
      </div>

      <div className='flex flex-col gap-3 mt-4'>
        {/* Tools section */}
        <div className='flex flex-col gap-2'>
          <p className='text-xs font-semibold text-altwhite/50 uppercase tracking-widest'>
            {t('settings.debug.tools')}
          </p>
          <div className='flex flex-wrap items-center gap-2'>
            <Button
              size='sm'
              className='bg-btn-secondary text-btn-text font-bold'
              radius='full'
              onPress={handleOpenLogFile}
              startContent={<TbFolders size={18} />}
            >
              {t('settings.debug.viewLogFile')}
            </Button>
            <OpenSettings />
            <ExportSettings />
            <Button
              size='sm'
              className='bg-btn-secondary text-btn-text font-bold'
              radius='full'
              onPress={() => setAutoScroll(v => !v)}
              startContent={<TbArrowBarToDown size={18} />}
            >
              {autoScroll ? t('settings.debug.autoScrollOn') : t('settings.debug.autoScrollOff')}
            </Button>
          </div>
        </div>

        {/* Danger zone section */}
        <div className='flex flex-col gap-2'>
          <p className='text-xs font-semibold text-red-400/70 uppercase tracking-widest flex items-center gap-1'>
            <TbAlertTriangle size={12} />
            {t('settings.debug.destructive')}
          </p>
          <div className='flex flex-wrap items-center gap-2'>
            <Button
              size='sm'
              variant='flat'
              radius='full'
              color='danger'
              onPress={onOpen}
              startContent={<TbEraser size={18} />}
            >
              {t('settings.debug.clearLogs')}
            </Button>
            <ResetSettings setRefreshKey={setRefreshKey} />
            <ClearData />
          </div>
        </div>
      </div>

      {/* Log panel */}
      <div className='border border-border rounded-lg overflow-hidden bg-base/20'>
        <div
          className='h-[calc(100vh-330px)] overflow-y-auto'
          ref={el => {
            if (el && autoScroll) el.scrollTop = el.scrollHeight
          }}
        >
          {logs.length > 0 ? (
            <div className='divide-y divide-border/30'>
              {logs.map((log, index) => (
                <div
                  key={log.id}
                  className='flex items-start gap-3 px-4 py-2 hover:bg-item-hover/30 transition-colors group duration-150'
                >
                  <div className='flex items-center gap-2 min-w-0 shrink-0'>
                    <span className='text-xs text-altwhite/60 font-mono tabular-nums'>
                      {String(index + 1).padStart(3, '0')}
                    </span>
                    <div
                      className={cn(
                        'w-1.5 h-1.5 rounded-full shrink-0',
                        log.message?.includes('Error') || log.message?.includes('[Error]')
                          ? 'bg-red-400'
                          : log.message?.includes('Warning') || log.message?.includes('[Warn]')
                            ? 'bg-yellow-400'
                            : 'bg-green-400',
                      )}
                    />
                  </div>

                  <div className='min-w-0 flex-1'>
                    <div className='flex items-baseline gap-3'>
                      <span
                        className={cn(
                          'text-xs text-altwhite/60 font-mono shrink-0',
                          GeistMono.className,
                        )}
                      >
                        [{log.timestamp}]
                      </span>
                      <span
                        className={cn(
                          'text-xs font-mono leading-relaxed break-all',
                          log.message?.includes('Error') || log.message?.includes('[Error]')
                            ? 'text-red-300'
                            : log.message?.includes('Warning') || log.message?.includes('[Warn]')
                              ? 'text-yellow-300'
                              : 'text-content',
                          GeistMono.className,
                        )}
                      >
                        {log.message}
                      </span>
                    </div>
                  </div>
                </div>
              ))}
            </div>
          ) : (
            <div className='flex items-center justify-center py-16'>
              <div className='text-center'>
                <div className='text-2xl text-altwhite/30 mb-2'>◯</div>
                <p className='text-sm text-altwhite/60 font-mono'>{t('settings.debug.noLogs')}</p>
              </div>
            </div>
          )}
        </div>
      </div>

      {/* Clear logs confirmation modal */}
      <CustomModal
        isOpen={isOpen}
        onOpenChange={onOpenChange}
        title={t('common.confirm')}
        body={t('confirmation.clearLogs')}
        buttons={
          <>
            <Button
              size='sm'
              color='danger'
              variant='light'
              radius='full'
              className='font-semibold'
              onPress={onOpenChange}
            >
              {t('common.cancel')}
            </Button>
            <Button
              size='sm'
              className='bg-btn-secondary text-btn-text font-bold'
              radius='full'
              onPress={() => handleConfirmClearLogs(onOpenChange)}
            >
              {t('common.confirm')}
            </Button>
          </>
        }
      />
    </div>
  )
}
