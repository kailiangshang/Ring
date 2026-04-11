import { create } from 'zustand'
import * as api from '../api/client'
import type { PrListItem, CommitLogEntry, ArchiveQueueResponse } from '../types'

interface GitState {
  prs: PrListItem[]
  current_pr: PrListItem | null
  commit_log: CommitLogEntry[]
  archive_queue: ArchiveQueueResponse | null
  loading: boolean
  error: string | null

  load_prs: (ring_id: string, state?: string) => Promise<void>
  load_pr_detail: (ring_id: string, pr_id: number) => Promise<void>
  merge_pr: (ring_id: string, pr_id: number) => Promise<void>
  reject_pr: (ring_id: string, pr_id: number) => Promise<void>
  load_commit_log: (ring_id: string, limit?: number) => Promise<void>
  load_archive_queue: (ring_id: string) => Promise<void>
  clear_error: () => void
}

export const useGitStore = create<GitState>((set) => ({
  prs: [],
  current_pr: null,
  commit_log: [],
  archive_queue: null,
  loading: false,
  error: null,

  load_prs: async (ring_id, state) => {
    set({ loading: true, error: null })
    try {
      const prs = await api.list_prs(ring_id, state)
      set({ prs, loading: false })
    } catch (e) {
      set({ error: (e as Error).message, loading: false })
    }
  },

  load_pr_detail: async (ring_id, pr_id) => {
    set({ loading: true, error: null })
    try {
      const current_pr = await api.get_pr_diff(ring_id, pr_id)
      set({ current_pr, loading: false })
    } catch (e) {
      set({ error: (e as Error).message, loading: false })
    }
  },

  merge_pr: async (ring_id, pr_id) => {
    set({ loading: true, error: null })
    try {
      await api.merge_pr(ring_id, pr_id)
      const prs = await api.list_prs(ring_id)
      set({ prs, loading: false })
    } catch (e) {
      set({ error: (e as Error).message, loading: false })
    }
  },

  reject_pr: async (ring_id, pr_id) => {
    set({ loading: true, error: null })
    try {
      await api.reject_pr(ring_id, pr_id)
      const prs = await api.list_prs(ring_id)
      set({ prs, loading: false })
    } catch (e) {
      set({ error: (e as Error).message, loading: false })
    }
  },

  load_commit_log: async (ring_id, limit) => {
    set({ loading: true, error: null })
    try {
      const data = await api.get_commit_log(ring_id, limit)
      set({ commit_log: data, loading: false })
    } catch (e) {
      set({ error: (e as Error).message, loading: false })
    }
  },

  load_archive_queue: async (ring_id) => {
    set({ loading: true, error: null })
    try {
      const archive_queue = await api.get_archive_queue(ring_id)
      set({ archive_queue, loading: false })
    } catch (e) {
      set({ error: (e as Error).message, loading: false })
    }
  },

  clear_error: () => set({ error: null }),
}))
