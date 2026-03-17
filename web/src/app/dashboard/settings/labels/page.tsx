"use client"

import { useCallback, useEffect, useState } from "react"
import type { RowSelectionState } from "@tanstack/react-table"
import Link from "next/link"
import { Trash2 } from "lucide-react"

import { useCurrentUser } from "@/hooks/use-current-user"
import { getLabelDefinitions, deleteLabelDefinition } from "@/lib/api"
import type { LabelDefinition } from "@/types/definitions"
import { SiteHeader } from "@/components/site-header"
import { LabelsTable } from "@/components/labels-table"
import { Button } from "@/components/ui/button"
import {
  Dialog,
  DialogClose,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog"
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu"

export default function LabelsSettingsPage() {
  const { isAdmin } = useCurrentUser()
  const [definitions, setDefinitions] = useState<LabelDefinition[]>([])
  const [error, setError] = useState<string | null>(null)
  const [rowSelection, setRowSelection] = useState<RowSelectionState>({})
  const [bulkDeleteOpen, setBulkDeleteOpen] = useState(false)
  const [deleting, setDeleting] = useState(false)

  const selectedCount = Object.keys(rowSelection).length

  const load = useCallback(() => {
    getLabelDefinitions()
      .then(setDefinitions)
      .catch((e) => setError(e instanceof Error ? e.message : String(e)))
  }, [])

  useEffect(() => {
    load()
  }, [load])

  const handleDelete = useCallback(
    async (id: number) => {
      await deleteLabelDefinition(id)
      load()
    },
    [load],
  )

  const handleBulkDelete = useCallback(async () => {
    setDeleting(true)
    try {
      const ids = Object.keys(rowSelection).map(Number)
      await Promise.all(ids.map((id) => deleteLabelDefinition(id)))
      setRowSelection({})
      setBulkDeleteOpen(false)
      load()
    } catch (e: unknown) {
      setError(e instanceof Error ? e.message : String(e))
    } finally {
      setDeleting(false)
    }
  }, [rowSelection, load])

  return (
    <>
      <SiteHeader title="Label Definitions" />
      <div className="flex flex-1 flex-col gap-4 overflow-hidden p-4 md:p-6">
        {error && <p className="text-destructive text-sm">{error}</p>}

        <LabelsTable
          definitions={definitions}
          isAdmin={isAdmin}
          rowSelection={rowSelection}
          onRowSelectionChange={setRowSelection}
          onDelete={handleDelete}
        >
          <div className="flex w-full items-center justify-between gap-2 p-1">
            <div>
              <h2 className="text-lg font-semibold">Label Definitions</h2>
              <p className="text-muted-foreground text-sm">
                Manage the label definitions published by your labeler.
              </p>
            </div>
            <div className="flex items-center gap-2">
              {isAdmin && (
                <DropdownMenu>
                  <DropdownMenuTrigger asChild>
                    <Button
                      variant="outline"
                      size="sm"
                      className="h-8"
                      disabled={selectedCount === 0}
                    >
                      Actions
                    </Button>
                  </DropdownMenuTrigger>
                  <DropdownMenuContent align="end">
                    <DropdownMenuItem
                      className="text-destructive focus:text-destructive"
                      onClick={() => setBulkDeleteOpen(true)}
                    >
                      <Trash2 className="size-4" />
                      Delete Selected
                    </DropdownMenuItem>
                  </DropdownMenuContent>
                </DropdownMenu>
              )}
              {isAdmin && (
                <Button asChild>
                  <Link href="/dashboard/settings/labels/new">Add Label</Link>
                </Button>
              )}
            </div>
          </div>
        </LabelsTable>
      </div>

      <Dialog open={bulkDeleteOpen} onOpenChange={setBulkDeleteOpen}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>Delete {selectedCount} label definition(s)?</DialogTitle>
            <DialogDescription>
              This will permanently remove the selected label definitions. Any existing
              labels using these definitions will remain but the definitions will no
              longer be published. This action cannot be undone.
            </DialogDescription>
          </DialogHeader>
          <DialogFooter>
            <DialogClose asChild>
              <Button variant="outline" disabled={deleting}>
                Cancel
              </Button>
            </DialogClose>
            <Button
              variant="destructive"
              disabled={deleting}
              onClick={handleBulkDelete}
            >
              {deleting ? "Deleting..." : "Delete"}
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </>
  )
}
