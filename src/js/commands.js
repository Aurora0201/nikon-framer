const { invoke } = window.__TAURI__.core;

export async function getFontList() {
  return await invoke("get_font_list");
}
export async function processImage(params) {
  return await invoke("process_single_image", params);
}
export async function debugShadowGrid() {
  return await invoke("debug_shadow_grid");
}
export async function debugWeightGrid() {
  return await invoke("debug_weight_grid");
}

export async function checkExif(filePath) {
  try {
    // 这里调用 Rust: fn check_file_exif(path: &str) -> bool
    return await invoke("check_file_exif", { path: filePath });
  } catch (e) {
    console.error(e);
    return false;
  }
}

// 🟢 [新增] 扫描文件夹 (如果前端做不了，就需要 Rust)
// 在 Tauri V2 中，前端无法直接列出文件夹内容，必须依靠 Rust Command
export async function scanFolder(folderPath) {
    return await invoke("scan_folder_for_images", { folderPath });
}