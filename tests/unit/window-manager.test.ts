import { describe, it, expect } from 'vitest';
import { computeSnapBounds } from '../../src/renderer/state/windowStore';

describe('Window Manager & Snapping Geometry', () => {
  const container = { width: 1920, height: 1080 };

  it('computes Left Half snapping bounds correctly', () => {
    const bounds = computeSnapBounds('left-half', container);
    expect(bounds).toEqual({
      x: 0,
      y: 0,
      width: 960,
      height: 1080,
    });
  });

  it('computes Right Half snapping bounds correctly', () => {
    const bounds = computeSnapBounds('right-half', container);
    expect(bounds).toEqual({
      x: 960,
      y: 0,
      width: 960,
      height: 1080,
    });
  });

  it('computes Top-Left quadrant bounds correctly', () => {
    const bounds = computeSnapBounds('top-left', container);
    expect(bounds).toEqual({
      x: 0,
      y: 0,
      width: 960,
      height: 540,
    });
  });

  it('computes Bottom-Right quadrant bounds correctly', () => {
    const bounds = computeSnapBounds('bottom-right', container);
    expect(bounds).toEqual({
      x: 960,
      y: 540,
      width: 960,
      height: 540,
    });
  });

  it('computes Center Third bounds correctly', () => {
    const bounds = computeSnapBounds('center-third', container);
    expect(bounds).toEqual({
      x: 640,
      y: 0,
      width: 640,
      height: 1080,
    });
  });

  it('computes Maximize bounds filling full container', () => {
    const bounds = computeSnapBounds('maximize', container);
    expect(bounds).toEqual({
      x: 0,
      y: 0,
      width: 1920,
      height: 1080,
    });
  });
});

