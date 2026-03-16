"use client";

import { useMemo } from "react";
import {
  type ColumnDef,
  type RowSelectionState,
  getCoreRowModel,
  useReactTable,
} from "@tanstack/react-table";
import Link from "next/link";
import { Eye } from "lucide-react";

import type { Report } from "@/types/reports";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Checkbox } from "@/components/ui/checkbox";
import { DataTable } from "@/components/data-table/data-table";
import { DataTableColumnHeader } from "@/components/data-table/data-table-column-header";
import { ResolvedHandle } from "@/components/resolved-handle";
import { formatReasonType } from "@/lib/reason-types";

function StatusBadge({ status }: { status: Report["status"] }) {
  const variants: Record<Report["status"], string> = {
    pending: "bg-yellow-100 text-yellow-800 border-yellow-300",
    in_review: "bg-blue-100 text-blue-800 border-blue-300",
    resolved: "bg-green-100 text-green-800 border-green-300",
    dismissed: "bg-neutral-100 text-neutral-600 border-neutral-300",
  };

  const labels: Record<Report["status"], string> = {
    pending: "Pending",
    in_review: "In Review",
    resolved: "Resolved",
    dismissed: "Dismissed",
  };

  return (
    <Badge variant="outline" className={variants[status]}>
      {labels[status]}
    </Badge>
  );
}

function AutoBadge() {
  return (
    <Badge
      variant="outline"
      className="bg-purple-100 text-purple-800 border-purple-300"
    >
      Auto
    </Badge>
  );
}

function formatAge(dateStr: string): string {
  const now = Date.now();
  const then = new Date(dateStr).getTime();
  const diffMs = now - then;
  const diffMins = Math.floor(diffMs / 60000);
  if (diffMins < 1) return "just now";
  if (diffMins < 60) return `${diffMins}m`;
  const diffHours = Math.floor(diffMins / 60);
  if (diffHours < 24) return `${diffHours}h`;
  const diffDays = Math.floor(diffHours / 24);
  if (diffDays < 30) return `${diffDays}d`;
  const diffMonths = Math.floor(diffDays / 30);
  return `${diffMonths}mo`;
}

interface QueueTableProps {
  reports: Report[];
  rowSelection: RowSelectionState;
  onRowSelectionChange: (selection: RowSelectionState) => void;
}

export function QueueTable({
  reports,
  rowSelection,
  onRowSelectionChange,
}: QueueTableProps) {
  const columns = useMemo<ColumnDef<Report>[]>(
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
        size: 40,
        enableSorting: false,
        enableHiding: false,
      },
      {
        id: "subject",
        accessorFn: (row) => row.subjectUri ?? row.subjectDid ?? "-",
        header: ({ column }) => (
          <DataTableColumnHeader column={column} label="Subject" />
        ),
        cell: ({ row }) => {
          const did = row.original.subjectDid;
          if (did && !row.original.subjectUri) {
            return (
              <ResolvedHandle
                did={did}
                className="text-xs truncate block max-w-xs"
              />
            );
          }
          const val = row.original.subjectUri ?? did ?? "-";
          return (
            <span
              className="font-mono text-xs truncate block max-w-xs"
              title={val}
            >
              {val}
            </span>
          );
        },
      },
      {
        id: "reasonType",
        accessorKey: "reasonType",
        header: ({ column }) => (
          <DataTableColumnHeader column={column} label="Reason" />
        ),
        cell: ({ getValue }) => (
          <span className="text-sm">{formatReasonType(getValue<string>())}</span>
        ),
      },
      {
        id: "status",
        accessorKey: "status",
        header: ({ column }) => (
          <DataTableColumnHeader column={column} label="Status" />
        ),
        cell: ({ getValue }) => (
          <StatusBadge status={getValue<Report["status"]>()} />
        ),
      },
      {
        id: "reportedBy",
        accessorKey: "reportedBy",
        header: ({ column }) => (
          <DataTableColumnHeader column={column} label="Reporter" />
        ),
        cell: ({ getValue }) => (
          <ResolvedHandle
            did={getValue<string>()}
            className="text-xs truncate block max-w-[200px]"
          />
        ),
      },
      {
        id: "priority",
        accessorKey: "priority",
        header: ({ column }) => (
          <DataTableColumnHeader column={column} label="Priority" />
        ),
        cell: ({ getValue }) => (
          <span className="text-sm tabular-nums">{getValue<number>()}</span>
        ),
        size: 80,
      },
      {
        id: "assignedTo",
        accessorKey: "assignedTo",
        header: ({ column }) => (
          <DataTableColumnHeader column={column} label="Assigned To" />
        ),
        cell: ({ getValue }) => {
          const val = getValue<string | null>();
          return val ? (
            <ResolvedHandle
              did={val}
              className="text-xs truncate block max-w-[200px]"
            />
          ) : (
            <span className="text-muted-foreground text-xs">Unassigned</span>
          );
        },
      },
      {
        id: "createdAt",
        accessorKey: "createdAt",
        header: ({ column }) => (
          <DataTableColumnHeader column={column} label="Age" />
        ),
        cell: ({ getValue }) => (
          <span className="text-muted-foreground text-xs whitespace-nowrap">
            {formatAge(getValue<string>())}
          </span>
        ),
        size: 60,
      },
      {
        id: "autoLabeled",
        accessorKey: "autoLabeled",
        header: ({ column }) => (
          <DataTableColumnHeader column={column} label="Auto" />
        ),
        cell: ({ getValue }) => (getValue<boolean>() ? <AutoBadge /> : null),
        size: 60,
      },
      {
        id: "actions",
        header: () => null,
        cell: ({ row }) => (
          <Button variant="ghost" size="sm" className="h-7 gap-1.5" asChild>
            <Link href={`/dashboard/queue/${row.original.id}`}>
              <Eye className="size-3.5" />
            </Link>
          </Button>
        ),
        size: 90,
        enableSorting: false,
        enableHiding: false,
      },
    ],
    [],
  );

  const table = useReactTable({
    data: reports,
    columns,
    state: {
      columnPinning: { left: ["select"], right: ["actions"] },
      rowSelection,
    },
    onRowSelectionChange: (updaterOrValue) => {
      const next =
        typeof updaterOrValue === "function"
          ? updaterOrValue(rowSelection)
          : updaterOrValue;
      onRowSelectionChange(next);
    },
    enableRowSelection: true,
    getCoreRowModel: getCoreRowModel(),
    getRowId: (row) => String(row.id),
  });

  return <DataTable table={table} />;
}
