"use client"

import { useState } from "react"
import { Plus, Trash2 } from "lucide-react"

import { Button } from "@/components/ui/button"
import { Checkbox } from "@/components/ui/checkbox"
import { Input } from "@/components/ui/input"
import { Label } from "@/components/ui/label"
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select"
import { Separator } from "@/components/ui/separator"
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs"
import { Textarea } from "@/components/ui/textarea"
import type { LabelDefinition, LabelDefinitionLocale } from "@/types/definitions"

export interface LabelDefinitionFormData {
  identifier: string
  severity: "inform" | "alert" | "none"
  blurs: "content" | "media" | "none"
  default_setting: "ignore" | "warn" | "hide"
  adult_only: boolean
  locales: LabelDefinitionLocale[]
}

interface LabelDefinitionFormProps {
  onSubmit: (data: LabelDefinitionFormData) => Promise<void>
  onCancel: () => void
  existing?: LabelDefinition
}

const IDENTIFIER_RE = /^[a-z][a-z0-9-]*$/

function languageName(code: string): string {
  try {
    const name = new Intl.DisplayNames(["en"], { type: "language" }).of(code)
    if (name) return name
  } catch {}
  return code.toUpperCase()
}

function initLocales(existing?: LabelDefinition): LabelDefinitionLocale[] {
  if (existing?.locales?.length) return existing.locales
  return [{ lang: "en", name: "", description: "" }]
}

