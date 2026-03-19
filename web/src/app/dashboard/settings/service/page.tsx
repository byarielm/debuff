"use client"

import { useCallback, useEffect, useState } from "react"

import { SiteHeader } from "@/components/site-header"
import {
  ServiceRecordForm,
  type ServiceRecordValues,
} from "@/components/service-record-form"
import { getServiceRecord, updateServiceRecord } from "@/lib/api"

export default function ServiceSettingsPage() {
  const [initialValues, setInitialValues] =
    useState<ServiceRecordValues | undefined>()
  const [loading, setLoading] = useState(true)
  const [loadError, setLoadError] = useState<string | null>(null)

  const load = useCallback(() => {
    setLoading(true)
    setLoadError(null)
    getServiceRecord()
      .then((data) => {
        setInitialValues(data)
      })
      .catch((e) => {
        setLoadError(e instanceof Error ? e.message : String(e))
      })
      .finally(() => setLoading(false))
  }, [])

  useEffect(() => {
    load()
  }, [load])

  async function handleSubmit(values: ServiceRecordValues) {
    await updateServiceRecord(values)
  }

  return (
    <>
      <SiteHeader title="Service Record" />
      <div className="flex flex-1 flex-col gap-4 p-4 md:p-6">
        <div>
          <h2 className="text-lg font-semibold">Service Record</h2>
          <p className="text-muted-foreground text-sm">
            Update the labeler service record published to the network. Changes
            take effect immediately.
          </p>
        </div>

        {loadError && (
          <p className="text-destructive text-sm">{loadError}</p>
        )}

        <ServiceRecordForm
          initialValues={initialValues}
          onSubmit={handleSubmit}
          submitLabel="Save Changes"
          loading={loading}
        />
      </div>
    </>
  )
}
