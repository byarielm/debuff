"use no memo"

import { useMemo, useState } from "react"
import {
  type ColumnDef,
  type RowSelectionState,
  getCoreRowModel,
  useReactTable,
} from "@tanstack/react-table"
import Link from "next/link"
import { Pencil, Trash2 } from "lucide-react"

import type { LabelDefinition } from "@/types/definitions"
import { Badge } from "@/components/ui/badge"
import { Button } from "@/components/ui/button"
import { Checkbox } from "@/components/ui/checkbox"
import {
  Dialog,
  DialogClose,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog"
import { DataTable } from "@/components/data-table/data-table"
import { DataTableColumnHeader } from "@/components/data-table/data-table-column-header"

function severityBadgeClass(severity: string) {
  switch (severity) {
    case "alert":
      return "bg-red-100 text-red-800 border-red-300"
    case "inform":
      return "bg-blue-100 text-blue-800 border-blue-300"
    default:
      return "bg-neutral-100 text-neutral-600 border-neutral-300"
  }
}

interface LabelsTableProps {
  definitions: LabelDefinition[]
  isAdmin: boolean
  rowSelection: RowSelectionState
  onRowSelectionChange: (state: RowSelectionState) => void
  onDelete?: (id: number) => Promise<void>
  children?: React.ReactNode
}

export function LabelsTable({
  definitions,
  isAdmin,
  rowSelection,
  onRowSelectionChange,
  onDelete,
  children,
}: LabelsTableProps) {
  const [deleteTarget, setDeleteTarget] = useState<LabelDefinition | null>(null)
  const [deleting, setDeleting] = useState(false)

  async function handleDelete() {
    if (!deleteTarget || !onDelete) return
    setDeleting(true)
    try {
      await onDelete(deleteTarget.id)
      setDeleteTarget(null)
    } finally {
      setDeleting(false)
    }
  }

  const columns = useMemo<ColumnDef<LabelDefinition>[]>(
    () => [
      {
        id: "select",
        header: ({ table }) => (
          <Checkbox
            checked={
              table.getIsAllPageRowsSelected() ||
              (table.getIsSomePageRowsSelected() && "indeterminate")
            }
            onCheckedChange={(value) =>
              table.toggleAllPageRowsSelected(!!value)
            }
            aria-label="Select all"
          />
        ),
        cell: ({ row }) => (
          <Checkbox
            checked={row.getIsSelected()}
            onCheckedChange={(value) => row.toggleSelected(!!value)}
            aria-label="Select row"
          />
        ),
        size: 32,
        minSize: 32,
        maxSize: 32,
        enableSorting: false,
        enableHiding: false,
        enableResizing: false,
      },
      {
        id: "identifier",
        accessorKey: "identifier",
        header: ({ column }) => (
          <DataTableColumnHeader column={column} label="Identifier" />
        ),
        cell: ({ getValue }) => (
          <code className="text-sm">{getValue<string>()}</code>
        ),
      },
      {
        id: "severity",
        accessorKey: "severity",
        header: ({ column }) => (
          <DataTableColumnHeader column={column} label="Severity" />
        ),
        cell: ({ getValue }) => (
          <Badge variant="outline" className={severityBadgeClass(getValue<string>())}>
            {getValue<string>()}
          </Badge>
        ),
        size: 100,
      },
      {
        id: "blurs",
        accessorKey: "blurs",
        header: ({ column }) => (
          <DataTableColumnHeader column={column} label="Blurs" />
        ),
        cell: ({ getValue }) => (
          <span className="text-sm">{getValue<string>()}</span>
        ),
        size: 100,
      },
      {
        id: "default_setting",
        accessorKey: "default_setting",
        header: ({ column }) => (
          <DataTableColumnHeader column={column} label="Default" />
        ),
        cell: ({ getValue }) => (
          <span className="text-sm">{getValue<string>()}</span>
        ),
        size: 100,
      },
      {
        id: "adult_only",
        accessorKey: "adult_only",
        header: ({ column }) => (
          <DataTableColumnHeader column={column} label="Adult Only" />
        ),
        cell: ({ getValue }) => (
          <span className="text-sm">{getValue<boolean>() ? "Yes" : "No"}</span>
        ),
        size: 90,
      },
      {
        id: "actions",
        header: () => null,
        cell: ({ row }) => (
          <div className="flex items-center gap-1">
            <Button
              variant="ghost"
              size="icon"
              className="size-8"
              title="Edit definition"
              aria-label="Edit definition"
              asChild
            >
              <Link href={`/dashboard/settings/labels/${row.original.id}`}>
                <Pencil className="size-4" />
              </Link>
            </Button>
            {isAdmin && onDelete && (
              <Button
                variant="ghost"
                size="icon"
                className="size-8 text-destructive hover:text-destructive"
                title="Delete definition"
                aria-label="Delete definition"
                onClick={() => setDeleteTarget(row.original)}
              >
                <Trash2 className="size-4" />
              </Button>
            )}
          </div>
        ),
        size: 80,
        minSize: 80,
        maxSize: 80,
        enableSorting: false,
        enableHiding: false,
        enableResizing: false,
      },
    ],
    [isAdmin, onDelete],
  )

  const table = useReactTable({
    data: definitions,
    columns,
    state: {
      columnPinning: { left: ["select"], right: ["actions"] },
      rowSelection,
    },
    onRowSelectionChange: (updater) => {
      const newState = typeof updater === "function" ? updater(rowSelection) : updater
      onRowSelectionChange(newState)
    },
    enableRowSelection: true,
    getCoreRowModel: getCoreRowModel(),
    getRowId: (row) => String(row.id),
  })

  return (
    <>
      <DataTable table={table}>{children}</DataTable>
      <Dialog open={deleteTarget !== null} onOpenChange={(open) => !open && setDeleteTarget(null)}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>Delete label definition?</DialogTitle>
            <DialogDescription>
              This will permanently remove <code>{deleteTarget?.identifier}</code>. Any existing
              labels using this definition will remain but the definition will no
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
              onClick={handleDelete}
            >
              {deleting ? "Deleting..." : "Delete"}
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </>
  )
}
