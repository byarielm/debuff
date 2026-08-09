"use client"

import { useEffect, useState } from "react"
import { useParams, useRouter } from "next/navigation"
import { Trash2 } from "lucide-react"

import { useCurrentUser } from "@/hooks/use-current-user"
import {
  getLabelDefinition,
  updateLabelDefinition,
  deleteLabelDefinition,
} from "@/lib/api"
import type { LabelDefinition } from "@/types/definitions"
import { SiteHeader } from "@/components/site-header"
import { LabelDefinitionForm } from "@/components/label-definition-form"
import type { LabelDefinitionFormData } from "@/components/label-definition-form"
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

export default function LabelDetail() {
  const params = useParams<{ id: string }>()
  const id = params?.id
  const waitingForParams = params === null
  const defId = id && /^\d+$/.test(id) ? Number(id) : null
  const router = useRouter()
  const { isAdmin } = useCurrentUser()

  const [definition, setDefinition] = useState<LabelDefinition | null>(null)
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)
  const [deleteOpen, setDeleteOpen] = useState(false)
  const [deleting, setDeleting] = useState(false)

  useEffect(() => {
    if (waitingForParams) return

    setError(null)
    setLoading(true)
    if (defId === null) {
      setError("Invalid label definition URL.")
      setLoading(false)
      return
    }
    let cancelled = false
    getLabelDefinition(defId)
      .then((def) => {
        if (!cancelled) setDefinition(def)
      })
      .catch((e) => {
        if (!cancelled) setError(e instanceof Error ? e.message : String(e))
      })
      .finally(() => {
        if (!cancelled) setLoading(false)
      })
    return () => {
      cancelled = true
    }
  }, [defId, waitingForParams])

  async function handleSubmit(data: LabelDefinitionFormData) {
    if (defId === null) return
    await updateLabelDefinition(defId, data)
    router.push("/dashboard/settings/labels")
  }

  async function handleDelete() {
    if (defId === null) return
    setDeleting(true)
    try {
      await deleteLabelDefinition(defId)
      router.push("/dashboard/settings/labels")
    } catch (e: unknown) {
      setError(e instanceof Error ? e.message : String(e))
      setDeleting(false)
    }
  }

  if (loading) {
    return (
      <>
        <SiteHeader title="Label Definition" backHref="/dashboard/settings/labels" />
        <div className="flex flex-1 items-center justify-center p-6">
          <p className="text-muted-foreground text-sm">Loading...</p>
        </div>
      </>
    )
  }

  if (error || !definition) {
    return (
      <>
        <SiteHeader title="Label Definition" backHref="/dashboard/settings/labels" />
        <div className="flex flex-1 items-center justify-center p-6">
          <p className="text-destructive text-sm">
            {error ?? "Label definition not found."}
          </p>
        </div>
      </>
    )
  }

  return (
    <>
      <SiteHeader
        title={definition.identifier}
        backHref="/dashboard/settings/labels"
      />
      <div className="flex flex-1 flex-col gap-4 p-4 md:p-6 max-w-2xl">
        <div className="flex items-center justify-between">
          <div>
            <h2 className="text-lg font-semibold">
              Edit Label Definition
            </h2>
            <p className="text-muted-foreground text-sm">
              Update the settings for <code>{definition.identifier}</code>.
            </p>
          </div>
          {isAdmin && (
            <Button
              variant="outline"
              size="sm"
              className="text-destructive hover:text-destructive"
              onClick={() => setDeleteOpen(true)}
            >
              <Trash2 className="size-4 mr-1.5" />
              Delete
            </Button>
          )}
        </div>

        <LabelDefinitionForm
          onSubmit={handleSubmit}
          onCancel={() => router.push("/dashboard/settings/labels")}
          existing={definition}
        />
      </div>

      <Dialog open={deleteOpen} onOpenChange={setDeleteOpen}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>Delete label definition?</DialogTitle>
            <DialogDescription>
              This will permanently remove this label definition. Any existing
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
