# Moderator Assignment System Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Make moderator assignment functional by adding queue filtering, backend validation, and cleanup on moderator removal.

**Architecture:** Four independent changes — (1) relax moderator list permissions so non-admins can populate the filter, (2) add an "Assigned To" filter to the queue page that passes `assigned_to` to the existing backend param (plus a new `unassigned` special value), (3) fix the "Assign to Me" bulk action to send the real DID, (4) add backend validation on assignment and cleanup on moderator deletion.

**Tech Stack:** Rust/axum backend, Next.js/React frontend, TanStack Table, shadcn/ui, sqlx/PostgreSQL

---

### Task 1: Relax `list_moderators` Permissions

The `list_moderators` endpoint currently requires admin role. Non-admin moderators need access to populate the assignment filter dropdown and the action panel's assign dropdown. Allow any authenticated moderator to list moderators.

**Files:**
- Modify: `src/api/moderators.rs:48-54`

**Step 1: Remove the admin check from `list_moderators`**

In `src/api/moderators.rs`, the `list_moderators` function has:

```rust
pub(super) async fn list_moderators(
    State(state): State<AppState>,
    auth: ModeratorAuth,
) -> Result<Json<Vec<ModeratorSummary>>, AppError> {
    if auth.role != "admin" {
        return Err(AppError::Forbidden);
    }
```

Remove the `if auth.role != "admin"` block. The `ModeratorAuth` extractor already ensures only authenticated moderators can access it. Change `auth` to `_auth` since the binding is no longer used.

```rust
pub(super) async fn list_moderators(
    State(state): State<AppState>,
    _auth: ModeratorAuth,
) -> Result<Json<Vec<ModeratorSummary>>, AppError> {
```

**Step 2: Verify it compiles**

Run: `cargo check` from `/Users/trezy/Development/clients/atproto/debuff`
Expected: compiles with no errors

**Step 3: Commit**

```bash
git add src/api/moderators.rs
git commit -m "feat: allow all moderators to list moderators"
```

---

### Task 2: Support `unassigned` Filter on Backend

The `list_queue` endpoint's `assigned_to` filter only matches exact values. We need a way to filter for reports with no assignee (NULL). Use the special value `__unassigned__` to mean `assigned_to IS NULL`.

**Files:**
- Modify: `src/api/queue.rs:151-154`

**Step 1: Update the `assigned_to` filter condition**

In `src/api/queue.rs`, replace the `assigned_to` filter block (lines 151-154):

```rust
if params.assigned_to.is_some() {
    bind_idx += 1;
    conditions.push(format!("r.assigned_to = ${bind_idx}"));
}
```

With:

```rust
if let Some(ref assigned_to) = params.assigned_to {
    if assigned_to == "__unassigned__" {
        conditions.push("r.assigned_to IS NULL".to_string());
    } else {
        bind_idx += 1;
        conditions.push(format!("r.assigned_to = ${bind_idx}"));
    }
}
```

And update the bind section (around line 213) from:

```rust
if let Some(ref assigned_to) = params.assigned_to {
    query = query.bind(assigned_to);
}
```

To:

```rust
if let Some(ref assigned_to) = params.assigned_to {
    if assigned_to != "__unassigned__" {
        query = query.bind(assigned_to);
    }
}
```

**Step 2: Verify it compiles**

Run: `cargo check` from `/Users/trezy/Development/clients/atproto/debuff`
Expected: compiles with no errors

**Step 3: Commit**

```bash
git add src/api/queue.rs
git commit -m "feat: support __unassigned__ filter for queue assigned_to param"
```

---

### Task 3: Add `assigned_to` Param to Frontend `getReports`

The `getReports` API function doesn't pass `assigned_to` to the backend.

**Files:**
- Modify: `web/src/lib/api.ts:56-65`

**Step 1: Add `assigned_to` to the params**

Change the `getReports` function from:

```typescript
export function getReports(
  params?: { status?: string; cursor?: string; limit?: number }
) {
  const searchParams = new URLSearchParams()
  if (params?.status) searchParams.set("status", params.status)
  if (params?.cursor) searchParams.set("cursor", params.cursor)
  if (params?.limit) searchParams.set("limit", String(params.limit))
```

To:

