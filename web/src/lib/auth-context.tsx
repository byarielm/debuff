"use client"

import {
  createContext,
  useCallback,
  useContext,
  useEffect,
  useState,
} from "react"

interface AuthContextType {
  did: string | null
  role: string | null
  login: (handle: string) => Promise<void>
  logout: () => Promise<void>
  refresh: () => Promise<void>
  loading: boolean
  error: string | null
}

const AuthContext = createContext<AuthContextType>({
  did: null,
  role: null,
  login: async () => {},
  logout: async () => {},
  refresh: async () => {},
  loading: true,
  error: null,
})

export function AuthProvider({ children }: { children: React.ReactNode }) {
  const [did, setDid] = useState<string | null>(null)
  const [role, setRole] = useState<string | null>(null)
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)

  useEffect(() => {
    let cancelled = false

    async function checkSession() {
      try {
        const res = await fetch("/auth/me", { credentials: "include" })
        if (res.ok) {
          const data = await res.json()
          if (!cancelled) {
            setDid(data.did)
            setRole(data.role ?? null)
          }
        }
      } catch (e) {
        if (!cancelled) {
          console.error("Session check failed:", e)
        }
      } finally {
        if (!cancelled) setLoading(false)
      }
    }

    checkSession()
    return () => { cancelled = true }
  }, [])

  const login = useCallback(async (handle: string) => {
    setError(null)
    try {
      const res = await fetch(`/auth/login?handle=${encodeURIComponent(handle)}`, {
        credentials: "include",
      })
      if (!res.ok) {
        const text = await res.text()
        throw new Error(text || "Login failed")
      }
      const data = await res.json()
      window.location.href = data.url
    } catch (e) {
      setError(e instanceof Error ? e.message : "Login failed")
      throw e
    }
  }, [])

  const refresh = useCallback(async () => {
    try {
      const res = await fetch("/auth/me", { credentials: "include" })
      if (res.ok) {
        const data = await res.json()
        setDid(data.did)
        setRole(data.role ?? null)
      }
    } catch {
      // Best-effort
    }
  }, [])

  const logout = useCallback(async () => {
    try {
      await fetch("/auth/logout", {
        method: "POST",
        credentials: "include",
      })
    } catch {
      // Best-effort
    }
    setDid(null)
    setRole(null)
  }, [])

  if (loading) return null

  return (
    <AuthContext.Provider
      value={{
        did,
        role,
        login,
        logout,
        refresh,
        loading,
        error,
      }}
    >
      {children}
    </AuthContext.Provider>
  )
}

export function useAuth() {
  return useContext(AuthContext)
}
