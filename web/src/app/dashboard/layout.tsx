"use client"

import { useEffect, useState } from "react"
import { useRouter } from "next/navigation"

import { useAuth } from "@/lib/auth-context"
import { AppSidebar } from "@/components/app-sidebar"
import { SidebarInset, SidebarProvider } from "@/components/ui/sidebar"

export default function DashboardLayout({
  children,
}: {
  children: React.ReactNode
}) {
  const { did } = useAuth()
  const router = useRouter()
  const [setupChecked, setSetupChecked] = useState(false)

  useEffect(() => {
    if (!did) {
      router.replace("/login")
      return
    }

    async function checkSetup() {
      try {
        const res = await fetch("/api/setup/status", {
          credentials: "include",
        })
        if (res.ok) {
          const data = await res.json()
          if (!data.setup_complete) {
            router.replace("/setup")
            return
          }
        }
      } catch {
        // If setup check fails, allow dashboard access
      }
      setSetupChecked(true)
    }

    checkSetup()
  }, [did, router])

  if (!did || !setupChecked) return null

  return (
    <SidebarProvider
      style={
        {
          "--sidebar-width": "calc(var(--spacing) * 72)",
          "--header-height": "calc(var(--spacing) * 12)",
        } as React.CSSProperties
      }
    >
      <AppSidebar variant="inset" />
      <SidebarInset>{children}</SidebarInset>
    </SidebarProvider>
  )
}
