import type { Game, InvokeCustomList, InvokeSettings } from '@/shared/types'
import { useEffect, useState } from 'react'
import { showDangerToast } from '@/shared/components'
import { useSearchStore, useStateStore, useUserStore } from '@/shared/stores'
import { invoke, isTauri } from '@/shared/utils/tauri'

export type CustomListTab = 'all' | 'list' | 'blacklist'

export function useCustomList(listName: string) {
  const isAchievementUnlocker = useStateStore(state => state.isAchievementUnlocker)
  const isCardFarming = useStateStore(state => state.isCardFarming)
  const userSummary = useUserStore(state => state.userSummary)
  const gamesList = useUserStore(state => state.gamesList)
  const setUserSettings = useUserStore(state => state.setUserSettings)
  const searchTerm = useSearchStore(state => state.customListQueryValue)
  const [list, setList] = useState<Game[]>([])
  const [isLoading, setIsLoading] = useState(true)
  const [activeTab, setActiveTab] = useState<CustomListTab>('list')

  const filteredGamesList = gamesList.filter(game =>
    game.name.toLowerCase().includes(searchTerm.toLowerCase()),
  )

  useEffect(() => {
    const getCustomLists = async () => {
      if (!isTauri()) {
        setList([])
        setIsLoading(false)
        return
      }

      const response = await invoke<InvokeCustomList>('get_custom_lists', {
        steamId: userSummary?.steamId,
        list: listName,
      })
      if (!response.error) {
        setList(response.list_data)
      } else {
        showDangerToast(response.error)
        setList([])
      }
      setIsLoading(false)
    }
    getCustomLists()
  }, [userSummary?.steamId, isAchievementUnlocker, isCardFarming, listName])

  const handleAddGame = async (game: Game) => {
    if (!isTauri()) return

    const response = await invoke<InvokeCustomList>('add_game_to_custom_list', {
      steamId: userSummary?.steamId,
      game: { appid: game.appid, name: game.name },
      list: listName,
    })
    if (!response.error) {
      setList(response.list_data)
    } else {
      showDangerToast(response.error)
    }
  }

  const handleAddAllGames = async (games: Game[]) => {
    if (!isTauri()) return

    const clearResponse = await invoke<InvokeCustomList>('update_custom_list', {
      steamId: userSummary?.steamId,
      list: listName,
      newList: [],
    })
    if (!clearResponse.error) {
      const addResponse = await invoke<InvokeCustomList>('update_custom_list', {
        steamId: userSummary?.steamId,
        list: listName,
        newList: games,
      })
      if (!addResponse.error) {
        setList(addResponse.list_data)
      } else {
        showDangerToast(addResponse.error)
      }
    } else {
      showDangerToast(clearResponse.error)
    }
  }

  const handleAddAllResults = async (games: Game[]) => {
    if (!isTauri()) return

    const newGames = games.filter(
      game => !list.some(existingGame => existingGame.appid === game.appid),
    )

    const combinedList = [...list, ...newGames]

    const addResponse = await invoke<InvokeCustomList>('update_custom_list', {
      steamId: userSummary?.steamId,
      list: listName,
      newList: combinedList,
    })
    if (!addResponse.error) {
      setList(addResponse.list_data)
    } else {
      showDangerToast(addResponse.error)
    }
  }

  const handleRemoveGame = async (game: Game) => {
    if (!isTauri()) return

    const response = await invoke<InvokeCustomList>('remove_game_from_custom_list', {
      steamId: userSummary?.steamId,
      game: { appid: game.appid, name: game.name },
      list: listName,
    })
    if (!response.error) {
      setList(response.list_data)
    } else {
      showDangerToast(response.error)
    }
  }

  const handleBlacklistGame = async (game: Game) => {
    if (!isTauri()) return

    const cachedUserSummary = await invoke<InvokeSettings>('get_user_settings', {
      steamId: userSummary?.steamId,
    })

    const currentBlacklist: number[] = cachedUserSummary.settings.cardFarming.blacklist || []

    const updatedBlacklist = currentBlacklist.includes(game.appid)
      ? currentBlacklist.filter(appid => appid !== game.appid)
      : [...currentBlacklist, game.appid]

    if (updatedBlacklist.length === 0 && activeTab === 'blacklist') {
      setActiveTab('all')
    }

    invoke<InvokeSettings>('update_user_settings', {
      steamId: userSummary?.steamId,
      key: 'cardFarming.blacklist',
      value: updatedBlacklist,
    })

    setUserSettings(prevSettings => ({
      ...prevSettings,
      cardFarming: {
        ...prevSettings.cardFarming,
        blacklist: updatedBlacklist,
      },
    }))
  }

  const handleUpdateListOrder = async (newList: Game[]) => {
    if (!isTauri()) return

    const response = await invoke<InvokeCustomList>('update_custom_list', {
      steamId: userSummary?.steamId,
      list: listName,
      newList,
    })
    if (!response.error) {
      setList(response.list_data)
    } else {
      showDangerToast(response.error)
    }
  }

  const handleClearList = async () => {
    if (!isTauri()) return

    const response = await invoke<InvokeCustomList>('update_custom_list', {
      steamId: userSummary?.steamId,
      list: listName,
      newList: [],
    })
    if (!response.error) {
      setList([])
    } else {
      showDangerToast(response.error)
    }
  }

  const handleClearBlacklist = () => {
    if (!isTauri()) return

    invoke<InvokeSettings>('update_user_settings', {
      steamId: userSummary?.steamId,
      key: 'cardFarming.blacklist',
      value: [],
    })
    setUserSettings(prevSettings => ({
      ...prevSettings,
      cardFarming: {
        ...prevSettings.cardFarming,
        blacklist: [],
      },
    }))
  }

  return {
    list,
    setList,
    isLoading,
    filteredGamesList,
    searchTerm,
    activeTab,
    setActiveTab,
    handleAddGame,
    handleAddAllGames,
    handleAddAllResults,
    handleRemoveGame,
    handleUpdateListOrder,
    handleClearList,
    handleClearBlacklist,
    handleBlacklistGame,
  }
}
