"use client"

import { useCallback, useEffect, useState } from "react"
import { Trash2 } from "lucide-react"

import { useCurrentUser } from "@/hooks/use-current-user"
import {
  getModerators,
  addModerator,
  deleteModerator,
} from "@/lib/api"
import type { Moderator } from "@/types/moderators"
import { SiteHeader } from "@/components/site-header"
import { Button } from "@/components/ui/button"
import { Badge } from "@/components/ui/badge"
import { Input } from "@/components/ui/input"
import { Label } from "@/components/ui/label"
import {
  Dialog,
  DialogClose,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
  DialogTrigger,
} from "@/components/ui/dialog"
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select"
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table"

export default function ModeratorsSettingsPage() {
  const { isAdmin } = useCurrentUser()
  const [moderators, setModerators] = useState<Moderator[]>([])
  const [handles, setHandles] = useState<Record<string, string>>({})
  const [error, setError] = useState<string | null>(null)
  const [deleteDid, setDeleteDid] = useState<string | null>(null)
  const [deleting, setDeleting] = useState(false)

  const load = useCallback(() => {
    getModerators()
      .then(setModerators)
      .catch((e) => setError(e instanceof Error ? e.message : String(e)))
  }, [])

  useEffect(() => {
    load()
  }, [load])

  // Resolve DIDs to handles via PLC directory
  useEffect(() => {
    const newDids = moderators
      .map((m) => m.did)
      .filter((did) => !(did in handles))
    if (newDids.length === 0) return
    for (const did of newDids) {
      fetch(`https://plc.directory/${encodeURIComponent(did)}`)
        .then((res) => (res.ok ? res.json() : null))
        .then((data) => {
          if (!data) return
          const handle = data.alsoKnownAs
            ?.find((aka: string) => aka.startsWith("at://"))
            ?.replace("at://", "")
          if (handle) {
            setHandles((prev) => ({ ...prev, [did]: handle }))
          }
        })
        .catch(() => {})
    }
  }, [moderators, handles])

  async function handleDelete(did: string) {
    setDeleting(true)
    try {
      await deleteModerator(did)
      setDeleteDid(null)
      load()
    } catch (e: unknown) {
      setError(e instanceof Error ? e.message : String(e))
    } finally {
      setDeleting(false)
    }
  }

  function roleBadgeClass(role: string) {
    switch (role) {
      case "admin":
        return "bg-purple-100 text-purple-800 dark:bg-purple-900 dark:text-purple-200"
      default:
        return "bg-blue-100 text-blue-800 dark:bg-blue-900 dark:text-blue-200"
    }
  }

  return (
    <>
      <SiteHeader title="Moderators" />
      <div className="flex flex-1 flex-col gap-4 p-4 md:p-6">
        {error && <p className="text-destructive text-sm">{error}</p>}

        <div className="flex items-center justify-between">
          <div>
            <h2 className="text-lg font-semibold">Moderators</h2>
            <p className="text-muted-foreground text-sm">
              Manage who can moderate content on your labeler.
            </p>
          </div>
          {isAdmin && (
            <AddModeratorDialog onSuccess={load} />
          )}
        </div>

        <div className="overflow-clip rounded-lg border">
          <Table>
            <TableHeader>
              <TableRow>
                <TableHead>User</TableHead>
                <TableHead>Role</TableHead>
                <TableHead>Last Active</TableHead>
                <TableHead>Created</TableHead>
                <TableHead className="w-20 sticky right-0 bg-inherit z-[1]" />
              </TableRow>
            </TableHeader>
            <TableBody>
              {moderators.length === 0 && (
                <TableRow>
                  <TableCell
                    colSpan={5}
                    className="text-muted-foreground text-center"
                  >
                    No moderators yet.
                  </TableCell>
                </TableRow>
              )}
              {moderators.map((mod) => (
                <TableRow key={mod.did}>
                  <TableCell>
                    <div className="flex flex-col">
                      {handles[mod.did] && (
                        <span className="font-medium">@{handles[mod.did]}</span>
                      )}
                      <span className="font-mono text-muted-foreground text-xs">
                        {mod.did}
                      </span>
                    </div>
                  </TableCell>
                  <TableCell>
                    <Badge className={roleBadgeClass(mod.role)}>
                      {mod.role}
                    </Badge>
                  </TableCell>
                  <TableCell className="text-sm">
                    {mod.last_used_at
                      ? new Date(mod.last_used_at).toLocaleString()
                      : "Never"}
                  </TableCell>
                  <TableCell className="text-sm">
                    {new Date(mod.created_at).toLocaleString()}
                  </TableCell>
                  <TableCell className="w-20 sticky right-0 bg-inherit z-[1]">
                    {isAdmin && (
                      <Button
                        variant="ghost"
                        size="icon"
                        className="size-8 text-muted-foreground hover:text-destructive"
                        title="Remove moderator"
                        aria-label="Remove moderator"
                        onClick={() => setDeleteDid(mod.did)}
                      >
                        <Trash2 className="size-4" />
                      </Button>
                    )}
                  </TableCell>
                </TableRow>
              ))}
            </TableBody>
          </Table>
        </div>
      </div>

      <Dialog
        open={!!deleteDid}
        onOpenChange={(open) => {
          if (!open) setDeleteDid(null)
        }}
      >
        <DialogContent>
          <DialogHeader>
            <DialogTitle>Remove moderator?</DialogTitle>
            <DialogDescription>
              This will revoke this moderator&apos;s access. They will no longer
              be able to review reports or apply labels.
            </DialogDescription>
          </DialogHeader>
          {deleteDid && (
            <code className="text-muted-foreground block truncate text-xs">
              {handles[deleteDid] ? `@${handles[deleteDid]} (${deleteDid})` : deleteDid}
            </code>
          )}
          <DialogFooter>
            <DialogClose asChild>
              <Button variant="outline" disabled={deleting}>
                Cancel
              </Button>
            </DialogClose>
            <Button
              variant="destructive"
              disabled={deleting}
              onClick={() => {
                if (deleteDid) handleDelete(deleteDid)
              }}
            >
              {deleting ? "Removing..." : "Remove"}
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </>
  )
}

