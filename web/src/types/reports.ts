export interface Report {
  id: number;
  subjectUri: string | null;
  subjectCid: string | null;
  subjectDid: string | null;
  reasonType: string;
  reason: string;
  reportedBy: string;
  status: "pending" | "in_review" | "resolved" | "dismissed";
  assignedTo: string | null;
  priority: number;
  autoLabeled: boolean;
  labelCount?: number;
  notes?: ReportNote[];
  labels?: ReportLabel[];
  createdAt: string;
  updatedAt: string;
}

export interface ReportNote {
  id: number;
  author: string;
  content: string;
  createdAt: string;
}

export interface ReportLabel {
  id: number;
  val: string;
  neg: boolean;
  createdAt: string;
}
