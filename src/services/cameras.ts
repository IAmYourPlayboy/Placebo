import { invoke } from '@tauri-apps/api/core';
import type { Camera } from '../types/camera';

export async function getCameras(): Promise<Camera[]> {
  return invoke<Camera[]>('get_cameras');
}

export async function getCameraById(id: string): Promise<Camera | null> {
  return invoke<Camera | null>('get_camera_by_id', { id });
}

export async function getCamerasByCity(city: string): Promise<Camera[]> {
  return invoke<Camera[]>('get_cameras_by_city', { city });
}

export async function searchCameras(query: string): Promise<Camera[]> {
  return invoke<Camera[]>('search_cameras', { query });
}

export async function getNearbyCameras(
  lat: number,
  lng: number,
  radiusKm: number
): Promise<Camera[]> {
  return invoke<Camera[]>('get_nearby_cameras', { lat, lng, radiusKm });
}

export async function getCameraCategories(): Promise<string[]> {
  return invoke<string[]>('get_camera_categories');
}

export async function getCamerasCount(): Promise<number> {
  return invoke<number>('get_cameras_count');
}

export async function seedCameras(): Promise<number> {
  return invoke<number>('seed_cameras');
}
