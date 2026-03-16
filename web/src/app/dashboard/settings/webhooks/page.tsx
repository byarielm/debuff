"use client"

import { useCallback, useEffect, useState } from "react"
import { Pencil, Trash2 } from "lucide-react"

import { useCurrentUser } from "@/hooks/use-current-user"
import {
  getWebhookSources,
  createWebhookSource,
  updateWebhookSource,
  deleteWebhookSource,
} from "@/lib/api"
import type { WebhookSource } from "@/types/webhooks"
import { SiteHeader } from "@/components/site-header"
import { WebhookCreateForm, WebhookEditForm } from "@/components/webhook-form"
import { Button } from "@/components/ui/button"
import { Badge } from "@/components/ui/badge"
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
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table"

export default function WebhooksSettingsPage() {
  const { isAdmin } = useCurrentUser()
  const [webhooks, setWebhooks] = useState<WebhookSource[]>([])
  const [error, setError] = useState<string | null>(null)
  const [createOpen, setCreateOpen] = useState(false)
  const [editWebhook, setEditWebhook] = useState<WebhookSource | null>(null)
  const [deleteId, setDeleteId] = useState<number | null>(null)
  const [deleting, setDeleting] = useState(false)

  const load = useCallback(() => {
    getWebhookSources()
      .then(setWebhooks)
      .catch((e) => setError(e instanceof Error ? e.message : String(e)))
  }, [])

  useEffect(() => {
    load()
  }, [load])

  async function handleCreate(data: { name: string }) {
    const result = await createWebhookSource(data)
    load()
    return result
  }

  async function handleEdit(
    data: Partial<Pick<WebhookSource, "active" | "auto_accept" | "auto_label" | "requires_review">>
  ) {
    if (!editWebhook) return
    await updateWebhookSource(editWebhook.id, data)
    load()
  }

  async function handleDelete(id: number) {
    setDeleting(true)
    try {
      await deleteWebhookSource(id)
      setDeleteId(null)
      load()
    } catch (e: unknown) {
      setError(e instanceof Error ? e.message : String(e))
    } finally {
      setDeleting(false)
    }
  }

  return (
    <>
      <SiteHeader title="Webhooks" />
      <div className="flex flex-1 flex-col gap-4 p-4 md:p-6">
        {error && <p className="text-destructive text-sm">{error}</p>}

        <div className="flex items-center justify-between">
          <div>
            <h2 className="text-lg font-semibold">Webhook Sources</h2>
            <p className="text-muted-foreground text-sm">
              Manage external webhook sources that submit reports.
            </p>
          </div>
          {isAdmin && (
            <Button onClick={() => setCreateOpen(true)}>Add Source</Button>
          )}
        </div>

        <div className="overflow-clip rounded-lg border">
          <Table>
            <TableHeader>
              <TableRow>
                <TableHead>Name</TableHead>
                <TableHead>Active</TableHead>
                <TableHead>Auto Accept</TableHead>
                <TableHead>Auto Label</TableHead>
                <TableHead>Requires Review</TableHead>
                <TableHead>Created</TableHead>
                <TableHead className="w-20 sticky right-0 bg-inherit z-[1]" />
              </TableRow>
            </TableHeader>
            <TableBody>
              {webhooks.length === 0 && (
                <TableRow>
                  <TableCell
                    colSpan={7}
                    className="text-muted-foreground text-center"
                  >
                    No webhook sources yet.
                  </TableCell>
                </TableRow>
              )}
              {webhooks.map((wh) => (
                <TableRow key={wh.id}>
                  <TableCell className="font-medium">{wh.name}</TableCell>
                  <TableCell>
                    <Badge
                      className={
                        wh.active
                          ? "bg-green-100 text-green-800 dark:bg-green-900 dark:text-green-200"
                          : "bg-neutral-100 text-neutral-800 dark:bg-neutral-800 dark:text-neutral-200"
                      }
                    >
                      {wh.active ? "Active" : "Inactive"}
                    </Badge>
                  </TableCell>
                  <TableCell className="text-sm">
                    {wh.auto_accept ? "Yes" : "No"}
                  </TableCell>
                  <TableCell className="text-sm">
                    {wh.auto_label ? "Yes" : "No"}
                  </TableCell>
                  <TableCell className="text-sm">
                    {wh.requires_review ? "Yes" : "No"}
                  </TableCell>
                  <TableCell className="text-sm">
                    {new Date(wh.created_at).toLocaleString()}
                  </TableCell>
                  <TableCell className="w-20 sticky right-0 bg-inherit z-[1]">
                    <div className="flex gap-1">
                      {isAdmin && (
                        <Button
                          variant="ghost"
                          size="icon"
                          className="size-8"
                          title="Edit webhook"
                          aria-label="Edit webhook"
                          onClick={() => setEditWebhook(wh)}
                        >
                          <Pencil className="size-4" />
                        </Button>
                      )}
                      {isAdmin && (
                        <Button
                          variant="ghost"
                          size="icon"
                          className="size-8 text-muted-foreground hover:text-destructive"
                          title="Delete webhook"
                          aria-label="Delete webhook"
                          onClick={() => setDeleteId(wh.id)}
                        >
                          <Trash2 className="size-4" />
                        </Button>
                      )}
                    </div>
                  </TableCell>
                </TableRow>
              ))}
            </TableBody>
          </Table>
        </div>
      </div>

      <WebhookCreateForm
        open={createOpen}
        onOpenChange={(o) => {
          setCreateOpen(o)
          if (!o) load()
        }}
        onSubmit={handleCreate}
      />

      {editWebhook && (
        <WebhookEditForm
          open={!!editWebhook}
          onOpenChange={(o) => {
            if (!o) setEditWebhook(null)
          }}
          webhook={editWebhook}
          onSubmit={handleEdit}
        />
      )}

      <Dialog
        open={deleteId !== null}
        onOpenChange={(open) => {
          if (!open) setDeleteId(null)
        }}
      >
        <DialogContent>
          <DialogHeader>
            <DialogTitle>Delete webhook source?</DialogTitle>
            <DialogDescription>
              This will permanently remove this webhook source and revoke its
              secret. Incoming webhooks using this source will be rejected. This
              action cannot be undone.
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
              onClick={() => {
                if (deleteId !== null) handleDelete(deleteId)
              }}
            >
              {deleting ? "Deleting..." : "Delete"}
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </>
  )
}
