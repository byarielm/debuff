"use client"

import { useEffect, useState } from "react"
import { Loader2 } from "lucide-react"
import { useAuth } from "@/lib/auth-context"
import { Button } from "@/components/ui/button"
import { Avatar, AvatarFallback, AvatarImage } from "@/components/ui/avatar"
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card"

const STORAGE_KEY = "debuff_setup_labeler_auth"
const APPVIEW_URL = "https://public.api.bsky.app"

interface LabelerProfile {
  did: string
  handle: string
  displayName?: string
  avatar?: string
}

interface SetupLabelerAuthProps {
  labelerDid: string
  onComplete: () => void
  onSkip: () => void
}

export function setupLabelerAuthReturning(): {
  labelerDid: string
  originalUserDid: string
} | null {
  try {
    const raw = localStorage.getItem(STORAGE_KEY)
    if (!raw) return null
    const data = JSON.parse(raw)
    if (data.labelerDid && data.originalUserDid) {
      return data
    }
  } catch {
    // ignore
  }
  return null
}

export function clearSetupLabelerAuth() {
  localStorage.removeItem(STORAGE_KEY)
}

export function SetupLabelerAuth({
  labelerDid,
  onComplete,
  onSkip,
}: SetupLabelerAuthProps) {
  const { did: userDid } = useAuth()
  const [loading, setLoading] = useState(false)
  const [error, setError] = useState<string | null>(null)
  const [profile, setProfile] = useState<LabelerProfile | null>(null)

  // Auto-skip if the logged-in user IS the labeler
  useEffect(() => {
    if (userDid && labelerDid && userDid === labelerDid) {
      onSkip()
    }
  }, [userDid, labelerDid, onSkip])

  // Fetch the labeler's profile from the public AppView
  useEffect(() => {
    if (!labelerDid) return

    async function fetchProfile() {
      try {
        const res = await fetch(
          `${APPVIEW_URL}/xrpc/app.bsky.actor.getProfile?actor=${encodeURIComponent(labelerDid)}`
        )
        if (res.ok) {
          const data = await res.json()
          setProfile({
            did: data.did,
            handle: data.handle,
            displayName: data.displayName,
            avatar: data.avatar,
          })
        }
      } catch {
        // Profile fetch is best-effort
      }
    }

    fetchProfile()
  }, [labelerDid])

  async function handleStartAuth() {
    setLoading(true)
    setError(null)

    try {
      const res = await fetch("/api/setup/labeler-auth", {
        method: "POST",
        credentials: "include",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ handle: profile?.handle || labelerDid }),
      })

      if (!res.ok) {
        const text = await res.text()
        throw new Error(text || "Failed to start labeler authentication")
      }

      const data = await res.json()

      // Save state before navigating away
      localStorage.setItem(
        STORAGE_KEY,
        JSON.stringify({
          labelerDid,
          originalUserDid: userDid,
        })
      )

      // Navigate the main window to OAuth (will redirect back after auth)
      window.location.href = data.url
    } catch (e) {
      setError(
        e instanceof Error
          ? e.message
          : "Failed to start labeler authentication"
      )
      setLoading(false)
    }
  }

  if (userDid === labelerDid) return null

  return (
    <Card>
      <CardHeader>
        <CardTitle>Authenticate Labeler Account</CardTitle>
        <CardDescription>
          The labeler account is different from your admin account. You need to
          authenticate the labeler account so Debuff can update its identity and
          create the service record. You will be redirected to sign in and then
          brought back here.
        </CardDescription>
      </CardHeader>
      <CardContent>
        <div className="flex flex-col gap-4">
          {error && <p className="text-destructive text-sm">{error}</p>}

          <div className="flex items-center gap-3 rounded-md border p-3">
            {profile ? (
              <>
                <Avatar>
                  {profile.avatar && (
                    <AvatarImage src={profile.avatar} alt="" />
                  )}
                  <AvatarFallback>
                    {(profile.displayName || profile.handle)
                      .charAt(0)
                      .toUpperCase()}
                  </AvatarFallback>
                </Avatar>
                <div className="min-w-0 flex-1">
                  {profile.displayName && (
                    <p className="truncate font-medium">
                      {profile.displayName}
                    </p>
                  )}
                  <p className="text-muted-foreground truncate text-sm">
                    @{profile.handle}
                  </p>
                  <p className="text-muted-foreground truncate font-mono text-xs">
                    {profile.did}
                  </p>
                </div>
              </>
            ) : (
              <>
                <Avatar>
                  <AvatarFallback>
                    <Loader2 className="size-4 animate-spin" />
                  </AvatarFallback>
                </Avatar>
                <div className="min-w-0 flex-1">
                  <p className="text-muted-foreground truncate font-mono text-sm">
                    {labelerDid}
                  </p>
                </div>
              </>
            )}
          </div>

          <Button onClick={handleStartAuth} disabled={loading}>
            {loading ? "Redirecting..." : "Authenticate Labeler Account"}
          </Button>
        </div>
      </CardContent>
    </Card>
  )
}
