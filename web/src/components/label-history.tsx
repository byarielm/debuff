"use client"

import { cn } from "@/lib/utils"
import { LabelBadge } from "@/components/label-badges"
import type { Label } from "@/types/labels"

interface LabelHistoryProps {
  labels: Label[]
}

function formatTimestamp(cts: string): string {
  const date = new Date(cts)
  return date.toLocaleString(undefined, {
    year: "numeric",
    month: "short",
    day: "numeric",
    hour: "2-digit",
    minute: "2-digit",
  })
}

function truncateDid(did: string): string {
  if (did.length <= 24) return did
  return `${did.slice(0, 16)}...${did.slice(-8)}`
}

export function LabelHistory({ labels }: LabelHistoryProps) {
  if (labels.length === 0) {
    return (
      <p className="text-muted-foreground text-sm py-4">
        No label history for this account.
      </p>
    )
  }

  // Sort by timestamp descending (newest first)
  const sorted = [...labels].sort(
    (a, b) => new Date(b.cts).getTime() - new Date(a.cts).getTime()
  )

  return (
    <div className="relative">
      {/* Vertical timeline line */}
      <div className="absolute left-3 top-2 bottom-2 w-px bg-border" />

      <div className="space-y-4">
        {sorted.map((label, i) => (
          <div key={`${label.src}-${label.val}-${label.cts}-${i}`} className="relative pl-8">
            {/* Timeline dot */}
            <div
              className={cn(
                "absolute left-1.5 top-1.5 size-3 rounded-full border-2 border-background",
                label.neg
                  ? "bg-muted-foreground"
                  : "bg-primary"
              )}
            />

            <div className={cn(
              "rounded-lg border p-3",
              label.neg && "opacity-70"
            )}>
              <div className="flex items-center justify-between gap-2">
                <div className="flex items-center gap-2">
                  <LabelBadge val={label.val} negated={label.neg} />
                  <span className={cn(
                    "text-xs font-medium",
                    label.neg
                      ? "text-muted-foreground"
                      : "text-foreground"
                  )}>
                    {label.neg ? "negated" : "applied"}
                  </span>
                </div>
                <time className="text-muted-foreground text-xs">
                  {formatTimestamp(label.cts)}
                </time>
              </div>

              <div className="mt-1.5 text-muted-foreground text-xs">
                Signed by{" "}
                <span className="font-mono" title={label.src}>
                  {truncateDid(label.src)}
                </span>
              </div>

              {label.exp && (
                <div className="mt-1 text-muted-foreground text-xs">
                  Expires: {formatTimestamp(label.exp)}
                </div>
              )}
            </div>
          </div>
        ))}
      </div>
    </div>
  )
}
