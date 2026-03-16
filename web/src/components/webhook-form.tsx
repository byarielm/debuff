"use client"

import { useState } from "react"
import { Copy, Check } from "lucide-react"

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
import { Input } from "@/components/ui/input"
import { Label } from "@/components/ui/label"
import { Switch } from "@/components/ui/switch"
import type { WebhookSource } from "@/types/webhooks"

interface WebhookCreateFormProps {
  open: boolean
  onOpenChange: (open: boolean) => void
  onSubmit: (data: { name: string }) => Promise<WebhookSource>
}

export function WebhookCreateForm({
  open,
  onOpenChange,
  onSubmit,
}: WebhookCreateFormProps) {
  const [name, setName] = useState("")
  const [submitting, setSubmitting] = useState(false)
  const [error, setError] = useState<string | null>(null)
  const [createdSecret, setCreatedSecret] = useState<string | null>(null)
  const [copied, setCopied] = useState(false)

  function reset() {
    setName("")
    setError(null)
    setCreatedSecret(null)
    setCopied(false)
  }

  async function handleCreate() {
    setError(null)
    setSubmitting(true)
    try {
      const result = await onSubmit({ name })
      if (result.secret) {
        setCreatedSecret(result.secret)
      } else {
        onOpenChange(false)
      }
    } catch (e: unknown) {
      setError(e instanceof Error ? e.message : String(e))
    } finally {
      setSubmitting(false)
    }
  }

  async function handleCopy() {
    if (!createdSecret) return
    await navigator.clipboard.writeText(createdSecret)
    setCopied(true)
    setTimeout(() => setCopied(false), 2000)
  }

  return (
    <Dialog
      open={open}
      onOpenChange={(o) => {
        if (o) reset()
        onOpenChange(o)
      }}
    >
      <DialogContent>
        {createdSecret ? (
          <>
            <DialogHeader>
              <DialogTitle>Webhook Secret</DialogTitle>
              <DialogDescription>
                Copy this secret now. It will not be shown again.
              </DialogDescription>
            </DialogHeader>
            <div className="flex items-center gap-2">
              <code className="bg-muted flex-1 truncate rounded px-3 py-2 font-mono text-sm">
                {createdSecret}
              </code>
              <Button
                variant="outline"
                size="icon"
                className="shrink-0"
                onClick={handleCopy}
              >
                {copied ? (
                  <Check className="size-4" />
                ) : (
                  <Copy className="size-4" />
                )}
              </Button>
            </div>
            <DialogFooter>
              <Button onClick={() => onOpenChange(false)}>Done</Button>
            </DialogFooter>
          </>
        ) : (
          <>
            <DialogHeader>
              <DialogTitle>Add Webhook Source</DialogTitle>
              <DialogDescription>
                Create a new webhook source. You will receive a secret to
                authenticate incoming webhooks.
              </DialogDescription>
            </DialogHeader>
            <div className="flex flex-col gap-4">
              {error && <p className="text-destructive text-sm">{error}</p>}
              <div className="flex flex-col gap-2">
                <Label htmlFor="webhook-name">Name</Label>
                <Input
                  id="webhook-name"
                  value={name}
                  onChange={(e) => setName(e.target.value)}
                  placeholder="My Webhook"
                />
              </div>
            </div>
            <DialogFooter>
              <DialogClose asChild>
                <Button variant="outline" disabled={submitting}>
                  Cancel
                </Button>
              </DialogClose>
              <Button
                onClick={handleCreate}
                disabled={!name.trim() || submitting}
              >
                {submitting ? "Creating..." : "Create"}
              </Button>
            </DialogFooter>
          </>
        )}
      </DialogContent>
    </Dialog>
  )
}

interface WebhookEditFormProps {
  open: boolean
  onOpenChange: (open: boolean) => void
  webhook: WebhookSource
  onSubmit: (data: Partial<Pick<WebhookSource, "active" | "auto_accept" | "auto_label" | "requires_review">>) => Promise<void>
}

export function WebhookEditForm({
  open,
  onOpenChange,
  webhook,
  onSubmit,
}: WebhookEditFormProps) {
  const [active, setActive] = useState(webhook.active)
  const [autoAccept, setAutoAccept] = useState(webhook.auto_accept)
  const [autoLabel, setAutoLabel] = useState(webhook.auto_label)
  const [requiresReview, setRequiresReview] = useState(webhook.requires_review)
  const [submitting, setSubmitting] = useState(false)
  const [error, setError] = useState<string | null>(null)

  function reset() {
    setActive(webhook.active)
    setAutoAccept(webhook.auto_accept)
    setAutoLabel(webhook.auto_label)
    setRequiresReview(webhook.requires_review)
    setError(null)
  }

  async function handleSave() {
    setError(null)
    setSubmitting(true)
    try {
      await onSubmit({
        active,
        auto_accept: autoAccept,
        auto_label: autoLabel,
        requires_review: requiresReview,
      })
      onOpenChange(false)
    } catch (e: unknown) {
      setError(e instanceof Error ? e.message : String(e))
    } finally {
      setSubmitting(false)
    }
  }

  return (
    <Dialog
      open={open}
      onOpenChange={(o) => {
        if (o) reset()
        onOpenChange(o)
      }}
    >
      <DialogContent>
        <DialogHeader>
          <DialogTitle>Edit Webhook: {webhook.name}</DialogTitle>
          <DialogDescription>
            Configure how this webhook source handles incoming reports.
          </DialogDescription>
        </DialogHeader>
        <div className="flex flex-col gap-4">
          {error && <p className="text-destructive text-sm">{error}</p>}

          <div className="flex items-center justify-between">
            <Label htmlFor="webhook-active" className="cursor-pointer">
              Active
            </Label>
            <Switch
              id="webhook-active"
              checked={active}
              onCheckedChange={setActive}
            />
          </div>

          <div className="flex items-center justify-between">
            <Label htmlFor="webhook-auto-accept" className="cursor-pointer">
              Auto Accept
            </Label>
            <Switch
              id="webhook-auto-accept"
              checked={autoAccept}
              onCheckedChange={setAutoAccept}
            />
          </div>

          <div className="flex items-center justify-between">
            <Label htmlFor="webhook-auto-label" className="cursor-pointer">
              Auto Label
            </Label>
            <Switch
              id="webhook-auto-label"
              checked={autoLabel}
              onCheckedChange={setAutoLabel}
            />
          </div>

          <div className="flex items-center justify-between">
            <Label htmlFor="webhook-requires-review" className="cursor-pointer">
              Requires Review
            </Label>
            <Switch
              id="webhook-requires-review"
              checked={requiresReview}
              onCheckedChange={setRequiresReview}
            />
          </div>
        </div>
        <DialogFooter>
          <DialogClose asChild>
            <Button variant="outline" disabled={submitting}>
              Cancel
            </Button>
          </DialogClose>
          <Button onClick={handleSave} disabled={submitting}>
            {submitting ? "Saving..." : "Save"}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
}
