"use client"

import { useCallback, useEffect, useState } from "react"
import type { RowSelectionState } from "@tanstack/react-table"
import { ChevronLeft, ChevronRight } from "lucide-react"

import { updateReport, assignReport, getModerators } from "@/lib/api"
import { useAuth } from "@/lib/auth-context"
import type { Moderator } from "@/types/moderators"
import { useResolveHandles } from "@/hooks/use-resolve-handle"
import { useQueueList } from "@/hooks/use-queue"
import { SiteHeader } from "@/components/site-header"
import { QueueTable } from "@/components/queue-table"
import { Button } from "@/components/ui/button"
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select"
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu"

const STATUS_OPTIONS = [
  { value: "all", label: "All Statuses" },
  { value: "pending", label: "Pending" },
  { value: "in_review", label: "In Review" },
  { value: "resolved", label: "Resolved" },
  { value: "dismissed", label: "Dismissed" },
]

export default function QueuePage() {
  const {
    reports,
    loading,
    error,
    cursorStack,
    nextCursor,
    filters,
    setFilters,
    goNext,
    goPrevious,
    refresh,
  } = useQueueList()

  const { did } = useAuth()
  const [moderators, setModerators] = useState<Moderator[]>([])
  const modHandles = useResolveHandles(moderators.map((m) => m.did))

  useEffect(() => {
    getModerators().then(setModerators).catch(() => {})
  }, [])

  const [rowSelection, setRowSelection] = useState<RowSelectionState>({})
  const [bulkLoading, setBulkLoading] = useState(false)

  const selectedCount = Object.keys(rowSelection).length

  const handleBulkDismiss = useCallback(async () => {
    setBulkLoading(true)
    try {
      const ids = Object.keys(rowSelection).map(Number)
      await Promise.all(
        ids.map((id) => updateReport(id, { status: "dismissed" })),
      )
      setRowSelection({})
      refresh()
    } finally {
      setBulkLoading(false)
    }
  }, [rowSelection, refresh])

  const handleBulkAssign = useCallback(
    async (did: string) => {
      setBulkLoading(true)
      try {
        const ids = Object.keys(rowSelection).map(Number)
        await Promise.all(
          ids.map((id) => assignReport(id, { did })),
        )
        setRowSelection({})
        refresh()
      } finally {
        setBulkLoading(false)
      }
    },
    [rowSelection, refresh],
  )

  return (
    <>
      <SiteHeader title="Moderation Queue" />
      <div className="flex flex-1 flex-col gap-4 overflow-hidden p-4 md:p-6">
        {error && <p className="text-destructive text-sm">{error}</p>}

        {/* Toolbar */}
        <div className="flex w-full items-center justify-between gap-2">
          <div className="flex items-center gap-2">
            <Select
              value={filters.status ?? "all"}
              onValueChange={(value) =>
                setFilters({
                  ...filters,
                  status: value === "all" ? undefined : value,
                })
              }
            >
              <SelectTrigger className="h-8 w-44 text-sm">
                <SelectValue placeholder="Filter by status" />
              </SelectTrigger>
              <SelectContent>
                {STATUS_OPTIONS.map((opt) => (
                  <SelectItem key={opt.value} value={opt.value}>
                    {opt.label}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>

            <Select
              value={
                filters.assigned_to === undefined
                  ? "all"
                  : filters.assigned_to === did
                    ? "__mine__"
                    : filters.assigned_to
              }
              onValueChange={(value) =>
                setFilters({
                  ...filters,
                  assigned_to: value === "all"
                    ? undefined
                    : value === "__mine__"
                      ? did ?? undefined
                      : value,
                })
              }
            >
              <SelectTrigger className="h-8 w-48 text-sm">
                <SelectValue placeholder="Filter by assignee" />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="all">All Assignees</SelectItem>
                <SelectItem value="__mine__">Assigned to Me</SelectItem>
                <SelectItem value="__unassigned__">Unassigned</SelectItem>
                {moderators.filter((mod) => mod.did !== did).map((mod) => {
                  const handle = modHandles.get(mod.did)
                  return (
                    <SelectItem key={mod.did} value={mod.did}>
                      {handle ? (
                        <span className="flex flex-col">
                          <span>@{handle}</span>
                          <span className="text-muted-foreground font-mono text-[0.65rem] leading-tight">{mod.did}</span>
                        </span>
                      ) : (
                        <span className="font-mono">{mod.did}</span>
                      )}
                    </SelectItem>
                  )
                })}
              </SelectContent>
            </Select>
          </div>

          <div className="flex items-center gap-2">
            {selectedCount > 0 && (
              <DropdownMenu>
                <DropdownMenuTrigger asChild>
                  <Button
                    variant="outline"
                    size="sm"
                    className="h-8"
                    disabled={bulkLoading}
                  >
                    Actions ({selectedCount})
                  </Button>
                </DropdownMenuTrigger>
                <DropdownMenuContent align="end">
                  <DropdownMenuItem onClick={handleBulkDismiss}>
                    Dismiss Selected
                  </DropdownMenuItem>
                  <DropdownMenuItem
                    onClick={() => did && handleBulkAssign(did)}
                    disabled={!did}
                  >
                    Assign to Me
                  </DropdownMenuItem>
                </DropdownMenuContent>
              </DropdownMenu>
            )}
          </div>
        </div>

        {/* Table */}
        <QueueTable
          reports={reports}
          rowSelection={rowSelection}
          onRowSelectionChange={setRowSelection}
        />

        {/* Cursor pagination */}
        <div className="flex w-full items-center justify-between gap-4 p-1">
          <p className="text-muted-foreground flex-1 whitespace-nowrap text-sm">
            {loading ? "Loading..." : `${reports.length} report(s) on this page.`}
          </p>
          <div className="flex items-center space-x-2">
            <Button
              aria-label="Go to previous page"
              title="Previous page"
              variant="outline"
              size="icon"
              className="size-8"
              disabled={cursorStack.length === 0 || loading}
              onClick={goPrevious}
            >
              <ChevronLeft />
            </Button>
            <Button
              aria-label="Go to next page"
              title="Next page"
              variant="outline"
              size="icon"
              className="size-8"
              disabled={!nextCursor || loading}
              onClick={goNext}
            >
              <ChevronRight />
            </Button>
          </div>
        </div>
      </div>
    </>
  )
}
