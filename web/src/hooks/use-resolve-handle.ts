"use client"

import { useEffect, useState } from "react"

const handleCache = new Map<string, string>()
const pendingRequests = new Map<string, Promise<string | null>>()

function resolveFromPlc(did: string): Promise<string | null> {
  const existing = pendingRequests.get(did)
  if (existing) return existing

  const promise = fetch(`https://plc.directory/${encodeURIComponent(did)}`)
    .then((res) => (res.ok ? res.json() : null))
    .then((data) => {
      if (!data) return null
      const handle = data.alsoKnownAs
        ?.find((aka: string) => aka.startsWith("at://"))
        ?.replace("at://", "")
      if (handle) {
        handleCache.set(did, handle)
        return handle
      }
      return null
    })
    .catch(() => null)
    .finally(() => {
      pendingRequests.delete(did)
    })

  pendingRequests.set(did, promise)
  return promise
}

export function useResolveHandle(did: string | null | undefined): string | null {
  const cached = did ? handleCache.get(did) ?? null : null
  const [handle, setHandle] = useState<string | null>(cached)

  useEffect(() => {
    if (!did) return
    const cachedHandle = handleCache.get(did)
    if (cachedHandle) {
      setHandle(cachedHandle)
      return
    }
    resolveFromPlc(did).then((h) => {
      if (h) setHandle(h)
    })
  }, [did])

  return handle
}

export function useResolveHandles(dids: string[]): Map<string, string> {
  const [handles, setHandles] = useState<Map<string, string>>(() => {
    const initial = new Map<string, string>()
    for (const did of dids) {
      const cached = handleCache.get(did)
      if (cached) initial.set(did, cached)
    }
    return initial
  })

  useEffect(() => {
    const unresolved = dids.filter((did) => !handleCache.has(did))
    if (unresolved.length === 0) {
      // All cached, sync state
      const all = new Map<string, string>()
      for (const did of dids) {
        const cached = handleCache.get(did)
        if (cached) all.set(did, cached)
      }
      setHandles(all)
      return
    }

    Promise.all(unresolved.map((did) => resolveFromPlc(did))).then(() => {
      const all = new Map<string, string>()
      for (const did of dids) {
        const cached = handleCache.get(did)
        if (cached) all.set(did, cached)
      }
      setHandles(all)
    })
  }, [dids.join(",")])

  return handles
}
