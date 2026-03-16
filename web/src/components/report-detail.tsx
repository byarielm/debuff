"use client"

import type { Report } from "@/types/reports"
import type { Label } from "@/types/labels"
import { Badge } from "@/components/ui/badge"
import { JsonViewer } from "@/components/json-viewer"
import { ResolvedHandle } from "@/components/resolved-handle"
import { formatReasonType } from "@/lib/reason-types"
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card"

function StatusBadge({ status }: { status: Report["status"] }) {
  const variants: Record<Report["status"], string> = {
    pending: "bg-yellow-100 text-yellow-800 border-yellow-300",
    in_review: "bg-blue-100 text-blue-800 border-blue-300",
    resolved: "bg-green-100 text-green-800 border-green-300",
    dismissed: "bg-neutral-100 text-neutral-600 border-neutral-300",
  }

  const labels: Record<Report["status"], string> = {
    pending: "Pending",
    in_review: "In Review",
    resolved: "Resolved",
    dismissed: "Dismissed",
  }

  return (
    <Badge variant="outline" className={variants[status]}>
      {labels[status]}
    </Badge>
  )
}

interface ReportDetailProps {
  report: Report
  labels: Label[]
  record?: unknown
}

function parseAtUri(uri: string): { did: string; collection: string; rkey: string } | null {
  const match = uri.match(/^at:\/\/(did:[^/]+)\/([^/]+)\/(.+)$/)
  if (!match) return null
  return { did: match[1], collection: match[2], rkey: match[3] }
}

export function ReportDetail({ report, labels, record }: ReportDetailProps) {
  const subject = report.subjectUri ?? report.subjectDid ?? "Unknown"
  const parsed = report.subjectUri ? parseAtUri(report.subjectUri) : null

  return (
    <div className="flex flex-col gap-4">
      <Card>
        <CardHeader>
          <div className="flex items-center justify-between gap-2">
            <CardTitle>Report #{report.id}</CardTitle>
            <div className="flex items-center gap-2">
              {report.autoLabeled && (
                <Badge
                  variant="outline"
                  className="bg-purple-100 text-purple-800 border-purple-300"
                >
                  Auto-labeled
                </Badge>
              )}
              <StatusBadge status={report.status} />
            </div>
          </div>
          <CardDescription className="font-mono text-xs break-all">
            {subject}
          </CardDescription>
        </CardHeader>
        <CardContent>
          <dl className="grid grid-cols-[auto_1fr] gap-x-4 gap-y-2 text-sm">
            <dt className="text-muted-foreground font-medium">Reason Type</dt>
            <dd>{formatReasonType(report.reasonType)}</dd>

            <dt className="text-muted-foreground font-medium">Reason</dt>
            <dd className="whitespace-pre-wrap">{report.reason || "-"}</dd>

            <dt className="text-muted-foreground font-medium">Reporter</dt>
            <dd className="text-xs break-all">
              <ResolvedHandle did={report.reportedBy} />
            </dd>

            {(report.subjectDid || parsed?.did) && (
              <>
                <dt className="text-muted-foreground font-medium">Record Owner</dt>
                <dd className="text-xs break-all">
                  <ResolvedHandle did={(parsed?.did ?? report.subjectDid)!} />
                </dd>
              </>
            )}

            {parsed?.collection && (
              <>
                <dt className="text-muted-foreground font-medium">Collection</dt>
                <dd className="font-mono text-xs">{parsed.collection}</dd>
              </>
            )}

            {report.subjectCid && (
              <>
                <dt className="text-muted-foreground font-medium">Subject CID</dt>
                <dd className="font-mono text-xs break-all">{report.subjectCid}</dd>
              </>
            )}

            <dt className="text-muted-foreground font-medium">Priority</dt>
            <dd>{report.priority}</dd>

            <dt className="text-muted-foreground font-medium">Assigned To</dt>
            <dd className="text-xs break-all">
              {report.assignedTo ? (
                <ResolvedHandle did={report.assignedTo} />
              ) : (
                "Unassigned"
              )}
            </dd>

            <dt className="text-muted-foreground font-medium">Created</dt>
            <dd>{new Date(report.createdAt).toLocaleString()}</dd>

            <dt className="text-muted-foreground font-medium">Updated</dt>
            <dd>{new Date(report.updatedAt).toLocaleString()}</dd>
          </dl>
        </CardContent>
      </Card>

      {labels.length > 0 && (
        <Card>
          <CardHeader>
            <CardTitle className="text-base">Existing Labels</CardTitle>
          </CardHeader>
          <CardContent>
            <div className="flex flex-wrap gap-1.5">
              {labels.map((label) => (
                <Badge
                  key={`${label.src}-${label.val}-${label.cts}`}
                  variant={label.neg ? "outline" : "secondary"}
                  title={`Source: ${label.src}`}
                >
                  {label.neg && "~"}
                  {label.val}
                </Badge>
              ))}
            </div>
          </CardContent>
        </Card>
      )}

      <Card>
        <CardHeader>
          <CardTitle className="text-base">Record Preview</CardTitle>
        </CardHeader>
        <CardContent>
          <JsonViewer
            data={record ?? {
              id: report.id,
              subjectUri: report.subjectUri,
              subjectCid: report.subjectCid,
              subjectDid: report.subjectDid,
              reasonType: report.reasonType,
              reason: report.reason,
              reportedBy: report.reportedBy,
              status: report.status,
              priority: report.priority,
              autoLabeled: report.autoLabeled,
              createdAt: report.createdAt,
            }}
            className="overflow-auto rounded-md [&_pre]:p-4 [&_pre]:text-xs [&_pre]:leading-relaxed"
          />
        </CardContent>
      </Card>
    </div>
  )
}
