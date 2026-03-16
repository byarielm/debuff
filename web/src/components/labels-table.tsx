"use no memo"

import { useMemo } from "react"
import { type ColumnDef, getCoreRowModel, useReactTable } from "@tanstack/react-table"
import Link from "next/link"
import { Pencil } from "lucide-react"

import type { LabelDefinition } from "@/types/definitions"
import { Badge } from "@/components/ui/badge"
import { Button } from "@/components/ui/button"
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
}

export function LabelsTable({ definitions, isAdmin }: LabelsTableProps) {
  const columns = useMemo<ColumnDef<LabelDefinition>[]>(
    () => [
      {
        id: "identifier",
        accessorKey: "identifier",
        header: ({ column }) => (
          <DataTableColumnHeader column={column} label="Identifier" />
        ),
        cell: ({ getValue, row }) => (
          <div className="flex items-center gap-2">
            <code className="text-sm">{getValue<string>()}</code>
            {row.original.builtin && (
              <Badge variant="secondary" className="text-xs">
                Built-in
              </Badge>
            )}
          </div>
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
        cell: ({ row }) =>
          !row.original.builtin ? (
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
          ) : null,
        size: 48,
        enableSorting: false,
        enableHiding: false,
      },
    ],
    [isAdmin],
  )

  const table = useReactTable({
    data: definitions,
    columns,
    state: {
      columnPinning: { right: ["actions"] },
    },
    getCoreRowModel: getCoreRowModel(),
    getRowId: (row) => String(row.id),
  })

  return <DataTable table={table} />
}
