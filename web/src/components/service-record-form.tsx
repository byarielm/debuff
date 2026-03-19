"use client"

import { useCallback, useEffect, useState } from "react"
import { Loader2, X } from "lucide-react"
import { Button } from "@/components/ui/button"
import { Input } from "@/components/ui/input"
import { Label } from "@/components/ui/label"
import { Checkbox } from "@/components/ui/checkbox"

interface NsidInfo {
  description?: string
  loading: boolean
}

export interface ServiceRecordValues {
  subjectTypes: string[]
  subjectCollections: string[]
  reasonTypes: string[]
}

interface ServiceRecordFormProps {
  initialValues?: ServiceRecordValues
  onSubmit: (values: ServiceRecordValues) => Promise<void>
  submitLabel: string
  loading?: boolean
}

const SUBJECT_TYPES = [
  { id: "account", label: "Account" },
  { id: "record", label: "Record" },
]

const SUBJECT_COLLECTIONS = [
  { id: "app.bsky.feed.post", label: "Posts", defaultChecked: true },
  { id: "app.bsky.actor.profile", label: "Profiles", defaultChecked: true },
  { id: "app.bsky.feed.generator", label: "Feed Generators", defaultChecked: false },
  { id: "app.bsky.graph.list", label: "Lists", defaultChecked: false },
]

const WELL_KNOWN_IDS = new Set(SUBJECT_COLLECTIONS.map((c) => c.id))

const REASON_TYPES = [
  { id: "com.atproto.moderation.defs#reasonSpam", label: "Spam" },
  { id: "com.atproto.moderation.defs#reasonMisleading", label: "Misleading" },
  { id: "com.atproto.moderation.defs#reasonSexual", label: "Sexual" },
  { id: "com.atproto.moderation.defs#reasonRude", label: "Rude" },
  { id: "com.atproto.moderation.defs#reasonViolation", label: "Violation" },
  { id: "com.atproto.moderation.defs#reasonAppeal", label: "Appeal" },
  { id: "com.atproto.moderation.defs#reasonOther", label: "Other" },
]

