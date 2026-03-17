"use client"

import { useState } from "react"
import { Button } from "@/components/ui/button"
import { Input } from "@/components/ui/input"
import { Label } from "@/components/ui/label"
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card"

interface SetupPlcUpdateProps {
  onComplete: () => void
}

export function SetupPlcUpdate({ onComplete }: SetupPlcUpdateProps) {
  const [phase, setPhase] = useState<"request" | "confirm">("request")
  const [token, setToken] = useState("")
  const [loading, setLoading] = useState(false)
  const [error, setError] = useState<string | null>(null)

  async function handleRequestCode() {
    setLoading(true)
    setError(null)

    try {
      const res = await fetch("/api/setup/plc/request", {
        method: "POST",
        credentials: "include",
      })

      if (!res.ok) {
        const text = await res.text()
        throw new Error(text || "Failed to send confirmation code")
      }

      setPhase("confirm")
    } catch (e) {
      setError(
        e instanceof Error ? e.message : "Failed to send confirmation code"
      )
    } finally {
      setLoading(false)
    }
  }

  async function handleSubmitToken(e: React.FormEvent) {
    e.preventDefault()
    if (!token.trim()) return

    setLoading(true)
    setError(null)

    try {
      const res = await fetch("/api/setup/plc/submit", {
        method: "POST",
        credentials: "include",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ token: token.trim() }),
      })

      if (!res.ok) {
        const text = await res.text()
        // Surface user-friendly messages for known errors
        if (text.includes("ExpiredToken") || text.includes("expired")) {
          throw new Error("Your confirmation code has expired. Please request a new one.")
        }
        if (text.includes("InvalidToken") || text.includes("invalid")) {
          throw new Error("Invalid confirmation code. Please check the code and try again.")
        }
        throw new Error(text || "Failed to update DID document")
      }

      onComplete()
    } catch (e) {
      setError(
        e instanceof Error ? e.message : "Failed to update DID document"
      )
    } finally {
      setLoading(false)
    }
  }

  async function handleRequestNewCode() {
    setLoading(true)
    setError(null)
    setToken("")

    try {
      const res = await fetch("/api/setup/plc/request", {
        method: "POST",
        credentials: "include",
      })

      if (!res.ok) {
        const text = await res.text()
        throw new Error(text || "Failed to send confirmation code")
      }

      setError(null)
    } catch (e) {
      setError(
        e instanceof Error ? e.message : "Failed to send confirmation code"
      )
    } finally {
      setLoading(false)
    }
  }

  return (
    <Card>
      <CardHeader>
        <CardTitle>Update DID Document</CardTitle>
        <CardDescription>
          {phase === "request"
            ? "Your labeler's DID document needs to be updated to include the signing key and service endpoint. A confirmation code will be sent to the email associated with the labeler account."
            : "A confirmation code has been sent to the email associated with the labeler account. Enter the code below to apply the DID document update."}
        </CardDescription>
      </CardHeader>
      <CardContent>
        <div className="flex flex-col gap-4">
          {error && <p className="text-destructive text-sm">{error}</p>}

          {phase === "request" ? (
            <>
              <Button onClick={handleRequestCode} disabled={loading}>
                {loading ? "Sending..." : "Send Confirmation Code"}
              </Button>
              <button
                type="button"
                className="text-muted-foreground hover:text-foreground text-sm underline"
                onClick={() => setPhase("confirm")}
                disabled={loading}
              >
                I already have a code
              </button>
            </>
          ) : (
            <form onSubmit={handleSubmitToken} className="flex flex-col gap-4">
              <div className="grid gap-2">
                <Label htmlFor="token">Confirmation Code</Label>
                <Input
                  id="token"
                  type="text"
                  placeholder="Enter the code from your email"
                  value={token}
                  onChange={(e) => setToken(e.target.value)}
                  required
                  disabled={loading}
                />
              </div>
              <Button type="submit" disabled={loading}>
                {loading ? "Submitting..." : "Confirm Update"}
              </Button>
              <button
                type="button"
                className="text-muted-foreground hover:text-foreground text-sm underline"
                onClick={handleRequestNewCode}
                disabled={loading}
              >
                Request a new code
              </button>
            </form>
          )}
        </div>
      </CardContent>
    </Card>
  )
}
