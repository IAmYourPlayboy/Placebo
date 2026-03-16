import { useRef, useEffect } from 'react';
import { useThree, useFrame } from '@react-three/fiber';
import * as THREE from 'three';

interface NavigationControlsProps {
  /** Колбэк когда пользователь начинает исследовать 3D-мир */
  onExplorationStart: () => void;
  /** Флаг: пользователь уже в режиме исследования */
  isExploring: boolean;
  /** Скорость перемещения (м/сек) */
  moveSpeed?: number;
  /** Скорость вращения (рад/пиксель) */
  lookSpeed?: number;
}

/**
 * NavigationControls — управление камерой в 3D-мире.
 *
 * Поведение по умолчанию:
 * - Камера смотрит прямо вперёд (на VideoPlane) — как обычный плеер
 * - При зажатии ПКМ или при drag — начинается вращение
 * - WASD / стрелки — перемещение (fly mode)
 * - Колёсико — zoom (приближение/удаление)
 * - Пробел — вернуться к "прямому" виду
 * - ESC — курсор обратно (выход из pointer lock)
 *
 * НЕ используем PointerLockControls из drei — они слишком агрессивны
 * (захватывают курсор сразу). Нам нужен мягкий вход.
 */
export function NavigationControls({
  onExplorationStart,
  isExploring,
  moveSpeed = 30,
  lookSpeed = 0.002,
}: NavigationControlsProps) {
  const { camera, gl } = useThree();
  const keysRef = useRef(new Set<string>());
  const isDraggingRef = useRef(false);
  const eulerRef = useRef(new THREE.Euler(0, 0, 0, 'YXZ'));
  const velocityRef = useRef(new THREE.Vector3());

  // ─── Keyboard ────────────────────────────────────────────

  useEffect(() => {
    const onKeyDown = (e: KeyboardEvent) => {
      keysRef.current.add(e.code);

      // Пробел — вернуть камеру в дефолтное положение
      if (e.code === 'Space') {
        eulerRef.current.set(0, 0, 0);
        camera.quaternion.setFromEuler(eulerRef.current);
      }
    };

    const onKeyUp = (e: KeyboardEvent) => {
      keysRef.current.delete(e.code);
    };

    window.addEventListener('keydown', onKeyDown);
    window.addEventListener('keyup', onKeyUp);
    return () => {
      window.removeEventListener('keydown', onKeyDown);
      window.removeEventListener('keyup', onKeyUp);
    };
  }, [camera]);

  // ─── Mouse ───────────────────────────────────────────────

  useEffect(() => {
    const canvas = gl.domElement;

    const onMouseDown = (e: MouseEvent) => {
      // Любая кнопка мыши — начало вращения (включая ЛКМ для trackpad)
      if (e.button === 0 || e.button === 1 || e.button === 2) {
        isDraggingRef.current = true;
        if (!isExploring) onExplorationStart();
        e.preventDefault();
      }
    };

    const onMouseUp = (e: MouseEvent) => {
      if (e.button === 0 || e.button === 1 || e.button === 2) {
        isDraggingRef.current = false;
      }
    };

    const onMouseMove = (e: MouseEvent) => {
      if (!isDraggingRef.current) return;

      const euler = eulerRef.current;
      euler.y -= e.movementX * lookSpeed;
      euler.x -= e.movementY * lookSpeed;

      // Ограничение вертикального вращения: не дальше 85° вверх/вниз
      euler.x = Math.max(-Math.PI * 0.47, Math.min(Math.PI * 0.47, euler.x));

      camera.quaternion.setFromEuler(euler);
    };

    const onWheel = (e: WheelEvent) => {
      const isPinch = e.ctrlKey; // trackpad pinch gesture
      const speed = isPinch ? 0.5 : 0.1;
      const delta = -e.deltaY * speed;

      const direction = new THREE.Vector3();
      camera.getWorldDirection(direction);
      camera.position.addScaledVector(direction, delta);

      if (!isExploring) onExplorationStart();
      e.preventDefault();
    };

    const onContextMenu = (e: MouseEvent) => {
      e.preventDefault();  // Убираем контекстное меню браузера при ПКМ
    };

    canvas.addEventListener('mousedown', onMouseDown);
    canvas.addEventListener('mouseup', onMouseUp);
    canvas.addEventListener('mousemove', onMouseMove);
    canvas.addEventListener('wheel', onWheel, { passive: false });
    canvas.addEventListener('contextmenu', onContextMenu);

    return () => {
      canvas.removeEventListener('mousedown', onMouseDown);
      canvas.removeEventListener('mouseup', onMouseUp);
      canvas.removeEventListener('mousemove', onMouseMove);
      canvas.removeEventListener('wheel', onWheel);
      canvas.removeEventListener('contextmenu', onContextMenu);
    };
  }, [camera, gl, lookSpeed, isExploring, onExplorationStart]);

  // ─── Update loop ─────────────────────────────────────────

  useFrame((_, delta) => {
    const keys = keysRef.current;
    const velocity = velocityRef.current;

    // Направления движения
    const forward = new THREE.Vector3();
    camera.getWorldDirection(forward);
    forward.y = 0;
    forward.normalize();

    const right = new THREE.Vector3();
    right.crossVectors(forward, camera.up).normalize();

    // Вычисляем целевую скорость
    const targetVelocity = new THREE.Vector3();

    if (keys.has('KeyW') || keys.has('ArrowUp')) targetVelocity.add(forward);
    if (keys.has('KeyS') || keys.has('ArrowDown')) targetVelocity.sub(forward);
    if (keys.has('KeyA') || keys.has('ArrowLeft')) targetVelocity.sub(right);
    if (keys.has('KeyD') || keys.has('ArrowRight')) targetVelocity.add(right);
    if (keys.has('KeyQ')) targetVelocity.y -= 1;  // вниз
    if (keys.has('KeyE')) targetVelocity.y += 1;   // вверх

    if (targetVelocity.lengthSq() > 0) {
      targetVelocity.normalize().multiplyScalar(moveSpeed);
      if (!isExploring) onExplorationStart();
    }

    // Плавное ускорение/торможение (lerp)
    velocity.lerp(targetVelocity, 0.15);

    // Применяем скорость
    if (velocity.lengthSq() > 0.01) {
      camera.position.addScaledVector(velocity, delta);
    }

    // Ограничение: не ниже земли
    if (camera.position.y < 1.5) {
      camera.position.y = 1.5;
    }
  });

  return null;  // Этот компонент невизуальный — только логика
}