```typescript
export function getReports(
  params?: { status?: string; assigned_to?: string; cursor?: string; limit?: number }
) {
  const searchParams = new URLSearchParams()
  if (params?.status) searchParams.set("status", params.status)
  if (params?.assigned_to) searchParams.set("assigned_to", params.assigned_to)
  if (params?.cursor) searchParams.set("cursor", params.cursor)
  if (params?.limit) searchParams.set("limit", String(params.limit))
```

**Step 2: Commit**

```bash
git add web/src/lib/api.ts
git commit -m "feat: add assigned_to param to getReports API function"
```

---

### Task 4: Wire `assigned_to` Filter Through `useQueueList`

The `useQueueList` hook defines `assigned_to` in its filter interface but never passes it to `getReports`.

**Files:**
- Modify: `web/src/hooks/use-queue.ts:52-73`

**Step 1: Pass `assigned_to` to `getReports`**

In the `fetchPage` callback, change:

```typescript
const data = await getReports({
  status: filters.status,
  cursor,
  limit: 25,
})
```

To:

```typescript
const data = await getReports({
  status: filters.status,
  assigned_to: filters.assigned_to,
  cursor,
  limit: 25,
})
```

And update the dependency array of the `useCallback` from `[filters.status]` to `[filters.status, filters.assigned_to]`.

**Step 2: Commit**

```bash
git add web/src/hooks/use-queue.ts
git commit -m "feat: pass assigned_to filter to getReports in useQueueList"
```

---

### Task 5: Add Assigned To Filter Dropdown to Queue Page

Add a second `Select` dropdown next to the status filter.

**Files:**
- Modify: `web/src/app/dashboard/queue/page.tsx`

**Step 1: Add moderator fetching and the filter dropdown**

Add imports at the top:

```typescript
import { useEffect, useState as useStateReact } from "react"
import { getModerators } from "@/lib/api"
import { useAuth } from "@/lib/auth-context"
import type { Moderator } from "@/types/moderators"
```

Note: `useState` is already imported from React, and `useCallback` too. We need `useEffect` added to the existing import. Also import `getModerators`, `useAuth`, and the `Moderator` type.

Inside `QueuePage`, add state and effect to fetch moderators:

```typescript
const { did } = useAuth()
const [moderators, setModerators] = useState<Moderator[]>([])

useEffect(() => {
  getModerators().then(setModerators).catch(() => {})
}, [])
```

Note: `useState` is already available since it's used for `rowSelection` (currently imported as `useState` from `react` — but actually it's not imported, `useCallback` and `useState` come from the page's own imports). Check the imports — currently the page imports `{ useCallback, useState }` from `"react"`. We need to add `useEffect` to that import.

Add the assigned-to filter `Select` next to the status filter, inside the `<div className="flex items-center gap-2">` on the left side of the toolbar:

```tsx
<Select
  value={filters.assigned_to ?? "all"}
  onValueChange={(value) =>
    setFilters({
      ...filters,
      assigned_to: value === "all" ? undefined : value,
    })
  }
>
  <SelectTrigger className="h-8 w-48 text-sm">
    <SelectValue placeholder="Filter by assignee" />
  </SelectTrigger>
  <SelectContent>
    <SelectItem value="all">All Assignees</SelectItem>
    <SelectItem value="__mine__">Assigned to Me</SelectItem>
    <SelectItem value="__unassigned__">Unassigned</SelectItem>
    {moderators.map((mod) => (
      <SelectItem key={mod.did} value={mod.did}>
        {mod.did}
      </SelectItem>
    ))}
  </SelectContent>
</Select>
```

**Step 2: Handle the `__mine__` value in the hook**

The `__mine__` value needs to be resolved to the user's DID before being sent to the API. The simplest approach: resolve it in the `setFilters` call on the page. When `__mine__` is selected, replace it with `did` from `useAuth()`:

```typescript
onValueChange={(value) =>
  setFilters({
    ...filters,
    assigned_to: value === "all"
      ? undefined
      : value === "__mine__"
        ? did ?? undefined
        : value,
  })
}
```

But keep the Select's `value` prop showing `__mine__` when the filter matches the user's DID. Update the `value` prop:

```typescript
value={
  filters.assigned_to === undefined
    ? "all"
    : filters.assigned_to === did
      ? "__mine__"
      : filters.assigned_to
}
```

**Step 3: Commit**

```bash
git add web/src/app/dashboard/queue/page.tsx
git commit -m "feat: add assigned-to filter dropdown to queue page"
```

