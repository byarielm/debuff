"use client"

import { useCallback, useEffect, useState } from "react"
import { useParams } from "next/navigation"
import {
  AlertTriangle,
  Ban,
  EyeOff,
  ShieldOff,
  Loader2,
} from "lucide-react"

import { getLabels, accountAction } from "@/lib/api"
import { useResolveHandle } from "@/hooks/use-resolve-handle"
import type { Label } from "@/types/labels"

import { SiteHeader } from "@/components/site-header"
import { LabelBadges } from "@/components/label-badges"
import { LabelHistory } from "@/components/label-history"

import { Button } from "@/components/ui/button"
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card"
import { Separator } from "@/components/ui/separator"
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog"

const ACCOUNT_ACTIONS = [
  {
    label: "Suspend",
    val: "!suspend",
    icon: Ban,
    description: "Prevent this account from interacting with the service. Can be reversed.",
    variant: "destructive" as const,
  },
  {
    label: "Takedown",
    val: "!takedown",
    icon: ShieldOff,
    description: "Remove this account and its content from the service. This is a severe action.",
    variant: "destructive" as const,
  },
  {
    label: "Hide",
    val: "!hide",
    icon: EyeOff,
    description: "Hide this account from discovery and feeds. Content is still accessible via direct link.",
    variant: "outline" as const,
  },
] as const

export default function AccountPage() {
  const params = useParams<{ did: string }>()
  const did = decodeURIComponent(params.did)
  const handle = useResolveHandle(did)
  const [labels, setLabels] = useState<Label[]>([])
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)

  const [confirmAction, setConfirmAction] = useState<typeof ACCOUNT_ACTIONS[number] | null>(null)
  const [actionLoading, setActionLoading] = useState(false)

  const loadData = useCallback(async () => {
    setLoading(true)
    setError(null)
    try {
      const labelsResult = await getLabels({ uri: did })
      setLabels(labelsResult ?? [])
    } catch (e) {
      setError(e instanceof Error ? e.message : "Failed to load account data")
    } finally {
      setLoading(false)
    }
  }, [did])

  useEffect(() => {
    loadData()
  }, [loadData])

  const activeLabels = labels.filter((l) => !l.neg)
  const activeVals = new Set(activeLabels.map((l) => l.val))

  async function handleAction(action: typeof ACCOUNT_ACTIONS[number]) {
    setActionLoading(true)
    try {
      const isActive = activeVals.has(action.val)
      await accountAction(did, {
        action: action.val,
        negate: isActive,
      })
      setConfirmAction(null)
      await loadData()
    } catch (e) {
      setError(e instanceof Error ? e.message : "Action failed")
    } finally {
      setActionLoading(false)
    }
  }

  return (
    <>
      <SiteHeader title="Account" backHref="/dashboard/queue" />

      <div className="p-4 lg:p-6 space-y-6">
        {/* Account Header */}
        <Card>
          <CardHeader>
            <CardTitle className="flex items-center gap-3">
              <div>
                {handle ? (
                  <>
                    <span className="text-lg font-semibold">@{handle}</span>
                    <span className="text-muted-foreground text-sm ml-2 font-mono">
                      {did}
                    </span>
                  </>
                ) : (
                  <span className="text-lg font-semibold font-mono">{did}</span>
                )}
              </div>
            </CardTitle>
          </CardHeader>
        </Card>

        {loading && (
          <div className="flex items-center justify-center py-12">
            <Loader2 className="size-6 animate-spin text-muted-foreground" />
          </div>
        )}

        {error && (
          <Card className="border-destructive">
            <CardContent className="pt-6">
              <div className="flex items-center gap-2 text-destructive">
                <AlertTriangle className="size-4" />
                <span className="text-sm">{error}</span>
              </div>
            </CardContent>
          </Card>
        )}

        {!loading && (
          <>
            {/* Active Labels */}
            <Card>
              <CardHeader>
                <CardTitle className="text-base">Active Labels</CardTitle>
              </CardHeader>
              <CardContent>
                {activeLabels.length === 0 ? (
                  <p className="text-muted-foreground text-sm">
                    No active labels on this account.
                  </p>
                ) : (
                  <LabelBadges labels={activeLabels} />
                )}
              </CardContent>
            </Card>

            {/* Account Actions */}
            <Card>
              <CardHeader>
                <CardTitle className="text-base">Account Actions</CardTitle>
              </CardHeader>
              <CardContent>
                <div className="flex flex-wrap gap-2">
                  {ACCOUNT_ACTIONS.map((action) => {
                    const isActive = activeVals.has(action.val)
                    return (
                      <Button
                        key={action.val}
                        variant={isActive ? "destructive" : action.variant}
                        size="sm"
                        onClick={() => setConfirmAction(action)}
                      >
                        <action.icon className="size-4 mr-1.5" />
                        {isActive ? `Remove ${action.label}` : action.label}
                      </Button>
                    )
                  })}
                </div>
              </CardContent>
            </Card>

            <Separator />

            {/* Label History */}
            <Card>
              <CardHeader>
                <CardTitle className="text-base">Label History</CardTitle>
              </CardHeader>
              <CardContent>
                <LabelHistory labels={labels} />
              </CardContent>
            </Card>
          </>
        )}
      </div>

      {/* Confirmation Dialog */}
      <Dialog
        open={confirmAction !== null}
        onOpenChange={(open) => {
          if (!open) setConfirmAction(null)
        }}
      >
        {confirmAction && (
          <DialogContent>
            <DialogHeader>
              <DialogTitle>
                {activeVals.has(confirmAction.val)
                  ? `Remove ${confirmAction.label}`
                  : `Apply ${confirmAction.label}`}
              </DialogTitle>
              <DialogDescription>
                {activeVals.has(confirmAction.val)
                  ? `This will negate the ${confirmAction.val} label on this account.`
                  : confirmAction.description}
              </DialogDescription>
            </DialogHeader>

            <div className="py-2">
              <p className="text-sm">
                <span className="text-muted-foreground">Account:</span>{" "}
                <span className="font-mono text-xs">
                  {handle ? `@${handle}` : did}
                </span>
              </p>
              <p className="text-sm mt-1">
                <span className="text-muted-foreground">Action:</span>{" "}
                <code className="text-xs bg-muted px-1 py-0.5 rounded">
                  {activeVals.has(confirmAction.val) ? "negate" : "apply"} {confirmAction.val}
                </code>
              </p>
            </div>

            <DialogFooter>
              <Button
                variant="outline"
                onClick={() => setConfirmAction(null)}
                disabled={actionLoading}
              >
                Cancel
              </Button>
              <Button
                variant="destructive"
                onClick={() => handleAction(confirmAction)}
                disabled={actionLoading}
              >
                {actionLoading && <Loader2 className="size-4 mr-1.5 animate-spin" />}
                Confirm
              </Button>
            </DialogFooter>
          </DialogContent>
        )}
      </Dialog>
    </>
  )
}
