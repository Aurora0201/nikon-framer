import { el } from './elements.js';

export const previewState = {
  scale: 1, panning: false, pointX: 0, pointY: 0, startX: 0, startY: 0,
};

export function resetViewState() {
  previewState.scale = 1;
  previewState.panning = false;
  previewState.pointX = 0;
  previewState.pointY = 0;
  updateTransform();
  el.modal.style.display = "none";
  el.previewImg.src = "";
}

export function updateTransform() {
  el.previewImg.style.transform = `translate(${previewState.pointX}px, ${previewState.pointY}px) scale(${previewState.scale})`;
}