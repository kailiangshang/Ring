import { create } from 'zustand'
import type { ArchiveRecord, RepoStatus } from '../types/archive'
import * as api from '../services/api'

interface ArchiveState {
  archives: ArchiveRecord[]
  queue: ArchiveRecord[]
  repoStatus: RepoStatus | null
  loading: boolean
  archiving: boolean
  progress: string

  fetchArchives: (ringId: string) => Promise<void>
  fetchQueue: (ringId: string) => Promise<void>
  fetchRepoStatus: (ringId: string) => Promise<void>
  triggerArchive: (
    ringId: string,
    content: string,
    title: string,
    sessionId?: string,
  ) => Promise<void>
  reviewArchive: (
    ringId: string,
    archiveId: string,
    action: 'merge' | 'reject',
  ) => Promise<void>
  initRepo: (ringId: string) => Promise<void>
}

export const useArchiveStore = create<ArchiveState>((set, get) => ({
  archives: [],
  queue: [],
  repoStatus: null,
  loading: false,
  archiving: false,
  progress: '',

  fetchArchives: async (ringId) => {
    set({ loading: true })
    try {
      const data = await api.api.get<{ archives: ArchiveRecord[] }>(
        `/rings/${ringId}/archives`,
      )
      set({ archives: data.archives })
    } finally {
      set({ loading: false })
    }
  },

  fetchQueue: async (ringId) => {
    const data = await api.api.get<{ queue: ArchiveRecord[] }>(
      `/rings/${ringId}/archive-queue`,
    )
    set({ queue: data.queue })
  },

  fetchRepoStatus: async (ringId) => {
    const status = await api.api.get<RepoStatus>(
      `/rings/${ringId}/repo/status`,
    )
    set({ repoStatus: status })
  },

  triggerArchive: async (ringId, content, title, sessionId) => {
    set({ archiving: true, progress: '' })
    try {
      await api.triggerArchiveSSE(
        ringId,
        {
          session_id: sessionId,
          content,
          suggested_title: title,
          node_suggestion: { action: 'create_new', node_title: title },
        },
        (event) => set({ progress: event.message }),
        () => {},
        () => {},
      )
      await get().fetchArchives(ringId)
    } finally {
      set({ archiving: false, progress: '' })
    }
  },

  reviewArchive: async (ringId, archiveId, action) => {
    await api.api.post(`/rings/${ringId}/archives/${archiveId}/review`, {
      action,
    })
    await Promise.all([
      get().fetchArchives(ringId),
      get().fetchQueue(ringId),
    ])
  },

  initRepo: async (ringId) => {
    const status = await api.api.post<RepoStatus>(
      `/rings/${ringId}/repo/init`,
      {},
    )
    set({ repoStatus: status })
  },
}))
