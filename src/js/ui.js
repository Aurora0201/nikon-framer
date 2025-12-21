import { el } from './elements.js';
import { previewState, updateTransform } from './state.js';

export function setStatus(text, type = "normal") {
  el.status.innerText = text;
  if (type === "error") el.status.style.color = "#ff4444";
  else if (type === "success") el.status.style.color = "#4caf50";
  else if (type === "loading") el.status.style.color = "#FF9800"; 
  else el.status.style.color = "#333";
}

export function toggleLoading(isLoading) {
  if (isLoading) {
    el.loadingSpinner.style.display = "block";
    el.btn.disabled = true;
    if(el.debugShadowBtn) el.debugShadowBtn.disabled = true;
    if(el.debugWeightBtn) el.debugWeightBtn.disabled = true;
  } else {
    el.loadingSpinner.style.display = "none";
    el.btn.disabled = false;
    if(el.debugShadowBtn) el.debugShadowBtn.disabled = false;
    if(el.debugWeightBtn) el.debugWeightBtn.disabled = false;
  }
}

export function showPreview(base64Data, defaultScale = 1.0) {
    previewState.scale = defaultScale;
    previewState.pointX = 0;
    previewState.pointY = 0;
    updateTransform();
    el.previewImg.src = base64Data; 
    el.modal.style.display = "flex";
}

// 🟢 [新增] 控制阴影滑块显示/隐藏的逻辑
function updateShadowVisibility() {
    const currentStyle = el.styleSelect.value;
    if (currentStyle === "GaussianBlur") {
        el.shadowControlGroup.style.display = "block";
    } else {
        el.shadowControlGroup.style.display = "none";
    }
}

// 🟢 [修改] 初始化 UI 监听
export function initUIEvents() {
    // 1. 滑块数值显示
    el.shadowInput.addEventListener("input", (e) => {
        el.shadowValDisplay.innerText = e.target.value;
    });

    // 2. 监听样式选择变化
    el.styleSelect.addEventListener("change", () => {
        updateShadowVisibility();
    });

    // 3. 初始化时执行一次检查 (设置默认状态)
    updateShadowVisibility();
}