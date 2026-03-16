"use client"

import { useCallback, useState } from "react"

import type { ReportNote } from "@/types/reports"
import { Button } from "@/components/ui/button"
import {
  Card,
  CardContent,
  CardHeader,
  CardTitle,
} from "@/components/ui/card"
import { Textarea } from "@/components/ui/textarea"
import { Separator } from "@/components/ui/separator"

function formatTimestamp(dateStr: string): string {
  return new Date(dateStr).toLocaleString()
}

function truncateDid(did: string): string {
  if (did.length <= 30) return did
  return did.slice(0, 16) + "..." + did.slice(-10)
}

interface NotesThreadProps {
  notes: ReportNote[]
  onAddNote: (content: string) => Promise<void>
}

export function NotesThread({ notes, onAddNote }: NotesThreadProps) {
  const [content, setContent] = useState("")
  const [submitting, setSubmitting] = useState(false)

  const handleSubmit = useCallback(async () => {
    if (!content.trim()) return
    setSubmitting(true)
    try {
      await onAddNote(content.trim())
      setContent("")
    } finally {
      setSubmitting(false)
    }
  }, [content, onAddNote])

  return (
    <Card>
      <CardHeader>
        <CardTitle className="text-base">Internal Notes</CardTitle>
      </CardHeader>
      <CardContent className="flex flex-col gap-4">
        {notes.length > 0 ? (
          <div className="flex flex-col gap-3">
            {notes.map((note) => (
              <div key={note.id} className="flex flex-col gap-1">
                <div className="flex items-center justify-between gap-2">
                  <span
                    className="font-mono text-xs text-muted-foreground"
                    title={note.author}
                  >
                    {truncateDid(note.author)}
                  </span>
                  <span className="text-muted-foreground text-xs whitespace-nowrap">
                    {formatTimestamp(note.createdAt)}
                  </span>
                </div>
                <p className="text-sm whitespace-pre-wrap">{note.content}</p>
                <Separator className="mt-2" />
              </div>
            ))}
          </div>
        ) : (
          <p className="text-muted-foreground text-sm">No notes yet.</p>
        )}

        <div className="flex flex-col gap-2">
          <Textarea
            value={content}
            onChange={(e) => setContent(e.target.value)}
            placeholder="Add an internal note..."
            rows={3}
          />
          <div className="flex justify-end">
            <Button
              size="sm"
              disabled={!content.trim() || submitting}
              onClick={handleSubmit}
            >
              {submitting ? "Adding..." : "Add Note"}
            </Button>
          </div>
        </div>
      </CardContent>
    </Card>
  )
}
