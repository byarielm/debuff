"use client"

import { useRouter } from "next/navigation"
import { CheckCircle2 } from "lucide-react"
import { Button } from "@/components/ui/button"
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card"

export function SetupComplete() {
  const router = useRouter()

  return (
    <Card>
      <CardHeader className="justify-items-center text-center">
        <CheckCircle2 className="mb-2 size-12 text-green-500" />
        <CardTitle>Setup Complete</CardTitle>
        <CardDescription>
          Your labeler service is configured and ready to go. You can now
          configure your first label or head to the dashboard to start
          moderating.
        </CardDescription>
      </CardHeader>
      <CardContent>
        <div className="flex flex-col gap-3">
          <Button onClick={() => router.push("/dashboard/settings/labels/new")}>
            Configure Your First Label
          </Button>
          <Button
            variant="outline"
            onClick={() => router.push("/dashboard/queue")}
          >
            Go to Dashboard
          </Button>
        </div>
      </CardContent>
    </Card>
  )
}
