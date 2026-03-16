"use client"

import { useEffect, useState } from "react"
import { codeToHtml } from "shiki"

interface JsonViewerProps {
  data: unknown
  className?: string
}

export function JsonViewer({ data, className }: JsonViewerProps) {
  const [html, setHtml] = useState<string>("")
  const json = JSON.stringify(data, null, 2)

  useEffect(() => {
    let cancelled = false
    codeToHtml(json, {
      lang: "json",
      themes: {
        light: "github-light",
        dark: "github-dark",
      },
    }).then((result) => {
      if (!cancelled) setHtml(result)
    })
    return () => { cancelled = true }
  }, [json])

  return (
    <div
      className={className}
      dangerouslySetInnerHTML={
        html
          ? { __html: html }
          : { __html: `<pre class="p-4 text-xs leading-relaxed overflow-auto"><code>${escapeHtml(json)}</code></pre>` }
      }
    />
  )
}

function escapeHtml(str: string): string {
  return str
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;")
}
