import {
  EffectComposer,
  Bloom,
  ToneMapping,
  Vignette,
} from '@react-three/postprocessing';
import { ToneMappingMode } from 'postprocessing';
import type { PostMode } from '../../../types/world3d';

interface PostEffectsProps {
  mode: PostMode;
}

export default function PostEffects({ mode }: PostEffectsProps) {
  if (mode === 'full') {
    return (
      <EffectComposer>
        <Bloom
          mipmapBlur
          luminanceThreshold={0.9}
          intensity={0.3}
        />
        <ToneMapping mode={ToneMappingMode.ACES_FILMIC} />
        <Vignette offset={0.3} darkness={0.6} />
      </EffectComposer>
    );
  }

  return (
    <EffectComposer>
      <Bloom
        mipmapBlur
        luminanceThreshold={0.9}
        intensity={0.3}
      />
      <ToneMapping mode={ToneMappingMode.ACES_FILMIC} />
    </EffectComposer>
  );
}
