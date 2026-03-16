"use client"

import { useRouter } from "next/navigation"

import { createLabelDefinition } from "@/lib/api"
import { SiteHeader } from "@/components/site-header"
import { LabelDefinitionForm } from "@/components/label-definition-form"
import type { LabelDefinitionFormData } from "@/components/label-definition-form"

export default function NewLabelPage() {
  const router = useRouter()

  async function handleSubmit(data: LabelDefinitionFormData) {
    await createLabelDefinition(data)
    router.push("/dashboard/settings/labels")
  }

  return (
    <>
      <SiteHeader title="New Label Definition" backHref="/dashboard/settings/labels" />
      <div className="flex flex-1 flex-col gap-4 p-4 md:p-6 max-w-2xl">
        <div>
          <h2 className="text-lg font-semibold">New Label Definition</h2>
          <p className="text-muted-foreground text-sm">
            Create a new label definition for your labeler.
          </p>
        </div>
        <LabelDefinitionForm
          onSubmit={handleSubmit}
          onCancel={() => router.push("/dashboard/settings/labels")}
        />
      </div>
    </>
  )
}
