import { lazy, Suspense } from 'react';
import { useQuality } from '../../../hooks/useQuality';

const PostEffects = lazy(() => import('./PostEffects'));

export function PostStack() {
  const { post } = useQuality();

  if (post.mode === 'none') return null;

  return (
    <Suspense fallback={null}>
      <PostEffects mode={post.mode} />
    </Suspense>
  );
}
