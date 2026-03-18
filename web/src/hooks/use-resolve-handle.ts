"use client"

import { useEffect, useState } from "react"

const handleCache = new Map<string, string>()
const pendingRequests = new Map<string, Promise<string | null>>()

function resolveHandle(did: string): Promise<string | null> {
  const existing = pendingRequests.get(did)
  if (existing) return existing

  const promise = fetch(`/api/resolve/handle?did=${encodeURIComponent(did)}`)
    .then((res) => (res.ok ? res.json() : null))
    .then((data) => {
      if (!data?.handle) return null
      handleCache.set(did, data.handle)
      return data.handle
    })
    .catch(() => null)
    .finally(() => {
      pendingRequests.delete(did)
    })

  pendingRequests.set(did, promise)
  return promise
}

export function useResolveHandle(did: string | null | undefined): string | null {
  const [handle, setHandle] = useState<string | null>(() => {
    if (!did) return null
    return handleCache.get(did) ?? null
  })

  useEffect(() => {
    if (!did) return
    // If already cached, state was initialized with it
    if (handleCache.has(did)) return
    resolveHandle(did).then((h) => {
      if (h) setHandle(h)
    })
  }, [did])

  return handle
}

export function useResolveHandles(dids: string[]): Map<string, string> {
  const didsKey = dids.join(",")
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
    // If all are cached, state was already initialized correctly
    if (unresolved.length === 0) return

    Promise.all(unresolved.map((did) => resolveHandle(did))).then(() => {
      const all = new Map<string, string>()
      for (const did of dids) {
        const cached = handleCache.get(did)
        if (cached) all.set(did, cached)
      }
      setHandles(all)
    })
  }, [dids, didsKey])

  return handles
}
