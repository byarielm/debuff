"use no memo"
"use client"

import { useCallback, useEffect, useState } from "react"

import {
  getReports,
  getReport,
  updateReport,
  assignReport,
  escalateReport,
  addReportNote,
  getLabels,
  getLabelDefinitions,
  applyLabels,
  getModerators,
  accountAction,
} from "@/lib/api"
import type { Report, ReportNote } from "@/types/reports"
import type { Label } from "@/types/labels"
import type { LabelDefinition } from "@/types/definitions"
import type { Moderator } from "@/types/moderators"

interface QueueFilters {
  status?: string
  reason_type?: string
  assigned_to?: string
  sort?: "newest" | "oldest" | "priority"
}

interface UseQueueListReturn {
  reports: Report[]
  loading: boolean
  error: string | null
  cursorStack: string[]
  nextCursor: string | undefined
  filters: QueueFilters
  setFilters: (filters: QueueFilters) => void
  fetchPage: (cursor?: string) => void
  goNext: () => void
  goPrevious: () => void
  refresh: () => void
}

export function useQueueList(): UseQueueListReturn {
  const [reports, setReports] = useState<Report[]>([])
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)
  const [cursorStack, setCursorStack] = useState<string[]>([])
  const [nextCursor, setNextCursor] = useState<string | undefined>()
  const [filters, setFilters] = useState<QueueFilters>({})

  const fetchPage = useCallback(
    async (cursor?: string) => {
      setLoading(true)
      setError(null)
      try {
        const data = await getReports({
          status: filters.status,
          assigned_to: filters.assigned_to,
          cursor,
          limit: 25,
        })
        setReports(data.items)
        setNextCursor(data.cursor ?? undefined)
      } catch (e: unknown) {
        setError(e instanceof Error ? e.message : String(e))
        setReports([])
        setNextCursor(undefined)
      } finally {
        setLoading(false)
      }
    },
    [filters.status, filters.assigned_to],
  )

  useEffect(() => {
    setCursorStack([])
    setNextCursor(undefined)
    fetchPage()
  }, [fetchPage])

  const goNext = useCallback(() => {
    if (!nextCursor) return
    setCursorStack((prev) => [...prev, nextCursor])
    fetchPage(nextCursor)
  }, [nextCursor, fetchPage])

  const goPrevious = useCallback(() => {
    if (cursorStack.length === 0) return
    const stack = [...cursorStack]
    stack.pop()
    const prevCursor = stack.length > 0 ? stack[stack.length - 1] : undefined
    setCursorStack(stack)
    fetchPage(prevCursor)
  }, [cursorStack, fetchPage])

  const refresh = useCallback(() => {
    const currentCursor =
      cursorStack.length > 0 ? cursorStack[cursorStack.length - 1] : undefined
    fetchPage(currentCursor)
  }, [cursorStack, fetchPage])

  return {
    reports,
    loading,
    error,
    cursorStack,
    nextCursor,
    filters,
    setFilters,
    fetchPage,
    goNext,
    goPrevious,
    refresh,
  }
}

interface UseQueueDetailReturn {
  report: Report | null
  notes: ReportNote[]
  labels: Label[]
  definitions: LabelDefinition[]
  moderators: Moderator[]
  loading: boolean
  error: string | null
  refresh: () => void
  updateStatus: (status: string) => Promise<void>
  assign: (did: string | null) => Promise<void>
  escalate: () => Promise<void>
  addNote: (content: string) => Promise<void>
  applyLabelsToSubject: (vals: string[]) => Promise<void>
  performAccountAction: (action: string) => Promise<void>
}

export function useQueueDetail(id: number): UseQueueDetailReturn {
  const [report, setReport] = useState<Report | null>(null)
  const [notes, setNotes] = useState<ReportNote[]>([])
  const [labels, setLabels] = useState<Label[]>([])
  const [definitions, setDefinitions] = useState<LabelDefinition[]>([])
  const [moderators, setModerators] = useState<Moderator[]>([])
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)

  const load = useCallback(async () => {
    setLoading(true)
    setError(null)
    try {
      const [reportData, defsData, modsData] = await Promise.all([
        getReport(id),
        getLabelDefinitions(),
        getModerators(),
      ])
      setReport(reportData)
      setNotes(reportData.notes ?? [])
      setDefinitions(defsData)
      setModerators(modsData)

      // Fetch existing labels for the subject
      const subjectUri = reportData.subjectUri ?? reportData.subjectDid
      if (subjectUri) {
        const existingLabels = await getLabels({ uri: subjectUri })
        setLabels(existingLabels)
      }
    } catch (e: unknown) {
      setError(e instanceof Error ? e.message : String(e))
    } finally {
      setLoading(false)
    }
  }, [id])

  useEffect(() => {
    load()
  }, [load])

  const updateStatus = useCallback(
    async (status: string) => {
      await updateReport(id, { status })
      setReport((prev) => prev ? { ...prev, status: status as Report["status"] } : prev)
    },
    [id],
  )

  const assign = useCallback(
    async (did: string | null) => {
      await assignReport(id, { did })
      setReport((prev) => prev ? { ...prev, assignedTo: did } : prev)
    },
    [id],
  )

  const doEscalate = useCallback(async () => {
    await escalateReport(id)
    setReport((prev) => prev ? { ...prev, priority: prev.priority + 1 } : prev)
  }, [id])

  const addNote = useCallback(
    async (content: string) => {
      const note = await addReportNote(id, { content })
      setNotes((prev) => [...prev, note])
    },
    [id],
  )

  const applyLabelsToSubject = useCallback(
    async (vals: string[]) => {
      if (!report) return
      const uri = report.subjectUri ?? report.subjectDid
      if (!uri) return
      const newLabels = await applyLabels({
        uri,
        cid: report.subjectCid ?? undefined,
        val: vals,
      })
      setLabels((prev) => [...prev, ...newLabels])
    },
    [report],
  )

  const performAccountAction = useCallback(
    async (action: string) => {
      if (!report?.subjectDid) return
      await accountAction(report.subjectDid, { action })
    },
    [report],
  )

  return {
    report,
    notes,
    labels,
    definitions,
    moderators,
    loading,
    error,
    refresh: load,
    updateStatus,
    assign,
    escalate: doEscalate,
    addNote,
    applyLabelsToSubject,
    performAccountAction,
  }
}
