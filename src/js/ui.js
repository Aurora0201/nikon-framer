import { el } from './elements.js';
import { previewState, updateTransform, fileQueue } from './state.js';
import { checkExif } from './commands.js';

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

export async function renderFileList() {
  const list = el.fileList;
  list.innerHTML = ""; // 清空a

  // 控制空状态提示的显示
  if (fileQueue.files.length === 0) {
    el.emptyTip.style.display = "block";
    list.style.display = "none";
    el.queueCount.innerText = "0 张照片";
    return;
  }

  el.emptyTip.style.display = "none";
  list.style.display = "block";
  el.queueCount.innerText = `${fileQueue.files.length} 张照片`;

  // 遍历生成 DOM
  for (let i = 0; i < fileQueue.files.length; i++) {
    const file = fileQueue.files[i];

    // 如果状态是 wait，异步去检查一下 EXIF
    if (file.exifStatus === 'wait') {
      checkExif(file.path).then(isOk => {
        file.exifStatus = isOk ? 'ok' : 'no';
        updateItemStatus(i, file.exifStatus); // 局部更新 DOM，不重绘整个列表
      });
    }

    const li = document.createElement("li");
    li.className = "file-item";
    li.innerHTML = `
      <div class="file-info">
        <span class="file-name" title="${file.path}">${file.name}</span>
        <span id="exif-tag-${i}" class="tag-exif ${file.exifStatus}">
          ${getExifLabel(file.exifStatus)}
        </span>
      </div>
      <button class="remove-item-btn" data-index="${i}">×</button>
    `;
    list.appendChild(li);
  }

  // 绑定删除按钮事件
  document.querySelectorAll('.remove-item-btn').forEach(btn => {
    btn.addEventListener('click', (e) => {
      const idx = parseInt(e.target.dataset.index);
      fileQueue.remove(idx);
      renderFileList(); // 重新渲染
    });
  });
}

function getExifLabel(status) {
  if (status === 'ok') return 'EXIF';      // 簡單明瞭
  if (status === 'no') return 'NO EXIF';   // 或者用 'PNG' / 'BASIC'
  return 'SCANNING...';
}

function updateItemStatus(index, status) {
  const tag = document.getElementById(`exif-tag-${index}`);
  if (tag) {
    tag.className = `tag-exif ${status}`;
    tag.innerText = getExifLabel(status);
  }
}