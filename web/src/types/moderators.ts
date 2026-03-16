export interface Moderator {
  did: string;
  role: "moderator" | "admin";
  created_at: string;
  last_used_at: string | null;
}