---

### Task 6: Fix "Assign to Me" Bulk Action

The bulk action sends the literal string `"self"` instead of the moderator's DID.

**Files:**
- Modify: `web/src/app/dashboard/queue/page.tsx:132-136`

**Step 1: Replace `"self"` with the user's DID**

The `useAuth` hook is already imported from Task 5. The `did` is already available. Change:

```tsx
<DropdownMenuItem
  onClick={() => handleBulkAssign("self")}
>
  Assign to Me
</DropdownMenuItem>
```

To:

```tsx
<DropdownMenuItem
  onClick={() => did && handleBulkAssign(did)}
  disabled={!did}
>
  Assign to Me
</DropdownMenuItem>
```

**Step 2: Commit**

```bash
git add web/src/app/dashboard/queue/page.tsx
git commit -m "fix: send actual DID instead of 'self' in bulk assign"
```

---

### Task 7: Validate `assigned_to` DID on Backend

When assigning a moderator to a report, verify the DID exists in the `moderators` table.

**Files:**
- Modify: `src/api/queue.rs:386-410`

**Step 1: Add validation to `assign_moderator`**

In the `assign_moderator` function, add a check before the UPDATE. Change:

```rust
pub async fn assign_moderator(
    State(state): State<AppState>,
    _auth: ModeratorAuth,
    Path(id): Path<i64>,
    Json(body): Json<AssignBody>,
) -> Result<Json<serde_json::Value>, AppError> {
    let result = sqlx::query(
```

To:

```rust
pub async fn assign_moderator(
    State(state): State<AppState>,
    _auth: ModeratorAuth,
    Path(id): Path<i64>,
    Json(body): Json<AssignBody>,
) -> Result<Json<serde_json::Value>, AppError> {
    // Validate that the DID belongs to a known moderator
    if let Some(ref did) = body.did {
        let exists: Option<(String,)> =
            sqlx::query_as("SELECT did FROM moderators WHERE did = $1")
                .bind(did)
                .fetch_optional(&state.db)
                .await
                .map_err(|e| AppError::Internal(format!("failed to check moderator: {e}")))?;

        if exists.is_none() {
            return Err(AppError::BadRequest(format!(
                "DID '{}' is not a known moderator",
                did
            )));
        }
    }

    let result = sqlx::query(
```

**Step 2: Verify it compiles**

Run: `cargo check` from `/Users/trezy/Development/clients/atproto/debuff`
Expected: compiles with no errors

**Step 3: Commit**

```bash
git add src/api/queue.rs
git commit -m "feat: validate assigned_to DID exists in moderators table"
```

---

### Task 8: Clean Up Assignments on Moderator Removal

When a moderator is deleted, null out `assigned_to` on all reports assigned to them.

**Files:**
- Modify: `src/api/moderators.rs:82-110`

**Step 1: Add cleanup query before the DELETE**

In the `remove_moderator` function, add a query to clear assignments after verifying the moderator exists but before (or after) deleting them. Add it after the DELETE succeeds:

Change:

```rust
let result = sqlx::query("DELETE FROM moderators WHERE did = $1")
    .bind(&did)
    .execute(&state.db)
    .await
    .map_err(|e| AppError::Internal(format!("failed to remove moderator: {e}")))?;

if result.rows_affected() == 0 {
    return Err(AppError::NotFound);
}

Ok(StatusCode::NO_CONTENT)
```

To:

```rust
let result = sqlx::query("DELETE FROM moderators WHERE did = $1")
    .bind(&did)
    .execute(&state.db)
    .await
    .map_err(|e| AppError::Internal(format!("failed to remove moderator: {e}")))?;

if result.rows_affected() == 0 {
    return Err(AppError::NotFound);
}

// Clear assignments for the removed moderator
sqlx::query("UPDATE reports SET assigned_to = NULL WHERE assigned_to = $1")
    .bind(&did)
    .execute(&state.db)
    .await
    .map_err(|e| AppError::Internal(format!("failed to clear assignments: {e}")))?;

Ok(StatusCode::NO_CONTENT)
```

**Step 2: Verify it compiles**

Run: `cargo check` from `/Users/trezy/Development/clients/atproto/debuff`
Expected: compiles with no errors

**Step 3: Commit**

```bash
git add src/api/moderators.rs
git commit -m "feat: clear report assignments when moderator is removed"
```
