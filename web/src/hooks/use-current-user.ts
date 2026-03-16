import { useCallback, useEffect, useState } from "react"

import { useAuth } from "@/lib/auth-context"
import { getModerators } from "@/lib/api"
import type { Moderator } from "@/types/moderators"

export function useCurrentUser() {
  const { did } = useAuth()
  const [currentUser, setCurrentUser] = useState<Moderator | null>(null)

  const load = useCallback(() => {
    getModerators()
      .then((moderators) => setCurrentUser(moderators.find((m) => m.did === did) ?? null))
      .catch(() => setCurrentUser(null))
  }, [did])

  useEffect(() => {
    load()
  }, [load])

  const isAdmin = currentUser?.role === "admin"

  return { currentUser, isAdmin, reload: load }
}
