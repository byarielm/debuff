"use client"

import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card"
import { ServiceRecordForm } from "@/components/service-record-form"

interface SetupServiceRecordProps {
  onComplete: () => void
}

export function SetupServiceRecord({ onComplete }: SetupServiceRecordProps) {
  async function handleSubmit(values: {
    subjectTypes: string[]
    subjectCollections: string[]
    reasonTypes: string[]
  }) {
    const res = await fetch("/api/setup/record", {
      method: "POST",
      credentials: "include",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify(values),
    })

    if (!res.ok) {
      const text = await res.text()
      throw new Error(text || "Failed to create service record")
    }

    const completeRes = await fetch("/api/setup/complete", {
      method: "POST",
      credentials: "include",
    })

    if (!completeRes.ok) {
      const text = await completeRes.text()
      throw new Error(text || "Failed to complete setup")
    }

    onComplete()
  }

  return (
    <Card>
      <CardHeader>
        <CardTitle>Service Record Configuration</CardTitle>
        <CardDescription>
          Configure what types of content your labeler will accept reports for.
          These settings define the labeler service record published to the
          network. You can update these settings later from the dashboard.
        </CardDescription>
      </CardHeader>
      <CardContent>
        <ServiceRecordForm
          onSubmit={handleSubmit}
          submitLabel="Create Service Record"
        />
      </CardContent>
    </Card>
  )
}
