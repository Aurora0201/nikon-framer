import { el } from './elements.js';
import { getFontList } from './commands.js';
import { setStatus } from './ui.js';

export async function loadFonts() {
  try {
    const fonts = await getFontList();
    
    // 1. 清空旧列表
    el.fontSelect.innerHTML = ''; 

    // 2. 如果列表为空，做个保底
    if (!fonts || fonts.length === 0) {
       const option = document.createElement("option");
       option.value = "default";
       option.innerText = "Error: No Fonts Found";
       el.fontSelect.appendChild(option);
       return;
    }

    // 3. 遍历渲染所有字体
    fonts.forEach(fontName => {
        const option = document.createElement("option");
        
        // 🟢 核心修改：value 传完整文件名，innerText 显示去除后缀的名字
        option.value = fontName; 
        
        // 使用正则去除最后的扩展名 (例如 "MyFont.ttf" -> "MyFont")
        const displayName = fontName.replace(/\.[^/.]+$/, "");
        option.innerText = displayName; 
        
        el.fontSelect.appendChild(option);
    });

    // 4. 默认选中列表的第一个 (内置字体)
    if (el.fontSelect.options.length > 0) {
        el.fontSelect.selectedIndex = 0;
    }

    console.log("字体列表已加载:", fonts);
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