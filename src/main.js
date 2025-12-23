import { open } from '@tauri-apps/plugin-dialog';
import { listen } from '@tauri-apps/api/event';
import { invoke } from '@tauri-apps/api/core';

import { el } from './js/elements.js';
// processImage 在单张处理时可能还会用到，但在批处理中逻辑已移至 Rust
import { processImage, debugShadowGrid, debugWeightGrid } from './js/commands.js';
import { loadFonts, initFontEvents } from './js/fonts.js';
import { initPreviewInteraction } from './js/preview.js';
import { initUIEvents, setStatus, toggleLoading, showPreview, renderFileList, updateProgress } from './js/ui.js';
import { fileQueue } from './js/state.js';

// 🔴 关键：彻底屏蔽浏览器的默认拖拽行为
document.addEventListener('dragover', e => e.preventDefault());
document.addEventListener('drop', e => e.preventDefault());

// --- 🟢 全局状态控制 ---
let isProcessing = false; // 是否正在批处理中
let canStop = false;      // 是否允许点击停止 (3秒防误触)
let stopTimer = null;     // 计时器引用

// 初始化
window.addEventListener("DOMContentLoaded", async () => {
  loadFonts();
  initFontEvents();
  initPreviewInteraction();
  initUIEvents();
  renderFileList();

  // ==========================================
  // 🟢 1. 注册 Rust 事件监听 (新增部分)
  // ==========================================
  console.log("🚀 注册事件监听...");

  // 监听进度更新
  await listen('process-progress', (event) => {
    const { current, total, filepath, status } = event.payload;
    
    // 更新状态栏文案
    if (status === 'skipped') {
      setStatus(`[${current}/${total}] 跳过(无EXIF): ${filepath}`, "loading");
    } else {
      setStatus(`[${current}/${total}] 正在处理: ${filepath}`, "loading");
    }

    // 💡 这里可以扩展真正的进度条 UI
    updateProgress(current, total); 
  });

  // 监听状态改变 (完成或停止)
  await listen('process-status', (event) => {
    const status = event.payload; // 'finished' | 'stopped'
    
    if (status === 'finished') {
      setStatus(`批处理完成！`, "success");
      resetBatchState();
    } else if (status === 'stopped') {
      setStatus("已终止批处理", "error");
      resetBatchState();
    }
  });

  // ==========================================
  // 🟢 2. 原有的 Drag & Drop 逻辑 (保持不变)
  // ==========================================
  const unlistenDrop = await listen('tauri://drag-drop', async (event) => {
    // 如果正在处理中，禁止拖入新文件
    if (isProcessing) return;

    const paths = event.payload.paths;
    console.log("拖入路径:", paths);

    if (paths && paths.length > 0) {
      if (el.dropZone) {
        try {
          const validFiles = await invoke('filter_files', { paths });
          if (validFiles.length > 0) {
            const hasNew = fileQueue.add(validFiles);
            if (hasNew) {
              renderFileList();
              if (validFiles.length < paths.length) {
                setStatus(`已添加 ${validFiles.length} 个文件 (已忽略文件夹)`, "success");
              } else {
                setStatus(`已添加 ${validFiles.length} 个文件`, "success");
              }
            } else {
              setStatus("文件已存在列表中", "normal");
            }
          } else {
            setStatus("未检测到图片文件 (文件夹已忽略)", "loading");
            setTimeout(() => setStatus("请拖入具体的图片文件", "normal"), 2000);
          }
        } catch (error) {
          console.error("文件过滤失败:", error);
          setStatus("文件读取错误", "error");
        }
      }
    }
    if(el.dropZone) el.dropZone.classList.remove('active');
  });

  const unlistenHover = await listen('tauri://drag-enter', (event) => {
    if (isProcessing) return;
    if(el.dropZone) el.dropZone.classList.add('active');
  });

  const unlistenCancel = await listen('tauri://drag-leave', (event) => {
    if(el.dropZone) el.dropZone.classList.remove('active');
  });

  console.log("✅ Tauri Listen 监听已注册");
});


// ==========================================
// 🟢 按钮逻辑
// ==========================================

// 辅助函数：重置 UI 状态
function resetBatchState() {
  isProcessing = false;
  canStop = false;
  if (stopTimer) clearTimeout(stopTimer);
  
  // 恢复按钮文字
  if (el.startBatchBtn) {
    el.startBatchBtn.textContent = "开始批处理";
    el.startBatchBtn.classList.remove("stop-mode"); // 可以加个红色样式类
    el.startBatchBtn.style.opacity = "1";
    el.startBatchBtn.style.cursor = "pointer";
  }

  // 恢复其他 UI 交互
  toggleLoading(false); 
  
  // 需求2：恢复列表移除功能
  // 假设 renderFileList 内部会根据 isProcessing 状态渲染删除按钮，或者这里手动移除禁用类
  if (el.fileList) el.fileList.classList.remove("disabled-interaction");
}


// 按钮：添加文件 (处理中禁用)
if (el.addFilesBtn) {
  el.addFilesBtn.addEventListener("click", async () => {
    if (isProcessing) return; // 🔒
    try {
      const selected = await open({
        multiple: true,
        filters: [{ name: 'Images', extensions: ['jpg', 'jpeg', 'png', 'nef', 'dng', 'arw'] }]
      });
      if (selected) {
        fileQueue.add(selected);
        renderFileList();
      }
    } catch (err) {
      console.error(err);
    }
  });
}

