export const el = {
  // --- 原有元素 ---
  status: document.getElementById("status"),
  loadingSpinner: document.getElementById("loading-spinner"),
  
  // 注意：原先的 select-btn 已经被你在 HTML 里改名为 start-batch-btn 了
  // 所以这里要对应修改，或者保留原名但获取新 ID
  startBatchBtn: document.getElementById("start-batch-btn"), 
  
  styleSelect: document.getElementById("style-select"),
  fontSelect: document.getElementById("font-select"),
  fontWeightSelect: document.getElementById("font-weight-select"),
  shadowControlGroup: document.getElementById("shadow-control-group"),
  shadowInput: document.getElementById("shadow-intensity"),
  shadowValDisplay: document.getElementById("shadow-val"),
  refreshFontsBtn: document.getElementById("refresh-fonts-btn"),
  
  debugShadowBtn: document.getElementById("debug-shadow-btn"),
  debugWeightBtn: document.getElementById("debug-weight-btn"),
  
  modal: document.getElementById("preview-modal"),
  previewImg: document.getElementById("preview-img"),
  closeBtn: document.getElementById("close-preview-btn"),

  // --- 🟢 [新增] 必须添加以下内容，否则 main.js 找不到元素 ---
  addFilesBtn: document.getElementById("add-files-btn"),
  addFolderBtn: document.getElementById("add-folder-btn"),
  dropZone: document.getElementById("drop-zone"),
  emptyTip: document.getElementById("empty-tip"),
  fileList: document.getElementById("file-list"),
  queueCount: document.getElementById("queue-count"),
  clearListBtn: document.getElementById("clear-list-btn"),
};