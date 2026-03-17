"use client"

import { use } from "react"

import { useQueueDetail } from "@/hooks/use-queue"
import { SiteHeader } from "@/components/site-header"
import { ReportDetail } from "@/components/report-detail"
import { ActionPanel } from "@/components/action-panel"
import { NotesThread } from "@/components/notes-thread"

export default function QueueDetail({
  params,
}: {
  params: Promise<{ id: string }>
}) {
  const { id } = use(params)
  const reportId = Number(id)

  const {
    report,
    notes,
    labels,
    definitions,
    moderators,
    loading,
    error,
    updateStatus,
    assign,
    escalate,
    addNote,
    applyLabelsToSubject,
    performAccountAction,
  } = useQueueDetail(reportId)

  if (loading) {
    return (
      <>
        <SiteHeader title="Report" backHref="/dashboard/queue" />
        <div className="flex flex-1 items-center justify-center p-6">
          <p className="text-muted-foreground text-sm">Loading report...</p>
        </div>
      </>
    )
  }

  if (error || !report) {
    return (
      <>
        <SiteHeader title="Report" backHref="/dashboard/queue" />
        <div className="flex flex-1 items-center justify-center p-6">
          <p className="text-destructive text-sm">
            {error ?? "Report not found."}
          </p>
        </div>
      </>
    )
  }

  return (
    <>
      <SiteHeader
        title={`Report #${report.id}`}
        backHref="/dashboard/queue"
      />
      <div className="flex flex-1 flex-col lg:flex-row gap-6 p-4 md:p-6">
        {/* Left panel: report details + notes */}
        <div className="flex flex-1 flex-col gap-4 min-w-0">
          <ReportDetail report={report} labels={labels} />
          <NotesThread notes={notes} onAddNote={addNote} />
        </div>

        {/* Right panel: actions */}
        <div className="w-full lg:w-80 shrink-0">
          <div className="lg:sticky lg:top-6">
            <ActionPanel
              report={report}
              definitions={definitions}
              moderators={moderators}
              onUpdateStatus={updateStatus}
              onAssign={assign}
              onEscalate={escalate}
              onApplyLabels={applyLabelsToSubject}
              onAddNote={addNote}
              onAccountAction={performAccountAction}
            />
          </div>
        </div>
      </div>
    </>
  )
}
