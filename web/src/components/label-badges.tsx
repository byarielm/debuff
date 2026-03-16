"use client"

import { Badge } from "@/components/ui/badge"
import {
  Tooltip,
  TooltipContent,
  TooltipProvider,
  TooltipTrigger,
} from "@/components/ui/tooltip"
import { cn } from "@/lib/utils"
import type { Label } from "@/types/labels"

const CONTENT_WARNING_LABELS = new Set([
  "nudity",
  "sexual",
  "porn",
  "gore",
  "nsfl",
  "graphic-media",
  "violence",
])

const MODERATION_LABELS = new Set([
  "spam",
  "impersonation",
  "doxxing",
])

const SYSTEM_LABELS = new Set([
  "!hide",
  "!warn",
  "!suspend",
  "!takedown",
])

type LabelCategory = "content-warning" | "moderation" | "system" | "other"

function getLabelCategory(val: string): LabelCategory {
  if (CONTENT_WARNING_LABELS.has(val)) return "content-warning"
  if (MODERATION_LABELS.has(val)) return "moderation"
  if (SYSTEM_LABELS.has(val)) return "system"
  return "other"
}

function getLabelVariant(category: LabelCategory): "destructive" | "outline" | "secondary" {
  switch (category) {
    case "content-warning":
      return "destructive"
    case "system":
      return "secondary"
    default:
      return "outline"
  }
}

function getLabelClassName(category: LabelCategory, negated?: boolean): string {
  const base = negated ? "opacity-50 line-through" : ""

  switch (category) {
    case "moderation":
      return cn(base, "border-amber-500 bg-amber-500 text-white hover:bg-amber-600")
    case "system":
      return cn(base, "border-purple-500 bg-purple-500 text-white hover:bg-purple-600")
    default:
      return base
  }
}

interface LabelBadgesProps {
  labels: Label[]
  showSource?: boolean
}

export function LabelBadges({ labels, showSource = true }: LabelBadgesProps) {
  if (labels.length === 0) return null

  return (
    <TooltipProvider>
      <div className="flex flex-wrap gap-1">
        {labels.map((label, i) => {
          const category = getLabelCategory(label.val)
          const variant = getLabelVariant(category)
          const className = getLabelClassName(category, label.neg)

          if (!showSource) {
            return (
              <Badge
                key={`${label.src}-${label.val}-${i}`}
                variant={variant}
                className={className}
              >
                {label.val}
              </Badge>
            )
          }

          return (
            <Tooltip key={`${label.src}-${label.val}-${i}`}>
              <TooltipTrigger asChild>
                <Badge variant={variant} className={className}>
                  {label.val}
                </Badge>
              </TooltipTrigger>
              <TooltipContent>
                <p className="text-xs">Source: {label.src}</p>
              </TooltipContent>
            </Tooltip>
          )
        })}
      </div>
    </TooltipProvider>
  )
}

interface LabelBadgeProps {
  val: string
  negated?: boolean
  src?: string
}

export function LabelBadge({ val, negated, src }: LabelBadgeProps) {
  const category = getLabelCategory(val)
  const variant = getLabelVariant(category)
  const className = getLabelClassName(category, negated)

  if (!src) {
    return (
      <Badge variant={variant} className={className}>
        {val}
      </Badge>
    )
  }

  return (
    <TooltipProvider>
      <Tooltip>
        <TooltipTrigger asChild>
          <Badge variant={variant} className={className}>
            {val}
          </Badge>
        </TooltipTrigger>
        <TooltipContent>
          <p className="text-xs">Source: {src}</p>
        </TooltipContent>
      </Tooltip>
    </TooltipProvider>
  )
}
