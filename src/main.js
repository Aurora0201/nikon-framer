import { open } from '@tauri-apps/plugin-dialog';
// 🟢 1. 引入 invoke 用于调用 Rust 指令，listen 用于监听事件
import { listen } from '@tauri-apps/api/event';
import { invoke } from '@tauri-apps/api/core';

import { el } from './js/elements.js';
import { processImage, debugShadowGrid, debugWeightGrid } from './js/commands.js';
import { loadFonts, initFontEvents } from './js/fonts.js';
import { initPreviewInteraction } from './js/preview.js';
import { initUIEvents, setStatus, toggleLoading, showPreview, renderFileList } from './js/ui.js';
import { fileQueue } from './js/state.js';

// 🔴 关键：彻底屏蔽浏览器的默认拖拽行为（防止打开图片）
document.addEventListener('dragover', e => e.preventDefault());
document.addEventListener('drop', e => e.preventDefault());

// 初始化
window.addEventListener("DOMContentLoaded", async () => {
  loadFonts();
  initFontEvents();
  initPreviewInteraction();
  initUIEvents();
  renderFileList();

  // ==========================================
  // 🟢 Tauri Listen API 方案 (dragDropEnabled: true)
  // ==========================================
  console.log("🚀 注册 Tauri 事件监听...");

  // 1. 监听文件真正“放下” (获取路径)
  const unlistenDrop = await listen('tauri://drag-drop', async (event) => {
    const paths = event.payload.paths;
    console.log("拖入路径:", paths);

    if (paths && paths.length > 0) {
      if (el.dropZone) {
        
        // 🟢 [修改] 调用 Rust 进行过滤，剔除文件夹
        try {
          // 调用我们在 Rust 中新增的 filter_files 指令
          const validFiles = await invoke('filter_files', { paths });

          if (validFiles.length > 0) {
            // 将过滤后的文件列表加入队列
            const hasNew = fileQueue.add(validFiles);
            
            if (hasNew) {
              renderFileList();
              // 如果过滤后数量变少了，说明剔除了文件夹
              if (validFiles.length < paths.length) {
                setStatus(`已添加 ${validFiles.length} 个文件 (已忽略文件夹)`, "success");
              } else {
                setStatus(`已添加 ${validFiles.length} 个文件`, "success");
              }
            } else {
              setStatus("文件已存在列表中", "normal");
            }
          } else {
            // 如果 validFiles 为空，说明拖进来的全是文件夹
            setStatus("未检测到图片文件 (文件夹已忽略)", "loading"); // 用 loading 颜色做个轻提示
            setTimeout(() => setStatus("请拖入具体的图片文件", "normal"), 2000);
          }
        } catch (error) {
          console.error("文件过滤失败:", error);
          setStatus("文件读取错误", "error");
        }
      }
    }
    
    // 移除高亮
    if(el.dropZone) el.dropZone.classList.remove('active');
  });

  // 2. 监听拖拽进入窗口 (Global Hover)
  const unlistenHover = await listen('tauri://drag-enter', (event) => {
    if(el.dropZone) el.dropZone.classList.add('active');
  });

  // 3. 监听拖拽取消/离开窗口
  const unlistenCancel = await listen('tauri://drag-leave', (event) => {
    if(el.dropZone) el.dropZone.classList.remove('active');
  });

  console.log("✅ Tauri Listen 监听已注册");
});


// ==========================================
// 🟢 按钮逻辑 (保持不变)
// ==========================================

// 按钮：添加文件
if (el.addFilesBtn) {
  el.addFilesBtn.addEventListener("click", async () => {
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

// 按钮：添加文件夹
if (el.addFolderBtn) {
  el.addFolderBtn.addEventListener("click", async () => {
    try {
      // 1. 打开文件夹选择对话框
      const folderPath = await open({
        directory: true, // 关键：设置为选择文件夹模式
        multiple: false, // 通常选一个文件夹即可
      });

      // 如果用户取消了选择，folderPath 会是 null
      if (folderPath) {
        setStatus(`正在扫描文件夹: ${folderPath}...`, "loading");
        
        // 2. 让 Rust 扫描该文件夹下的图片
        const files = await invoke('scan_folder', { folderPath });

        if (files && files.length > 0) {
          // 3. 加入队列
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

// 按钮：清空列表
if (el.clearListBtn) {
  el.clearListBtn.addEventListener("click", () => {
    fileQueue.clear();
    renderFileList();
    setStatus("列表已清空", "normal");
  });
}

// ==========================================
// 🟢 核心生成逻辑 (批处理) (保持不变)
// ==========================================
if (el.startBatchBtn) {
  el.startBatchBtn.addEventListener("click", async () => {
    if (fileQueue.files.length === 0) {
      setStatus("列表为空，请先添加照片！", "error");
      return;
    }

    const selectedStyle = el.styleSelect.value;
    const selectedFont = el.fontSelect.value;
    const selectedWeight = el.fontWeightSelect.value;
    const shadowInt = parseFloat(el.shadowInput.value);

    setStatus("正在批处理中...", "loading");
    toggleLoading(true);

    try {
      for (let i = 0; i < fileQueue.files.length; i++) {
        const file = fileQueue.files[i];
        setStatus(`正在处理 (${i + 1}/${fileQueue.files.length}): ${file.name}`, "loading");

        const savedData = await processImage({
          filePath: file.path,
          style: selectedStyle,
          fontFilename: selectedFont,
          fontWeight: selectedWeight,
          shadowIntensity: shadowInt
        });

        if (i === fileQueue.files.length - 1) {
          showPreview(savedData);
        }
      }
      setStatus(`全部完成！共处理 ${fileQueue.files.length} 张照片。`, "success");
    } catch (error) {
      setStatus("处理中断: " + error, "error");
    } finally {
      toggleLoading(false);
    }
  });
}

// Debug 按钮 (保持不变)
if (el.debugShadowBtn) { el.debugShadowBtn.addEventListener("click", async () => { debugShadowGrid(); }); }
if (el.debugWeightBtn) { el.debugWeightBtn.addEventListener("click", async () => { debugWeightGrid(); }); }