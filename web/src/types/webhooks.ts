export interface WebhookSource {
  id: number;
  name: string;
  secret?: string;
  active: boolean;
  auto_accept: boolean;
  auto_label: boolean;
  requires_review: boolean;
  created_at: string;
}
