"use client"

import { useCallback, useEffect, useState } from "react"
import Link from "next/link"

import { useCurrentUser } from "@/hooks/use-current-user"
import { getLabelDefinitions } from "@/lib/api"
import type { LabelDefinition } from "@/types/definitions"
import { SiteHeader } from "@/components/site-header"
import { LabelsTable } from "@/components/labels-table"
import { Button } from "@/components/ui/button"

export default function LabelsSettingsPage() {
  const { isAdmin } = useCurrentUser()
  const [definitions, setDefinitions] = useState<LabelDefinition[]>([])
  const [error, setError] = useState<string | null>(null)

  const load = useCallback(() => {
    getLabelDefinitions()
      .then(setDefinitions)
      .catch((e) => setError(e instanceof Error ? e.message : String(e)))
  }, [])

  useEffect(() => {
    load()
  }, [load])

  return (
    <>
      <SiteHeader title="Label Definitions" />
      <div className="flex flex-1 flex-col gap-4 overflow-hidden p-4 md:p-6">
        {error && <p className="text-destructive text-sm">{error}</p>}

        <div className="flex items-center justify-between">
          <div>
            <h2 className="text-lg font-semibold">Label Definitions</h2>
            <p className="text-muted-foreground text-sm">
              Manage the label definitions published by your labeler.
            </p>
          </div>
          {isAdmin && (
            <Button asChild>
              <Link href="/dashboard/settings/labels/new">Add Label</Link>
            </Button>
          )}
        </div>

        <LabelsTable definitions={definitions} isAdmin={isAdmin} />
      </div>
    </>
  )
}
