import type { Report, ReportNote } from "@/types/reports"
import type { Label } from "@/types/labels"
import type { LabelDefinition } from "@/types/definitions"
import type { WebhookSource } from "@/types/webhooks"
import type { Moderator } from "@/types/moderators"

export type { Report, ReportNote } from "@/types/reports"
export type { Label } from "@/types/labels"
export type { LabelDefinition, LabelDefinitionLocale } from "@/types/definitions"
export type { WebhookSource } from "@/types/webhooks"
export type { Moderator } from "@/types/moderators"

export class ApiError extends Error {
  status: number
  constructor(status: number, message: string) {
    super(message)
    this.status = status
  }
}

async function apiFetch<T = unknown>(
  path: string,
  options?: RequestInit,
): Promise<T> {
  const headers: Record<string, string> = {}
  if (
    options?.method === "POST" ||
    options?.method === "PUT" ||
    options?.method === "PATCH"
  ) {
    headers["Content-Type"] = "application/json"
  }

  const res = await fetch(path, {
    ...options,
    headers: { ...headers, ...options?.headers },
    credentials: "include",
  })

  if (!res.ok) {
    const text = await res.text().catch(() => res.statusText)
    throw new ApiError(res.status, text)
  }
  if (res.status === 204) return null as T
  const text = await res.text()
  if (!text) return null as T
  return JSON.parse(text)
}

// Reports / Queue
export interface QueueListResponse {
  items: Report[]
  cursor: string | null
}

export function getReports(
  params?: { status?: string; assigned_to?: string; cursor?: string; limit?: number }
) {
  const searchParams = new URLSearchParams()
  if (params?.status) searchParams.set("status", params.status)
  if (params?.assigned_to) searchParams.set("assigned_to", params.assigned_to)
  if (params?.cursor) searchParams.set("cursor", params.cursor)
  if (params?.limit) searchParams.set("limit", String(params.limit))
  const qs = searchParams.toString()
  return apiFetch<QueueListResponse>(`/api/queue${qs ? `?${qs}` : ""}`)
}

export function getReport(id: number) {
  return apiFetch<Report>(`/api/queue/${id}`)
}

export function updateReport(
  id: number,
  body: { status?: string; assigned_to?: string | null; priority?: number }
) {
  return apiFetch<Report>(`/api/queue/${id}`, {
    method: "PATCH",
    body: JSON.stringify(body),
  })
}

export function getReportNotes(reportId: number) {
  return apiFetch<ReportNote[]>(`/api/queue/${reportId}/notes`)
}

export function addReportNote(
  reportId: number,
  body: { content: string }
) {
  return apiFetch<ReportNote>(`/api/queue/${reportId}/notes`, {
    method: "POST",
    body: JSON.stringify(body),
  })
}

export function escalateReport(id: number) {
  return apiFetch<Report>(`/api/queue/${id}/escalate`, {
    method: "POST",
  })
}

export function assignReport(
  id: number,
  body: { did: string | null }
) {
  return apiFetch<Report>(`/api/queue/${id}/assign`, {
    method: "PATCH",
    body: JSON.stringify(body),
  })
}

// Labels
export function getLabels(
  params?: { uri?: string; cursor?: string; limit?: number }
) {
  const searchParams = new URLSearchParams()
  if (params?.uri) searchParams.set("uri", params.uri)
  if (params?.cursor) searchParams.set("cursor", params.cursor)
  if (params?.limit) searchParams.set("limit", String(params.limit))
  const qs = searchParams.toString()
  return apiFetch<Label[]>(`/api/labels${qs ? `?${qs}` : ""}`)
}

export function applyLabels(
  body: { uri: string; cid?: string; val: string[] }
) {
  return apiFetch<Label[]>("/api/labels", {
    method: "POST",
    body: JSON.stringify(body),
  })
}

// Label Definitions
export function getLabelDefinitions() {
  return apiFetch<LabelDefinition[]>("/api/definitions")
}

export function getLabelDefinition(id: number) {
  return apiFetch<LabelDefinition>(`/api/definitions/${id}`)
}

export function createLabelDefinition(
  body: Omit<LabelDefinition, "id" | "created_at">
) {
  return apiFetch<LabelDefinition>("/api/definitions", {
    method: "POST",
    body: JSON.stringify(body),
  })
}

export function updateLabelDefinition(
  id: number,
  body: Partial<Omit<LabelDefinition, "id" | "created_at">>
) {
  return apiFetch<LabelDefinition>(`/api/definitions/${id}`, {
    method: "PATCH",
    body: JSON.stringify(body),
  })
}

export function deleteLabelDefinition(id: number) {
  return apiFetch(`/api/definitions/${id}`, {
    method: "DELETE",
  })
}

// Webhook Sources
export function getWebhookSources() {
  return apiFetch<WebhookSource[]>("/api/webhooks")
}

export function createWebhookSource(
  body: { name: string; auto_accept?: boolean; auto_label?: boolean; requires_review?: boolean }
) {
  return apiFetch<WebhookSource>("/api/webhooks", {
    method: "POST",
    body: JSON.stringify(body),
  })
}

export function updateWebhookSource(
  id: number,
  body: Partial<Pick<WebhookSource, "name" | "active" | "auto_accept" | "auto_label" | "requires_review">>
) {
  return apiFetch<WebhookSource>(`/api/webhooks/${id}`, {
    method: "PATCH",
    body: JSON.stringify(body),
  })
}

export function deleteWebhookSource(id: number) {
  return apiFetch(`/api/webhooks/${id}`, {
    method: "DELETE",
  })
}

// Moderators
export function getModerators() {
  return apiFetch<Moderator[]>("/api/moderators")
}

export function addModerator(
  body: { did: string; role: "moderator" | "admin" }
) {
  return apiFetch<Moderator>("/api/moderators", {
    method: "POST",
    body: JSON.stringify(body),
  })
}

export function updateModerator(
  did: string,
  body: { role: "moderator" | "admin" }
) {
  return apiFetch<Moderator>(`/api/moderators/${encodeURIComponent(did)}`, {
    method: "PATCH",
    body: JSON.stringify(body),
  })
}

export function deleteModerator(did: string) {
  return apiFetch(`/api/moderators/${encodeURIComponent(did)}`, {
    method: "DELETE",
  })
}

// Accounts
export interface AccountInfo {
  did: string
  handle: string | null
  labels: Label[]
}

export function getAccount(did: string) {
  return apiFetch<AccountInfo>(`/api/accounts/${encodeURIComponent(did)}`)
}

export function getAccountLabels(did: string) {
  return apiFetch<Label[]>(`/api/labels?uri=${encodeURIComponent(`did:${did}`)}`)
}

export function accountAction(
  did: string,
  body: { action: string; negate?: boolean }
) {
  return apiFetch<Label>(`/api/accounts/${encodeURIComponent(did)}/action`, {
    method: "POST",
    body: JSON.stringify(body),
  })
}
