import { el } from './elements.js';
import { previewState, updateTransform, resetViewState } from './state.js';

export function initPreviewInteraction() {
  el.previewImg.addEventListener("wheel", (e) => {
    e.preventDefault();
    const delta = -e.deltaY;
    const factor = delta > 0 ? 1.1 : 0.9;
    let newScale = previewState.scale * factor;
    if (newScale < 0.1) newScale = 0.1;
    if (newScale > 10) newScale = 10;
    previewState.scale = newScale;
    updateTransform();
  });

  el.previewImg.addEventListener("mousedown", (e) => {
    e.preventDefault();
    previewState.startX = e.clientX - previewState.pointX;
    previewState.startY = e.clientY - previewState.pointY;
    previewState.panning = true;
    el.previewImg.style.cursor = "grabbing";
  });

  window.addEventListener("mousemove", (e) => {
    if (!previewState.panning) return;
    e.preventDefault();
    previewState.pointX = e.clientX - previewState.startX;
    previewState.pointY = e.clientY - previewState.startY;
    updateTransform();
  });

  window.addEventListener("mouseup", () => {
    previewState.panning = false;
    el.previewImg.style.cursor = "grab";
  });

  el.closeBtn.addEventListener("click", resetViewState);
  el.modal.addEventListener("click", (e) => {
    if (e.target === el.modal) resetViewState();
  });
}