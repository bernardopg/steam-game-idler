import { platform } from '@tauri-apps/plugin-os'
import { relaunch } from '@tauri-apps/plugin-process'
import { check } from '@tauri-apps/plugin-updater'
import { useState } from 'react'
import { useTranslation } from 'react-i18next'
import { TbCircleArrowDown } from 'react-icons/tb'
import { Spinner } from '@heroui/react'
import { CustomTooltip, showDangerToast, showPrimaryToast } from '@/shared/components'
import { fetchLatest, logEvent, preserveKeysAndClearData } from '@/shared/utils'
import { invoke } from '@/shared/utils/tauri'

export const UpdateButton = () => {
  const { t } = useTranslation()
  const [isLoading, setIsLoading] = useState(false)

  const handleUpdate = async () => {
    try {
      setIsLoading(true)
      const update = await check()
      if (update) {
        const latest = await fetchLatest()
        if ((await platform()) === 'linux' && !latest?.platforms?.['linux-x86_64']) {
          setIsLoading(false)
          return
        }
        localStorage.setItem('hasUpdated', 'true')
        await invoke('kill_all_steamutil_processes')
        await update.downloadAndInstall()
        if (latest?.major) {
          await preserveKeysAndClearData()
        }
        await relaunch()
      } else {
        setIsLoading(false)
      }
    } catch (error) {
      setIsLoading(false)
      const msg = String(error)
      if (msg.includes('linux-x86_64') && msg.includes('platforms')) {
        showPrimaryToast(t('toast.checkUpdate.none'))
        return
      }
      showDangerToast(t('toast.checkUpdate.error'))
      console.error('Error in (handleUpdate):', error)
      logEvent(`Error in (handleUpdate): ${error}`)
    }
  }

  return (
    <div>
      {isLoading ? (
        <div className='flex items-center p-2 rounded-full'>
          <Spinner size='sm' variant='simple' />
        </div>
      ) : (
        <CustomTooltip content={t('common.updateReady')}>
          <div className='flex justify-center items-center cursor-pointer' onClick={handleUpdate}>
            <div className='flex items-center px-1 py-1.5 text-success hover:text-success/80 duration-150'>
              <TbCircleArrowDown fontSize={20} />
            </div>
          </div>
        </CustomTooltip>
      )}
    </div>
  )
}