export function ServiceRecordForm({
  initialValues,
  onSubmit,
  submitLabel,
  loading: externalLoading,
}: ServiceRecordFormProps) {
  const [subjectTypes, setSubjectTypes] = useState<Set<string>>(
    new Set(initialValues?.subjectTypes ?? ["account", "record"])
  )
  const [subjectCollections, setSubjectCollections] = useState<Set<string>>(
    new Set(
      initialValues?.subjectCollections ??
        SUBJECT_COLLECTIONS.filter((c) => c.defaultChecked).map((c) => c.id)
    )
  )
  const [reasonTypes, setReasonTypes] = useState<Set<string>>(
    new Set(initialValues?.reasonTypes ?? REASON_TYPES.map((r) => r.id))
  )
  const [customCollections, setCustomCollections] = useState<string[]>(
    () => initialValues?.subjectCollections.filter((c) => !WELL_KNOWN_IDS.has(c)) ?? []
  )
  const [customCollection, setCustomCollection] = useState("")
  const [nsidInfo, setNsidInfo] = useState<Record<string, NsidInfo>>({})
  const [submitting, setSubmitting] = useState(false)
  const [error, setError] = useState<string | null>(null)
  const [success, setSuccess] = useState(false)

  // Re-initialize when initialValues change (e.g. after fetch completes)
  useEffect(() => {
    if (!initialValues) return
    setSubjectTypes(new Set(initialValues.subjectTypes))
    setSubjectCollections(new Set(initialValues.subjectCollections))
    setReasonTypes(new Set(initialValues.reasonTypes))
    setCustomCollections(
      initialValues.subjectCollections.filter((c) => !WELL_KNOWN_IDS.has(c))
    )
  }, [initialValues])

  const resolveNsid = useCallback((nsid: string) => {
    setNsidInfo((prev) => ({ ...prev, [nsid]: { loading: true } }))
    fetch(`/api/setup/resolve-nsid?nsid=${encodeURIComponent(nsid)}`)
      .then((res) => (res.ok ? res.json() : null))
      .then((data) => {
        setNsidInfo((prev) => ({
          ...prev,
          [nsid]: { description: data?.description, loading: false },
        }))
      })
      .catch(() => {
        setNsidInfo((prev) => ({
          ...prev,
          [nsid]: { loading: false },
        }))
      })
  }, [])

  function toggleSet(
    set: Set<string>,
    setter: (s: Set<string>) => void,
    value: string
  ) {
    const next = new Set(set)
    if (next.has(value)) {
      next.delete(value)
    } else {
      next.add(value)
    }
    setter(next)
  }

  function addCollections(input: string) {
    const items = input
      .split(/[,\n]+/)
      .map((s) => s.trim())
      .filter(
        (s) =>
          s && !subjectCollections.has(s) && !customCollections.includes(s)
      )
    if (items.length === 0) return
    setCustomCollections((prev) => [...prev, ...items])
    setSubjectCollections((prev) => new Set([...prev, ...items]))
    setCustomCollection("")
    for (const item of items) {
      resolveNsid(item)
    }
  }

  function handleRemoveCustomCollection(col: string) {
    setCustomCollections((prev) => prev.filter((c) => c !== col))
    setSubjectCollections((prev) => {
      const next = new Set(prev)
      next.delete(col)
      return next
    })
  }

  async function handleSubmit(e: React.FormEvent) {
    e.preventDefault()
    setSubmitting(true)
    setError(null)
    setSuccess(false)

    try {
      await onSubmit({
        subjectTypes: Array.from(subjectTypes),
        subjectCollections: Array.from(subjectCollections),
        reasonTypes: Array.from(reasonTypes),
      })
      setSuccess(true)
    } catch (e) {
      setError(
        e instanceof Error ? e.message : "Failed to update service record"
      )
    } finally {
      setSubmitting(false)
    }
  }

  const disabled = externalLoading || submitting

  return (
    <form onSubmit={handleSubmit} className="flex flex-col gap-6">
      {error && <p className="text-destructive text-sm">{error}</p>}
      {success && (
        <p className="text-sm text-green-600 dark:text-green-400">
          Service record updated successfully.
        </p>
      )}

      {/* Subject Types */}
      <div className="flex flex-col gap-3">
        <Label>Subject Types</Label>
        <p className="text-muted-foreground text-sm">
          What types of subjects can be reported to your labeler?
        </p>
        <div className="flex flex-col gap-2">
          {SUBJECT_TYPES.map((type) => (
            <div key={type.id} className="flex items-center gap-2">
              <Checkbox
                id={`subject-type-${type.id}`}
                checked={subjectTypes.has(type.id)}
                disabled={disabled}
                onCheckedChange={() =>
                  toggleSet(subjectTypes, setSubjectTypes, type.id)
                }
              />
              <Label
                htmlFor={`subject-type-${type.id}`}
                className="font-normal"
              >
                {type.label}
              </Label>
            </div>
          ))}
        </div>
      </div>

      {/* Subject Collections */}
      <div className="flex flex-col gap-3">
        <Label>Subject Collections</Label>
        <p className="text-muted-foreground text-sm">
          Which record collections can be reported?
        </p>
        <div className="flex flex-col gap-2">
          {SUBJECT_COLLECTIONS.map((col) => (
            <div key={col.id} className="flex items-center gap-2">
              <Checkbox
                id={`collection-${col.id}`}
                checked={subjectCollections.has(col.id)}
                disabled={disabled}
                onCheckedChange={() =>
                  toggleSet(
                    subjectCollections,
                    setSubjectCollections,
                    col.id
                  )
                }
              />
              <Label
                htmlFor={`collection-${col.id}`}
                className="font-normal"
              >
                {col.label}{" "}
                <span className="text-muted-foreground">({col.id})</span>
              </Label>
            </div>
          ))}
          {customCollections.map((col) => {
            const info = nsidInfo[col]
            return (
              <div key={col}>
                <div className="flex items-center gap-2">
                  <Checkbox
                    id={`collection-custom-${col}`}
                    checked={subjectCollections.has(col)}
                    disabled={disabled}
                    onCheckedChange={() =>
                      toggleSet(
                        subjectCollections,
                        setSubjectCollections,
                        col
                      )
                    }
                  />
                  <Label
                    htmlFor={`collection-custom-${col}`}
                    className="flex-1 font-normal"
                  >
                    {col}
                    {info?.loading && (
                      <Loader2 className="text-muted-foreground ml-1 inline size-3 animate-spin" />
                    )}
                  </Label>
                  <button
                    type="button"
                    className="text-muted-foreground hover:text-destructive"
                    onClick={() => handleRemoveCustomCollection(col)}
                    disabled={disabled}
                    aria-label={`Remove ${col}`}
                    title="Remove collection"
                  >
                    <X className="size-4" />
                  </button>
                </div>
                {info?.description && (
                  <p className="text-muted-foreground ml-6 text-xs">
                    {info.description}
                  </p>
                )}
              </div>
            )
          })}
        </div>
        <div className="flex gap-2">
          <Input
            placeholder="com.example.record"
            value={customCollection}
            disabled={disabled}
            onChange={(e) => setCustomCollection(e.target.value)}
            onKeyDown={(e) => {
              if (e.key === "Enter") {
                e.preventDefault()
                addCollections(customCollection)
              }
            }}
            onPaste={(e) => {
              const pasted = e.clipboardData.getData("text")
              if (pasted.includes(",") || pasted.includes("\n")) {
                e.preventDefault()
                addCollections(pasted)
              }
            }}
          />
          <Button
            type="button"
            variant="outline"
            disabled={disabled}
            onClick={() => addCollections(customCollection)}
          >
            Add
          </Button>
        </div>
      </div>

      {/* Reason Types */}
      <div className="flex flex-col gap-3">
        <Label>Reason Types</Label>
        <p className="text-muted-foreground text-sm">
          What reasons can reporters select when submitting a report?
        </p>
        <div className="grid grid-cols-2 gap-2">
          {REASON_TYPES.map((reason) => (
            <div key={reason.id} className="flex items-center gap-2">
              <Checkbox
                id={`reason-${reason.id}`}
                checked={reasonTypes.has(reason.id)}
                disabled={disabled}
                onCheckedChange={() =>
                  toggleSet(reasonTypes, setReasonTypes, reason.id)
                }
              />
              <Label
                htmlFor={`reason-${reason.id}`}
                className="font-normal"
              >
                {reason.label}
              </Label>
            </div>
          ))}
        </div>
      </div>

      <Button type="submit" disabled={disabled}>
        {submitting ? "Saving..." : submitLabel}
      </Button>
    </form>
  )
}
