import { el } from './elements.js';
import { getFontList } from './commands.js';
import { setStatus } from './ui.js';

export async function loadFonts() {
  try {
    const fonts = await getFontList();
    el.fontSelect.innerHTML = '<option value="default">默认内置字体 (Default)</option>';
    fonts.forEach(fontName => {
      const option = document.createElement("option");
      option.value = fontName;
      option.innerText = fontName;
      el.fontSelect.appendChild(option);
    });
    console.log("字体列表已刷新:", fonts);
  } catch (err) {
    console.error("加载字体失败:", err);
    setStatus("警告：加载字体列表失败", "error");
  }
}

export function initFontEvents() {
  el.refreshFontsBtn.addEventListener("click", () => {
    el.refreshFontsBtn.style.transform = "rotate(360deg)";
    setTimeout(() => el.refreshFontsBtn.style.transform = "none", 500);
    loadFonts();
  });
}