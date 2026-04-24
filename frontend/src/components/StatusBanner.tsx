export function StatusBanner({ loading, error }: { loading: string | null; error: string | null }) {
  if (error) {
    return <div className="status-banner status-banner--error">{error}</div>;
  }

  if (loading) {
    return <div className="status-banner">{loading}</div>;
  }

  return null;
}
