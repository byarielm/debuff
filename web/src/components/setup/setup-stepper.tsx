"use client"

import {
  Stepper,
  StepperDescription,
  StepperIndicator,
  StepperItem,
  StepperList,
  StepperSeparator,
  StepperTitle,
  StepperTrigger,
} from "@/components/ui/stepper"

export type SetupStep = "identity" | "auth" | "plc" | "record" | "complete"

const STEPS: { value: SetupStep; title: string; description: string }[] = [
  { value: "identity", title: "Identity", description: "Labeler identity" },
  { value: "auth", title: "Auth", description: "Authenticate labeler" },
  { value: "plc", title: "DID Update", description: "Update DID document" },
  { value: "record", title: "Service Record", description: "Configure record" },
  { value: "complete", title: "Done", description: "Setup complete" },
]

interface SetupStepperProps {
  currentStep: SetupStep
}

export function SetupStepper({ currentStep }: SetupStepperProps) {
  return (
    <div className="mb-8">
      <Stepper value={currentStep}>
        <StepperList>
          {STEPS.map((step, index) => (
            <StepperItem key={step.value} value={step.value}>
              <StepperTrigger>
                <StepperIndicator>{index + 1}</StepperIndicator>
                <div className="hidden flex-col gap-0.5 whitespace-nowrap sm:flex">
                  <StepperTitle>{step.title}</StepperTitle>
                  <StepperDescription className="hidden lg:block">
                    {step.description}
                  </StepperDescription>
                </div>
              </StepperTrigger>
              <StepperSeparator className="mx-4" />
            </StepperItem>
          ))}
        </StepperList>
      </Stepper>
    </div>
  )
}
