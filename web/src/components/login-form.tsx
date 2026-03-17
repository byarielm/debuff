"use client"

import { useCallback, useEffect, useRef, useState } from "react"

import { cn } from "@/lib/utils"
import { useAuth } from "@/lib/auth-context"
import { Avatar, AvatarFallback, AvatarImage } from "@/components/ui/avatar"
import { Button } from "@/components/ui/button"
import { Input } from "@/components/ui/input"
import { Label } from "@/components/ui/label"

interface Actor {
  did: string
  handle: string
  displayName?: string
  avatar?: string
}

export function LoginForm({
  className,
  ...props
}: React.ComponentProps<"div">) {
  const [handle, setHandle] = useState("")
  const [loading, setLoading] = useState(false)
  const [error, setError] = useState<string | null>(null)
  const [suggestions, setSuggestions] = useState<Actor[]>([])
  const [showSuggestions, setShowSuggestions] = useState(false)
  const [selectedIndex, setSelectedIndex] = useState(-1)
  const debounceRef = useRef<ReturnType<typeof setTimeout> | null>(null)
  const containerRef = useRef<HTMLDivElement>(null)
  const { login } = useAuth()

  // Close suggestions on outside click
  useEffect(() => {
    function handleClickOutside(e: MouseEvent) {
      if (
        containerRef.current &&
        !containerRef.current.contains(e.target as Node)
      ) {
        setShowSuggestions(false)
      }
    }
    document.addEventListener("mousedown", handleClickOutside)
    return () => document.removeEventListener("mousedown", handleClickOutside)
  }, [])

  const searchActors = useCallback(async (query: string) => {
    if (query.length < 2 || query.startsWith("did:")) {
      setSuggestions([])
      return
    }

    try {
      const res = await fetch(
        `/api/resolve/search?q=${encodeURIComponent(query)}&limit=6`
      )
      if (res.ok) {
        const data = await res.json()
        setSuggestions(data.actors ?? [])
        setShowSuggestions(true)
        setSelectedIndex(-1)
      }
    } catch {
      // Silently fail — typeahead is optional
    }
  }, [])

  function handleInputChange(value: string) {
    setHandle(value)

    if (debounceRef.current) clearTimeout(debounceRef.current)
    debounceRef.current = setTimeout(() => searchActors(value), 250)
  }

  function selectActor(actor: Actor) {
    setHandle(actor.handle)
    setShowSuggestions(false)
    setSuggestions([])
  }

  function handleKeyDown(e: React.KeyboardEvent) {
    if (!showSuggestions || suggestions.length === 0) return

    if (e.key === "ArrowDown") {
      e.preventDefault()
      setSelectedIndex((i) => Math.min(i + 1, suggestions.length - 1))
    } else if (e.key === "ArrowUp") {
      e.preventDefault()
      setSelectedIndex((i) => Math.max(i - 1, 0))
    } else if (e.key === "Enter" && selectedIndex >= 0) {
      e.preventDefault()
      selectActor(suggestions[selectedIndex])
    } else if (e.key === "Escape") {
      setShowSuggestions(false)
    }
  }

  async function handleSubmit(e: React.FormEvent) {
    e.preventDefault()
    if (!handle.trim()) return
    setLoading(true)
    setError(null)
    try {
      await login(handle.trim())
    } catch (e: unknown) {
      setError(e instanceof Error ? e.message : "Login failed")
      setLoading(false)
    }
  }

  return (
    <div className={cn("flex flex-col gap-6", className)} {...props}>
      <form onSubmit={handleSubmit}>
        <div className="flex flex-col gap-6">
          <div className="flex flex-col items-center gap-2 text-center">
            <h1 className="text-xl font-bold">Debuff Admin</h1>
            <p className="text-muted-foreground text-sm">
              Sign in with your ATProto account to manage moderation.
            </p>
          </div>
          {error && (
            <p className="text-destructive text-center text-sm">{error}</p>
          )}
          <div className="grid gap-3" ref={containerRef}>
            <Label htmlFor="handle">Handle</Label>
            <div className="relative">
              <Input
                id="handle"
                type="text"
                placeholder="you.bsky.social"
                value={handle}
                onChange={(e) => handleInputChange(e.target.value)}
                onFocus={() => {
                  if (suggestions.length > 0) setShowSuggestions(true)
                }}
                onKeyDown={handleKeyDown}
                autoComplete="off"
                required
                disabled={loading}
              />
              {showSuggestions && suggestions.length > 0 && (
                <div className="absolute z-50 mt-1 w-full rounded-md border bg-popover shadow-md">
                  {suggestions.map((actor, index) => (
                    <button
                      key={actor.did}
                      type="button"
                      className={`flex w-full items-center gap-3 px-3 py-2 text-left text-sm transition-colors hover:bg-accent ${
                        index === selectedIndex ? "bg-accent" : ""
                      } ${index === 0 ? "rounded-t-md" : ""} ${
                        index === suggestions.length - 1 ? "rounded-b-md" : ""
                      }`}
                      onMouseDown={(e) => {
                        e.preventDefault()
                        selectActor(actor)
                      }}
                    >
                      <Avatar size="sm">
                        {actor.avatar && (
                          <AvatarImage src={actor.avatar} alt="" />
                        )}
                        <AvatarFallback>
                          {(actor.displayName || actor.handle)
                            .charAt(0)
                            .toUpperCase()}
                        </AvatarFallback>
                      </Avatar>
                      <div className="min-w-0 flex-1">
                        {actor.displayName && (
                          <p className="truncate font-medium">
                            {actor.displayName}
                          </p>
                        )}
                        <p className="text-muted-foreground truncate text-xs">
                          @{actor.handle}
                        </p>
                      </div>
                    </button>
                  ))}
                </div>
              )}
            </div>
          </div>
          <Button type="submit" className="w-full" disabled={loading}>
            {loading ? "Signing in..." : "Sign in"}
          </Button>
        </div>
      </form>
    </div>
  )
}