function AddModeratorDialog({
  onSuccess,
}: {
  onSuccess: () => void
}) {
  const [did, setDid] = useState("")
  const [role, setRole] = useState<"moderator" | "admin">("moderator")
  const [error, setError] = useState<string | null>(null)
  const [open, setOpen] = useState(false)

  async function handleAdd() {
    setError(null)
    try {
      await addModerator({ did, role })
      setDid("")
      setRole("moderator")
      setOpen(false)
      onSuccess()
    } catch (e: unknown) {
      setError(e instanceof Error ? e.message : String(e))
    }
  }

  return (
    <Dialog
      open={open}
      onOpenChange={(o) => {
        setOpen(o)
        if (o) {
          setDid("")
          setRole("moderator")
          setError(null)
        }
      }}
    >
      <DialogTrigger asChild>
        <Button>Add Moderator</Button>
      </DialogTrigger>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>Add Moderator</DialogTitle>
          <DialogDescription>
            Add a new moderator by entering their DID and assigning a role.
          </DialogDescription>
        </DialogHeader>
        <div className="flex flex-col gap-4">
          {error && <p className="text-destructive text-sm">{error}</p>}
          <div className="flex flex-col gap-2">
            <Label htmlFor="mod-did">DID</Label>
            <Input
              id="mod-did"
              value={did}
              onChange={(e) => setDid(e.target.value)}
              placeholder="did:plc:..."
              className="font-mono"
            />
          </div>
          <div className="flex flex-col gap-2">
            <Label htmlFor="mod-role">Role</Label>
            <Select value={role} onValueChange={(v) => setRole(v as typeof role)}>
              <SelectTrigger id="mod-role">
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="moderator">Moderator</SelectItem>
                <SelectItem value="admin">Admin</SelectItem>
              </SelectContent>
            </Select>
          </div>
        </div>
        <DialogFooter>
          <DialogClose asChild>
            <Button variant="outline">Cancel</Button>
          </DialogClose>
          <Button onClick={handleAdd} disabled={!did.trim()}>
            Add
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
}