export function LabelDefinitionForm({
  onSubmit,
  onCancel,
  existing,
}: LabelDefinitionFormProps) {
  const isEdit = !!existing
  const [identifier, setIdentifier] = useState(existing?.identifier ?? "")
  const [severity, setSeverity] = useState<"inform" | "alert" | "none">(
    existing?.severity ?? "inform"
  )
  const [blurs, setBlurs] = useState<"content" | "media" | "none">(
    existing?.blurs ?? "none"
  )
  const [defaultSetting, setDefaultSetting] = useState<"ignore" | "warn" | "hide">(
    existing?.default_setting ?? "warn"
  )
  const [adultOnly, setAdultOnly] = useState(existing?.adult_only ?? false)
  const [locales, setLocales] = useState<LabelDefinitionLocale[]>(() => initLocales(existing))
  const [activeTab, setActiveTab] = useState("en")
  const [submitting, setSubmitting] = useState(false)
  const [error, setError] = useState<string | null>(null)

  const identifierValid = IDENTIFIER_RE.test(identifier)
  const hasEnglish = locales.some((l) => l.lang === "en" && l.name.trim() && l.description.trim())
  const allLocalesValid = locales.every((l) => l.lang.trim() && l.name.trim() && l.description.trim())
  const noDuplicateLangs = new Set(locales.map((l) => l.lang)).size === locales.length
  const canSubmit =
    identifier.trim() !== "" &&
    identifierValid &&
    hasEnglish &&
    allLocalesValid &&
    noDuplicateLangs &&
    !submitting

  function updateLocale(index: number, field: keyof LabelDefinitionLocale, value: string) {
    setLocales((prev) =>
      prev.map((l, i) => (i === index ? { ...l, [field]: value } : l))
    )
  }

  function addLocale() {
    const lang = prompt("Enter a language code (e.g. fr, de, ja):")
    if (!lang) return
    const normalized = lang.toLowerCase().trim()
    if (!normalized) return
    if (locales.some((l) => l.lang === normalized)) return
    setLocales((prev) => [...prev, { lang: normalized, name: "", description: "" }])
    setActiveTab(normalized)
  }

  function removeLocale(index: number) {
    const removing = locales[index]
    setLocales((prev) => prev.filter((_, i) => i !== index))
    if (activeTab === removing.lang) {
      setActiveTab("en")
    }
  }

  async function handleSubmit(e: React.FormEvent) {
    e.preventDefault()
    setError(null)
    setSubmitting(true)
    try {
      await onSubmit({
        identifier,
        severity,
        blurs,
        default_setting: defaultSetting,
        adult_only: adultOnly,
        locales,
      })
    } catch (e: unknown) {
      setError(e instanceof Error ? e.message : String(e))
    } finally {
      setSubmitting(false)
    }
  }

  return (
    <form onSubmit={handleSubmit} className="flex flex-col gap-6">
      {error && <p className="text-destructive text-sm">{error}</p>}

      <div className="flex flex-col gap-2">
        <Label htmlFor="def-identifier">Identifier</Label>
        <Input
          id="def-identifier"
          value={identifier}
          onChange={(e) => setIdentifier(e.target.value.toLowerCase())}
          placeholder="my-label"
          className="font-mono max-w-sm"
          disabled={isEdit}
        />
        {identifier && !identifierValid && (
          <p className="text-destructive text-xs">
            Must start with a letter. Only lowercase letters, numbers, and hyphens allowed.
          </p>
        )}
      </div>

      <div className="grid grid-cols-1 gap-4 sm:grid-cols-3">
        <div className="flex flex-col gap-2">
          <Label htmlFor="def-severity">Severity</Label>
          <Select value={severity} onValueChange={(v) => setSeverity(v as typeof severity)}>
            <SelectTrigger id="def-severity">
              <SelectValue />
            </SelectTrigger>
            <SelectContent>
              <SelectItem value="inform">Inform</SelectItem>
              <SelectItem value="alert">Alert</SelectItem>
              <SelectItem value="none">None</SelectItem>
            </SelectContent>
          </Select>
        </div>

        <div className="flex flex-col gap-2">
          <Label htmlFor="def-blurs">Blurs</Label>
          <Select value={blurs} onValueChange={(v) => setBlurs(v as typeof blurs)}>
            <SelectTrigger id="def-blurs">
              <SelectValue />
            </SelectTrigger>
            <SelectContent>
              <SelectItem value="content">Content</SelectItem>
              <SelectItem value="media">Media</SelectItem>
              <SelectItem value="none">None</SelectItem>
            </SelectContent>
          </Select>
        </div>

        <div className="flex flex-col gap-2">
          <Label htmlFor="def-default">Default Setting</Label>
          <Select
            value={defaultSetting}
            onValueChange={(v) => setDefaultSetting(v as typeof defaultSetting)}
          >
            <SelectTrigger id="def-default">
              <SelectValue />
            </SelectTrigger>
            <SelectContent>
              <SelectItem value="ignore">Ignore</SelectItem>
              <SelectItem value="warn">Warn</SelectItem>
              <SelectItem value="hide">Hide</SelectItem>
            </SelectContent>
          </Select>
        </div>
      </div>

      <div className="flex items-center gap-2">
        <Checkbox
          id="def-adult-only"
          checked={adultOnly}
          onCheckedChange={(checked) => setAdultOnly(checked === true)}
        />
        <Label htmlFor="def-adult-only" className="cursor-pointer">
          Adult only
        </Label>
      </div>

      <Separator />

      <div className="flex flex-col gap-4">
        <div className="flex items-center justify-between">
          <div>
            <p className="text-sm font-medium">Translations</p>
            <p className="text-muted-foreground text-xs">
              English is required. Add translations for other languages.
            </p>
          </div>
          <Button type="button" variant="outline" size="sm" onClick={addLocale}>
            <Plus className="size-4 mr-1.5" />
            Add Translation
          </Button>
        </div>

        {!noDuplicateLangs && (
          <p className="text-destructive text-xs">
            Each language code must be unique.
          </p>
        )}

        <Tabs value={activeTab} onValueChange={setActiveTab}>
          <TabsList variant="line">
            {locales.map((locale) => (
              <TabsTrigger key={locale.lang} value={locale.lang}>
                {languageName(locale.lang)}
              </TabsTrigger>
            ))}
          </TabsList>

          {locales.map((locale, index) => (
            <TabsContent key={locale.lang} value={locale.lang} className="flex flex-col gap-4 pt-2">
              <div className="flex flex-col gap-2">
                <Label htmlFor={`def-locale-name-${index}`}>Name</Label>
                <Input
                  id={`def-locale-name-${index}`}
                  value={locale.name}
                  onChange={(e) => updateLocale(index, "name", e.target.value)}
                  placeholder="My Label"
                  className="max-w-sm"
                />
              </div>
              <div className="flex flex-col gap-2">
                <Label htmlFor={`def-locale-desc-${index}`}>Description</Label>
                <Textarea
                  id={`def-locale-desc-${index}`}
                  value={locale.description}
                  onChange={(e) => updateLocale(index, "description", e.target.value)}
                  placeholder="A description of what this label means."
                  rows={3}
                />
              </div>
              {locale.lang !== "en" && (
                <div>
                  <Button
                    type="button"
                    variant="outline"
                    size="sm"
                    className="text-destructive hover:text-destructive"
                    onClick={() => removeLocale(index)}
                  >
                    <Trash2 className="size-4 mr-1.5" />
                    Remove Translation
                  </Button>
                </div>
              )}
            </TabsContent>
          ))}
        </Tabs>
      </div>

      <div className="flex gap-2">
        <Button type="submit" disabled={!canSubmit}>
          {submitting ? "Saving..." : isEdit ? "Save Changes" : "Create Label"}
        </Button>
        <Button type="button" variant="outline" onClick={onCancel} disabled={submitting}>
          Cancel
        </Button>
      </div>
    </form>
  )
}
