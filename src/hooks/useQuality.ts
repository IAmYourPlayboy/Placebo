import { createContext, useContext } from 'react';
import { QualityConfig, DEFAULT_QUALITY } from '../types/world3d';

export const QualityContext = createContext<QualityConfig>(DEFAULT_QUALITY);

export function useQuality(): QualityConfig {
  return useContext(QualityContext);
}
