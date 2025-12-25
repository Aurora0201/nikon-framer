# Nikon Framer 📷

![Tauri](https://img.shields.io/badge/Tauri-v2-24C8DB?style=flat&logo=tauri&logoColor=black)
![Rust](https://img.shields.io/badge/Rust-1.75+-000000?style=flat&logo=rust&logoColor=white)
![Vue](https://img.shields.io/badge/Vue.js-3.x-4FC08D?style=flat&logo=vuedotjs&logoColor=white)
![Platform](https://img.shields.io/badge/Platform-macOS%20%7C%20Windows-lightgrey)
![License](https://img.shields.io/badge/License-MIT-green)

**Nikon Framer** 是一个专为尼康（Nikon）摄影师打造的高性能、隐私安全的本地水印相框生成工具。

基于 **Rust** 和 **Tauri v2** 构建，前端采用 **Vue 3** 现代化框架。它结合了系统级原生应用的极致性能与响应式 UI 的丝滑体验。无需上传图片到云端，利用本地算力毫秒级完成合成，完美支持 4000万+ 像素 RAW 转 JPG 大图处理。

---

## ✨ 核心特性 (Features)

### 🚀 强劲内核 (Rust Core)
* **极速处理**: 依托 Rust 的 `image-rs` 库与多线程并发模型 (Rayon/Threads)，处理高分辨率照片流畅无卡顿。
* **架构设计**: 后端采用 **策略模式 (Strategy Pattern)** 重构，支持无缝扩展新的相框风格，代码高内聚低耦合。
* **智能 EXIF**: 自动解析相机型号（如 Nikon Z8）、镜头参数（如 50mm f/1.8）、ISO、快门及光圈数据。
* **隐私优先**: 纯本地运行，断网可用，您的照片数据永远不会离开您的硬盘。

### 🛠️ 智能批处理 (Smart Batch)
* **响应式 UI**: 基于 **Vue 3 Reactivity** 系统，提供丝滑的文件列表管理与状态反馈。
* **安全控制**: 
    * **防误触机制**: 启动时的倒计时保护与二次确认逻辑。
    * **可中断任务**: 支持在处理数千张照片的过程中随时安全终止 (Graceful Shutdown)。
* **可视化进度**: 底部实时进度条显示处理百分比与当前文件名，任务状态一目了然。
* **智能过滤**: 后端直接拦截非图片文件，前端仅接收有效数据，极大降低 IPC 通信开销。

### 🎨 影廊级美学 (Aesthetics)
* **高斯模糊 (Atmosphere)**: 智能提取画面主色调生成模糊背景，辅以磨砂玻璃质感与弥散投影，营造高端氛围。
* **极简白底 (Gallery)**: 经典的画廊风格白边，仿拍立得布局，专注于影像本身的纯粹。
* **完美排版**:
    * 专为 Nikon Z 系列优化的 Logo 布局（Z Logo + 品牌字）。
    * **动态字体**: 支持自定义字体权重与阴影强度调节。

---

## 📸 效果预览 (Preview)

| **高斯模糊风格 (Blur)** | **极简白底风格 (White)** |
| :---: | :---: |
| ![blur](imgs/blur.jpg) | ![white](imgs/white.jpg) |

*(此处建议放一张软件主界面的截图，展示 Vue 重构后的控制面板和文件列表)*

---

## 🛠️ 技术栈 (Tech Stack)

本项目采用 **Clean Architecture** 指导下的现代化混合架构：

### Frontend (User Interface)
* **Framework**: Vue.js 3 (Composition API) + Vite
* **State Management**: Reactive Store (Vue Reactivity API)
* **Logic Reuse**: Custom Composables (`useBatchProcess`, `useGlobalEvents`) 封装核心逻辑
* **Styling**: Scoped CSS + CSS Variables

### Backend (Core Logic)
* **Language**: Rust (Edition 2021)
* **Architecture**: 
    * **DTO Pattern**: 使用强类型结构体 (`BatchContext`) 管理前后端通信。
    * **Enum Dispatch**: 利用 Rust 枚举分发不同的处理策略。
* **Key Crates**:
    * `tauri`: v2.0 跨平台框架
    * `image`: 图像处理管线
    * `kamadak-exif`: 元数据解析
    * `serde`: 高效序列化 (JSON <-> Rust Struct)
    * `rayon` / `std::thread`: 并发任务调度

---

## 🚀 开发指南 (Development)

如果您想在本地运行或贡献代码，请确保已安装 Rust 和 Node.js 环境。

### 1. 克隆项目

```bash
git clone [https://github.com/Aurora0201/nikon-framer.git](https://github.com/Aurora0201/nikon-framer.git)
cd nikon-framer
```

### 2. 安装依赖
```Bash

npm install
```
### 3. 开发模式运行
启动 Vite 前端热重载与 Rust 后端：

```Bash

npm run tauri dev
```
⚠️ 性能提示: 在 dev 模式下，Rust 编译器未开启优化，大图处理速度可能会较慢。如需体验真实性能，请使用 release 模式构建。

### 4. 打包构建
构建 Windows 安装包 (.msi / .exe) 或 macOS 应用 (.dmg / .app)：

```Bash

npm run tauri build
```
构建产物将位于 src-tauri/target/release/bundle 目录下。

### 📝 待办事项 (Todo)
- [x] 前端重构: 迁移至 Vue 3，实现组件化开发 ✅

- [x] 后端重构: 引入策略模式与 DTO，提升代码健壮性 ✅

- [x] 批量处理: 队列管理、进度条与终止功能 ✅

- [ ] 实时预览: 基于内存缓存的所见即所得 (WYSIWYG) 调节

- [ ] 更多样式: 胶片风格 (Film Look)、拍立得风格

- [ ] 品牌扩展: 支持 Sony, Canon, Fujifilm 等更多品牌 Logo

- [ ] 自定义签名: 支持用户上传个人水印

📄 许可证 (License)
MIT License © 2025 Aurora0201


