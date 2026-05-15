import { useEffect, useState } from 'react'
import tauriConfig from '../../../../src-tauri/tauri.conf.json'
import { getAppVersion } from '@/shared/utils'

export function useSettings() {
  const [version, setVersion] = useState(tauriConfig.version)
  const [refreshKey, setRefreshKey] = useState(0)

  useEffect(() => {
    const getAndSetVersion = async () => {
      const version = await getAppVersion()
      if (version) setVersion(version)
    }

    getAndSetVersion()

    window.addEventListener('focus', getAndSetVersion)
    document.addEventListener('visibilitychange', getAndSetVersion)

    const intervalId = window.setInterval(getAndSetVersion, 60 * 1000)

    return () => {
      window.removeEventListener('focus', getAndSetVersion)
      document.removeEventListener('visibilitychange', getAndSetVersion)
      window.clearInterval(intervalId)
    }
  }, [])

  return { version, refreshKey, setRefreshKey }
}
