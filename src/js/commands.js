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