// 按钮：添加文件夹 (处理中禁用)
if (el.addFolderBtn) {
  el.addFolderBtn.addEventListener("click", async () => {
    if (isProcessing) return; // 🔒
    try {
      const folderPath = await open({
        directory: true,
        multiple: false,
      });

      if (folderPath) {
        setStatus(`正在扫描文件夹: ${folderPath}...`, "loading");
        const files = await invoke('scan_folder', { folderPath });

        if (files && files.length > 0) {
          const hasNew = fileQueue.add(files);
          if (hasNew) {
            renderFileList();
            setStatus(`成功添加 ${files.length} 张照片`, "success");
          } else {
            setStatus("文件夹中的照片已在列表中", "normal");
          }
        } else {
          setStatus("该文件夹内没有发现支持的图片", "error");
        }
      }
    } catch (err) {
      console.error(err);
      setStatus("读取文件夹失败", "error");
    }
  });
}

// 按钮：清空列表 (处理中禁用)
if (el.clearListBtn) {
  el.clearListBtn.addEventListener("click", () => {
    if (isProcessing) return; // 🔒
    fileQueue.clear();
    renderFileList();
    setStatus("列表已清空", "normal");
  });
}


// ==========================================
// 🟢 核心生成逻辑 (重构为 Rust 批处理 + 状态控制)
// ==========================================
// ==========================================
// 🟢 核心生成逻辑 (Debug 版)
// ==========================================
if (el.startBatchBtn) {
  console.log("✅ 批处理按钮已找到，监听器已挂载"); // 🟢 检查1：确认按钮元素存在

  el.startBatchBtn.addEventListener("click", async () => {
    console.log("🖱️ [Debug] 批处理按钮被点击"); // 🟢 检查2：确认点击事件触发
    console.log("   当前状态: isProcessing =", isProcessing, "canStop =", canStop);

    // --- 场景 A: 正在处理中 -> 处理“终止”逻辑 ---
    if (isProcessing) {
      console.log("   进入终止逻辑分支");
      if (canStop) {
        setStatus("正在终止...", "loading");
        console.log("🚀 [Debug] 调用 stop_batch_process...");
        await invoke('stop_batch_process');
      } else {
        console.log("⚠️ [Debug] 3秒防误触保护期，忽略点击");
      }
      return;
    }

    // --- 场景 B: 未处理 -> 处理“开始”逻辑 ---
    console.log("   进入启动逻辑分支");
    
    if (fileQueue.files.length === 0) {
      console.warn("⚠️ [Debug] 列表为空，中止");
      setStatus("列表为空，请先添加照片！", "error");
      return;
    }

    // 1. 获取参数
    const selectedStyle = el.styleSelect.value;
    const selectedFont = el.fontSelect.value;
    const selectedWeight = el.fontWeightSelect.value;
    // 确保是数字类型
    const shadowInt = parseFloat(el.shadowInput.value) || 0.0; 
    
    // 提取纯路径数组
    const filePaths = fileQueue.files.map(f => f.path);

    console.log("📦 [Debug] 准备发送参数:", {
        filePaths: filePaths, // 重点检查这个数组是否为空
        style: selectedStyle,
        fontFilename: selectedFont,
        fontWeight: selectedWeight,
        shadowIntensity: shadowInt
    });

    // 2. 更新 UI 状态
    isProcessing = true;
    canStop = false;
    toggleLoading(true); 
    
    if (el.fileList) el.fileList.classList.add("disabled-interaction");

    el.startBatchBtn.textContent = "启动中...";
    el.startBatchBtn.style.cursor = "not-allowed";
    el.startBatchBtn.classList.add("processing-mode");

    stopTimer = setTimeout(() => {
      if (isProcessing) {
        canStop = true;
        el.startBatchBtn.textContent = "终止处理";
        el.startBatchBtn.style.cursor = "pointer";
        el.startBatchBtn.classList.add("can-stop");
        console.log("⏱️ [Debug] 3秒倒计时结束，允许终止");
      }
    }, 3000);

    setStatus("准备开始批处理...", "loading");

    // 3. 调用 Rust
    try {
      console.log("🚀 [Debug] 正在执行 invoke('start_batch_process')...");
      
      // 注意：Tauri v1/v2 默认会自动将 JS 的驼峰 (filePaths) 转为 Rust 的蛇形 (file_paths)
      // 但为了保险，我们在这里打印一下 invoke 结果
      const res = await invoke('start_batch_process', {
        filePaths: filePaths,      // 对应 Rust: file_paths
        style: selectedStyle,      // 对应 Rust: style
        fontFilename: selectedFont,// 对应 Rust: font_filename
        fontWeight: selectedWeight,// 对应 Rust: font_weight
        shadowIntensity: shadowInt // 对应 Rust: shadow_intensity
      });

      console.log("✅ [Debug] Rust start_batch_process 返回:", res);
      
    } catch (error) {
      console.error("❌ [Debug] 批处理启动异常:", error);
      setStatus("批处理启动失败: " + error, "error");
      resetBatchState();
    }
  });
} else {
    console.error("❌ [Debug] 致命错误：无法在 DOM 中找到 startBatchBtn 元素！检查 elements.js 的 ID 是否匹配");
}

// Debug 按钮
if (el.debugShadowBtn) { el.debugShadowBtn.addEventListener("click", async () => { debugShadowGrid(); }); }
if (el.debugWeightBtn) { el.debugWeightBtn.addEventListener("click", async () => { debugWeightGrid(); }); }