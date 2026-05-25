/**
 * Stub. Portal-based 3D canvas virtualization is a deferred alpha
 * decision (alpha-design.md §10): each World3DScreen owns its own
 * R3F <Canvas> directly, and this component intentionally renders
 * nothing. Revisit in M7+ if multiple concurrent 3D scenes become a
 * memory issue.
 */
export default function GlobalCanvas() {
  return null;
}
