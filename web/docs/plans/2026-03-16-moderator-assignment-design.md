# Moderator Assignment System Design

**Goal:** Make moderator assignment functional by adding queue filtering, input validation, and cleanup on moderator removal.

**Constraints:** Assignment remains organizational — all moderators can see and act on all reports regardless of assignment. No notifications.

---

## Changes

### 1. Queue Filter

Add an "Assigned To" filter dropdown to the queue page toolbar. Options:

- **All** (default) — no filter applied
- **Assigned to me** — filters to current user's DID
- **Unassigned** — filters to reports with no assignee
- **Each moderator** — listed by DID (fetched via `GET /api/moderators`)

The filter passes the `assigned_to` query parameter to the existing backend `list_queue` endpoint, which already supports this param but the frontend never used it.

### 2. Fix "Assign to Me" Bulk Action

The bulk action currently sends the literal string `"self"` as the `assigned_to` value. Change the frontend to resolve the current user's DID from `useAuth()` and send that instead.

### 3. Backend Validation on Assignment

When `PATCH /api/queue/:id/assign` is called, validate that the `assigned_to` DID exists in the `moderators` table. Return 400 if the DID is not a known moderator. Null values (unassign) are always allowed.

### 4. Cleanup on Moderator Removal

When `DELETE /api/moderators/:did` is called, null out `assigned_to` on all reports currently assigned to that moderator. This prevents orphaned assignments.
