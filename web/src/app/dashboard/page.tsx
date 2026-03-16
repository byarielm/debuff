"use client"

import { useEffect } from "react"
import { useRouter } from "next/navigation"

export default function DashboardRedirect() {
  const router = useRouter()
  useEffect(() => {
    router.replace("/dashboard/queue")
  }, [router])
  return null
}
