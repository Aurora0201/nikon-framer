import { open } from '@tauri-apps/plugin-dialog';

// 导入同级 js 目录下的模块
import { el } from './js/elements.js';
import { processImage, debugShadowGrid, debugWeightGrid } from './js/commands.js';
import { loadFonts, initFontEvents } from './js/fonts.js';
import { initPreviewInteraction } from './js/preview.js';
import { initUIEvents, setStatus, toggleLoading, showPreview } from './js/ui.js';

// 初始化
window.addEventListener("DOMContentLoaded", () => {
  loadFonts();
  initFontEvents();
  initPreviewInteraction();
  initUIEvents();
});

// 生成按钮事件
el.btn.addEventListener("click", async () => {
  const selectedStyle = el.styleSelect.value;
  const selectedFont = el.fontSelect.value;
  const selectedWeight = el.fontWeightSelect.value;
  const shadowInt = parseFloat(el.shadowInput.value);

  try {
    const selected = await open({
      multiple: false,
      filters: [{ name: 'Images', extensions: ['jpg', 'jpeg', 'png', 'nef', 'dng', 'arw'] }]
    });

    if (selected === null) return;

    setStatus("正在处理中...", "loading");
    toggleLoading(true);

    const savedData = await processImage({
      filePath: selected,
      style: selectedStyle,
      fontFilename: selectedFont,
      fontWeight: selectedWeight,
      shadowIntensity: shadowInt
    });

    showPreview(savedData);
    setStatus("处理完成！文件已保存。", "success");

  } catch (error) {
    setStatus("发生错误: " + error, "error");
    console.error(error);
  } finally {
    toggleLoading(false);
  }
});

// Debug 按钮事件
if (el.debugShadowBtn) {
  el.debugShadowBtn.addEventListener("click", async () => {
    try {
      setStatus("正在生成阴影图例...", "loading");
      toggleLoading(true);
      const savedData = await debugShadowGrid();
      showPreview(savedData, 0.5); 
      setStatus("Debug 图例已生成", "success");
    } catch (error) {
      setStatus("错误: " + error, "error");
      console.error(error);
    } finally {
      toggleLoading(false);
    }
  });
}

if (el.debugWeightBtn) {
  el.debugWeightBtn.addEventListener("click", async () => {
    try {
      setStatus("正在生成粗细图例...", "loading");
      toggleLoading(true);
      const savedData = await debugWeightGrid();
      showPreview(savedData, 0.5);
      setStatus("Debug 图例已生成", "success");
    } catch (error) {
      setStatus("错误: " + error, "error");
      console.error(error);
    } finally {
      toggleLoading(false);
    }
  });
}