export const el = {
  status: document.getElementById("status"),
  loadingSpinner: document.getElementById("loading-spinner"),
  btn: document.getElementById("select-btn"),
  styleSelect: document.getElementById("style-select"),
  fontSelect: document.getElementById("font-select"),
  fontWeightSelect: document.getElementById("font-weight-select"),
// 🟢 [新增] 注册阴影控制组容器
  shadowControlGroup: document.getElementById("shadow-control-group"),

  shadowInput: document.getElementById("shadow-intensity"),
  shadowValDisplay: document.getElementById("shadow-val"),
  refreshFontsBtn: document.getElementById("refresh-fonts-btn"),
  debugShadowBtn: document.getElementById("debug-shadow-btn"),
  debugWeightBtn: document.getElementById("debug-weight-btn"),
  modal: document.getElementById("preview-modal"),
  previewImg: document.getElementById("preview-img"),
  closeBtn: document.getElementById("close-preview-btn"),
};