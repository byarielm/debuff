export interface LabelDefinition {
  id: number;
  identifier: string;
  severity: "inform" | "alert" | "none";
  blurs: "content" | "media" | "none";
  default_setting: "ignore" | "warn" | "hide";
  adult_only: boolean;
  builtin: boolean;
  locales: LabelDefinitionLocale[];
  created_at: string;
}

export interface LabelDefinitionLocale {
  lang: string;
  name: string;
  description: string;
}
