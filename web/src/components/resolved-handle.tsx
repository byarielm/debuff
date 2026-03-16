"use client"

import { useResolveHandle } from "@/hooks/use-resolve-handle"
import { cn } from "@/lib/utils"

interface ResolvedHandleProps {
  did: string
  className?: string
}

export function ResolvedHandle({ did, className }: ResolvedHandleProps) {
  const handle = useResolveHandle(did)

  if (!handle) {
    return (
      <span className={cn("font-mono", className)}>{did}</span>
    )
  }

  return (
    <span className={cn("flex flex-col", className)}>
      <span>@{handle}</span>
      <span className="text-muted-foreground font-mono text-[0.65rem] leading-tight">{did}</span>
    </span>
  )
}
