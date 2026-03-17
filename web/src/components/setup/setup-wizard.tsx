"use client"

import { useCallback, useEffect, useState } from "react"
import { useAuth } from "@/lib/auth-context"
import { SetupStepper, type SetupStep } from "./setup-stepper"
import { SetupLabelerIdentity } from "./setup-labeler-identity"
import {
  SetupLabelerAuth,
  setupLabelerAuthReturning,
  clearSetupLabelerAuth,
} from "./setup-labeler-auth"
import { SetupPlcUpdate } from "./setup-plc-update"
import { SetupServiceRecord } from "./setup-service-record"
import { SetupComplete } from "./setup-complete"

export function SetupWizard() {
  const { refresh } = useAuth()
  const [currentStep, setCurrentStep] = useState<SetupStep>("identity")
  const [labelerDid, setLabelerDid] = useState<string>("")
  const [loading, setLoading] = useState(true)

  useEffect(() => {
    async function init() {
      // Check if we're returning from a labeler OAuth redirect
      const returning = setupLabelerAuthReturning()
      if (returning) {
        clearSetupLabelerAuth()

        // The cookie currently has the labeler's DID (set by OAuth callback).
        // Call confirm to store labeler DID in backend and restore the original user cookie.
        try {
          const res = await fetch("/api/setup/labeler-auth/confirm", {
            method: "POST",
            credentials: "include",
            headers: { "Content-Type": "application/json" },
            body: JSON.stringify({
              did: returning.labelerDid,
              restore_did: returning.originalUserDid,
            }),
          })

          if (res.ok) {
            // Refresh auth context so it reflects the restored admin session
            await refresh()
            setLabelerDid(returning.labelerDid)
            setCurrentStep("plc")
            setLoading(false)
            return
          }
        } catch {
          // Fall through to normal status check
        }
      }

      // Normal flow: check setup status
      try {
        const res = await fetch("/api/setup/status", {
          credentials: "include",
        })
        if (!res.ok) {
          setLoading(false)
          return
        }

        const data = await res.json()

        if (data.labeler_did) {
          setLabelerDid(data.labeler_did)
        }

        if (data.setup_complete) {
          setCurrentStep("complete")
        } else if (data.service_record_configured) {
          setCurrentStep("complete")
        } else if (data.plc_configured) {
          setCurrentStep("record")
        } else if (data.labeler_did_configured) {
          setCurrentStep("auth")
        }
      } catch {
        // Start from the beginning on error
      } finally {
        setLoading(false)
      }
    }

    init()
  }, [])

  const handleIdentityComplete = useCallback((did: string) => {
    setLabelerDid(did)
    setCurrentStep("auth")
  }, [])

  const handleAuthComplete = useCallback(() => {
    setCurrentStep("plc")
  }, [])

  const handlePlcComplete = useCallback(() => {
    setCurrentStep("record")
  }, [])

  const handleRecordComplete = useCallback(() => {
    setCurrentStep("complete")
  }, [])

  if (loading) {
    return null
  }

  return (
    <div>
      <SetupStepper currentStep={currentStep} />

      {currentStep === "identity" && (
        <SetupLabelerIdentity
          initialDid={labelerDid}
          onComplete={handleIdentityComplete}
        />
      )}

      {currentStep === "auth" && (
        <SetupLabelerAuth
          labelerDid={labelerDid}
          onComplete={handleAuthComplete}
          onSkip={handleAuthComplete}
        />
      )}

      {currentStep === "plc" && (
        <SetupPlcUpdate onComplete={handlePlcComplete} />
      )}

      {currentStep === "record" && (
        <SetupServiceRecord onComplete={handleRecordComplete} />
      )}

      {currentStep === "complete" && <SetupComplete />}
    </div>
  )
}
