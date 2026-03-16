const REASON_DESCRIPTIONS: Record<string, string> = {
  // com.atproto.moderation.defs tokens
  "com.atproto.moderation.defs#reasonSpam":
    "Spam: frequent unwanted promotion, replies, mentions",
  "com.atproto.moderation.defs#reasonViolation":
    "Direct violation of server rules, laws, terms of service",
  "com.atproto.moderation.defs#reasonMisleading":
    "Misleading identity, affiliation, or content",
  "com.atproto.moderation.defs#reasonSexual":
    "Unwanted or mislabeled sexual content",
  "com.atproto.moderation.defs#reasonRude":
    "Rude, harassing, explicit, or otherwise unwelcoming behavior",
  "com.atproto.moderation.defs#reasonOther":
    "Reports not falling under another report category",
  "com.atproto.moderation.defs#reasonAppeal":
    "Appeal a previously taken moderation action",

  // tools.ozone.report.defs duplicates of legacy
  "tools.ozone.report.defs#reasonAppeal":
    "Appeal a previously taken moderation action",
  "tools.ozone.report.defs#reasonOther":
    "Reports not falling under another report category",
}

export function formatReasonType(reasonType: string): string {
  if (REASON_DESCRIPTIONS[reasonType]) {
    return REASON_DESCRIPTIONS[reasonType]
  }

  // For tools.ozone.report.defs subcategories, parse the token name
  // e.g. "tools.ozone.report.defs#reasonViolenceAnimal" → "Violence — Animal"
  const token = reasonType.split("#").pop()
  if (!token) return reasonType

  // Strip "reason" prefix
  const name = token.replace(/^reason/, "")
  if (!name) return reasonType

  // Split on camelCase boundaries: "ViolenceAnimal" → ["Violence", "Animal"]
  const parts = name.replace(/([a-z])([A-Z])/g, "$1 $2").split(" ")

  if (parts.length >= 2) {
    const category = parts[0]
    const sub = parts.slice(1).join(" ")
    return `${category} — ${sub}`
  }

  return parts[0]
}
