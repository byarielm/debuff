import AccountDetail from "./account-detail"

// https://github.com/vercel/next.js/issues/71862
// Returning [] fails with output:"export", so provide a dummy param.
export async function generateStaticParams() {
  return [{ did: "_" }]
}

export default function AccountPage() {
  return <AccountDetail />
}
