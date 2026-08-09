"use client"

import { useCallback, useState } from "react"

import type { Report } from "@/types/reports"
import type { LabelDefinition } from "@/types/definitions"
import type { Moderator } from "@/types/moderators"
import { Button } from "@/components/ui/button"
import {
  Card,
  CardContent,
  CardHeader,
  CardTitle,
} from "@/components/ui/card"
import {
  Dialog,
  DialogClose,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
  DialogTrigger,
} from "@/components/ui/dialog"
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select"
import { Textarea } from "@/components/ui/textarea"
import { Badge } from "@/components/ui/badge"
import { Separator } from "@/components/ui/separator"
import { ResolvedHandle } from "@/components/resolved-handle"

interface ActionPanelProps {
  report: Report
  definitions: LabelDefinition[]
  moderators: Moderator[]
  onUpdateStatus: (status: string) => Promise<void>
  onAssign: (did: string | null) => Promise<void>
  onEscalate: () => Promise<void>
  onApplyLabels: (vals: string[]) => Promise<void>
  onAddNote: (content: string) => Promise<void>
  onAccountAction: (action: string) => Promise<void>
}

export function ActionPanel({
  report,
  definitions,
  moderators,
  onUpdateStatus,
  onAssign,
  onEscalate,
  onApplyLabels,
  onAddNote,
  onAccountAction,
}: ActionPanelProps) {
  const [selectedLabels, setSelectedLabels] = useState<string[]>([])
  const [applyingLabels, setApplyingLabels] = useState(false)
  const [applyError, setApplyError] = useState<string | null>(null)

  const [dismissNote, setDismissNote] = useState("")
  const [statusLoading, setStatusLoading] = useState(false)
  const [escalating, setEscalating] = useState(false)
  const [accountActionLoading, setAccountActionLoading] = useState(false)

  const handleApplyLabels = useCallback(async () => {
    if (selectedLabels.length === 0) return
    setApplyingLabels(true)
    setApplyError(null)
    try {
      await onApplyLabels(selectedLabels)
      setSelectedLabels([])
    } catch (error) {
      setApplyError(
        error instanceof Error ? error.message : "Failed to apply labels.",
      )
    } finally {
      setApplyingLabels(false)
    }
  }, [selectedLabels, onApplyLabels])

  const handleAcknowledge = useCallback(async () => {
    setStatusLoading(true)
    try {
      await onUpdateStatus("resolved")
    } finally {
      setStatusLoading(false)
    }
  }, [onUpdateStatus])

  const handleDismiss = useCallback(async () => {
    setStatusLoading(true)
    try {
      if (dismissNote.trim()) {
        await onAddNote(dismissNote.trim())
      }
      await onUpdateStatus("dismissed")
      setDismissNote("")
    } finally {
      setStatusLoading(false)
    }
  }, [onUpdateStatus, onAddNote, dismissNote])

  const handleEscalate = useCallback(async () => {
    setEscalating(true)
    try {
      await onEscalate()
    } finally {
      setEscalating(false)
    }
  }, [onEscalate])

  const handleAccountAction = useCallback(
    async (action: string) => {
      setAccountActionLoading(true)
      try {
        await onAccountAction(action)
      } finally {
        setAccountActionLoading(false)
      }
    },
    [onAccountAction],
  )

  const toggleLabel = (identifier: string) => {
    setSelectedLabels((prev) =>
      prev.includes(identifier)
        ? prev.filter((l) => l !== identifier)
        : [...prev, identifier],
    )
  }

  const isResolved = report.status === "resolved" || report.status === "dismissed"

  return (
    <div className="flex flex-col gap-4">
      {/* Label Application */}
      <Card>
        <CardHeader>
          <CardTitle className="text-base">Apply Labels</CardTitle>
        </CardHeader>
        <CardContent className="flex flex-col gap-3">
          {definitions.length > 0 ? (
            <div className="flex flex-wrap gap-1.5">
              {definitions.map((def) => (
                <Badge
                  key={def.id}
                  variant={selectedLabels.includes(def.identifier) ? "default" : "outline"}
                  className="cursor-pointer select-none"
                  onClick={() => toggleLabel(def.identifier)}
                >
                  {def.identifier}
                </Badge>
              ))}
            </div>
          ) : (
            <p className="text-muted-foreground text-sm">No label definitions configured.</p>
          )}

          <Button
            size="sm"
            disabled={selectedLabels.length === 0 || applyingLabels}
            onClick={handleApplyLabels}
          >
            {applyingLabels ? "Applying..." : `Apply ${selectedLabels.length} Label${selectedLabels.length !== 1 ? "s" : ""}`}
          </Button>
          {applyError && (
            <p className="text-destructive text-sm" role="alert">
              {applyError}
            </p>
          )}
        </CardContent>
      </Card>

      {/* Status Actions */}
      <Card>
        <CardHeader>
          <CardTitle className="text-base">Actions</CardTitle>
        </CardHeader>
        <CardContent className="flex flex-col gap-3">
          <div className="flex flex-wrap gap-2">
            <Button
              size="sm"
              disabled={isResolved || statusLoading}
              onClick={handleAcknowledge}
            >
              {statusLoading ? "..." : "Acknowledge"}
            </Button>

            <Dialog>
              <DialogTrigger asChild>
                <Button
                  size="sm"
                  variant="secondary"
                  disabled={isResolved || statusLoading}
                >
                  Dismiss
                </Button>
              </DialogTrigger>
              <DialogContent>
                <DialogHeader>
                  <DialogTitle>Dismiss Report</DialogTitle>
                  <DialogDescription>
                    Optionally add a note explaining why this report is being dismissed.
                  </DialogDescription>
                </DialogHeader>
                <Textarea
                  value={dismissNote}
                  onChange={(e) => setDismissNote(e.target.value)}
                  placeholder="Optional note..."
                  rows={3}
                />
                <DialogFooter>
                  <DialogClose asChild>
                    <Button variant="outline">Cancel</Button>
                  </DialogClose>
                  <DialogClose asChild>
                    <Button
                      variant="secondary"
                      disabled={statusLoading}
                      onClick={handleDismiss}
                    >
                      Dismiss Report
                    </Button>
                  </DialogClose>
                </DialogFooter>
              </DialogContent>
            </Dialog>

            <Button
              size="sm"
              variant="outline"
              disabled={isResolved || escalating}
              onClick={handleEscalate}
            >
              {escalating ? "Escalating..." : "Escalate"}
            </Button>
          </div>
        </CardContent>
      </Card>

      {/* Assign Moderator */}
      <Card>
        <CardHeader>
          <CardTitle className="text-base">Assign Moderator</CardTitle>
        </CardHeader>
        <CardContent>
          <Select
            value={report.assignedTo ?? "unassigned"}
            onValueChange={(value) =>
              onAssign(value === "unassigned" ? null : value)
            }
          >
            <SelectTrigger className="w-full">
              <SelectValue placeholder="Select moderator" />
            </SelectTrigger>
            <SelectContent>
              <SelectItem value="unassigned">Unassigned</SelectItem>
              {moderators.map((mod) => (
                <SelectItem key={mod.did} value={mod.did}>
                  <ResolvedHandle did={mod.did} className="text-xs" />
                </SelectItem>
              ))}
            </SelectContent>
          </Select>
        </CardContent>
      </Card>

      {/* Account Actions */}
      {report.subjectDid && (
        <Card>
          <CardHeader>
            <CardTitle className="text-base">Account Actions</CardTitle>
          </CardHeader>
          <CardContent className="flex flex-col gap-3">
            <p className="text-muted-foreground text-xs font-mono break-all">
              {report.subjectDid}
            </p>
            <Separator />
            <div className="flex flex-wrap gap-2">
              <Dialog>
                <DialogTrigger asChild>
                  <Button
                    size="sm"
                    variant="destructive"
                    disabled={accountActionLoading}
                  >
                    Suspend Account
                  </Button>
                </DialogTrigger>
                <DialogContent>
                  <DialogHeader>
                    <DialogTitle>Suspend Account?</DialogTitle>
                    <DialogDescription>
                      This will apply a !suspend label to the account. The account will be
                      restricted from participating. This action can be reversed.
                    </DialogDescription>
                  </DialogHeader>
                  <DialogFooter>
                    <DialogClose asChild>
                      <Button variant="outline">Cancel</Button>
                    </DialogClose>
                    <DialogClose asChild>
                      <Button
                        variant="destructive"
                        disabled={accountActionLoading}
                        onClick={() => handleAccountAction("!suspend")}
                      >
                        Suspend
                      </Button>
                    </DialogClose>
                  </DialogFooter>
                </DialogContent>
              </Dialog>

              <Dialog>
                <DialogTrigger asChild>
                  <Button
                    size="sm"
                    variant="destructive"
                    disabled={accountActionLoading}
                  >
                    Takedown Account
                  </Button>
                </DialogTrigger>
                <DialogContent>
                  <DialogHeader>
                    <DialogTitle>Takedown Account?</DialogTitle>
                    <DialogDescription>
                      This will apply a !takedown label to the account. The account and all
                      its content will be hidden. This is a severe action.
                    </DialogDescription>
                  </DialogHeader>
                  <DialogFooter>
                    <DialogClose asChild>
                      <Button variant="outline">Cancel</Button>
                    </DialogClose>
                    <DialogClose asChild>
                      <Button
                        variant="destructive"
                        disabled={accountActionLoading}
                        onClick={() => handleAccountAction("!takedown")}
                      >
                        Takedown
                      </Button>
                    </DialogClose>
                  </DialogFooter>
                </DialogContent>
              </Dialog>
            </div>
          </CardContent>
        </Card>
      )}
    </div>
  )
}
